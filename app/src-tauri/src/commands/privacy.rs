use crate::core::config::Config;
use license_control_suite::desktop::tauri::AuthCommandError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const USER_AGENT: &str = "ai-youtube-shorts-privacy/0.1.0";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserDataDeletionInput {
    pub license_key: String,
    pub purchaser_email: Option<String>,
    pub confirmation: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserDataDeletionStatusInput {
    pub request_id: String,
    pub lookup_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserDataDeletionView {
    pub request_id: String,
    pub lookup_token: Option<String>,
    pub status: String,
    pub message: Option<String>,
    pub completed_at_ms: Option<u64>,
    pub error_code: Option<String>,
}

#[derive(Deserialize)]
struct WorkerEnvelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<WorkerError>,
}

#[derive(Deserialize)]
struct WorkerError {
    code: String,
    message: String,
}

#[derive(Serialize)]
struct WorkerDeletionRequest<'a> {
    license_key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    purchaser_email: Option<&'a str>,
    confirmation: &'a str,
    app_version: &'a str,
    timestamp_ms: u64,
}

#[derive(Serialize)]
struct WorkerDeletionStatusRequest<'a> {
    request_id: &'a str,
    lookup_token: &'a str,
}

#[tauri::command]
pub async fn request_user_data_deletion(
    input: UserDataDeletionInput,
) -> Result<UserDataDeletionView, AuthCommandError> {
    let license_key = input.license_key.trim();
    let confirmation = input.confirmation.trim();
    if license_key.is_empty() || confirmation != "DELETE" {
        return Err(command_error(
            "invalid_deletion_request",
            "Deletion request requires a license key and DELETE confirmation.",
        ));
    }
    let purchaser_email = input
        .purchaser_email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let body = WorkerDeletionRequest {
        license_key,
        purchaser_email,
        confirmation,
        app_version: env!("CARGO_PKG_VERSION"),
        timestamp_ms: now_epoch_ms(),
    };
    send_worker_request(
        "/v1/privacy/delete/request",
        &body,
        Some(&generate_idempotency_key("privacy_delete_request", license_key)),
    )
    .await
}

#[tauri::command]
pub async fn get_user_data_deletion_status(
    input: UserDataDeletionStatusInput,
) -> Result<UserDataDeletionView, AuthCommandError> {
    let request_id = input.request_id.trim();
    let lookup_token = input.lookup_token.trim();
    if request_id.is_empty() || lookup_token.is_empty() {
        return Err(command_error(
            "bad_request",
            "Deletion status requires a request id and lookup token.",
        ));
    }
    let body = WorkerDeletionStatusRequest {
        request_id,
        lookup_token,
    };
    send_worker_request("/v1/privacy/delete/status", &body, None).await
}

async fn send_worker_request<T, B>(
    path: &str,
    body: &B,
    idempotency_key: Option<&str>,
) -> Result<T, AuthCommandError>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    crate::core::config::load_env_files_near_current_dir();
    let cfg = Config::from_env().map_err(|_| command_error("storage", "Could not load Worker configuration."))?;
    let url = format!("{}{}", cfg.license_worker_config().base_url, path);
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_millis(
            cfg.license_worker_config().timeout_ms,
        ))
        .build()
        .map_err(|_| command_error("storage", "Could not initialize Worker client."))?;
    let mut request = client.post(url).json(body);
    if let Some(key) = idempotency_key {
        request = request.header("x-idempotency-key", key);
    }
    let response = request
        .send()
        .await
        .map_err(|_| command_error("worker_unreachable", "Unable to reach the license service right now."))?;
    let text = response
        .text()
        .await
        .map_err(|_| command_error("serialization", "Worker response could not be read."))?;
    parse_worker_response(&text)
}

fn parse_worker_response<T>(text: &str) -> Result<T, AuthCommandError>
where
    T: DeserializeOwned,
{
    let envelope: WorkerEnvelope<T> = serde_json::from_str(text)
        .map_err(|_| command_error("serialization", "Worker returned an invalid response."))?;
    if envelope.ok {
        return envelope
            .data
            .ok_or_else(|| command_error("serialization", "Worker response was missing data."));
    }
    let error = envelope.error;
    Err(command_error(
        error
            .as_ref()
            .map(|value| value.code.as_str())
            .unwrap_or("unknown"),
        error
            .as_ref()
            .map(|value| value.message.as_str())
            .unwrap_or("Worker request failed."),
    ))
}

fn command_error(code: impl Into<String>, message: impl Into<String>) -> AuthCommandError {
    AuthCommandError {
        code: code.into(),
        message: message.into(),
    }
}

fn generate_idempotency_key(action: &str, license_key: &str) -> String {
    let seed = format!("{action}:{}:{}", license_key.trim(), now_epoch_ms());
    let digest = Sha256::digest(seed.as_bytes());
    format!("privacy_{}", bytes_to_hex(&digest)[..32].to_string())
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

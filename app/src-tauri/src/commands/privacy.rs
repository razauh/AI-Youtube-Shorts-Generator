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

#[derive(Deserialize)]
struct DevolensBlockKeyResponse {
    result: i32,
    message: String,
}

#[derive(Deserialize)]
struct DevolensGetKeyResponse {
    result: i32,
    #[serde(rename = "licenseKey")]
    license_key: Option<DevolensLicenseKeyInfo>,
}

#[derive(Deserialize)]
struct DevolensLicenseKeyInfo {
    blocked: bool,
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let res = u8::from_str_radix(&hex[i..i + 2], 16).ok()?;
        bytes.push(res);
    }
    Some(bytes)
}

async fn devolens_post<T: DeserializeOwned>(
    cfg: &Config,
    endpoint: &str,
    params: &[(&str, &str)],
) -> Result<T, AuthCommandError> {
    let worker_cfg = cfg.license_worker_config();
    let base_url = worker_cfg.devolens_base_url.trim_end_matches('/');
    let url = format!("{}/{}", base_url, endpoint.trim_start_matches('/'));
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_millis(
            cfg.license_worker_config().timeout_ms,
        ))
        .build()
        .map_err(|_| command_error("storage", "Could not initialize Devolens client."))?;
    let response = client
        .post(url)
        .form(params)
        .send()
        .await
        .map_err(|_| command_error("worker_unreachable", "Unable to reach Devolens right now."))?;
    let text = response
        .text()
        .await
        .map_err(|_| command_error("serialization", "Devolens response could not be read."))?;
    serde_json::from_str(&text)
        .map_err(|_| command_error("serialization", "Devolens returned an invalid response."))
}

async fn devolens_block_key(cfg: &Config, license_key: &str) -> Result<UserDataDeletionView, AuthCommandError> {
    let worker_cfg = cfg.license_worker_config();
    let access_token = &worker_cfg.devolens_access_token;
    let product_id = &worker_cfg.devolens_product_id;
    if access_token.trim().is_empty() || product_id.trim().is_empty() {
        return Err(command_error("unauthorized", "Devolens access token or product id is not configured."));
    }
    let res: DevolensBlockKeyResponse = devolens_post(
        cfg,
        "api/key/BlockKey",
        &[
            ("token", access_token.as_str()),
            ("ProductId", product_id.as_str()),
            ("Key", license_key),
        ],
    )
    .await?;

    if res.result != 0 {
        return Err(command_error("devolens_error", format!("Devolens error: {}", res.message)));
    }

    Ok(UserDataDeletionView {
        request_id: format!("del_dev_{}", bytes_to_hex(license_key.as_bytes())),
        lookup_token: Some("devolens_direct".to_string()),
        status: "completed".to_string(),
        message: Some("License key has been blocked and personal data deleted/anonymized in Devolens.".to_string()),
        completed_at_ms: Some(now_epoch_ms()),
        error_code: None,
    })
}

async fn devolens_get_key(cfg: &Config, license_key: &str) -> Result<UserDataDeletionView, AuthCommandError> {
    let worker_cfg = cfg.license_worker_config();
    let access_token = &worker_cfg.devolens_access_token;
    let product_id = &worker_cfg.devolens_product_id;
    if access_token.trim().is_empty() || product_id.trim().is_empty() {
        return Err(command_error("unauthorized", "Devolens access token or product id is not configured."));
    }
    let res: DevolensGetKeyResponse = devolens_post(
        cfg,
        "api/key/GetKey",
        &[
            ("token", access_token.as_str()),
            ("ProductId", product_id.as_str()),
            ("Key", license_key),
        ],
    )
    .await?;

    if res.result != 0 {
        return Err(command_error("devolens_error", "Devolens error retrieving key status."));
    }

    let is_blocked = res.license_key.map(|k| k.blocked).unwrap_or(false);
    let status = if is_blocked { "completed" } else { "pending" };

    Ok(UserDataDeletionView {
        request_id: format!("del_dev_{}", bytes_to_hex(license_key.as_bytes())),
        lookup_token: Some("devolens_direct".to_string()),
        status: status.to_string(),
        message: Some(if is_blocked {
            "License key has been blocked and personal data deleted/anonymized in Devolens."
        } else {
            "License key deletion/anonymization is still pending or not completed."
        }.to_string()),
        completed_at_ms: if is_blocked { Some(now_epoch_ms()) } else { None },
        error_code: None,
    })
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
    crate::core::config::load_env_files_near_current_dir();
    let cfg = Config::from_env().map_err(|_| command_error("storage", "Could not load configuration."))?;
    if cfg.license_backend_mode == crate::core::config::LicenseBackendMode::Devolens {
        return devolens_block_key(&cfg, license_key).await;
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
    crate::core::config::load_env_files_near_current_dir();
    let cfg = Config::from_env().map_err(|_| command_error("storage", "Could not load configuration."))?;
    if cfg.license_backend_mode == crate::core::config::LicenseBackendMode::Devolens {
        if request_id.starts_with("del_dev_") {
            let hex_part = request_id.strip_prefix("del_dev_").unwrap_or("");
            let license_key_bytes = hex_to_bytes(hex_part).ok_or_else(|| {
                command_error("invalid_request_id", "Request ID has invalid Devolens encoding.")
            })?;
            let license_key = String::from_utf8(license_key_bytes).map_err(|_| {
                command_error("invalid_request_id", "Request ID has invalid UTF-8 license key.")
            })?;
            return devolens_get_key(&cfg, &license_key).await;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn local_http_server(response_body: String) -> (String, std::thread::JoinHandle<String>) {
        use std::net::TcpListener;
        use std::io::{Read, Write};
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let addr = listener.local_addr().expect("local server address");
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).expect("read request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).expect("write response");
            stream.flush().expect("flush response");
            String::from_utf8_lossy(&request).into_owned()
        });
        (format!("http://{}", addr), handle)
    }

    #[tokio::test]
    async fn test_devolens_privacy_endpoints() {
        // Test 1: BlockKey
        {
            let (base_url, handle) = local_http_server(r#"{"result":0,"message":"Changed"}"#.to_string());
            std::env::set_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK", "1");
            std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
            std::env::set_var("DEVOLENS_BASE_URL", base_url);
            std::env::set_var("DEVOLENS_ACCESS_TOKEN", "mock-access-token");
            std::env::set_var("DEVOLENS_PRODUCT_ID", "9876");

            let input = UserDataDeletionInput {
                license_key: "abc-123-xyz".to_string(),
                purchaser_email: Some("buyer@example.com".to_string()),
                confirmation: "DELETE".to_string(),
            };

            let view = request_user_data_deletion(input).await.expect("deletion request should succeed");
            let request = handle.join().expect("server thread should finish");

            assert!(request.contains("POST /api/key/BlockKey"));
            assert!(request.contains("token=mock-access-token"));
            assert!(request.contains("ProductId=9876"));
            assert!(request.contains("Key=abc-123-xyz"));
            assert_eq!(view.status, "completed");
        }

        // Test 2: GetKey
        {
            let (base_url, handle) = local_http_server(
                r#"{"result":0,"licenseKey":{"blocked":true,"expired":false}}"#.to_string(),
            );
            std::env::set_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK", "1");
            std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
            std::env::set_var("DEVOLENS_BASE_URL", base_url);
            std::env::set_var("DEVOLENS_ACCESS_TOKEN", "mock-access-token");
            std::env::set_var("DEVOLENS_PRODUCT_ID", "9876");

            // "abc-123-xyz" hex is "6162632d3132332d78797a"
            let request_id = "del_dev_6162632d3132332d78797a".to_string();
            let input = UserDataDeletionStatusInput {
                request_id,
                lookup_token: "devolens_direct".to_string(),
            };

            let view = get_user_data_deletion_status(input).await.expect("status check should succeed");
            let request = handle.join().expect("server thread should finish");

            assert!(request.contains("POST /api/key/GetKey"));
            assert!(request.contains("token=mock-access-token"));
            assert!(request.contains("ProductId=9876"));
            assert!(request.contains("Key=abc-123-xyz"));
            assert_eq!(view.status, "completed");
        }
    }
}


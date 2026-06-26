use crate::commands::runtime::{secure_store_delete, secure_store_load, secure_store_save};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ADMIN_BASE_URL_KEY: &str = "ADMIN_DESKTOP_WORKER_BASE_URL";
const ADMIN_TOKEN_KEY: &str = "ADMIN_DESKTOP_API_TOKEN";
const ADMIN_USER_AGENT: &str = "ai-youtube-shorts-admin-desktop/0.1.0";
static IDEMPOTENCY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResetRequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeletionRequestStatus {
    Pending,
    Approved,
    Processing,
    Rejected,
    Completed,
    Failed,
}

impl DeletionRequestStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Processing => "processing",
            Self::Rejected => "rejected",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl ResetRequestStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseState {
    BoundActive,
    Unbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminResetRequestItem {
    pub reset_request_id: String,
    pub status: ResetRequestStatus,
    pub license_state: LicenseState,
    pub message: String,
    pub masked_license_key: Option<String>,
    pub has_license_hash: bool,
    pub purchaser_email: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminResetListData {
    pub requests: Vec<AdminResetRequestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminResetDecisionData {
    pub reset_request_id: String,
    pub status: ResetRequestStatus,
    pub license_state: LicenseState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminDeletionRequestItem {
    pub deletion_request_id: String,
    pub status: DeletionRequestStatus,
    pub masked_license_key: Option<String>,
    pub has_license_hash: bool,
    pub license_hash_prefix: Option<String>,
    pub purchaser_email: Option<String>,
    pub requested_scope: String,
    pub deletion_preview: Option<serde_json::Value>,
    pub deletion_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub privacy_review: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message_safe: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub decided_at_ms: Option<u64>,
    pub completed_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminDeletionListData {
    pub requests: Vec<AdminDeletionRequestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminDeletionDecisionData {
    pub deletion_request_id: String,
    pub status: DeletionRequestStatus,
    pub deletion_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub privacy_review: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminDisableLicenseData {
    pub license_hash_prefix: String,
    pub entitlement_status: String,
    pub deactivate_bindings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminOverviewData {
    pub total_licenses: u64,
    pub entitlement_counts: std::collections::BTreeMap<String, u64>,
    pub device_binding_counts: std::collections::BTreeMap<String, u64>,
    pub reset_request_counts: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    pub deletion_request_counts: std::collections::BTreeMap<String, u64>,
    pub recent_audit_events_24h: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminAuditEventItem {
    pub event_type: String,
    pub actor: Option<String>,
    pub created_at_ms: u64,
    pub metadata_summary: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminAuditEventListData {
    pub events: Vec<AdminAuditEventItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminIdempotencyRecordItem {
    pub op: String,
    pub idempotency_key_prefix: String,
    pub payload_hash_prefix: String,
    pub response_status: u16,
    pub response_body_size: usize,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminIdempotencyRecordListData {
    pub records: Vec<AdminIdempotencyRecordItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminConfigView {
    pub base_url: Option<String>,
    pub token_configured: bool,
    pub token_redacted: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandError {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
    pub retryable: bool,
}

#[derive(Debug, Deserialize)]
struct WorkerEnvelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<WorkerError>,
}

#[derive(Debug, Deserialize)]
struct WorkerError {
    code: String,
    message: String,
    request_id: String,
    retryable: bool,
}

#[derive(Debug, Serialize)]
struct AdminDecisionRequest<'a> {
    request_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct AdminDeletionApproveRequest<'a> {
    request_id: &'a str,
    confirmation: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct AdminDisableLicenseRequest<'a> {
    license_hash_prefix: &'a str,
    reason: &'a str,
    deactivate_bindings: bool,
}

#[derive(Debug, Clone)]
struct AdminConfig {
    base_url: String,
    token: String,
}

#[derive(Debug, Clone, Copy)]
enum AdminAction {
    TestConnection,
    List,
    Overview,
    AuditEvents,
    IdempotencyRecords,
    Approve,
    Reject,
    DisableLicense,
    ListDeletionRequests,
    ApproveDeletionRequest,
    RejectDeletionRequest,
}

impl AdminAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::TestConnection => "test_connection",
            Self::List => "list_reset_requests",
            Self::Overview => "overview",
            Self::AuditEvents => "audit_events",
            Self::IdempotencyRecords => "idempotency_records",
            Self::Approve => "approve_reset_request",
            Self::Reject => "reject_reset_request",
            Self::DisableLicense => "disable_license",
            Self::ListDeletionRequests => "list_deletion_requests",
            Self::ApproveDeletionRequest => "approve_deletion_request",
            Self::RejectDeletionRequest => "reject_deletion_request",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HttpMethod {
    Get,
    Post,
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn command_error(
    code: impl Into<String>,
    message: impl Into<String>,
    request_id: Option<String>,
    retryable: bool,
) -> AdminCommandError {
    AdminCommandError {
        code: code.into(),
        message: message.into(),
        request_id,
        retryable,
    }
}

fn validate_base_url(value: &str) -> Result<String, AdminCommandError> {
    let trimmed = value.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        return Err(command_error(
            "bad_request",
            "Worker API base URL is required.",
            None,
            false,
        ));
    }
    let parsed = reqwest::Url::parse(&trimmed).map_err(|_| {
        command_error(
            "bad_request",
            "Worker API base URL must be a valid http or https URL.",
            None,
            false,
        )
    })?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err(command_error(
            "bad_request",
            "Worker API base URL must be a valid http or https URL.",
            None,
            false,
        ));
    }
    Ok(trimmed)
}

fn validate_token(value: &str) -> Result<String, AdminCommandError> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(command_error(
            "unauthorized",
            "Admin API token is required.",
            None,
            false,
        ));
    }
    Ok(trimmed)
}

fn validate_support_reason(value: Option<&str>) -> Result<&str, AdminCommandError> {
    value.map(str::trim).filter(|value| !value.is_empty()).ok_or_else(|| {
        command_error(
            "bad_request",
            "Support decision reason is required.",
            None,
            false,
        )
    })
}

pub fn redact_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 4 {
        return "[redacted]".to_string();
    }
    let suffix: String = chars[chars.len().saturating_sub(4)..].iter().collect();
    format!("[redacted]...{suffix}")
}

pub fn normalize_worker_base_url(value: &str) -> Result<String, AdminCommandError> {
    validate_base_url(value)
}

pub fn generate_admin_idempotency_key(action: &str, request_id: &str) -> String {
    let seed = format!(
        "admin:{action}:{request_id}:{}:{}:{}",
        now_epoch_ms(),
        std::process::id(),
        IDEMPOTENCY_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let digest = Sha256::digest(seed.as_bytes());
    format!("admin_{}", bytes_to_hex(&digest)[..32].to_string())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn log_admin_event(
    action: AdminAction,
    path: &str,
    status: Option<StatusCode>,
    request_id: Option<&str>,
    category: &str,
) {
    eprintln!(
        "admin_api action={} endpoint={} status={} worker_request_id={} category={} timestamp_ms={}",
        action.as_str(),
        path,
        status
            .map(|value| value.as_u16().to_string())
            .unwrap_or_else(|| "network".to_string()),
        request_id.unwrap_or("none"),
        category,
        now_epoch_ms()
    );
}

fn config_from_store() -> Result<AdminConfig, AdminCommandError> {
    let base_url = secure_store_load(ADMIN_BASE_URL_KEY.to_string())
        .map_err(|_| command_error("storage", "Could not load admin base URL.", None, true))?
        .ok_or_else(|| command_error("bad_request", "Admin configuration is incomplete.", None, false))?;
    let token = secure_store_load(ADMIN_TOKEN_KEY.to_string())
        .map_err(|_| command_error("storage", "Could not load admin token.", None, true))?
        .ok_or_else(|| command_error("unauthorized", "Admin token is not configured.", None, false))?;
    Ok(AdminConfig {
        base_url: validate_base_url(&base_url)?,
        token: validate_token(&token)?,
    })
}

async fn send_admin_request<T, B>(
    action: AdminAction,
    method: HttpMethod,
    path: &str,
    body: Option<&B>,
    idempotency_key: Option<&str>,
) -> Result<T, AdminCommandError>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let config = config_from_store()?;
    let url = format!("{}{}", config.base_url, path);
    let client = reqwest::Client::builder()
        .user_agent(ADMIN_USER_AGENT)
        .build()
        .map_err(|_| command_error("storage", "Could not initialize admin API client.", None, true))?;
    let mut request = match method {
        HttpMethod::Get => client.get(url),
        HttpMethod::Post => client.post(url),
    }
    .bearer_auth(&config.token);

    if let Some(value) = idempotency_key {
        request = request.header("x-idempotency-key", value);
    }
    if matches!(
        action,
        AdminAction::ApproveDeletionRequest | AdminAction::RejectDeletionRequest
    ) {
        request = request.header("x-admin-actor", "admin_desktop");
    }
    if let Some(value) = body {
        request = request.json(value);
    }

    let response = request.send().await.map_err(|_| {
        log_admin_event(action, path, None, None, "network");
        command_error("network", "Could not reach the Worker API.", None, true)
    })?;
    let status = response.status();
    let text = response.text().await.map_err(|_| {
        log_admin_event(action, path, Some(status), None, "serialization");
        command_error(
            "serialization",
            "Worker API response could not be read.",
            None,
            true,
        )
    })?;
    parse_worker_response(action, path, status, &text)
}

fn parse_worker_response<T>(
    action: AdminAction,
    path: &str,
    status: StatusCode,
    text: &str,
) -> Result<T, AdminCommandError>
where
    T: DeserializeOwned,
{
    let envelope: WorkerEnvelope<T> = serde_json::from_str(text).map_err(|_| {
        log_admin_event(action, path, Some(status), None, "serialization");
        command_error(
            "serialization",
            "Worker API returned an invalid response.",
            None,
            true,
        )
    })?;

    if status.is_success() && envelope.ok {
        log_admin_event(action, path, Some(status), None, "success");
        return envelope.data.ok_or_else(|| {
            command_error(
                "serialization",
                "Worker API response was missing data.",
                None,
                true,
            )
        });
    }

    let error = envelope.error;
    log_admin_event(
        action,
        path,
        Some(status),
        error.as_ref().map(|value| value.request_id.as_str()),
        error
            .as_ref()
            .map(|value| value.code.as_str())
            .unwrap_or("unknown"),
    );
    Err(command_error(
        error
            .as_ref()
            .map(|value| value.code.as_str())
            .unwrap_or("unknown"),
        error
            .as_ref()
            .map(|value| value.message.as_str())
            .unwrap_or("Worker API request failed."),
        error.as_ref().map(|value| value.request_id.clone()),
        error.as_ref().map(|value| value.retryable).unwrap_or(false),
    ))
}

#[tauri::command]
pub fn admin_config_load() -> Result<AdminConfigView, AdminCommandError> {
    let base_url = secure_store_load(ADMIN_BASE_URL_KEY.to_string())
        .map_err(|_| command_error("storage", "Could not load admin base URL.", None, true))?;
    let token = secure_store_load(ADMIN_TOKEN_KEY.to_string())
        .map_err(|_| command_error("storage", "Could not load admin token.", None, true))?;
    Ok(AdminConfigView {
        base_url,
        token_configured: token.as_ref().is_some_and(|value| !value.trim().is_empty()),
        token_redacted: token.as_ref().map(|value| redact_token(value)),
    })
}

#[tauri::command]
pub fn admin_config_save(base_url: String, token: String) -> Result<AdminConfigView, AdminCommandError> {
    let base_url = validate_base_url(&base_url)?;
    let token = validate_token(&token)?;
    secure_store_save(ADMIN_BASE_URL_KEY.to_string(), base_url.clone())
        .map_err(|_| command_error("storage", "Could not save admin base URL.", None, true))?;
    secure_store_save(ADMIN_TOKEN_KEY.to_string(), token.clone())
        .map_err(|_| command_error("storage", "Could not save admin token.", None, true))?;
    Ok(AdminConfigView {
        base_url: Some(base_url),
        token_configured: true,
        token_redacted: Some(redact_token(&token)),
    })
}

#[tauri::command]
pub fn admin_config_clear() -> Result<AdminConfigView, AdminCommandError> {
    secure_store_delete(ADMIN_BASE_URL_KEY.to_string())
        .map_err(|_| command_error("storage", "Could not clear admin base URL.", None, true))?;
    secure_store_delete(ADMIN_TOKEN_KEY.to_string())
        .map_err(|_| command_error("storage", "Could not clear admin token.", None, true))?;
    Ok(AdminConfigView {
        base_url: None,
        token_configured: false,
        token_redacted: None,
    })
}

#[tauri::command]
pub async fn admin_test_connection() -> Result<AdminOverviewData, AdminCommandError> {
    send_admin_request::<AdminOverviewData, ()>(
        AdminAction::TestConnection,
        HttpMethod::Get,
        "/v1/admin/overview",
        None,
        None,
    )
    .await
}

#[tauri::command]
pub async fn admin_overview() -> Result<AdminOverviewData, AdminCommandError> {
    send_admin_request::<AdminOverviewData, ()>(
        AdminAction::Overview,
        HttpMethod::Get,
        "/v1/admin/overview",
        None,
        None,
    )
    .await
}

#[tauri::command]
pub async fn admin_list_audit_events(
    event_type: Option<String>,
    actor: Option<String>,
    limit: Option<u32>,
) -> Result<AdminAuditEventListData, AdminCommandError> {
    let mut query = vec![];
    if let Some(v) = event_type.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        query.push(("event_type", v.to_string()));
    }
    if let Some(v) = actor.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        query.push(("actor", v.to_string()));
    }
    if let Some(v) = limit {
        query.push(("limit", v.to_string()));
    }
    let query_string = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{}={}", k, encode_query_component(v)))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let path = format!("/v1/admin/audit-events{query_string}");
    send_admin_request::<AdminAuditEventListData, ()>(
        AdminAction::AuditEvents,
        HttpMethod::Get,
        &path,
        None,
        None,
    )
    .await
}

#[tauri::command]
pub async fn admin_list_idempotency_records(
    op: Option<String>,
    limit: Option<u32>,
) -> Result<AdminIdempotencyRecordListData, AdminCommandError> {
    let mut query = vec![];
    if let Some(v) = op.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        query.push(("op", v.to_string()));
    }
    if let Some(v) = limit {
        query.push(("limit", v.to_string()));
    }
    let query_string = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{}={}", k, encode_query_component(v)))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let path = format!("/v1/admin/idempotency-records{query_string}");
    send_admin_request::<AdminIdempotencyRecordListData, ()>(
        AdminAction::IdempotencyRecords,
        HttpMethod::Get,
        &path,
        None,
        None,
    )
    .await
}

#[tauri::command]
pub async fn admin_list_reset_requests(
    status: ResetRequestStatus,
) -> Result<AdminResetListData, AdminCommandError> {
    let path = format!("/v1/admin/reset/requests?status={}", status.as_str());
    send_admin_request::<AdminResetListData, ()>(
        AdminAction::List,
        HttpMethod::Get,
        &path,
        None,
        None,
    )
    .await
}

#[tauri::command]
pub async fn admin_list_deletion_requests(
    status: DeletionRequestStatus,
) -> Result<AdminDeletionListData, AdminCommandError> {
    let path = format!("/v1/admin/privacy/delete-requests?status={}", status.as_str());
    send_admin_request::<AdminDeletionListData, ()>(
        AdminAction::ListDeletionRequests,
        HttpMethod::Get,
        &path,
        None,
        None,
    )
    .await
}

#[tauri::command]
pub async fn admin_approve_reset_request(
    request_id: String,
    reason: Option<String>,
) -> Result<AdminResetDecisionData, AdminCommandError> {
    let idempotency_key = generate_admin_idempotency_key("approve", &request_id);
    let reason = reason
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let body = AdminDecisionRequest {
        request_id: request_id.as_str(),
        reason,
    };
    send_admin_request(
        AdminAction::Approve,
        HttpMethod::Post,
        "/v1/admin/reset/approve",
        Some(&body),
        Some(&idempotency_key),
    )
    .await
}

#[tauri::command]
pub async fn admin_approve_deletion_request(
    request_id: String,
    confirmation: String,
    reason: Option<String>,
) -> Result<AdminDeletionDecisionData, AdminCommandError> {
    let normalized_confirmation = confirmation.trim();
    if normalized_confirmation != "DELETE USER DATA" {
        return Err(command_error(
            "bad_request",
            "Deletion approval requires DELETE USER DATA confirmation.",
            None,
            false,
        ));
    }
    let idempotency_key = generate_admin_idempotency_key("approve_deletion", &request_id);
    let reason = validate_support_reason(reason.as_deref())?;
    let body = AdminDeletionApproveRequest {
        request_id: request_id.as_str(),
        confirmation: normalized_confirmation,
        reason: Some(reason),
    };
    send_admin_request(
        AdminAction::ApproveDeletionRequest,
        HttpMethod::Post,
        "/v1/admin/privacy/delete/approve",
        Some(&body),
        Some(&idempotency_key),
    )
    .await
}

#[tauri::command]
pub async fn admin_reject_deletion_request(
    request_id: String,
    reason: Option<String>,
) -> Result<AdminDeletionDecisionData, AdminCommandError> {
    let idempotency_key = generate_admin_idempotency_key("reject_deletion", &request_id);
    let reason = validate_support_reason(reason.as_deref())?;
    let body = AdminDecisionRequest {
        request_id: request_id.as_str(),
        reason: Some(reason),
    };
    send_admin_request(
        AdminAction::RejectDeletionRequest,
        HttpMethod::Post,
        "/v1/admin/privacy/delete/reject",
        Some(&body),
        Some(&idempotency_key),
    )
    .await
}

#[tauri::command]
pub async fn admin_reject_reset_request(
    request_id: String,
    reason: Option<String>,
) -> Result<AdminResetDecisionData, AdminCommandError> {
    let idempotency_key = generate_admin_idempotency_key("reject", &request_id);
    let reason = reason
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let body = AdminDecisionRequest {
        request_id: request_id.as_str(),
        reason,
    };
    send_admin_request(
        AdminAction::Reject,
        HttpMethod::Post,
        "/v1/admin/reset/reject",
        Some(&body),
        Some(&idempotency_key),
    )
    .await
}

#[tauri::command]
pub async fn admin_disable_license(
    license_hash_prefix: String,
    reason: String,
    deactivate_bindings: bool,
) -> Result<AdminDisableLicenseData, AdminCommandError> {
    let normalized_prefix = license_hash_prefix.trim();
    let normalized_reason = reason.trim();
    if normalized_prefix.is_empty() || normalized_reason.is_empty() {
        return Err(command_error(
            "bad_request",
            "License hash prefix and reason are required.",
            None,
            false,
        ));
    }
    let idempotency_key = generate_admin_idempotency_key("disable_license", normalized_prefix);
    let body = AdminDisableLicenseRequest {
        license_hash_prefix: normalized_prefix,
        reason: normalized_reason,
        deactivate_bindings,
    };
    send_admin_request(
        AdminAction::DisableLicense,
        HttpMethod::Post,
        "/v1/admin/licenses/disable",
        Some(&body),
        Some(&idempotency_key),
    )
    .await
}

fn encode_query_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn admin_normalizes_base_url() {
        assert_eq!(
            normalize_worker_base_url(" https://worker.example.test/// ").unwrap(),
            "https://worker.example.test"
        );
    }

    #[test]
    fn admin_rejects_invalid_base_url() {
        assert!(normalize_worker_base_url("file:///tmp/token").is_err());
    }

    #[test]
    fn admin_redacts_token_without_exposing_full_value() {
        let redacted = redact_token("admin-token-secret-1234");
        assert_eq!(redacted, "[redacted]...1234");
        assert!(!redacted.contains("admin-token-secret"));
    }

    #[test]
    fn admin_idempotency_keys_are_unique_per_action() {
        let first = generate_admin_idempotency_key("approve", "reset-1");
        let second = generate_admin_idempotency_key("approve", "reset-1");
        assert!(first.starts_with("admin_"));
        assert_ne!(first, second);
    }

    #[test]
    fn admin_support_reason_is_required_for_deletion_decisions() {
        assert_eq!(validate_support_reason(Some(" privacy request ")).unwrap(), "privacy request");
        assert_eq!(validate_support_reason(Some("")).unwrap_err().code, "bad_request");
        assert_eq!(validate_support_reason(None).unwrap_err().code, "bad_request");
    }

    #[test]
    fn admin_parses_worker_success_envelope() {
        let body = r#"{
          "ok": true,
          "data": {
            "requests": [{
              "reset_request_id": "reset-1",
              "status": "pending",
              "license_state": "BOUND_ACTIVE",
              "message": "pending",
              "masked_license_key": "••••-1234",
              "has_license_hash": true,
              "purchaser_email": "b***@example.com",
              "created_at_ms": 1,
              "updated_at_ms": 2
            }]
          }
        }"#;
        let parsed: AdminResetListData = parse_worker_response(
            AdminAction::List,
            "/v1/admin/reset/requests?status=pending",
            StatusCode::OK,
            body,
        )
        .unwrap();
        assert_eq!(parsed.requests[0].reset_request_id, "reset-1");
    }

    #[test]
    fn admin_parses_worker_error_envelope() {
        let body = r#"{
          "ok": false,
          "error": {
            "code": "invalid_transition",
            "message": "Reset request has already been decided.",
            "request_id": "req_123",
            "retryable": false
          }
        }"#;
        let parsed = parse_worker_response::<AdminResetDecisionData>(
            AdminAction::Approve,
            "/v1/admin/reset/approve",
            StatusCode::CONFLICT,
            body,
        )
        .unwrap_err();
        assert_eq!(parsed.code, "invalid_transition");
        assert_eq!(parsed.request_id.as_deref(), Some("req_123"));
    }

    #[test]
    fn admin_parses_disable_license_success_envelope() {
        let body = r#"{
          "ok": true,
          "data": {
            "license_hash_prefix": "abc123",
            "entitlement_status": "disabled",
            "deactivate_bindings": true
          }
        }"#;
        let parsed: AdminDisableLicenseData = parse_worker_response(
            AdminAction::DisableLicense,
            "/v1/admin/licenses/disable",
            StatusCode::OK,
            body,
        )
        .unwrap();
        assert_eq!(parsed.license_hash_prefix, "abc123");
        assert_eq!(parsed.entitlement_status, "disabled");
        assert!(parsed.deactivate_bindings);
    }
}

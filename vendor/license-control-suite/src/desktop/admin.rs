//! Curated desktop-only admin console facade.
//!
//! This surface exposes the existing admin domain as a desktop-only admin
//! console boundary. It is intentionally separate from the six end-user/client
//! licensing commands and is not a web dashboard target.

pub use crate::modules::admin_dashboard::{
    adapters, auth, authz, compatibility, ops, queue, realtime,
};

#[cfg(feature = "desktop-tauri")]
use crate::modules::admin_dashboard::adapters::AdminApi;
#[cfg(feature = "desktop-tauri")]
use crate::modules::shared_contracts::{
    dto::{
        AdminAuthChallengeRequest, AdminAuthChallengeResponse, AdminAuthVerifyRequest,
        AdminAuthVerifyResponse, AdminResetDecisionResponse, DeviceResetStatusResponse,
    },
    errors::{ApiError, ErrorCode},
};
#[cfg(feature = "desktop-tauri")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[cfg(feature = "desktop-tauri")]
pub struct WorkerAdminApi {
    base_url: String,
    admin_access_token: String,
    client: reqwest::Client,
}

#[cfg(feature = "desktop-tauri")]
impl WorkerAdminApi {
    pub fn new(
        base_url: impl Into<String>,
        admin_access_token: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Err(api_error(
                ErrorCode::AdminAuthInvalid,
                "admin worker base URL is required",
            ));
        }
        let admin_access_token = admin_access_token.into();
        if admin_access_token.trim().is_empty() {
            return Err(api_error(
                ErrorCode::AdminAuthInvalid,
                "admin bearer token is required",
            ));
        }
        Ok(Self {
            base_url,
            admin_access_token,
            client: reqwest::Client::new(),
        })
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.admin_access_token)
            .send()
            .await
            .map_err(|_| api_error(ErrorCode::AdminForbidden, "admin worker request failed"))?;
        parse_worker_response(response).await
    }

    async fn post_json<T, B>(
        &self,
        path: &str,
        body: &B,
        idempotency_key: &str,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.admin_access_token)
            .header("x-idempotency-key", idempotency_key)
            .json(body)
            .send()
            .await
            .map_err(|_| api_error(ErrorCode::AdminForbidden, "admin worker request failed"))?;
        parse_worker_response(response).await
    }
}

#[cfg(feature = "desktop-tauri")]
impl AdminApi for WorkerAdminApi {
    fn get_challenge(
        &self,
        _req: &AdminAuthChallengeRequest,
    ) -> Result<AdminAuthChallengeResponse, ApiError> {
        Err(api_error(
            ErrorCode::AdminAuthInvalid,
            "bearer-token admin API does not use challenge login",
        ))
    }

    fn verify_challenge(
        &self,
        _req: &AdminAuthVerifyRequest,
    ) -> Result<AdminAuthVerifyResponse, ApiError> {
        Err(api_error(
            ErrorCode::AdminAuthInvalid,
            "bearer-token admin API does not use challenge login",
        ))
    }

    fn list_pending_resets(&self) -> Result<Vec<DeviceResetStatusResponse>, ApiError> {
        let response: AdminResetListResponse = tauri::async_runtime::block_on(
            self.get_json("/v1/admin/reset/requests?status=pending"),
        )?;
        Ok(response.requests)
    }

    fn approve_reset(
        &self,
        reset_request_id: &str,
        idempotency_key: &str,
    ) -> Result<AdminResetDecisionResponse, ApiError> {
        tauri::async_runtime::block_on(self.post_json(
            "/v1/admin/reset/approve",
            &AdminResetDecisionRequest {
                request_id: reset_request_id,
                reason: None,
            },
            idempotency_key,
        ))
    }

    fn reject_reset(
        &self,
        reset_request_id: &str,
        idempotency_key: &str,
    ) -> Result<AdminResetDecisionResponse, ApiError> {
        tauri::async_runtime::block_on(self.post_json(
            "/v1/admin/reset/reject",
            &AdminResetDecisionRequest {
                request_id: reset_request_id,
                reason: None,
            },
            idempotency_key,
        ))
    }
}

#[cfg(feature = "desktop-tauri")]
#[derive(Deserialize)]
struct WorkerEnvelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<WorkerError>,
}

#[cfg(feature = "desktop-tauri")]
#[derive(Deserialize)]
struct WorkerError {
    message: String,
    retryable: bool,
    request_id: String,
}

#[cfg(feature = "desktop-tauri")]
#[derive(Deserialize)]
struct AdminResetListResponse {
    requests: Vec<DeviceResetStatusResponse>,
}

#[cfg(feature = "desktop-tauri")]
#[derive(Serialize)]
struct AdminResetDecisionRequest<'a> {
    request_id: &'a str,
    reason: Option<&'a str>,
}

#[cfg(feature = "desktop-tauri")]
async fn parse_worker_response<T>(response: reqwest::Response) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|_| api_error(ErrorCode::AdminForbidden, "admin worker response failed"))?;
    let envelope: WorkerEnvelope<T> = serde_json::from_str(&body).map_err(|_| {
        api_error(
            ErrorCode::AdminForbidden,
            "admin worker response was invalid",
        )
    })?;
    if status.is_success() && envelope.ok {
        return envelope.data.ok_or_else(|| {
            api_error(
                ErrorCode::AdminForbidden,
                "admin worker response missing data",
            )
        });
    }
    let error = envelope.error;
    Err(ApiError::new(
        ErrorCode::AdminForbidden,
        error
            .as_ref()
            .map(|value| value.message.as_str())
            .unwrap_or("admin worker request was rejected"),
        error.as_ref().map(|value| value.retryable).unwrap_or(false),
        error
            .as_ref()
            .map(|value| value.request_id.as_str())
            .unwrap_or("admin-worker"),
    ))
}

#[cfg(feature = "desktop-tauri")]
fn api_error(code: ErrorCode, message: impl Into<String>) -> ApiError {
    ApiError::new(code, message, false, "admin-worker")
}

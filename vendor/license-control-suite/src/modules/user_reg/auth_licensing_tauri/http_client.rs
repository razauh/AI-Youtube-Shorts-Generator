use async_trait::async_trait;
use crate::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationOutcome, ActivationRequest, AuthError, BoundDeviceSummary,
    DeviceFingerprint, DeviceId, DevicePublicKey, DeviceResetRequest, DeviceResetStatus,
    EntitlementStatus, MaskedLicenseKey, ResetRequestId, ValidationOutcome, WorkerClient,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpWorkerClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpWorkerClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, AuthError> {
        Self::with_timeout(base_url, Duration::from_secs(10))
    }

    pub fn with_timeout(
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, AuthError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|err| AuthError::Storage(err.to_string()))?;
        Ok(Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client,
        })
    }

    async fn post_json<T, R>(&self, path: &str, body: &T) -> Result<R, AuthError>
    where
        T: serde::Serialize + ?Sized,
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .send()
            .await
            .map_err(|_| AuthError::WorkerUnreachable)?;
        parse_response(response).await
    }

    async fn get_json<R>(&self, path: &str) -> Result<R, AuthError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|_| AuthError::WorkerUnreachable)?;
        parse_response(response).await
    }
}

async fn parse_response<R>(response: reqwest::Response) -> Result<R, AuthError>
where
    R: for<'de> serde::Deserialize<'de>,
{
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|_| AuthError::WorkerUnreachable)?;
    if status.is_success() {
        return serde_json::from_str::<R>(&body)
            .map_err(|err| AuthError::Serialization(err.to_string()));
    }
    if let Ok(error) = serde_json::from_str::<WorkerErrorBody>(&body) {
        return Err(error.into_auth_error(status));
    }
    match status {
        StatusCode::BAD_REQUEST => Err(AuthError::InvalidResetRequest),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AuthError::Unauthorized),
        StatusCode::NOT_FOUND => Err(AuthError::ResetRequestNotFound),
        StatusCode::CONFLICT => Err(AuthError::DeviceAlreadyBound),
        StatusCode::GONE => Err(AuthError::ReauthRequired),
        _ => Err(AuthError::WorkerUnreachable),
    }
}

#[derive(Debug, Deserialize)]
struct WorkerErrorBody {
    code: Option<String>,
}

impl WorkerErrorBody {
    fn into_auth_error(self, fallback_status: StatusCode) -> AuthError {
        match self.code.as_deref() {
            Some("invalid_license_key") => AuthError::InvalidLicenseKey,
            Some("invalid_purchase_email") => AuthError::InvalidPurchaseEmail,
            Some("invalid_device_identity") => AuthError::InvalidDeviceIdentity,
            Some("invalid_reset_request") => AuthError::InvalidResetRequest,
            Some("device_already_bound") => AuthError::DeviceAlreadyBound,
            Some("reauth_required") | Some("token_expired") | Some("token_revoked") => {
                AuthError::ReauthRequired
            }
            Some("reset_request_not_found") => AuthError::ResetRequestNotFound,
            Some("unauthorized") => AuthError::Unauthorized,
            Some("worker_unreachable") => AuthError::WorkerUnreachable,
            _ => match fallback_status {
                StatusCode::BAD_REQUEST => AuthError::InvalidResetRequest,
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AuthError::Unauthorized,
                StatusCode::NOT_FOUND => AuthError::ResetRequestNotFound,
                StatusCode::CONFLICT => AuthError::DeviceAlreadyBound,
                StatusCode::GONE => AuthError::ReauthRequired,
                _ => AuthError::WorkerUnreachable,
            },
        }
    }
}

#[derive(Serialize)]
struct ActivationRequestBody {
    license_key: String,
    device_public_key: String,
    fingerprint: DeviceFingerprint,
    app_version: String,
    timestamp_ms: i64,
}

impl From<ActivationRequest> for ActivationRequestBody {
    fn from(value: ActivationRequest) -> Self {
        Self {
            license_key: value.license_key.expose_secret().to_string(),
            device_public_key: value.device_public_key.as_str().to_string(),
            fingerprint: value.fingerprint,
            app_version: value.app_version,
            timestamp_ms: value.timestamp_ms,
        }
    }
}

impl std::fmt::Debug for ActivationRequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivationRequestBody")
            .field("license_key", &"<redacted>")
            .field("device_public_key", &self.device_public_key)
            .field("fingerprint", &self.fingerprint)
            .field("app_version", &self.app_version)
            .field("timestamp_ms", &self.timestamp_ms)
            .finish()
    }
}

#[derive(Serialize)]
struct ValidationRequestBody {
    access_token: String,
}

impl From<AccessToken> for ValidationRequestBody {
    fn from(value: AccessToken) -> Self {
        Self {
            access_token: value.expose_secret().to_string(),
        }
    }
}

impl std::fmt::Debug for ValidationRequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationRequestBody")
            .field("access_token", &"<redacted>")
            .finish()
    }
}

#[derive(Serialize)]
struct DeviceResetRequestBody {
    license_key: Option<String>,
    masked_license_key: Option<String>,
    purchaser_email: String,
    device_public_key: String,
    fingerprint: DeviceFingerprint,
    app_version: String,
    timestamp_ms: i64,
    receipt_reference: Option<String>,
}

impl From<DeviceResetRequest> for DeviceResetRequestBody {
    fn from(value: DeviceResetRequest) -> Self {
        Self {
            license_key: value
                .license_key
                .map(|license_key| license_key.expose_secret().to_string()),
            masked_license_key: value.masked_license_key.map(|key| key.as_str().to_string()),
            purchaser_email: value.purchaser_email.as_str().to_string(),
            device_public_key: value.device_public_key.as_str().to_string(),
            fingerprint: value.fingerprint,
            app_version: value.app_version,
            timestamp_ms: value.timestamp_ms,
            receipt_reference: value.receipt_reference,
        }
    }
}

impl std::fmt::Debug for DeviceResetRequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceResetRequestBody")
            .field(
                "license_key",
                &self.license_key.as_ref().map(|_| "<redacted>"),
            )
            .field("masked_license_key", &self.masked_license_key)
            .field("purchaser_email", &self.purchaser_email)
            .field("device_public_key", &self.device_public_key)
            .field("fingerprint", &self.fingerprint)
            .field("app_version", &self.app_version)
            .field("timestamp_ms", &self.timestamp_ms)
            .field("receipt_reference", &self.receipt_reference)
            .finish()
    }
}

#[derive(Deserialize)]
struct ActivationResponseBody {
    access_token: String,
    masked_license_key: String,
    bound_device: BoundDeviceBody,
    entitlement: EntitlementStatusBody,
    token_expires_at_ms: i64,
}

impl TryFrom<ActivationResponseBody> for ActivationOutcome {
    type Error = AuthError;

    fn try_from(value: ActivationResponseBody) -> Result<Self, Self::Error> {
        Ok(Self {
            access_token: AccessToken::new(value.access_token)?,
            masked_license_key: MaskedLicenseKey::new(value.masked_license_key)?,
            bound_device: value.bound_device.try_into()?,
            entitlement: value.entitlement.into(),
            token_expires_at_ms: value.token_expires_at_ms,
        })
    }
}

#[derive(Deserialize)]
struct BoundDeviceBody {
    device_id: String,
    public_key: String,
    fingerprint: DeviceFingerprint,
}

impl TryFrom<BoundDeviceBody> for BoundDeviceSummary {
    type Error = AuthError;

    fn try_from(value: BoundDeviceBody) -> Result<Self, Self::Error> {
        Ok(Self {
            device_id: DeviceId::new(value.device_id)?,
            public_key: DevicePublicKey::new(value.public_key)?,
            fingerprint: value.fingerprint,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntitlementStatusBody {
    Active,
    Disabled,
    Refunded,
}

impl From<EntitlementStatusBody> for EntitlementStatus {
    fn from(value: EntitlementStatusBody) -> Self {
        match value {
            EntitlementStatusBody::Active => Self::Active,
            EntitlementStatusBody::Disabled => Self::Disabled,
            EntitlementStatusBody::Refunded => Self::Refunded,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ValidationResponseBody {
    Active {
        masked_license_key: String,
        bound_device: BoundDeviceBody,
        token_expires_at_ms: i64,
    },
    ReauthRequired,
    Revoked,
}

impl TryFrom<ValidationResponseBody> for ValidationOutcome {
    type Error = AuthError;

    fn try_from(value: ValidationResponseBody) -> Result<Self, Self::Error> {
        match value {
            ValidationResponseBody::Active {
                masked_license_key,
                bound_device,
                token_expires_at_ms,
            } => Ok(Self::Active {
                masked_license_key: MaskedLicenseKey::new(masked_license_key)?,
                bound_device: bound_device.try_into()?,
                token_expires_at_ms,
            }),
            ValidationResponseBody::ReauthRequired => Ok(Self::ReauthRequired),
            ValidationResponseBody::Revoked => Ok(Self::Revoked),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum DeviceResetStatusBody {
    Pending {
        request_id: String,
        created_at_ms: i64,
    },
    Approved {
        request_id: String,
        decided_at_ms: i64,
    },
    Rejected {
        request_id: String,
        decided_at_ms: i64,
        reason: Option<String>,
    },
    Expired {
        request_id: String,
        expired_at_ms: i64,
    },
    NotFound {
        request_id: String,
    },
}

impl TryFrom<DeviceResetStatusBody> for DeviceResetStatus {
    type Error = AuthError;

    fn try_from(value: DeviceResetStatusBody) -> Result<Self, Self::Error> {
        match value {
            DeviceResetStatusBody::Pending {
                request_id,
                created_at_ms,
            } => Ok(Self::Pending {
                request_id: ResetRequestId::new(request_id)?,
                created_at_ms,
            }),
            DeviceResetStatusBody::Approved {
                request_id,
                decided_at_ms,
            } => Ok(Self::Approved {
                request_id: ResetRequestId::new(request_id)?,
                decided_at_ms,
            }),
            DeviceResetStatusBody::Rejected {
                request_id,
                decided_at_ms,
                reason,
            } => Ok(Self::Rejected {
                request_id: ResetRequestId::new(request_id)?,
                decided_at_ms,
                reason,
            }),
            DeviceResetStatusBody::Expired {
                request_id,
                expired_at_ms,
            } => Ok(Self::Expired {
                request_id: ResetRequestId::new(request_id)?,
                expired_at_ms,
            }),
            DeviceResetStatusBody::NotFound { request_id } => Ok(Self::NotFound {
                request_id: ResetRequestId::new(request_id)?,
            }),
        }
    }
}

#[async_trait]
impl WorkerClient for HttpWorkerClient {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        let body = ActivationRequestBody::from(request);
        let response: ActivationResponseBody = self.post_json("/v1/activate", &body).await?;
        response.try_into()
    }

    async fn validate_session(&self, token: AccessToken) -> Result<ValidationOutcome, AuthError> {
        let body = ValidationRequestBody::from(token);
        let response: ValidationResponseBody =
            self.post_json("/v1/session/validate", &body).await?;
        response.try_into()
    }

    async fn request_device_reset(
        &self,
        request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        let body = DeviceResetRequestBody::from(request);
        let response: DeviceResetStatusBody =
            self.post_json("/v1/device-reset/request", &body).await?;
        response.try_into()
    }

    async fn get_device_reset_status(
        &self,
        request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        let response: DeviceResetStatusBody = self
            .get_json(&format!("/v1/device-reset/status/{}", request_id.as_str()))
            .await?;
        response.try_into()
    }
}

impl std::fmt::Debug for HttpWorkerClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpWorkerClient")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user_reg::auth_licensing_core::{
        DeviceFingerprint, DeviceId, DevicePublicKey, LicenseKey, PurchaseEmail,
    };
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn activation_request() -> ActivationRequest {
        ActivationRequest {
            license_key: LicenseKey::new("SECRET-LICENSE").unwrap(),
            device_public_key: DevicePublicKey::new("public").unwrap(),
            fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
            app_version: "1.0.0".into(),
            timestamp_ms: 10,
        }
    }

    fn activation_response_json() -> serde_json::Value {
        let public_key = DevicePublicKey::new("public").unwrap();
        let device_id = DeviceId::from_public_key(&public_key);
        json!({
            "access_token": "SECRET-TOKEN",
            "masked_license_key": "••••-ENSE",
            "bound_device": {
                "device_id": device_id.as_str(),
                "public_key": "public",
                "fingerprint": {
                    "platform": "linux",
                    "os": "linux",
                    "arch": "x86_64",
                    "hostname_hash": null
                }
            },
            "entitlement": "active",
            "token_expires_at_ms": 100
        })
    }

    #[tokio::test]
    async fn activation_uses_expected_method_path_and_parses_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/activate"))
            .and(body_json(json!({
                "license_key": "SECRET-LICENSE",
                "device_public_key": "public",
                "fingerprint": {
                    "platform": "linux",
                    "os": "linux",
                    "arch": "x86_64",
                    "hostname_hash": null
                },
                "app_version": "1.0.0",
                "timestamp_ms": 10
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(activation_response_json()))
            .mount(&server)
            .await;
        let client = HttpWorkerClient::new(server.uri()).unwrap();
        let result = client.activate(activation_request()).await.unwrap();
        assert_eq!(result.masked_license_key.as_str(), "••••-ENSE");
    }

    #[tokio::test]
    async fn activation_error_body_maps_without_echoing_worker_message() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/activate"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "code": "invalid_license_key",
                "message": "SECRET-LICENSE"
            })))
            .mount(&server)
            .await;
        let client = HttpWorkerClient::new(server.uri()).unwrap();
        let error = client.activate(activation_request()).await.unwrap_err();
        assert_eq!(error, AuthError::InvalidLicenseKey);
        assert!(!error.to_string().contains("SECRET-LICENSE"));
    }

    #[tokio::test]
    async fn activation_conflict_maps_to_device_bound() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/activate"))
            .respond_with(ResponseTemplate::new(409))
            .mount(&server)
            .await;
        let client = HttpWorkerClient::new(server.uri()).unwrap();
        assert_eq!(
            client.activate(activation_request()).await.unwrap_err(),
            AuthError::DeviceAlreadyBound
        );
    }

    #[tokio::test]
    async fn validation_gone_maps_to_reauth_required() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/session/validate"))
            .respond_with(ResponseTemplate::new(410))
            .mount(&server)
            .await;
        let client = HttpWorkerClient::new(server.uri()).unwrap();
        assert_eq!(
            client
                .validate_session(AccessToken::new("SECRET-TOKEN").unwrap())
                .await
                .unwrap_err(),
            AuthError::ReauthRequired
        );
    }

    #[tokio::test]
    async fn reset_request_uses_expected_path_and_parses_pending() {
        let server = MockServer::start().await;
        let request_id = ResetRequestId::new("reset-1").unwrap();
        Mock::given(method("POST"))
            .and(path("/v1/device-reset/request"))
            .and(body_json(json!({
                "license_key": "SECRET-LICENSE",
                "masked_license_key": null,
                "purchaser_email": "buyer@example.com",
                "device_public_key": "public",
                "fingerprint": {
                    "platform": "linux",
                    "os": "linux",
                    "arch": "x86_64",
                    "hostname_hash": null
                },
                "app_version": "1.0.0",
                "timestamp_ms": 10,
                "receipt_reference": null
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "pending",
                "request_id": "reset-1",
                "created_at_ms": 10
            })))
            .mount(&server)
            .await;
        let client = HttpWorkerClient::new(server.uri()).unwrap();
        let request = DeviceResetRequest {
            license_key: Some(LicenseKey::new("SECRET-LICENSE").unwrap()),
            masked_license_key: None,
            purchaser_email: PurchaseEmail::new("buyer@example.com").unwrap(),
            device_public_key: DevicePublicKey::new("public").unwrap(),
            fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
            app_version: "1.0.0".into(),
            timestamp_ms: 10,
            receipt_reference: None,
        };
        assert_eq!(
            client
                .request_device_reset(request)
                .await
                .unwrap()
                .request_id(),
            &request_id
        );
    }

    #[tokio::test]
    async fn reset_status_get_parses_approved() {
        let server = MockServer::start().await;
        let request_id = ResetRequestId::new("reset-1").unwrap();
        Mock::given(method("GET"))
            .and(path("/v1/device-reset/status/reset-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "approved",
                "request_id": "reset-1",
                "decided_at_ms": 10
            })))
            .mount(&server)
            .await;
        let client = HttpWorkerClient::new(server.uri()).unwrap();
        assert!(matches!(
            client.get_device_reset_status(request_id).await.unwrap(),
            DeviceResetStatus::Approved { .. }
        ));
    }

    #[test]
    fn http_worker_client_route_contracts_remain_unchanged() {
        let source = include_str!("http_client.rs");
        assert!(source.contains("\"/v1/activate\""));
        assert!(source.contains("\"/v1/session/validate\""));
        assert!(source.contains("\"/v1/device-reset/request\""));
        assert!(source.contains("\"/v1/device-reset/status/{}\""));
    }

    #[test]
    fn debug_output_does_not_contain_request_secrets() {
        let request = ActivationRequestBody::from(activation_request());
        let validation = ValidationRequestBody::from(AccessToken::new("SECRET-TOKEN").unwrap());
        assert!(!format!("{request:?}").contains("SECRET-LICENSE"));
        assert!(!format!("{validation:?}").contains("SECRET-TOKEN"));
    }
}

use crate::modules::user_reg::auth_licensing_core::{
    AuthError, AuthService, DeviceResetStatus, LicenseKey, PurchaseEmail, ResetRequestId,
    SessionState,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Clone)]
pub struct AuthAppState {
    pub service: Arc<AuthService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationView {
    pub auth_state: AuthStateView,
    pub masked_license_key: String,
    pub entitlement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionView {
    pub auth_state: AuthStateView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceResetInput {
    pub purchaser_email: String,
    pub receipt_reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceResetView {
    pub request_id: String,
    pub status: String,
    pub message: Option<String>,
    pub auth_state: AuthStateView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AuthStateView {
    Unauthenticated,
    Licensed {
        masked_license_key: String,
        device_id: String,
        token_expires_at_ms: i64,
    },
    ReauthRequired {
        masked_license_key: Option<String>,
    },
    ResetPending {
        request_id: String,
        masked_license_key: Option<String>,
    },
    ResetApprovedUnbound {
        request_id: String,
        masked_license_key: Option<String>,
        message: String,
    },
    ResetRejected {
        request_id: String,
        masked_license_key: Option<String>,
    },
    ResetExpired {
        request_id: String,
        masked_license_key: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthCommandError {
    pub code: String,
    pub message: String,
}

impl From<AuthError> for AuthCommandError {
    fn from(value: AuthError) -> Self {
        Self {
            code: value.code().to_string(),
            message: value.to_string(),
        }
    }
}

#[tauri::command]
pub async fn activate_license(
    license_key: String,
    state: State<'_, AuthAppState>,
) -> Result<ActivationView, AuthCommandError> {
    activate_license_with_service(license_key, &state.service).await
}

pub async fn activate_license_with_service(
    license_key: String,
    service: &AuthService,
) -> Result<ActivationView, AuthCommandError> {
    let outcome = service
        .activate_license(LicenseKey::new(license_key)?)
        .await?;
    Ok(ActivationView {
        auth_state: AuthStateView::from(SessionState::after_activation(
            outcome.masked_license_key.clone(),
            outcome.bound_device,
            outcome.token_expires_at_ms,
        )),
        masked_license_key: outcome.masked_license_key.to_string(),
        entitlement: format!("{:?}", outcome.entitlement).to_ascii_lowercase(),
    })
}

#[tauri::command]
pub async fn validate_session(
    state: State<'_, AuthAppState>,
) -> Result<SessionView, AuthCommandError> {
    validate_session_with_service(&state.service).await
}

pub async fn validate_session_with_service(
    service: &AuthService,
) -> Result<SessionView, AuthCommandError> {
    let state = service.validate_session().await?;
    Ok(SessionView {
        auth_state: AuthStateView::from(state),
    })
}

#[tauri::command]
pub async fn request_device_reset(
    input: DeviceResetInput,
    state: State<'_, AuthAppState>,
) -> Result<DeviceResetView, AuthCommandError> {
    request_device_reset_with_service(input, &state.service).await
}

pub async fn request_device_reset_with_service(
    input: DeviceResetInput,
    service: &AuthService,
) -> Result<DeviceResetView, AuthCommandError> {
    let status = service
        .request_device_reset(
            PurchaseEmail::new(input.purchaser_email)?,
            input.receipt_reference,
        )
        .await?;
    reset_view(status, service).await
}

#[tauri::command]
pub async fn get_device_reset_status(
    request_id: String,
    state: State<'_, AuthAppState>,
) -> Result<DeviceResetView, AuthCommandError> {
    get_device_reset_status_with_service(request_id, &state.service).await
}

pub async fn get_device_reset_status_with_service(
    request_id: String,
    service: &AuthService,
) -> Result<DeviceResetView, AuthCommandError> {
    let status = service
        .get_device_reset_status(ResetRequestId::new(request_id)?)
        .await?;
    reset_view(status, service).await
}

#[tauri::command]
pub async fn clear_local_session(state: State<'_, AuthAppState>) -> Result<(), AuthCommandError> {
    clear_local_session_with_service(&state.service).await
}

pub async fn clear_local_session_with_service(
    service: &AuthService,
) -> Result<(), AuthCommandError> {
    service.clear_local_session().await.map_err(Into::into)
}

#[tauri::command]
pub async fn get_auth_state(
    state: State<'_, AuthAppState>,
) -> Result<AuthStateView, AuthCommandError> {
    get_auth_state_with_service(&state.service).await
}

pub async fn get_auth_state_with_service(
    service: &AuthService,
) -> Result<AuthStateView, AuthCommandError> {
    Ok(AuthStateView::from(service.get_auth_state().await?))
}

pub fn command_names() -> &'static [&'static str] {
    &[
        "activate_license",
        "validate_session",
        "request_device_reset",
        "get_device_reset_status",
        "clear_local_session",
        "get_auth_state",
    ]
}

/// Returns an invoke handler for the auth command set only.
///
/// Host apps that need a single final `invoke_handler` should prefer using the
/// public command functions directly inside their own `tauri::generate_handler!`
/// invocation so they can compose crate commands with app-specific commands.
pub fn auth_command_handler<R>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: tauri::Runtime,
{
    tauri::generate_handler![
        activate_license,
        validate_session,
        request_device_reset,
        get_device_reset_status,
        clear_local_session,
        get_auth_state
    ]
}

/// Compatibility alias for the auth-only command invoke handler.
pub fn command_handler<R>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: tauri::Runtime,
{
    auth_command_handler::<R>()
}

/// Convenience helper for apps that only want to register this crate's auth
/// commands. Host apps with additional commands should compose their own final
/// invoke handler with the public command functions instead.
pub fn register_auth_commands<R>(builder: tauri::Builder<R>) -> tauri::Builder<R>
where
    R: tauri::Runtime,
{
    builder.invoke_handler(auth_command_handler::<R>())
}

async fn reset_view(
    status: DeviceResetStatus,
    service: &AuthService,
) -> Result<DeviceResetView, AuthCommandError> {
    let request_id = status.request_id().as_str().to_string();
    let (status_text, message) = match &status {
        DeviceResetStatus::Pending { .. } => ("pending", None),
        DeviceResetStatus::Approved { .. } => (
            "approved",
            Some(DeviceResetStatus::approved_message().to_string()),
        ),
        DeviceResetStatus::Rejected { .. } => ("rejected", None),
        DeviceResetStatus::Expired { .. } => ("expired", None),
        DeviceResetStatus::NotFound { .. } => ("not_found", None),
    };
    Ok(DeviceResetView {
        request_id,
        status: status_text.to_string(),
        message,
        auth_state: AuthStateView::from(service.get_auth_state().await?),
    })
}

impl From<SessionState> for AuthStateView {
    fn from(value: SessionState) -> Self {
        match value {
            SessionState::Unauthenticated => Self::Unauthenticated,
            SessionState::Licensed {
                masked_license_key,
                bound_device,
                token_expires_at_ms,
            } => Self::Licensed {
                masked_license_key: masked_license_key.to_string(),
                device_id: bound_device.device_id.as_str().to_string(),
                token_expires_at_ms,
            },
            SessionState::ReauthRequired { masked_license_key } => Self::ReauthRequired {
                masked_license_key: masked_license_key.map(|v| v.to_string()),
            },
            SessionState::ResetPending {
                request_id,
                masked_license_key,
            } => Self::ResetPending {
                request_id: request_id.as_str().to_string(),
                masked_license_key: masked_license_key.map(|v| v.to_string()),
            },
            SessionState::ResetApprovedUnbound {
                request_id,
                masked_license_key,
            } => Self::ResetApprovedUnbound {
                request_id: request_id.as_str().to_string(),
                masked_license_key: masked_license_key.map(|v| v.to_string()),
                message: DeviceResetStatus::approved_message().to_string(),
            },
            SessionState::ResetRejected {
                request_id,
                masked_license_key,
            } => Self::ResetRejected {
                request_id: request_id.as_str().to_string(),
                masked_license_key: masked_license_key.map(|v| v.to_string()),
            },
            SessionState::ResetExpired {
                request_id,
                masked_license_key,
            } => Self::ResetExpired {
                request_id: request_id.as_str().to_string(),
                masked_license_key: masked_license_key.map(|v| v.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user_reg::auth_licensing_core::test_support::{FakeWorkerClient, TestService};
    use crate::modules::user_reg::auth_licensing_core::{
        AccessToken, ActivationOutcome, BoundDeviceSummary, DeviceFingerprint, DeviceId,
        DevicePublicKey, EntitlementStatus, MaskedLicenseKey,
    };

    fn outcome() -> ActivationOutcome {
        let public_key = DevicePublicKey::new("public").unwrap();
        ActivationOutcome {
            access_token: AccessToken::new("secret-token").unwrap(),
            masked_license_key: MaskedLicenseKey::new("••••-1234").unwrap(),
            bound_device: BoundDeviceSummary {
                device_id: DeviceId::from_public_key(&public_key),
                public_key,
                fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
            },
            entitlement: EntitlementStatus::Active,
            token_expires_at_ms: 100,
        }
    }

    #[tokio::test]
    async fn activate_license_returns_frontend_safe_view() {
        let harness = TestService::new(FakeWorkerClient::new().with_activation(Ok(outcome())));
        let view = activate_license_with_service("LICENSE-1234".into(), &harness.service)
            .await
            .unwrap();
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("••••-1234"));
        assert!(!json.contains("LICENSE-1234"));
        assert!(!json.contains("secret-token"));
    }

    #[tokio::test]
    async fn approved_reset_maps_to_unbound_frontend_state() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let harness = TestService::new(FakeWorkerClient::new().with_reset_status(Ok(
            DeviceResetStatus::Approved {
                request_id,
                decided_at_ms: 10,
            },
        )));
        let view = get_device_reset_status_with_service("reset-1".into(), &harness.service)
            .await
            .unwrap();
        assert_eq!(view.status, "approved");
        assert!(matches!(
            view.auth_state,
            AuthStateView::ResetApprovedUnbound { .. }
        ));
    }

    #[test]
    fn command_errors_serialize_predictably() {
        let error = AuthCommandError::from(AuthError::WorkerUnreachable);
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("worker_unreachable"));
    }

    #[test]
    fn every_launch_command_is_registered_by_name() {
        assert_eq!(
            command_names(),
            &[
                "activate_license",
                "validate_session",
                "request_device_reset",
                "get_device_reset_status",
                "clear_local_session",
                "get_auth_state"
            ]
        );
    }

    #[test]
    fn command_handler_is_constructible_for_tauri_builder_registration() {
        let _handler = command_handler::<tauri::Wry>();
    }
}

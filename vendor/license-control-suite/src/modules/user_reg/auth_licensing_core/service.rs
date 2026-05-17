use super::{
    domain::{
        ActivationOutcome, ActivationRequest, AuthError, LicenseKey, MaskedLicenseKey,
        DeviceResetRequest, PurchaseEmail, ResetRequestId, ValidationOutcome,
    },
    state::{DeviceResetStatus, SessionState},
    traits::{Clock, DeviceIdentityProvider, LocalStateStore, SecretStore, WorkerClient},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthService {
    worker: Arc<dyn WorkerClient>,
    secrets: Arc<dyn SecretStore>,
    state: Arc<dyn LocalStateStore>,
    identity: Arc<dyn DeviceIdentityProvider>,
    clock: Arc<dyn Clock>,
    app_version: String,
}

impl AuthService {
    pub fn new(
        worker: Arc<dyn WorkerClient>,
        secrets: Arc<dyn SecretStore>,
        state: Arc<dyn LocalStateStore>,
        identity: Arc<dyn DeviceIdentityProvider>,
        clock: Arc<dyn Clock>,
        app_version: impl Into<String>,
    ) -> Self {
        Self {
            worker,
            secrets,
            state,
            identity,
            clock,
            app_version: app_version.into(),
        }
    }

    pub async fn activate_license(
        &self,
        license_key: LicenseKey,
    ) -> Result<ActivationOutcome, AuthError> {
        let keypair = self.identity.get_or_create_keypair().await?;
        let fingerprint = self.identity.collect_fingerprint().await?;
        let request = ActivationRequest {
            license_key: license_key.clone(),
            device_public_key: keypair.public_key().clone(),
            fingerprint,
            app_version: self.app_version.clone(),
            timestamp_ms: self.clock.now_ms(),
        };
        let outcome = self.worker.activate(request).await?;

        self.secrets.put_device_keypair(keypair).await?;
        self.secrets.put_license_key(license_key).await?;
        self.secrets
            .put_access_token(outcome.access_token.clone())
            .await?;
        self.state
            .save_session_state(SessionState::after_activation(
                outcome.masked_license_key.clone(),
                outcome.bound_device.clone(),
                outcome.token_expires_at_ms,
            ))
            .await?;

        Ok(outcome)
    }

    pub async fn validate_session(&self) -> Result<SessionState, AuthError> {
        let Some(token) = self.secrets.get_access_token().await? else {
            let current = self.state.load_session_state().await?;
            let masked = masked_from_state(&current);
            let reauth = SessionState::require_reauth(masked);
            self.state.save_session_state(reauth.clone()).await?;
            return Ok(reauth);
        };

        match self.worker.validate_session(token).await {
            Ok(ValidationOutcome::Active {
                masked_license_key,
                bound_device,
                token_expires_at_ms,
            }) => {
                let next = SessionState::after_activation(
                    masked_license_key,
                    bound_device,
                    token_expires_at_ms,
                );
                self.state.save_session_state(next.clone()).await?;
                Ok(next)
            }
            Ok(ValidationOutcome::ReauthRequired | ValidationOutcome::Revoked) => {
                self.secrets.clear_session_secrets().await?;
                let current = self.state.load_session_state().await?;
                let next = SessionState::require_reauth(masked_from_state(&current));
                self.state.save_session_state(next.clone()).await?;
                Ok(next)
            }
            Err(AuthError::WorkerUnreachable) => Err(AuthError::WorkerUnreachable),
            Err(err) => Err(err),
        }
    }

    pub async fn request_device_reset(
        &self,
        purchaser_email: PurchaseEmail,
        receipt_reference: Option<String>,
    ) -> Result<DeviceResetStatus, AuthError> {
        let keypair = self.identity.get_or_create_keypair().await?;
        let fingerprint = self.identity.collect_fingerprint().await?;
        let license_key = self.secrets.get_license_key().await?;
        let masked_license_key = match &license_key {
            Some(key) => Some(key.masked()),
            None => masked_from_state(&self.state.load_session_state().await?),
        };
        if license_key.is_none() && masked_license_key.is_none() {
            return Err(AuthError::InvalidResetRequest);
        }

        let request = DeviceResetRequest {
            license_key,
            masked_license_key,
            purchaser_email,
            device_public_key: keypair.public_key().clone(),
            fingerprint,
            app_version: self.app_version.clone(),
            timestamp_ms: self.clock.now_ms(),
            receipt_reference,
        };
        let status = self.worker.request_device_reset(request).await?;
        self.state.save_reset_status(status.clone()).await?;
        let current = self.state.load_session_state().await?;
        let next = SessionState::after_reset_status(&status, masked_from_state(&current))?;
        self.state.save_session_state(next).await?;
        Ok(status)
    }

    pub async fn get_device_reset_status(
        &self,
        request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        let status = self.worker.get_device_reset_status(request_id).await?;
        if matches!(status, DeviceResetStatus::NotFound { .. }) {
            return Err(AuthError::ResetRequestNotFound);
        }
        self.state.save_reset_status(status.clone()).await?;
        let current = self.state.load_session_state().await?;
        let next = SessionState::after_reset_status(&status, masked_from_state(&current))?;
        if matches!(status, DeviceResetStatus::Approved { .. }) {
            self.secrets.clear_session_secrets().await?;
        }
        self.state.save_session_state(next).await?;
        Ok(status)
    }

    pub async fn clear_local_session(&self) -> Result<(), AuthError> {
        self.secrets.clear_session_secrets().await?;
        self.state
            .save_session_state(SessionState::Unauthenticated)
            .await
    }

    pub async fn get_auth_state(&self) -> Result<SessionState, AuthError> {
        self.state.load_session_state().await
    }
}

fn masked_from_state(state: &SessionState) -> Option<MaskedLicenseKey> {
    match state {
        SessionState::Licensed {
            masked_license_key, ..
        }
        | SessionState::ReauthRequired {
            masked_license_key: Some(masked_license_key),
        }
        | SessionState::ResetPending {
            masked_license_key: Some(masked_license_key),
            ..
        }
        | SessionState::ResetApprovedUnbound {
            masked_license_key: Some(masked_license_key),
            ..
        }
        | SessionState::ResetRejected {
            masked_license_key: Some(masked_license_key),
            ..
        }
        | SessionState::ResetExpired {
            masked_license_key: Some(masked_license_key),
            ..
        } => Some(masked_license_key.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::domain::{
        AccessToken, BoundDeviceSummary, DeviceFingerprint, DeviceId, DevicePublicKey,
        EntitlementStatus,
    };
    use super::super::test_support::*;

    fn outcome() -> ActivationOutcome {
        let public_key = DevicePublicKey::new("public").unwrap();
        ActivationOutcome {
            access_token: AccessToken::new("token").unwrap(),
            masked_license_key: MaskedLicenseKey::new("••••-1234").unwrap(),
            bound_device: BoundDeviceSummary {
                device_id: DeviceId::from_public_key(&public_key),
                public_key,
                fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
            },
            entitlement: EntitlementStatus::Active,
            token_expires_at_ms: 200,
        }
    }

    #[tokio::test]
    async fn activation_sends_device_metadata_and_persists_success() {
        let worker = FakeWorkerClient::new().with_activation(Ok(outcome()));
        let service = TestService::new(worker.clone()).service;
        let result = service
            .activate_license(LicenseKey::new("LICENSE-1234").unwrap())
            .await
            .unwrap();

        let calls = worker.activation_requests();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].license_key.expose_secret(), "LICENSE-1234");
        assert_eq!(calls[0].device_public_key.as_str(), "public");
        assert_eq!(calls[0].app_version, "1.0.0");
        assert_eq!(result.masked_license_key.as_str(), "••••-1234");
    }

    #[tokio::test]
    async fn invalid_activation_persists_nothing() {
        let worker = FakeWorkerClient::new().with_activation(Err(AuthError::InvalidLicenseKey));
        let harness = TestService::new(worker);
        let err = harness
            .service
            .activate_license(LicenseKey::new("bad").unwrap())
            .await
            .unwrap_err();

        assert_eq!(err, AuthError::InvalidLicenseKey);
        assert!(harness.secrets.get_license_key().await.unwrap().is_none());
        assert_eq!(
            harness.state.load_session_state().await.unwrap(),
            SessionState::Unauthenticated
        );
    }

    #[tokio::test]
    async fn missing_token_moves_to_reauth_required() {
        let harness = TestService::new(FakeWorkerClient::new());
        let state = harness.service.validate_session().await.unwrap();
        assert_eq!(
            state,
            SessionState::ReauthRequired {
                masked_license_key: None
            }
        );
    }

    #[tokio::test]
    async fn active_validation_preserves_licensed_state() {
        let worker = FakeWorkerClient::new().with_validation(Ok(ValidationOutcome::Active {
            masked_license_key: MaskedLicenseKey::new("••••-1234").unwrap(),
            bound_device: outcome().bound_device,
            token_expires_at_ms: 300,
        }));
        let harness = TestService::new(worker);
        harness
            .secrets
            .put_access_token(AccessToken::new("token").unwrap())
            .await
            .unwrap();

        let state = harness.service.validate_session().await.unwrap();
        assert!(matches!(state, SessionState::Licensed { .. }));
    }

    #[tokio::test]
    async fn revoked_validation_clears_active_credential() {
        let worker = FakeWorkerClient::new().with_validation(Ok(ValidationOutcome::Revoked));
        let harness = TestService::new(worker);
        harness
            .secrets
            .put_access_token(AccessToken::new("token").unwrap())
            .await
            .unwrap();

        let state = harness.service.validate_session().await.unwrap();
        assert!(matches!(state, SessionState::ReauthRequired { .. }));
        assert!(harness.secrets.get_access_token().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn reset_request_sends_required_metadata_and_persists_pending() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let status = DeviceResetStatus::Pending {
            request_id: request_id.clone(),
            created_at_ms: 10,
        };
        let worker = FakeWorkerClient::new().with_reset_request(Ok(status));
        let harness = TestService::new(worker.clone());
        harness
            .secrets
            .put_license_key(LicenseKey::new("LICENSE-1234").unwrap())
            .await
            .unwrap();

        harness
            .service
            .request_device_reset(
                PurchaseEmail::new("buyer@example.com").unwrap(),
                Some("receipt".into()),
            )
            .await
            .unwrap();

        let calls = worker.reset_requests();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].purchaser_email.as_str(), "buyer@example.com");
        assert_eq!(calls[0].receipt_reference.as_deref(), Some("receipt"));
        assert_eq!(
            harness
                .state
                .load_reset_status()
                .await
                .unwrap()
                .unwrap()
                .request_id(),
            &request_id
        );
    }

    #[tokio::test]
    async fn approved_reset_clears_credentials_and_marks_unbound() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let worker = FakeWorkerClient::new().with_reset_status(Ok(DeviceResetStatus::Approved {
            request_id: request_id.clone(),
            decided_at_ms: 10,
        }));
        let harness = TestService::new(worker);
        harness
            .secrets
            .put_access_token(AccessToken::new("token").unwrap())
            .await
            .unwrap();

        harness
            .service
            .get_device_reset_status(request_id)
            .await
            .unwrap();

        assert!(harness.secrets.get_access_token().await.unwrap().is_none());
        assert!(matches!(
            harness.state.load_session_state().await.unwrap(),
            SessionState::ResetApprovedUnbound { .. }
        ));
    }
}

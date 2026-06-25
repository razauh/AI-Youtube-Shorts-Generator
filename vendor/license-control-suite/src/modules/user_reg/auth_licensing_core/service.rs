use super::{
    domain::{
        ActivationOutcome, ActivationRequest, AuthError, DeviceResetRequest, LicenseKey,
        MaskedLicenseKey, ResetRequestId, ValidationOutcome,
    },
    state::{DeviceResetStatus, SessionState},
    traits::{Clock, DeviceIdentityProvider, LocalStateStore, SecretStore, WorkerClient},
};
use std::sync::Arc;

const VALIDATION_INTERVAL_MS: i64 = 24 * 60 * 60 * 1000;
const OFFLINE_GRACE_MS: i64 = 7 * 24 * 60 * 60 * 1000;

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
    async fn reactivate_with_stored_license(
        &self,
        license_key: LicenseKey,
    ) -> Result<SessionState, AuthError> {
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
        let next = SessionState::after_activation(
            outcome.masked_license_key,
            outcome.bound_device,
            outcome.token_expires_at_ms,
            self.clock.now_ms(),
            VALIDATION_INTERVAL_MS,
        );
        self.state.save_session_state(next.clone()).await?;
        Ok(next)
    }

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
                self.clock.now_ms(),
                VALIDATION_INTERVAL_MS,
            ))
            .await?;

        Ok(outcome)
    }

    pub async fn validate_session(&self) -> Result<SessionState, AuthError> {
        let Some(token) = self.secrets.get_access_token().await? else {
            if let Some(license_key) = self.secrets.get_license_key().await? {
                match self.reactivate_with_stored_license(license_key).await {
                    Ok(next) => return Ok(next),
                    Err(AuthError::WorkerUnreachable) => {
                        let current = self.state.load_session_state().await?;
                        match current
                            .clone()
                            .into_offline_grace(self.clock.now_ms(), OFFLINE_GRACE_MS)
                        {
                            Ok(next) => {
                                self.state.save_session_state(next.clone()).await?;
                                return Ok(next);
                            }
                            Err(_) => {
                                let masked = masked_from_state(&current);
                                let next = SessionState::require_reauth(masked);
                                self.state.save_session_state(next.clone()).await?;
                                return Ok(next);
                            }
                        }
                    }
                    Err(_) => {
                        self.secrets.clear_session_secrets().await?;
                    }
                }
            }
            let current = self.state.load_session_state().await?;
            let next = match current {
                SessionState::Unauthenticated => SessionState::Unauthenticated,
                _ => SessionState::require_reauth(masked_from_state(&current)),
            };
            self.state.save_session_state(next.clone()).await?;
            return Ok(next);
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
                    self.clock.now_ms(),
                    VALIDATION_INTERVAL_MS,
                );
                self.state.save_session_state(next.clone()).await?;
                Ok(next)
            }
            Ok(ValidationOutcome::ReauthRequired) => {
                if let Some(license_key) = self.secrets.get_license_key().await? {
                    if let Ok(next) = self.reactivate_with_stored_license(license_key).await {
                        return Ok(next);
                    }
                }
                self.secrets.clear_session_secrets().await?;
                let current = self.state.load_session_state().await?;
                let next = SessionState::require_reauth(masked_from_state(&current));
                self.state.save_session_state(next.clone()).await?;
                Ok(next)
            }
            Ok(ValidationOutcome::Revoked) => {
                self.secrets.clear_session_secrets().await?;
                let current = self.state.load_session_state().await?;
                let next = SessionState::require_reauth(masked_from_state(&current));
                self.state.save_session_state(next.clone()).await?;
                Ok(next)
            }
            Err(AuthError::WorkerUnreachable) => {
                let current = self.state.load_session_state().await?;
                match current
                    .clone()
                    .into_offline_grace(self.clock.now_ms(), OFFLINE_GRACE_MS)
                {
                    Ok(next) => {
                        self.state.save_session_state(next.clone()).await?;
                        Ok(next)
                    }
                    Err(_) => {
                        let masked = masked_from_state(&current);
                        let next = SessionState::require_reauth(masked);
                        self.state.save_session_state(next.clone()).await?;
                        Ok(next)
                    }
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn request_device_reset_with_license_key(
        &self,
        license_key_override: Option<LicenseKey>,
    ) -> Result<DeviceResetStatus, AuthError> {
        let current = self.state.load_session_state().await?;
        let keypair = self.identity.get_or_create_keypair().await?;
        let fingerprint = self.identity.collect_fingerprint().await?;
        let license_key = if let Some(override_key) = license_key_override {
            Some(override_key)
        } else {
            self.secrets.get_license_key().await?
        };
        if license_key.is_none() {
            // Hosted reset requests require a real license key so the worker can authenticate
            // the reset request. Older sessions might be licensed via access token without the
            // original license key present in secure storage.
            return Err(AuthError::InvalidResetRequest);
        }
        let masked_license_key = license_key.as_ref().map(|key| key.masked());

        let request = DeviceResetRequest {
            license_key,
            masked_license_key,
            purchaser_email: None,
            device_public_key: keypair.public_key().clone(),
            fingerprint,
            app_version: self.app_version.clone(),
            timestamp_ms: self.clock.now_ms(),
        };
        let status = self.worker.request_device_reset(request).await?;
        self.state.save_reset_status(status.clone()).await?;
        if !is_licensed_session(&current) {
            let next = SessionState::after_reset_status(&status, masked_from_state(&current))?;
            self.state.save_session_state(next).await?;
        }
        Ok(status)
    }

    pub async fn request_device_reset(
        &self,
    ) -> Result<DeviceResetStatus, AuthError> {
        self.request_device_reset_with_license_key(None).await
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
        if matches!(status, DeviceResetStatus::Approved { .. }) {
            let next = SessionState::after_reset_status(&status, masked_from_state(&current))?;
            self.secrets.clear_session_secrets().await?;
            self.state.save_session_state(next).await?;
        } else if !is_licensed_session(&current) {
            let next = SessionState::after_reset_status(&status, masked_from_state(&current))?;
            self.state.save_session_state(next).await?;
        }
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

    pub async fn deactivate_current_device(&self) -> Result<(), AuthError> {
        let keypair = self.identity.get_or_create_keypair().await?;
        self.secrets.put_device_keypair(keypair.clone()).await?;
        let fingerprint = self.identity.collect_fingerprint().await?;
        let license_key = self.secrets.get_license_key().await?;
        if license_key.is_none() {
            return Err(AuthError::InvalidResetRequest);
        }
        let request = DeviceResetRequest {
            license_key,
            masked_license_key: None,
            purchaser_email: None,
            device_public_key: keypair.public_key().clone(),
            fingerprint,
            app_version: self.app_version.clone(),
            timestamp_ms: self.clock.now_ms(),
        };
        match self.worker.request_device_reset(request).await {
            Ok(status) => {
                self.secrets.clear_session_secrets().await?;
                self.state.save_session_state(SessionState::Unauthenticated).await?;
                self.state.save_reset_status(status).await?;
                Ok(())
            }
            Err(err) => {
                if err == AuthError::WorkerUnreachable {
                    Err(err)
                } else {
                    self.secrets.clear_session_secrets().await?;
                    self.state.save_session_state(SessionState::Unauthenticated).await?;
                    Err(err)
                }
            }
        }
    }
}

fn masked_from_state(state: &SessionState) -> Option<MaskedLicenseKey> {
    match state {
        SessionState::Licensed {
            masked_license_key, ..
        }
        | SessionState::LicensedOfflineGrace {
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

fn is_licensed_session(state: &SessionState) -> bool {
    matches!(
        state,
        SessionState::Licensed { .. } | SessionState::LicensedOfflineGrace { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::super::domain::{
        AccessToken, BoundDeviceSummary, DeviceFingerprint, DeviceId, DevicePublicKey,
        EntitlementStatus,
    };
    use super::super::test_support::*;
    use super::*;

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
    async fn missing_token_keeps_unauthenticated_state() {
        let harness = TestService::new(FakeWorkerClient::new());
        let state = harness.service.validate_session().await.unwrap();
        assert_eq!(state, SessionState::Unauthenticated);
    }

    #[tokio::test]
    async fn missing_token_moves_prior_authenticated_state_to_reauth_required() {
        let harness = TestService::new(FakeWorkerClient::new());
        harness
            .state
            .save_session_state(SessionState::after_activation(
                MaskedLicenseKey::new("••••-1234").unwrap(),
                outcome().bound_device.clone(),
                200,
                10,
                24 * 60 * 60 * 1000,
            ))
            .await
            .unwrap();

        let state = harness.service.validate_session().await.unwrap();
        assert!(matches!(state, SessionState::ReauthRequired { .. }));
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
        harness
            .secrets
            .put_license_key(LicenseKey::new("LICENSE-1234").unwrap())
            .await
            .unwrap();

        let state = harness.service.validate_session().await.unwrap();
        assert!(matches!(state, SessionState::Licensed { .. }));
    }

    #[tokio::test]
    async fn active_validation_without_stored_license_key_keeps_licensed_state() {
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
        assert!(harness.secrets.get_access_token().await.unwrap().is_some());
    }

    #[tokio::test]
    async fn reauth_required_validation_with_stored_license_reactivates_session() {
        let worker = FakeWorkerClient::new()
            .with_validation(Ok(ValidationOutcome::ReauthRequired))
            .with_activation(Ok(outcome()));
        let harness = TestService::new(worker);
        harness
            .secrets
            .put_access_token(AccessToken::new("expired-token").unwrap())
            .await
            .unwrap();
        harness
            .secrets
            .put_license_key(LicenseKey::new("LICENSE-1234").unwrap())
            .await
            .unwrap();

        let state = harness.service.validate_session().await.unwrap();
        assert!(matches!(state, SessionState::Licensed { .. }));
        assert!(harness.secrets.get_access_token().await.unwrap().is_some());
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
    async fn worker_unreachable_moves_licensed_session_to_offline_grace() {
        let worker = FakeWorkerClient::new().with_validation(Err(AuthError::WorkerUnreachable));
        let harness = TestService::new(worker);
        harness
            .secrets
            .put_access_token(AccessToken::new("token").unwrap())
            .await
            .unwrap();
        harness
            .state
            .save_session_state(SessionState::after_activation(
                MaskedLicenseKey::new("••••-1234").unwrap(),
                outcome().bound_device.clone(),
                200,
                10,
                24 * 60 * 60 * 1000,
            ))
            .await
            .unwrap();

        let state = harness.service.validate_session().await.unwrap();
        assert!(matches!(state, SessionState::LicensedOfflineGrace { .. }));
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
            .request_device_reset()
            .await
            .unwrap();

        let calls = worker.reset_requests();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].purchaser_email.is_none());
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
    async fn reset_request_does_not_force_logout_for_licensed_sessions() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let status = DeviceResetStatus::Pending {
            request_id,
            created_at_ms: 10,
        };
        let worker = FakeWorkerClient::new().with_reset_request(Ok(status));
        let harness = TestService::new(worker);
        harness
            .state
            .save_session_state(SessionState::after_activation(
                MaskedLicenseKey::new("••••-1234").unwrap(),
                outcome().bound_device.clone(),
                200,
                10,
                24 * 60 * 60 * 1000,
            ))
            .await
            .unwrap();
        harness
            .secrets
            .put_license_key(LicenseKey::new("LICENSE-1234").unwrap())
            .await
            .unwrap();

        harness.service.request_device_reset().await.unwrap();

        assert!(matches!(
            harness.state.load_session_state().await.unwrap(),
            SessionState::Licensed { .. }
        ));
    }

    #[tokio::test]
    async fn reset_request_accepts_ephemeral_license_key_when_secure_store_is_empty() {
        let request_id = ResetRequestId::new("reset-ephemeral").unwrap();
        let status = DeviceResetStatus::Pending {
            request_id: request_id.clone(),
            created_at_ms: 10,
        };
        let worker = FakeWorkerClient::new().with_reset_request(Ok(status));
        let harness = TestService::new(worker.clone());

        harness
            .service
            .request_device_reset_with_license_key(Some(LicenseKey::new("LICENSE-1234").unwrap()))
            .await
            .unwrap();

        let calls = worker.reset_requests();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0]
                .license_key
                .as_ref()
                .expect("ephemeral license key should be forwarded")
                .expose_secret(),
            "LICENSE-1234"
        );
        assert!(
            harness.secrets.get_license_key().await.unwrap().is_none(),
            "ephemeral reset key must not be persisted"
        );
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
    async fn pending_reset_status_preserves_licensed_state() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let worker = FakeWorkerClient::new().with_reset_status(Ok(DeviceResetStatus::Pending {
            request_id: request_id.clone(),
            created_at_ms: 10,
        }));
        let harness = TestService::new(worker);
        harness
            .state
            .save_session_state(SessionState::after_activation(
                MaskedLicenseKey::new("••••-1234").unwrap(),
                outcome().bound_device.clone(),
                200,
                10,
                24 * 60 * 60 * 1000,
            ))
            .await
            .unwrap();

        harness
            .service
            .get_device_reset_status(request_id)
            .await
            .unwrap();

        assert!(matches!(
            harness.state.load_session_state().await.unwrap(),
            SessionState::Licensed { .. }
        ));
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

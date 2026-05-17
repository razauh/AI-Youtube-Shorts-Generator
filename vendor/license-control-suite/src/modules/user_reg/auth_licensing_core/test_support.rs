use super::domain::{
    AccessToken, ActivationOutcome, ActivationRequest, AuthError, DeviceFingerprint, DeviceKeyPair,
    DevicePublicKey, DeviceResetRequest, LicenseKey, ResetRequestId, ValidationOutcome,
};
use super::service::AuthService;
use super::state::{DeviceResetStatus, SessionState};
use super::traits::{Clock, DeviceIdentityProvider, LocalStateStore, SecretStore, WorkerClient};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct InMemorySecretStore {
    license_key: Arc<Mutex<Option<LicenseKey>>>,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    device_keypair: Arc<Mutex<Option<DeviceKeyPair>>>,
}

#[async_trait]
impl SecretStore for InMemorySecretStore {
    async fn put_license_key(&self, value: LicenseKey) -> Result<(), AuthError> {
        *self.license_key.lock().unwrap() = Some(value);
        Ok(())
    }

    async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError> {
        Ok(self.license_key.lock().unwrap().clone())
    }

    async fn put_access_token(&self, value: AccessToken) -> Result<(), AuthError> {
        *self.access_token.lock().unwrap() = Some(value);
        Ok(())
    }

    async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
        Ok(self.access_token.lock().unwrap().clone())
    }

    async fn put_device_keypair(&self, value: DeviceKeyPair) -> Result<(), AuthError> {
        *self.device_keypair.lock().unwrap() = Some(value);
        Ok(())
    }

    async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
        Ok(self.device_keypair.lock().unwrap().clone())
    }

    async fn clear_session_secrets(&self) -> Result<(), AuthError> {
        *self.license_key.lock().unwrap() = None;
        *self.access_token.lock().unwrap() = None;
        Ok(())
    }
}

#[derive(Clone)]
pub struct InMemoryLocalStateStore {
    session_state: Arc<Mutex<SessionState>>,
    reset_status: Arc<Mutex<Option<DeviceResetStatus>>>,
}

impl Default for InMemoryLocalStateStore {
    fn default() -> Self {
        Self {
            session_state: Arc::new(Mutex::new(SessionState::Unauthenticated)),
            reset_status: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl LocalStateStore for InMemoryLocalStateStore {
    async fn save_session_state(&self, state: SessionState) -> Result<(), AuthError> {
        *self.session_state.lock().unwrap() = state;
        Ok(())
    }

    async fn load_session_state(&self) -> Result<SessionState, AuthError> {
        Ok(self.session_state.lock().unwrap().clone())
    }

    async fn save_reset_status(&self, status: DeviceResetStatus) -> Result<(), AuthError> {
        *self.reset_status.lock().unwrap() = Some(status);
        Ok(())
    }

    async fn load_reset_status(&self) -> Result<Option<DeviceResetStatus>, AuthError> {
        Ok(self.reset_status.lock().unwrap().clone())
    }
}

#[derive(Clone)]
pub struct FakeDeviceIdentityProvider {
    keypair: DeviceKeyPair,
    fingerprint: DeviceFingerprint,
}

impl Default for FakeDeviceIdentityProvider {
    fn default() -> Self {
        Self {
            keypair: DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "private")
                .unwrap(),
            fingerprint: DeviceFingerprint::new(
                "linux",
                "linux",
                "x86_64",
                Some("host-hash".into()),
            )
            .unwrap(),
        }
    }
}

#[async_trait]
impl DeviceIdentityProvider for FakeDeviceIdentityProvider {
    async fn get_or_create_keypair(&self) -> Result<DeviceKeyPair, AuthError> {
        Ok(self.keypair.clone())
    }

    async fn collect_fingerprint(&self) -> Result<DeviceFingerprint, AuthError> {
        Ok(self.fingerprint.clone())
    }
}

#[derive(Clone)]
pub struct FixedClock(pub i64);

impl Clock for FixedClock {
    fn now_ms(&self) -> i64 {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct FakeWorkerClient {
    activation_result: Arc<Mutex<Option<Result<ActivationOutcome, AuthError>>>>,
    validation_result: Arc<Mutex<Option<Result<ValidationOutcome, AuthError>>>>,
    reset_request_result: Arc<Mutex<Option<Result<DeviceResetStatus, AuthError>>>>,
    reset_status_result: Arc<Mutex<Option<Result<DeviceResetStatus, AuthError>>>>,
    activation_requests: Arc<Mutex<Vec<ActivationRequest>>>,
    validation_requests: Arc<Mutex<Vec<AccessToken>>>,
    reset_requests: Arc<Mutex<Vec<DeviceResetRequest>>>,
    reset_status_requests: Arc<Mutex<Vec<ResetRequestId>>>,
}

impl FakeWorkerClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_activation(self, result: Result<ActivationOutcome, AuthError>) -> Self {
        *self.activation_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_validation(self, result: Result<ValidationOutcome, AuthError>) -> Self {
        *self.validation_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_reset_request(self, result: Result<DeviceResetStatus, AuthError>) -> Self {
        *self.reset_request_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_reset_status(self, result: Result<DeviceResetStatus, AuthError>) -> Self {
        *self.reset_status_result.lock().unwrap() = Some(result);
        self
    }

    pub fn activation_requests(&self) -> Vec<ActivationRequest> {
        self.activation_requests.lock().unwrap().clone()
    }

    pub fn reset_requests(&self) -> Vec<DeviceResetRequest> {
        self.reset_requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl WorkerClient for FakeWorkerClient {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        self.activation_requests.lock().unwrap().push(request);
        self.activation_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(Err(AuthError::WorkerUnreachable))
    }

    async fn validate_session(&self, token: AccessToken) -> Result<ValidationOutcome, AuthError> {
        self.validation_requests.lock().unwrap().push(token);
        self.validation_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(Err(AuthError::WorkerUnreachable))
    }

    async fn request_device_reset(
        &self,
        request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        self.reset_requests.lock().unwrap().push(request);
        self.reset_request_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(Err(AuthError::WorkerUnreachable))
    }

    async fn get_device_reset_status(
        &self,
        request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        self.reset_status_requests.lock().unwrap().push(request_id);
        self.reset_status_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(Err(AuthError::WorkerUnreachable))
    }
}

pub struct TestService {
    pub service: AuthService,
    pub secrets: Arc<InMemorySecretStore>,
    pub state: Arc<InMemoryLocalStateStore>,
}

impl TestService {
    pub fn new(worker: FakeWorkerClient) -> Self {
        let secrets = Arc::new(InMemorySecretStore::default());
        let state = Arc::new(InMemoryLocalStateStore::default());
        let service = AuthService::new(
            Arc::new(worker),
            secrets.clone(),
            state.clone(),
            Arc::new(FakeDeviceIdentityProvider::default()),
            Arc::new(FixedClock(10)),
            "1.0.0",
        );
        Self {
            service,
            secrets,
            state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_secret_store_persists_and_clears() {
        let store = InMemorySecretStore::default();
        store
            .put_license_key(LicenseKey::new("key").unwrap())
            .await
            .unwrap();
        store
            .put_access_token(AccessToken::new("token").unwrap())
            .await
            .unwrap();
        assert!(store.get_license_key().await.unwrap().is_some());
        store.clear_session_secrets().await.unwrap();
        assert!(store.get_license_key().await.unwrap().is_none());
        assert!(store.get_access_token().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn local_state_defaults_to_unauthenticated() {
        let store = InMemoryLocalStateStore::default();
        assert_eq!(
            store.load_session_state().await.unwrap(),
            SessionState::Unauthenticated
        );
    }

    #[tokio::test]
    async fn fake_device_provider_returns_stable_identity() {
        let provider = FakeDeviceIdentityProvider::default();
        assert_eq!(
            provider
                .get_or_create_keypair()
                .await
                .unwrap()
                .public_key()
                .as_str(),
            "public"
        );
        assert_eq!(
            provider.collect_fingerprint().await.unwrap().platform,
            "linux"
        );
    }
}

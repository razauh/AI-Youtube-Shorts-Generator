use async_trait::async_trait;
use license_control_suite::core::{
    AccessToken, ActivationOutcome, ActivationRequest, AuthError, AuthService, BoundDeviceSummary,
    Clock, DeviceFingerprint, DeviceId, DeviceIdentityProvider, DeviceKeyPair,
    DevicePublicKey, DeviceResetRequest, DeviceResetStatus, EntitlementStatus, LicenseKey,
    LocalStateStore, ResetRequestId, SecretStore, SessionState, ValidationOutcome, WorkerClient,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ExampleWorkerClient;

#[async_trait]
impl WorkerClient for ExampleWorkerClient {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        let public_key = request.device_public_key.clone();
        Ok(ActivationOutcome {
            access_token: AccessToken::new("example-access-token")?,
            masked_license_key: request.license_key.masked(),
            bound_device: BoundDeviceSummary {
                device_id: DeviceId::from_public_key(&public_key),
                public_key,
                fingerprint: request.fingerprint,
            },
            entitlement: EntitlementStatus::Active,
            token_expires_at_ms: 1_700_000_000_000,
        })
    }

    async fn validate_session(
        &self,
        _token: AccessToken,
    ) -> Result<ValidationOutcome, AuthError> {
        Err(AuthError::WorkerUnreachable)
    }

    async fn request_device_reset(
        &self,
        _request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        Err(AuthError::InvalidResetRequest)
    }

    async fn get_device_reset_status(
        &self,
        _request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        Err(AuthError::ResetRequestNotFound)
    }
}

#[derive(Clone, Default)]
struct ExampleSecretStore {
    license_key: Arc<Mutex<Option<LicenseKey>>>,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    device_keypair: Arc<Mutex<Option<DeviceKeyPair>>>,
}

#[async_trait]
impl SecretStore for ExampleSecretStore {
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

#[derive(Clone, Default)]
struct ExampleLocalStateStore {
    session_state: Arc<Mutex<SessionState>>,
    reset_status: Arc<Mutex<Option<DeviceResetStatus>>>,
}

#[async_trait]
impl LocalStateStore for ExampleLocalStateStore {
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
struct ExampleIdentityProvider;

#[async_trait]
impl DeviceIdentityProvider for ExampleIdentityProvider {
    async fn get_or_create_keypair(&self) -> Result<DeviceKeyPair, AuthError> {
        DeviceKeyPair::new(DevicePublicKey::new("fake-public-key")?, "fake-private-key")
    }

    async fn collect_fingerprint(&self) -> Result<DeviceFingerprint, AuthError> {
        DeviceFingerprint::new("linux", "linux", "x86_64", Some("host-hash".into()))
    }
}

struct ExampleClock;

impl Clock for ExampleClock {
    fn now_ms(&self) -> i64 {
        1_700_000_000_000
    }
}

pub async fn run_fake_injection_activation_flow() -> Result<SessionState, AuthError> {
    let service = AuthService::new(
        Arc::new(ExampleWorkerClient),
        Arc::new(ExampleSecretStore::default()),
        Arc::new(ExampleLocalStateStore::default()),
        Arc::new(ExampleIdentityProvider),
        Arc::new(ExampleClock),
        "1.0.0",
    );

    service
        .activate_license(LicenseKey::new("LICENSE-1234")?)
        .await?;

    service.get_auth_state().await
}

fn main() {}

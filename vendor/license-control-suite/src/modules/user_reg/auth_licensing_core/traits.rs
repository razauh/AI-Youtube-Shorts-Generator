use super::domain::{
    AccessToken, ActivationOutcome, ActivationRequest, AuthError, DeviceFingerprint, DeviceKeyPair,
    DeviceResetRequest, LicenseKey, ResetRequestId, ValidationOutcome,
};
use super::state::{DeviceResetStatus, SessionState};
use async_trait::async_trait;

#[async_trait]
pub trait WorkerClient: Send + Sync {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError>;
    async fn validate_session(
        &self,
        token: AccessToken,
    ) -> Result<ValidationOutcome, AuthError>;
    async fn request_device_reset(
        &self,
        request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError>;
    async fn get_device_reset_status(
        &self,
        request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError>;
}

#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn put_license_key(&self, value: LicenseKey) -> Result<(), AuthError>;
    async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError>;
    async fn put_access_token(&self, value: AccessToken) -> Result<(), AuthError>;
    async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError>;
    async fn put_device_keypair(&self, value: DeviceKeyPair) -> Result<(), AuthError>;
    async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError>;
    async fn clear_session_secrets(&self) -> Result<(), AuthError>;
}

#[async_trait]
pub trait LocalStateStore: Send + Sync {
    async fn save_session_state(&self, state: SessionState) -> Result<(), AuthError>;
    async fn load_session_state(&self) -> Result<SessionState, AuthError>;
    async fn save_reset_status(&self, status: DeviceResetStatus) -> Result<(), AuthError>;
    async fn load_reset_status(&self) -> Result<Option<DeviceResetStatus>, AuthError>;
}

#[async_trait]
pub trait DeviceIdentityProvider: Send + Sync {
    async fn get_or_create_keypair(&self) -> Result<DeviceKeyPair, AuthError>;
    async fn collect_fingerprint(&self) -> Result<DeviceFingerprint, AuthError>;
}

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
}

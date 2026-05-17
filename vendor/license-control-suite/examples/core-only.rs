use license_control_suite::core::{
    test_support::{
        FakeDeviceIdentityProvider, FakeWorkerClient, FixedClock, InMemoryLocalStateStore,
        InMemorySecretStore,
    },
    AccessToken, ActivationOutcome, AuthError, AuthService, BoundDeviceSummary,
    DeviceFingerprint, DeviceId, DevicePublicKey, EntitlementStatus, LicenseKey,
    MaskedLicenseKey, SessionState,
};
use std::sync::Arc;

fn example_activation_outcome() -> ActivationOutcome {
    let public_key = DevicePublicKey::new("example-public-key").unwrap();
    ActivationOutcome {
        access_token: AccessToken::new("example-access-token").unwrap(),
        masked_license_key: MaskedLicenseKey::new("••••-1234").unwrap(),
        bound_device: BoundDeviceSummary {
            device_id: DeviceId::from_public_key(&public_key),
            public_key,
            fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
        },
        entitlement: EntitlementStatus::Active,
        token_expires_at_ms: 1_700_000_000_000,
    }
}

pub async fn run_core_only_activation_flow() -> Result<SessionState, AuthError> {
    let service = AuthService::new(
        Arc::new(FakeWorkerClient::new().with_activation(Ok(example_activation_outcome()))),
        Arc::new(InMemorySecretStore::default()),
        Arc::new(InMemoryLocalStateStore::default()),
        Arc::new(FakeDeviceIdentityProvider::default()),
        Arc::new(FixedClock(1_700_000_000_000)),
        "1.0.0",
    );

    service
        .activate_license(LicenseKey::new("LICENSE-1234")?)
        .await?;

    service.get_auth_state().await
}

fn main() {}

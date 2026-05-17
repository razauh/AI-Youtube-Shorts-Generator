use license_control_suite::modules::user_reg::auth_licensing_core::test_support::{FakeWorkerClient, TestService};
use license_control_suite::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationOutcome, BoundDeviceSummary, DeviceFingerprint, DeviceId,
    DevicePublicKey, DeviceResetStatus, EntitlementStatus, LocalStateStore, MaskedLicenseKey,
    ResetRequestId, SecretStore, SessionState, ValidationOutcome,
};
use license_control_suite::modules::user_reg::auth_licensing_tauri::{
    activate_license_with_service, get_device_reset_status_with_service,
    validate_session_with_service, AuthStateView,
};

fn activation_outcome() -> ActivationOutcome {
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
        token_expires_at_ms: 100,
    }
}

#[tokio::test]
async fn fresh_install_activation_becomes_licensed() {
    let harness =
        TestService::new(FakeWorkerClient::new().with_activation(Ok(activation_outcome())));

    let view = activate_license_with_service("LICENSE-1234".into(), &harness.service)
        .await
        .unwrap();

    assert!(matches!(view.auth_state, AuthStateView::Licensed { .. }));
    assert!(matches!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Licensed { .. }
    ));
}

#[tokio::test]
async fn relaunch_validation_keeps_session_licensed() {
    let outcome = activation_outcome();
    let harness = TestService::new(FakeWorkerClient::new().with_validation(Ok(
        ValidationOutcome::Active {
            masked_license_key: outcome.masked_license_key,
            bound_device: outcome.bound_device,
            token_expires_at_ms: 200,
        },
    )));
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();

    let view = validate_session_with_service(&harness.service)
        .await
        .unwrap();

    assert!(matches!(view.auth_state, AuthStateView::Licensed { .. }));
}

#[tokio::test]
async fn reset_approval_clears_old_local_credential_and_marks_unbound() {
    let request_id = ResetRequestId::new("reset-1").unwrap();
    let harness = TestService::new(FakeWorkerClient::new().with_reset_status(Ok(
        DeviceResetStatus::Approved {
            request_id,
            decided_at_ms: 100,
        },
    )));
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();

    let view = get_device_reset_status_with_service("reset-1".into(), &harness.service)
        .await
        .unwrap();

    assert!(matches!(
        view.auth_state,
        AuthStateView::ResetApprovedUnbound { .. }
    ));
    assert!(harness.secrets.get_access_token().await.unwrap().is_none());
}

#[tokio::test]
async fn worker_unavailable_does_not_corrupt_local_credential() {
    let harness = TestService::new(FakeWorkerClient::new());
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();

    let result = validate_session_with_service(&harness.service).await;

    assert!(result.is_err());
    assert!(harness.secrets.get_access_token().await.unwrap().is_some());
}

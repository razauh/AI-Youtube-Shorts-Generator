use license_control_suite::modules::user_reg::auth_licensing_core::test_support::{
    FakeWorkerClient, TestService,
};
use license_control_suite::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationOutcome, BoundDeviceSummary, DeviceFingerprint, DeviceId,
    DevicePublicKey, DeviceResetStatus, EntitlementStatus, LicenseKey, MaskedLicenseKey,
    PurchaseEmail, ResetRequestId, SessionState, ValidationOutcome,
};

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

#[tokio::test]
async fn baseline_auth_service_activation_validation_reset_flow_remains_current() {
    let request_id = ResetRequestId::new("reset-1").unwrap();
    let worker = FakeWorkerClient::new()
        .with_activation(Ok(outcome()))
        .with_validation(Ok(ValidationOutcome::Active {
            masked_license_key: MaskedLicenseKey::new("••••-1234").unwrap(),
            bound_device: outcome().bound_device.clone(),
            token_expires_at_ms: 300,
        }))
        .with_reset_status(Ok(DeviceResetStatus::Approved {
            request_id: request_id.clone(),
            decided_at_ms: 10,
        }));
    let harness = TestService::new(worker);

    let activation = harness
        .service
        .activate_license(LicenseKey::new("LICENSE-1234").unwrap())
        .await
        .unwrap();
    assert_eq!(activation.masked_license_key.as_str(), "••••-1234");
    assert!(harness.secrets.get_license_key().await.unwrap().is_some());
    assert!(harness.secrets.get_access_token().await.unwrap().is_some());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert!(matches!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Licensed { .. }
    ));

    let validated = harness.service.validate_session().await.unwrap();
    assert!(matches!(validated, SessionState::Licensed { .. }));

    harness
        .service
        .get_device_reset_status(request_id)
        .await
        .unwrap();

    assert!(harness.secrets.get_license_key().await.unwrap().is_none());
    assert!(harness.secrets.get_access_token().await.unwrap().is_none());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert!(matches!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::ResetApprovedUnbound { .. }
    ));
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

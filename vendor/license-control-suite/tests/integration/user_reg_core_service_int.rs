use license_control_suite::modules::user_reg::auth_licensing_core::test_support::{
    FakeWorkerClient, TestService,
};
use license_control_suite::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationOutcome, AuthError, BoundDeviceSummary, DeviceFingerprint, DeviceId,
    DeviceKeyPair, DevicePublicKey, DeviceResetStatus, EntitlementStatus, LicenseKey,
    LocalStateStore, MaskedLicenseKey, ResetRequestId, SecretStore, SessionState,
    ValidationOutcome,
};

fn stored_activation_keypair() -> DeviceKeyPair {
    DeviceKeyPair::new(
        DevicePublicKey::new("stored-activation-public").unwrap(),
        "stored-activation-private",
    )
    .unwrap()
}

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
async fn deactivate_current_device_success_clears_only_approved_state() {
    let worker = FakeWorkerClient::new().with_reset_request(Ok(DeviceResetStatus::Approved {
        request_id: ResetRequestId::new("reset-success").unwrap(),
        decided_at_ms: 10,
    }));
    let harness = TestService::new(worker);
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_device_keypair(stored_activation_keypair())
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

    harness.service.deactivate_current_device().await.unwrap();

    assert!(harness.secrets.get_license_key().await.unwrap().is_none());
    assert!(harness.secrets.get_access_token().await.unwrap().is_none());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert_eq!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Unauthenticated
    );
}

#[tokio::test]
async fn deactivate_current_device_reuses_stored_activation_identity() {
    let worker = FakeWorkerClient::new().with_reset_request(Ok(DeviceResetStatus::Approved {
        request_id: ResetRequestId::new("reset-success").unwrap(),
        decided_at_ms: 10,
    }));
    let harness = TestService::new(worker.clone());
    let stored_keypair = stored_activation_keypair();
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_device_keypair(stored_keypair.clone())
        .await
        .unwrap();

    harness.service.deactivate_current_device().await.unwrap();

    let calls = worker.reset_requests();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].device_public_key.as_str(),
        stored_keypair.public_key().as_str()
    );
}

#[tokio::test]
async fn deactivate_current_device_reuses_keypair_persisted_by_activation() {
    let worker = FakeWorkerClient::new()
        .with_activation(Ok(outcome()))
        .with_reset_request(Ok(DeviceResetStatus::Approved {
            request_id: ResetRequestId::new("reset-success").unwrap(),
            decided_at_ms: 10,
        }));
    let harness = TestService::new(worker.clone());

    harness
        .service
        .activate_license(LicenseKey::new("LICENSE-1234").unwrap())
        .await
        .unwrap();
    harness.service.deactivate_current_device().await.unwrap();

    let activation_calls = worker.activation_requests();
    let reset_calls = worker.reset_requests();
    assert_eq!(activation_calls.len(), 1);
    assert_eq!(reset_calls.len(), 1);
    assert_eq!(
        reset_calls[0].device_public_key.as_str(),
        activation_calls[0].device_public_key.as_str()
    );
}

#[tokio::test]
async fn deactivate_current_device_fails_without_regenerating_missing_identity() {
    let worker = FakeWorkerClient::new().with_reset_request(Ok(DeviceResetStatus::Approved {
        request_id: ResetRequestId::new("reset-success").unwrap(),
        decided_at_ms: 10,
    }));
    let harness = TestService::new(worker.clone());
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();

    let err = harness.service.deactivate_current_device().await.unwrap_err();

    assert_eq!(err, AuthError::InvalidDeviceIdentity);
    assert!(worker.reset_requests().is_empty());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_none());
}

#[tokio::test]
async fn deactivate_current_device_failure_preserves_retryable_state() {
    let worker = FakeWorkerClient::new().with_reset_request(Err(AuthError::WorkerUnreachable));
    let harness = TestService::new(worker);
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_device_keypair(stored_activation_keypair())
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

    let err = harness.service.deactivate_current_device().await.unwrap_err();
    assert_eq!(err, AuthError::WorkerUnreachable);

    assert!(harness.secrets.get_license_key().await.unwrap().is_some());
    assert!(harness.secrets.get_access_token().await.unwrap().is_some());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert!(matches!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Licensed { .. }
    ));
}

#[tokio::test]
async fn deactivate_current_device_pending_status_preserves_retryable_state() {
    let request_id = ResetRequestId::new("reset-pending").unwrap();
    let worker = FakeWorkerClient::new().with_reset_request(Ok(DeviceResetStatus::Pending {
        request_id: request_id.clone(),
        created_at_ms: 10,
    }));
    let harness = TestService::new(worker);
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_device_keypair(stored_activation_keypair())
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

    let err = harness.service.deactivate_current_device().await.unwrap_err();

    assert_eq!(err.code(), "invalid_transition");
    assert!(harness.secrets.get_license_key().await.unwrap().is_some());
    assert!(harness.secrets.get_access_token().await.unwrap().is_some());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert!(matches!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Licensed { .. }
    ));
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
async fn deactivate_current_device_non_terminal_error_preserves_retryable_state() {
    let worker = FakeWorkerClient::new().with_reset_request(Err(AuthError::Unauthorized));
    let harness = TestService::new(worker);
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_device_keypair(stored_activation_keypair())
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

    let err = harness.service.deactivate_current_device().await.unwrap_err();

    assert_eq!(err, AuthError::Unauthorized);
    assert!(harness.secrets.get_license_key().await.unwrap().is_some());
    assert!(harness.secrets.get_access_token().await.unwrap().is_some());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert!(matches!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Licensed { .. }
    ));
}

#[tokio::test]
async fn deactivate_current_device_terminal_already_deactivated_clears_state() {
    let worker = FakeWorkerClient::new().with_reset_request(Err(AuthError::InvalidResetRequest));
    let harness = TestService::new(worker);
    harness
        .secrets
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    harness
        .secrets
        .put_device_keypair(stored_activation_keypair())
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

    let err = harness.service.deactivate_current_device().await.unwrap_err();
    // Assuming InvalidResetRequest (or another terminal variant) is treated as terminal
    assert_eq!(err, AuthError::InvalidResetRequest);

    assert!(harness.secrets.get_license_key().await.unwrap().is_none());
    assert!(harness.secrets.get_access_token().await.unwrap().is_none());
    assert!(harness.secrets.get_device_keypair().await.unwrap().is_some());
    assert_eq!(
        harness.state.load_session_state().await.unwrap(),
        SessionState::Unauthenticated
    );
}

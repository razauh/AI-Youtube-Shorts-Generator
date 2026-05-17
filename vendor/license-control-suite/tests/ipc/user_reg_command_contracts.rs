use license_control_suite::modules::user_reg::auth_licensing_core::{
    BoundDeviceSummary, DeviceFingerprint, DeviceId, DevicePublicKey, MaskedLicenseKey, ResetRequestId,
    SessionState,
};
use license_control_suite::modules::user_reg::auth_licensing_tauri::{
    activate_license_with_service, clear_local_session_with_service, get_auth_state_with_service,
    get_device_reset_status_with_service, request_device_reset_with_service,
    validate_session_with_service, AuthCommandError, AuthStateView, DeviceResetInput,
    DeviceResetView,
};
use license_control_suite::modules::user_reg::auth_licensing_core::test_support::{FakeWorkerClient, TestService};
use license_control_suite::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationOutcome, DeviceResetStatus, EntitlementStatus,
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

#[test]
fn auth_command_error_serializes_with_code_and_message() {
    let err = AuthCommandError {
        code: "worker_unreachable".into(),
        message: "worker is unreachable".into(),
    };
    let value = serde_json::to_value(err).unwrap();
    assert!(value.get("code").is_some());
    assert!(value.get("message").is_some());
}

#[test]
fn auth_state_view_uses_tagged_shape() {
    let value = AuthStateView::ResetPending {
        request_id: "reset-1".into(),
        masked_license_key: Some("••••-1234".into()),
    };
    let json = serde_json::to_value(value).unwrap();
    assert_eq!(json["status"], "reset_pending");
    assert_eq!(json["request_id"], "reset-1");
}

#[tokio::test]
async fn activation_helper_response_shape_is_frontend_safe() {
    let harness = TestService::new(FakeWorkerClient::new().with_activation(Ok(outcome())));
    let view = activate_license_with_service("LICENSE-1234".into(), &harness.service)
        .await
        .unwrap();
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("••••-1234"));
    assert!(!json.contains("LICENSE-1234"));
    assert!(!json.contains("secret-token"));
}

#[test]
fn reset_view_shape_has_expected_fields() {
    let view = DeviceResetView {
        request_id: "reset-1".into(),
        status: "approved".into(),
        message: Some(DeviceResetStatus::approved_message().to_string()),
        auth_state: AuthStateView::from(SessionState::ResetApprovedUnbound {
            request_id: ResetRequestId::new("reset-1").unwrap(),
            masked_license_key: Some(MaskedLicenseKey::new("••••-1234").unwrap()),
        }),
    };
    let value = serde_json::to_value(view).unwrap();
    assert_eq!(value["request_id"], "reset-1");
    assert_eq!(value["status"], "approved");
    assert!(value.get("message").is_some());
    assert!(value.get("auth_state").is_some());
}

#[tokio::test]
async fn service_helper_functions_remain_behaviorally_equivalent() {
    let request_id = ResetRequestId::new("reset-1").unwrap();
    let harness = TestService::new(
        FakeWorkerClient::new()
            .with_activation(Ok(outcome()))
            .with_validation(Ok(
                license_control_suite::modules::user_reg::auth_licensing_core::ValidationOutcome::Active {
                    masked_license_key: MaskedLicenseKey::new("••••-1234").unwrap(),
                    bound_device: outcome().bound_device.clone(),
                    token_expires_at_ms: 200,
                },
            ))
            .with_reset_request(Ok(DeviceResetStatus::Pending {
                request_id: request_id.clone(),
                created_at_ms: 10,
            }))
            .with_reset_status(Ok(DeviceResetStatus::Approved {
                request_id,
                decided_at_ms: 20,
            })),
    );

    let activated = activate_license_with_service("LICENSE-1234".into(), &harness.service)
        .await
        .unwrap();
    assert!(matches!(activated.auth_state, AuthStateView::Licensed { .. }));

    let validated = validate_session_with_service(&harness.service).await.unwrap();
    assert!(matches!(validated.auth_state, AuthStateView::Licensed { .. }));

    let requested = request_device_reset_with_service(
        DeviceResetInput {
            purchaser_email: "buyer@example.com".into(),
            receipt_reference: Some("receipt".into()),
        },
        &harness.service,
    )
    .await
    .unwrap();
    assert_eq!(requested.status, "pending");

    let polled = get_device_reset_status_with_service("reset-1".into(), &harness.service)
        .await
        .unwrap();
    assert_eq!(polled.status, "approved");

    let state = get_auth_state_with_service(&harness.service).await.unwrap();
    assert!(matches!(state, AuthStateView::ResetApprovedUnbound { .. }));

    clear_local_session_with_service(&harness.service).await.unwrap();
    let cleared = get_auth_state_with_service(&harness.service).await.unwrap();
    assert!(matches!(cleared, AuthStateView::Unauthenticated));
}

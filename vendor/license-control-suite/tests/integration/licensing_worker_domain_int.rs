use license_control_suite::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationRequest, AdminResetDecision, AuthError, DeviceFingerprint, DevicePublicKey,
    DeviceResetRequest, DeviceResetStatus, LicenseKey, PurchaseEmail, ResetRequestId, ValidationOutcome,
};
use license_control_suite::modules::user_reg::licensing_worker::{InMemoryWorkerStore, WorkerApp};

fn license() -> LicenseKey {
    LicenseKey::new("LICENSE-1234").unwrap()
}

fn activation_request(public_key: &str) -> ActivationRequest {
    ActivationRequest {
        license_key: license(),
        device_public_key: DevicePublicKey::new(public_key).unwrap(),
        fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
        app_version: "1.0.0".into(),
        timestamp_ms: 10,
    }
}

fn reset_request() -> DeviceResetRequest {
    DeviceResetRequest {
        license_key: Some(license()),
        masked_license_key: None,
        purchaser_email: PurchaseEmail::new("buyer@example.com").unwrap(),
        device_public_key: DevicePublicKey::new("public").unwrap(),
        fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
        app_version: "1.0.0".into(),
        timestamp_ms: 20,
        receipt_reference: Some("receipt".into()),
    }
}

fn app() -> WorkerApp {
    let store = InMemoryWorkerStore::default();
    store.insert_license(&license(), "buyer@example.com");
    WorkerApp::new(store, "admin-secret")
}

#[test]
fn valid_unbound_activation_binds_device() {
    let app = app();
    let outcome = app.activate(activation_request("public")).unwrap();
    assert_eq!(outcome.masked_license_key.as_str(), "••••-1234");
    assert_eq!(
        app.store
            .license(&license())
            .unwrap()
            .current_binding
            .unwrap()
            .public_key
            .as_str(),
        "public"
    );
}

#[test]
fn validation_succeeds_for_active_token_and_fails_after_reset_approval() {
    let app = app();
    let outcome = app.activate(activation_request("public")).unwrap();
    assert!(matches!(
        app.validate_session(outcome.access_token.clone()).unwrap(),
        ValidationOutcome::Active { .. }
    ));
    let status = app.request_device_reset(reset_request()).unwrap();
    let request_id = status.request_id().clone();
    app.decide_reset("admin-secret", request_id, AdminResetDecision::Approve, 30)
        .unwrap();
    assert!(matches!(
        app.validate_session(outcome.access_token).unwrap(),
        ValidationOutcome::ReauthRequired
    ));
}

#[test]
fn admin_routes_require_authorization() {
    let app = app();
    let status = app.request_device_reset(reset_request()).unwrap();
    assert_eq!(
        app.decide_reset(
            "wrong",
            status.request_id().clone(),
            AdminResetDecision::Approve,
            30
        )
        .unwrap_err(),
        AuthError::Unauthorized
    );
}

#[test]
fn reset_status_returns_not_found_for_missing_request() {
    let app = app();
    assert!(matches!(
        app.get_reset_status(ResetRequestId::new("missing").unwrap()),
        DeviceResetStatus::NotFound { .. }
    ));
}

#[test]
fn audit_events_do_not_store_raw_access_tokens() {
    let app = app();
    let token: AccessToken = app
        .activate(activation_request("public"))
        .unwrap()
        .access_token;
    let audit_json = serde_json::to_string(&app.store.audit_events()).unwrap();
    assert!(!audit_json.contains(token.expose_secret()));
}

#[test]
fn reference_worker_behavior_remains_current() {
    let app = app();
    let activated = app.activate(activation_request("public")).unwrap();
    assert!(matches!(
        app.validate_session(activated.access_token.clone()).unwrap(),
        ValidationOutcome::Active { .. }
    ));

    let pending = app.request_device_reset(reset_request()).unwrap();
    assert!(matches!(pending, DeviceResetStatus::Pending { .. }));

    let approved = app
        .decide_reset(
            "admin-secret",
            pending.request_id().clone(),
            AdminResetDecision::Approve,
            30,
        )
        .unwrap();
    assert!(matches!(approved, DeviceResetStatus::Approved { .. }));
    assert!(matches!(
        app.validate_session(activated.access_token).unwrap(),
        ValidationOutcome::ReauthRequired
    ));

    let audit_json = serde_json::to_string(&app.store.audit_events()).unwrap();
    assert!(audit_json.contains("activation_success"));
    assert!(audit_json.contains("reset_request_created"));
    assert!(audit_json.contains("reset_approved"));
}

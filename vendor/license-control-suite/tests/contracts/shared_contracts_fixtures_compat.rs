use license_control_suite::modules::shared_contracts::dto::{
    ActivateRequest, AdminAuthVerifyResponse, DeviceResetStatusResponse,
};
use license_control_suite::modules::shared_contracts::errors::ApiError;
use license_control_suite::modules::shared_contracts::events::AuditEvent;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/fixtures/shared_contracts/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(path).expect("fixture file should exist")
}

#[test]
fn activate_request_fixture_matches_contract() {
    let raw = fixture("activate_request_minimal.json");
    let parsed: ActivateRequest = serde_json::from_str(&raw).unwrap();
    let serialized = serde_json::to_value(parsed).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(serialized, expected);
}

#[test]
fn reset_status_approved_fixture_matches_contract() {
    let raw = fixture("device_reset_status_approved.json");
    let parsed: DeviceResetStatusResponse = serde_json::from_str(&raw).unwrap();
    let serialized = serde_json::to_value(parsed).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(serialized, expected);
}

#[test]
fn api_error_fixture_matches_contract() {
    let raw = fixture("api_error_license_invalid.json");
    let parsed: ApiError = serde_json::from_str(&raw).unwrap();
    let serialized = serde_json::to_value(parsed).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(serialized, expected);
}

#[test]
fn admin_verify_fixture_matches_contract() {
    let raw = fixture("admin_verify_response.json");
    let parsed: AdminAuthVerifyResponse = serde_json::from_str(&raw).unwrap();
    let serialized = serde_json::to_value(parsed).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(serialized, expected);
}

#[test]
fn audit_event_fixture_matches_contract() {
    let raw = fixture("audit_event_admin_auth_succeeded.json");
    let parsed: AuditEvent = serde_json::from_str(&raw).unwrap();
    let serialized = serde_json::to_value(parsed).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(serialized, expected);
}

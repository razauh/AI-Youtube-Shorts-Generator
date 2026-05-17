use license_control_suite::modules::shared_contracts::dto::*;
use license_control_suite::modules::shared_contracts::state::{LicenseState, ResetRequestState};

#[test]
fn activate_request_roundtrip() {
    let req = ActivateRequest {
        license_key: "XXXX-XXXX-XXXX".to_string(),
        device: DeviceMetadata {
            device_id: "dev_abc123".to_string(),
            platform: Platform::Windows,
            app_version: "1.0.0".to_string(),
        },
    };

    let json = serde_json::to_string(&req).unwrap();
    let parsed: ActivateRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.license_key, req.license_key);
    assert_eq!(parsed.device.device_id, req.device.device_id);
}

#[test]
fn reset_status_approved_message_is_stable() {
    let res = DeviceResetStatusResponse {
        reset_request_id: "rr_123".to_string(),
        status: ResetRequestState::Approved,
        license_state: LicenseState::Unbound,
        message: "Device reset approved. You can now use this license key to activate a device."
            .to_string(),
    };

    let json = serde_json::to_value(&res).unwrap();
    assert_eq!(json["status"], "APPROVED");
    assert_eq!(json["license_state"], "UNBOUND");
}

#[test]
fn admin_verify_response_contains_scopes() {
    let res = AdminAuthVerifyResponse {
        admin_access_token: "token".to_string(),
        expires_in_sec: 600,
        scopes: vec!["admin:read".to_string(), "admin:reset:write".to_string()],
    };

    let json = serde_json::to_value(&res).unwrap();
    assert_eq!(json["expires_in_sec"], 600);
    assert_eq!(json["scopes"].as_array().unwrap().len(), 2);
}

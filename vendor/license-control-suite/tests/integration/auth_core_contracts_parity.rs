use license_control_suite::modules::shared_contracts::dto::{ActivateRequest, DeviceMetadata, Platform};

#[test]
fn activation_request_uses_shared_contract_shape() {
    let req = ActivateRequest {
        license_key: "XXXX-XXXX-XXXX".into(),
        device: DeviceMetadata { device_id: "dev_1".into(), platform: Platform::Windows, app_version: "1.0.0".into() },
    };

    let json = serde_json::to_value(req).unwrap();
    assert!(json.get("license_key").is_some());
    assert!(json.get("device").is_some());
}

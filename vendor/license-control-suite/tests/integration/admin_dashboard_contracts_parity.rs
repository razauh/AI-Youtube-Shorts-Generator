use license_control_suite::modules::shared_contracts::dto::AdminAuthVerifyResponse;

#[test]
fn admin_verify_contract_shape_is_stable() {
    let dto = AdminAuthVerifyResponse {
        admin_access_token: "tok".into(),
        expires_in_sec: 600,
        scopes: vec!["admin:read".into(), "admin:reset:write".into()],
    };

    let json = serde_json::to_value(dto).unwrap();
    assert!(json.get("admin_access_token").is_some());
    assert!(json.get("expires_in_sec").is_some());
    assert!(json.get("scopes").is_some());
}

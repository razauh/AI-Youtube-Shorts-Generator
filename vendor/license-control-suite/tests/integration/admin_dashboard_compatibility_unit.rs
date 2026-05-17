use license_control_suite::modules::admin_dashboard::compatibility::{
    assert_shared_contracts_compatibility, supported_shared_contracts_range,
};

#[test]
fn exposes_supported_contract_range() {
    let r = supported_shared_contracts_range();
    assert_eq!(r.min, "1.0.0");
    assert_eq!(r.max_exclusive, "2.0.0");
}

#[test]
fn compatibility_guard_accepts_current_contract_major() {
    assert!(assert_shared_contracts_compatibility().is_ok());
}

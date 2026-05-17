use license_control_suite::modules::admin_dashboard::authz::{can_read, can_write_reset};

#[test]
fn read_scope_allows_read() {
    let scopes = vec!["admin:read".to_string()];
    assert!(can_read(&scopes));
}

#[test]
fn write_scope_required_for_reset_write() {
    let scopes = vec!["admin:read".to_string()];
    assert!(!can_write_reset(&scopes));

    let scopes2 = vec!["admin:reset:write".to_string()];
    assert!(can_write_reset(&scopes2));
}

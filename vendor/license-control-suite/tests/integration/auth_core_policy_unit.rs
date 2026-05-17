use license_control_suite::modules::auth_core::policy::{offline_access, OfflineDecision};

#[test]
fn offline_within_grace_is_allowed() {
    let d = offline_access(1_000_000, 1_000_000 + 3600, 72 * 3600);
    assert_eq!(d, OfflineDecision::Allowed);
}

#[test]
fn offline_beyond_grace_is_denied() {
    let d = offline_access(1_000_000, 1_000_000 + (73 * 3600), 72 * 3600);
    assert!(matches!(d, OfflineDecision::Denied { .. }));
}

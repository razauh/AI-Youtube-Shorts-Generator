use license_control_suite::modules::auth_core::session::{should_force_reauth, SessionState};

#[test]
fn reauth_required_when_epoch_mismatch() {
    let s = SessionState { binding_epoch: 2, token_epoch: 1 };
    assert!(should_force_reauth(&s));
}

#[test]
fn no_reauth_when_epoch_matches() {
    let s = SessionState { binding_epoch: 2, token_epoch: 2 };
    assert!(!should_force_reauth(&s));
}

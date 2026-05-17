use license_control_suite::modules::admin_dashboard::realtime::{backoff_step_sec, should_apply_delta};

#[test]
fn backoff_sequence_is_expected() {
    let seq = [
        backoff_step_sec(0),
        backoff_step_sec(1),
        backoff_step_sec(2),
        backoff_step_sec(3),
        backoff_step_sec(4),
        backoff_step_sec(5),
    ];
    assert_eq!(seq, [1, 2, 5, 10, 30, 30]);
}

#[test]
fn deltas_blocked_until_resync() {
    assert!(!should_apply_delta(false));
    assert!(should_apply_delta(true));
}

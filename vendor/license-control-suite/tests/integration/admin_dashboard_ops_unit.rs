use license_control_suite::modules::admin_dashboard::ops::{compute_health, HealthInput};

#[test]
fn health_marks_stale_when_threshold_exceeded() {
    let input = HealthInput {
        pending_count: 3,
        now_epoch_sec: 1_000,
        last_event_epoch_sec: 900,
        stale_threshold_sec: 60,
    };
    let out = compute_health(input);
    assert!(out.is_stale);
}

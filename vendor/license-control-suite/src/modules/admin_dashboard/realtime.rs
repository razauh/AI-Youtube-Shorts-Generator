pub fn backoff_step_sec(attempt: u32) -> u64 {
    match attempt {
        0 => 1,
        1 => 2,
        2 => 5,
        3 => 10,
        _ => 30,
    }
}

pub fn should_apply_delta(has_completed_resync: bool) -> bool {
    has_completed_resync
}

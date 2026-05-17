#[derive(Debug, Clone, Copy)]
pub struct HealthInput {
    pub pending_count: u64,
    pub now_epoch_sec: u64,
    pub last_event_epoch_sec: u64,
    pub stale_threshold_sec: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthOutput {
    pub pending_count: u64,
    pub is_stale: bool,
}

pub fn compute_health(input: HealthInput) -> HealthOutput {
    let elapsed = input.now_epoch_sec.saturating_sub(input.last_event_epoch_sec);
    HealthOutput {
        pending_count: input.pending_count,
        is_stale: elapsed > input.stale_threshold_sec,
    }
}

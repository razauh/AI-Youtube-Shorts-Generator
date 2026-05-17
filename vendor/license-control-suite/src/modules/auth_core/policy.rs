#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfflineDecision {
    Allowed,
    Denied { reason: String },
}

pub fn offline_access(last_success_epoch_sec: u64, now_epoch_sec: u64, grace_window_sec: u64) -> OfflineDecision {
    if now_epoch_sec.saturating_sub(last_success_epoch_sec) <= grace_window_sec {
        OfflineDecision::Allowed
    } else {
        OfflineDecision::Denied {
            reason: "offline grace expired".to_string(),
        }
    }
}

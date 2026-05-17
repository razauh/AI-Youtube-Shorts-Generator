#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionState {
    pub binding_epoch: u64,
    pub token_epoch: u64,
}

pub fn should_force_reauth(state: &SessionState) -> bool {
    state.token_epoch != state.binding_epoch
}

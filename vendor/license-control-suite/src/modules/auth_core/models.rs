#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSession {
    pub license_id: String,
    pub access_token: String,
    pub expires_in_sec: u64,
    pub binding_epoch: u64,
    pub token_epoch: u64,
}

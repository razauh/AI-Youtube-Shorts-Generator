use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseState {
    Unbound,
    BoundActive,
    ResetPending,
    Suspended,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResetRequestState {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Expired,
    Cancelled,
}

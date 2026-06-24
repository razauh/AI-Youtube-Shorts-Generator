use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditEventType {
    ActivationSucceeded,
    ActivationDenied,
    SessionRenewed,
    ResetRequestSubmitted,
    ResetRequestUnderReview,
    ResetApproved,
    ResetRejected,
    AdminAuthChallengeIssued,
    AdminAuthSucceeded,
    AdminAuthFailed,
    LicenseRevoked,
    LicenseSuspended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Admin,
    System,
    Customer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "type")]
    pub actor_type: ActorType,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub ip: Option<String>,
    pub app_version: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub occurred_at: String,
    pub actor: Actor,
    pub license_id: Option<String>,
    pub reset_request_id: Option<String>,
    pub old_device_id: Option<String>,
    pub new_device_id: Option<String>,
    pub request_id: String,
    pub metadata: EventMetadata,
}

use serde::{Deserialize, Serialize};

use super::state::{LicenseState, ResetRequestState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub device_id: String,
    pub platform: Platform,
    pub app_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivateRequest {
    pub license_key: String,
    pub device: DeviceMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivateResponse {
    pub license_id: String,
    pub state: LicenseState,
    pub access_token: String,
    pub expires_in_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRenewResponse {
    pub access_token: String,
    pub expires_in_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceResetRequest {
    pub license_key: String,
    pub purchaser_email: String,
    pub device: DeviceMetadata,
    pub order_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceResetRequestAccepted {
    pub reset_request_id: String,
    pub status: ResetRequestState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceResetStatusResponse {
    pub reset_request_id: String,
    pub status: ResetRequestState,
    pub license_state: LicenseState,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAuthChallengeRequest {
    pub operator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAuthChallengeResponse {
    pub challenge_id: String,
    pub nonce: String,
    pub expires_in_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAuthVerifyRequest {
    pub operator_id: String,
    pub challenge_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAuthVerifyResponse {
    pub admin_access_token: String,
    pub expires_in_sec: u64,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminResetDecisionResponse {
    pub reset_request_id: String,
    pub status: ResetRequestState,
    pub license_state: LicenseState,
}

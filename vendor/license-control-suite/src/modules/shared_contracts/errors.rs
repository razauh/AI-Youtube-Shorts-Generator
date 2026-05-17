use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    LicenseInvalid,
    LicenseRevoked,
    LicenseSuspended,
    LicenseAlreadyBound,
    SessionReauthRequired,
    ResetRequestNotFound,
    ResetRequestConflict,
    AdminAuthInvalid,
    AdminForbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: ErrorBody,
    pub request_id: String,
}

impl ApiError {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        retryable: bool,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            error: ErrorBody {
                code,
                message: message.into(),
                retryable,
            },
            request_id: request_id.into(),
        }
    }
}

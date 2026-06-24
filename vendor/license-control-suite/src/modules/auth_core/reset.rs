use thiserror::Error;

use super::adapters::{ApiClient, LocalStore};
use crate::modules::shared_contracts::dto::{DeviceResetRequest, DeviceResetRequestAccepted};
use crate::modules::shared_contracts::state::ResetRequestState;

#[derive(Debug, Error)]
pub enum ResetError {
    #[error("remote error: {0}")]
    Remote(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("poll attempts exhausted")]
    PollAttemptsExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollStatus {
    Continue,
    TerminalApproved,
    TerminalRejected,
    TerminalExpired,
}

pub fn submit_reset_request(
    api: &dyn ApiClient,
    store: &mut dyn LocalStore,
    req: &DeviceResetRequest,
    idempotency_key: &str,
) -> Result<DeviceResetRequestAccepted, ResetError> {
    let accepted = api
        .submit_reset(req, idempotency_key)
        .map_err(|e| ResetError::Remote(e.error.message))?;

    store
        .save_reset_request_id(accepted.reset_request_id.clone())
        .map_err(ResetError::Storage)?;

    Ok(accepted)
}

pub fn poll_until_terminal(
    api: &dyn ApiClient,
    reset_request_id: &str,
    max_attempts: usize,
) -> Result<PollStatus, ResetError> {
    for _ in 0..max_attempts {
        let status = api
            .reset_status(reset_request_id)
            .map_err(|e| ResetError::Remote(e.error.message))?;

        let mapped = match status.status {
            ResetRequestState::Approved => PollStatus::TerminalApproved,
            ResetRequestState::Rejected => PollStatus::TerminalRejected,
            ResetRequestState::Expired => PollStatus::TerminalExpired,
            _ => PollStatus::Continue,
        };

        if mapped != PollStatus::Continue {
            return Ok(mapped);
        }
    }

    Err(ResetError::PollAttemptsExhausted)
}

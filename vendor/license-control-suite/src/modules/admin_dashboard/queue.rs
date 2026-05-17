use thiserror::Error;

use crate::modules::shared_contracts::dto::AdminResetDecisionResponse;

use super::adapters::AdminApi;

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("remote error: {0}")]
    Remote(String),
}

pub fn approve(
    api: &dyn AdminApi,
    reset_request_id: &str,
    idempotency_key: &str,
) -> Result<AdminResetDecisionResponse, QueueError> {
    api.approve_reset(reset_request_id, idempotency_key)
        .map_err(|e| QueueError::Remote(e.error.message))
}

pub fn reject(
    api: &dyn AdminApi,
    reset_request_id: &str,
    idempotency_key: &str,
) -> Result<AdminResetDecisionResponse, QueueError> {
    api.reject_reset(reset_request_id, idempotency_key)
        .map_err(|e| QueueError::Remote(e.error.message))
}

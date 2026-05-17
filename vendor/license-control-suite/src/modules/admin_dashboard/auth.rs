use thiserror::Error;

use crate::modules::shared_contracts::dto::{AdminAuthChallengeRequest, AdminAuthVerifyRequest};

use super::adapters::{AdminApi, SessionStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSession {
    pub operator_id: String,
    pub admin_access_token: String,
    pub expires_in_sec: u64,
    pub scopes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("remote error: {0}")]
    Remote(String),
    #[error("storage error: {0}")]
    Storage(String),
}

pub fn login_with_challenge(
    api: &dyn AdminApi,
    store: &mut dyn SessionStore,
    operator_id: &str,
    signature: &str,
) -> Result<AuthSession, AuthError> {
    let ch = api
        .get_challenge(&AdminAuthChallengeRequest {
            operator_id: operator_id.to_string(),
        })
        .map_err(|e| AuthError::Remote(e.error.message))?;

    let verified = api
        .verify_challenge(&AdminAuthVerifyRequest {
            operator_id: operator_id.to_string(),
            challenge_id: ch.challenge_id,
            signature: signature.to_string(),
        })
        .map_err(|e| AuthError::Remote(e.error.message))?;

    let session = AuthSession {
        operator_id: operator_id.to_string(),
        admin_access_token: verified.admin_access_token,
        expires_in_sec: verified.expires_in_sec,
        scopes: verified.scopes,
    };

    store
        .save_session(session.clone())
        .map_err(AuthError::Storage)?;

    Ok(session)
}

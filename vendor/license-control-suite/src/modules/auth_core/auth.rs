use thiserror::Error;

use crate::modules::shared_contracts::dto::{ActivateRequest, ActivateResponse, SessionRenewResponse};
use crate::modules::shared_contracts::errors::{ApiError, ErrorCode};

use super::adapters::{ApiClient, LocalStore};
use super::models::LocalSession;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("license invalid")]
    LicenseInvalid,
    #[error("license revoked")]
    LicenseRevoked,
    #[error("license suspended")]
    LicenseSuspended,
    #[error("license already bound")]
    LicenseAlreadyBound,
    #[error("reauth required")]
    SessionReauthRequired,
    #[error("storage failure: {0}")]
    Storage(String),
    #[error("remote error: {0}")]
    Remote(String),
}

pub fn activate(
    api: &dyn ApiClient,
    store: &mut dyn LocalStore,
    req: &ActivateRequest,
    idempotency_key: &str,
) -> Result<ActivateResponse, AuthError> {
    let res = api.activate(req, idempotency_key).map_err(map_api_error)?;

    let session = LocalSession {
        license_id: res.license_id.clone(),
        access_token: res.access_token.clone(),
        expires_in_sec: res.expires_in_sec,
        binding_epoch: 1,
        token_epoch: 1,
    };

    store.save_session(session).map_err(AuthError::Storage)?;
    Ok(res)
}

pub fn renew(
    api: &dyn ApiClient,
    store: &mut dyn LocalStore,
    access_token: &str,
) -> Result<SessionRenewResponse, AuthError> {
    let renewed = api.renew_session(access_token).map_err(map_api_error)?;

    if let Some(mut session) = store.load_session() {
        session.access_token = renewed.access_token.clone();
        session.expires_in_sec = renewed.expires_in_sec;
        session.token_epoch = session.binding_epoch;
        store.save_session(session).map_err(AuthError::Storage)?;
    }

    Ok(renewed)
}

fn map_api_error(e: ApiError) -> AuthError {
    match e.error.code {
        ErrorCode::LicenseInvalid => AuthError::LicenseInvalid,
        ErrorCode::LicenseRevoked => AuthError::LicenseRevoked,
        ErrorCode::LicenseSuspended => AuthError::LicenseSuspended,
        ErrorCode::LicenseAlreadyBound => AuthError::LicenseAlreadyBound,
        ErrorCode::SessionReauthRequired => AuthError::SessionReauthRequired,
        _ => AuthError::Remote(e.error.message),
    }
}

use crate::modules::shared_contracts::dto::{
    AdminAuthChallengeRequest, AdminAuthChallengeResponse, AdminAuthVerifyRequest,
    AdminAuthVerifyResponse, AdminResetDecisionResponse, DeviceResetStatusResponse,
};
use crate::modules::shared_contracts::errors::ApiError;

use super::auth::AuthSession;

pub trait AdminApi {
    fn get_challenge(&self, req: &AdminAuthChallengeRequest) -> Result<AdminAuthChallengeResponse, ApiError>;
    fn verify_challenge(&self, req: &AdminAuthVerifyRequest) -> Result<AdminAuthVerifyResponse, ApiError>;

    fn list_pending_resets(&self) -> Result<Vec<DeviceResetStatusResponse>, ApiError>;
    fn approve_reset(&self, reset_request_id: &str, idempotency_key: &str) -> Result<AdminResetDecisionResponse, ApiError>;
    fn reject_reset(&self, reset_request_id: &str, idempotency_key: &str) -> Result<AdminResetDecisionResponse, ApiError>;
}

pub trait SessionStore {
    fn save_session(&mut self, session: AuthSession) -> Result<(), String>;
    fn load_session(&self) -> Option<AuthSession>;
}

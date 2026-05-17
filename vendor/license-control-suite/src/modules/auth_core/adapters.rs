use crate::modules::shared_contracts::dto::{
    ActivateRequest, ActivateResponse, DeviceResetRequest, DeviceResetRequestAccepted,
    DeviceResetStatusResponse, SessionRenewResponse,
};
use crate::modules::shared_contracts::errors::ApiError;

use super::models::LocalSession;

pub trait ApiClient {
    fn activate(&self, req: &ActivateRequest, idempotency_key: &str) -> Result<ActivateResponse, ApiError>;
    fn renew_session(&self, access_token: &str) -> Result<SessionRenewResponse, ApiError>;
    fn submit_reset(
        &self,
        req: &DeviceResetRequest,
        idempotency_key: &str,
    ) -> Result<DeviceResetRequestAccepted, ApiError>;
    fn reset_status(&self, reset_request_id: &str) -> Result<DeviceResetStatusResponse, ApiError>;
}

pub trait LocalStore {
    fn load_session(&self) -> Option<LocalSession>;
    fn save_session(&mut self, session: LocalSession) -> Result<(), String>;
    fn save_reset_request_id(&mut self, reset_request_id: String) -> Result<(), String>;
    fn load_reset_request_id(&self) -> Option<String>;
}

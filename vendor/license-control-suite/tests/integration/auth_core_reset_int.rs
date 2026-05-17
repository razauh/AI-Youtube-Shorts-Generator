use license_control_suite::modules::auth_core::adapters::{ApiClient, LocalStore};
use license_control_suite::modules::auth_core::models::LocalSession;
use license_control_suite::modules::auth_core::reset::PollStatus;
use license_control_suite::modules::auth_core::reset::{poll_until_terminal, submit_reset_request};
use license_control_suite::modules::shared_contracts::dto::{
    ActivateRequest, ActivateResponse, DeviceMetadata, DeviceResetRequest, DeviceResetRequestAccepted,
    DeviceResetStatusResponse, Platform, SessionRenewResponse,
};
use license_control_suite::modules::shared_contracts::errors::ApiError;
use license_control_suite::modules::shared_contracts::state::{LicenseState, ResetRequestState};

#[derive(Default)]
struct FakeStore {
    rrid: Option<String>,
}
impl LocalStore for FakeStore {
    fn load_session(&self) -> Option<LocalSession> { None }
    fn save_session(&mut self, _session: LocalSession) -> Result<(), String> { Ok(()) }
    fn save_reset_request_id(&mut self, reset_request_id: String) -> Result<(), String> { self.rrid = Some(reset_request_id); Ok(()) }
    fn load_reset_request_id(&self) -> Option<String> { self.rrid.clone() }
}

struct FakeApi;
impl ApiClient for FakeApi {
    fn activate(&self, _req: &ActivateRequest, _idempotency_key: &str) -> Result<ActivateResponse, ApiError> { unimplemented!() }
    fn renew_session(&self, _access_token: &str) -> Result<SessionRenewResponse, ApiError> { unimplemented!() }
    fn submit_reset(&self, _req: &DeviceResetRequest, _idempotency_key: &str) -> Result<DeviceResetRequestAccepted, ApiError> {
        Ok(DeviceResetRequestAccepted { reset_request_id: "rr_123".into(), status: ResetRequestState::Submitted })
    }
    fn reset_status(&self, _id: &str) -> Result<DeviceResetStatusResponse, ApiError> {
        Ok(DeviceResetStatusResponse {
            reset_request_id: "rr_123".into(),
            status: ResetRequestState::Approved,
            license_state: LicenseState::Unbound,
            message: "Device reset approved. You can now use this license key to activate a device.".into(),
        })
    }
}

#[test]
fn submit_reset_persists_request_id() {
    let api = FakeApi;
    let mut store = FakeStore::default();
    let req = DeviceResetRequest {
        license_key: "XXXX-XXXX-XXXX".into(),
        purchaser_email: "u@example.com".into(),
        device: DeviceMetadata { device_id: "dev_1".into(), platform: Platform::Windows, app_version: "1.0.0".into() },
        order_ref: None,
    };

    let accepted = submit_reset_request(&api, &mut store, &req, "idem_2").unwrap();
    assert_eq!(accepted.reset_request_id, "rr_123");
    assert_eq!(store.load_reset_request_id().as_deref(), Some("rr_123"));
}

#[test]
fn polling_stops_on_terminal_state() {
    let api = FakeApi;
    let out = poll_until_terminal(&api, "rr_123", 3).unwrap();
    assert_eq!(out, PollStatus::TerminalApproved);
}

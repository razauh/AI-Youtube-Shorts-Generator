use license_control_suite::modules::auth_core::adapters::{ApiClient, LocalStore};
use license_control_suite::modules::auth_core::auth::{activate, AuthError};
use license_control_suite::modules::auth_core::models::LocalSession;
use license_control_suite::modules::shared_contracts::dto::{
    ActivateRequest, ActivateResponse, DeviceMetadata, DeviceResetRequest, DeviceResetRequestAccepted,
    DeviceResetStatusResponse, Platform, SessionRenewResponse,
};
use license_control_suite::modules::shared_contracts::errors::{ApiError, ErrorCode};
use license_control_suite::modules::shared_contracts::state::LicenseState;

#[derive(Default)]
struct FakeStore {
    saved: Option<LocalSession>,
}

impl LocalStore for FakeStore {
    fn load_session(&self) -> Option<LocalSession> { None }
    fn save_session(&mut self, session: LocalSession) -> Result<(), String> {
        self.saved = Some(session);
        Ok(())
    }
    fn save_reset_request_id(&mut self, _reset_request_id: String) -> Result<(), String> { Ok(()) }
    fn load_reset_request_id(&self) -> Option<String> { None }
}

struct FakeApiOk;
impl ApiClient for FakeApiOk {
    fn activate(&self, _req: &ActivateRequest, _idempotency_key: &str) -> Result<ActivateResponse, ApiError> {
        Ok(ActivateResponse {
            license_id: "lic_123".into(),
            state: LicenseState::BoundActive,
            access_token: "tok".into(),
            expires_in_sec: 900,
        })
    }
    fn renew_session(&self, _access_token: &str) -> Result<SessionRenewResponse, ApiError> { unimplemented!() }
    fn submit_reset(&self, _req: &DeviceResetRequest, _idempotency_key: &str) -> Result<DeviceResetRequestAccepted, ApiError> { unimplemented!() }
    fn reset_status(&self, _id: &str) -> Result<DeviceResetStatusResponse, ApiError> { unimplemented!() }
}

struct FakeApiInvalid;
impl ApiClient for FakeApiInvalid {
    fn activate(&self, _req: &ActivateRequest, _idempotency_key: &str) -> Result<ActivateResponse, ApiError> {
        Err(ApiError::new(ErrorCode::LicenseInvalid, "bad", false, "req_1"))
    }
    fn renew_session(&self, _access_token: &str) -> Result<SessionRenewResponse, ApiError> { unimplemented!() }
    fn submit_reset(&self, _req: &DeviceResetRequest, _idempotency_key: &str) -> Result<DeviceResetRequestAccepted, ApiError> { unimplemented!() }
    fn reset_status(&self, _id: &str) -> Result<DeviceResetStatusResponse, ApiError> { unimplemented!() }
}

#[test]
fn activation_success_saves_session() {
    let api = FakeApiOk;
    let mut store = FakeStore::default();
    let req = ActivateRequest {
        license_key: "XXXX-XXXX-XXXX".into(),
        device: DeviceMetadata { device_id: "dev_1".into(), platform: Platform::Windows, app_version: "1.0.0".into() },
    };

    let res = activate(&api, &mut store, &req, "idem_1").unwrap();
    assert_eq!(res.license_id, "lic_123");
    assert!(store.saved.is_some());
}

#[test]
fn activation_invalid_maps_error() {
    let api = FakeApiInvalid;
    let mut store = FakeStore::default();
    let req = ActivateRequest {
        license_key: "XXXX-XXXX-XXXX".into(),
        device: DeviceMetadata { device_id: "dev_1".into(), platform: Platform::Windows, app_version: "1.0.0".into() },
    };

    let err = activate(&api, &mut store, &req, "idem_1").unwrap_err();
    assert!(matches!(err, AuthError::LicenseInvalid));
}

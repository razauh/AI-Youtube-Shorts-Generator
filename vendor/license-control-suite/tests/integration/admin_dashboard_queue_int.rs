use license_control_suite::modules::admin_dashboard::adapters::AdminApi;
use license_control_suite::modules::admin_dashboard::queue::{approve, reject};
use license_control_suite::modules::shared_contracts::dto::{
    AdminResetDecisionResponse, DeviceResetStatusResponse,
};
use license_control_suite::modules::shared_contracts::errors::ApiError;
use license_control_suite::modules::shared_contracts::state::{LicenseState, ResetRequestState};

struct FakeApi;

impl AdminApi for FakeApi {
    fn get_challenge(
        &self,
        _req: &license_control_suite::modules::shared_contracts::dto::AdminAuthChallengeRequest,
    ) -> Result<license_control_suite::modules::shared_contracts::dto::AdminAuthChallengeResponse, ApiError>
    {
        unimplemented!()
    }

    fn verify_challenge(
        &self,
        _req: &license_control_suite::modules::shared_contracts::dto::AdminAuthVerifyRequest,
    ) -> Result<license_control_suite::modules::shared_contracts::dto::AdminAuthVerifyResponse, ApiError>
    {
        unimplemented!()
    }

    fn list_pending_resets(&self) -> Result<Vec<DeviceResetStatusResponse>, ApiError> {
        Ok(vec![])
    }

    fn approve_reset(
        &self,
        id: &str,
        _idempotency_key: &str,
    ) -> Result<AdminResetDecisionResponse, ApiError> {
        Ok(AdminResetDecisionResponse {
            reset_request_id: id.to_string(),
            status: ResetRequestState::Approved,
            license_state: LicenseState::Unbound,
        })
    }

    fn reject_reset(
        &self,
        id: &str,
        _idempotency_key: &str,
    ) -> Result<AdminResetDecisionResponse, ApiError> {
        Ok(AdminResetDecisionResponse {
            reset_request_id: id.to_string(),
            status: ResetRequestState::Rejected,
            license_state: LicenseState::BoundActive,
        })
    }
}

#[test]
fn approve_maps_expected_state() {
    let api = FakeApi;
    let out = approve(&api, "rr_1", "idem_1").unwrap();
    assert_eq!(out.status, ResetRequestState::Approved);
}

#[test]
fn reject_maps_expected_state() {
    let api = FakeApi;
    let out = reject(&api, "rr_1", "idem_2").unwrap();
    assert_eq!(out.status, ResetRequestState::Rejected);
}

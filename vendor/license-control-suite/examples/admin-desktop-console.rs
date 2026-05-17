use license_control_suite::desktop::admin::{
    adapters::{AdminApi, SessionStore},
    auth::{login_with_challenge, AuthError, AuthSession},
};
use license_control_suite::modules::shared_contracts::{
    dto::{
        AdminAuthChallengeRequest, AdminAuthChallengeResponse, AdminAuthVerifyRequest,
        AdminAuthVerifyResponse, AdminResetDecisionResponse, DeviceResetStatusResponse,
    },
    errors::ApiError,
    state::{LicenseState, ResetRequestState},
};

struct ExampleAdminApi;

impl AdminApi for ExampleAdminApi {
    fn get_challenge(
        &self,
        req: &AdminAuthChallengeRequest,
    ) -> Result<AdminAuthChallengeResponse, ApiError> {
        Ok(AdminAuthChallengeResponse {
            challenge_id: format!("challenge-for-{}", req.operator_id),
            nonce: "desktop-only-nonce".into(),
            expires_in_sec: 300,
        })
    }

    fn verify_challenge(
        &self,
        req: &AdminAuthVerifyRequest,
    ) -> Result<AdminAuthVerifyResponse, ApiError> {
        if req.signature.is_empty() {
            return Err(ApiError::new(
                license_control_suite::modules::shared_contracts::errors::ErrorCode::AdminAuthInvalid,
                "missing signature",
                false,
                "req-1",
            ));
        }

        Ok(AdminAuthVerifyResponse {
            admin_access_token: "desktop-admin-token".into(),
            expires_in_sec: 3600,
            scopes: vec!["reset:read".into(), "reset:write".into()],
        })
    }

    fn list_pending_resets(&self) -> Result<Vec<DeviceResetStatusResponse>, ApiError> {
        Ok(vec![DeviceResetStatusResponse {
            reset_request_id: "reset-1".into(),
            status: ResetRequestState::UnderReview,
            license_state: LicenseState::BoundActive,
            message: "pending".into(),
        }])
    }

    fn approve_reset(
        &self,
        reset_request_id: &str,
        _idempotency_key: &str,
    ) -> Result<AdminResetDecisionResponse, ApiError> {
        Ok(AdminResetDecisionResponse {
            reset_request_id: reset_request_id.into(),
            status: ResetRequestState::Approved,
            license_state: LicenseState::BoundActive,
        })
    }

    fn reject_reset(
        &self,
        reset_request_id: &str,
        _idempotency_key: &str,
    ) -> Result<AdminResetDecisionResponse, ApiError> {
        Ok(AdminResetDecisionResponse {
            reset_request_id: reset_request_id.into(),
            status: ResetRequestState::Rejected,
            license_state: LicenseState::BoundActive,
        })
    }
}

#[derive(Default)]
struct ExampleSessionStore {
    session: Option<AuthSession>,
}

impl SessionStore for ExampleSessionStore {
    fn save_session(&mut self, session: AuthSession) -> Result<(), String> {
        self.session = Some(session);
        Ok(())
    }

    fn load_session(&self) -> Option<AuthSession> {
        self.session.clone()
    }
}

pub fn run_admin_login_flow() -> Result<AuthSession, AuthError> {
    let api = ExampleAdminApi;
    let mut store = ExampleSessionStore::default();

    login_with_challenge(&api, &mut store, "operator-1", "signed-challenge")
}

fn main() {}

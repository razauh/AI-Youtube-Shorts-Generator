use license_control_suite::modules::admin_dashboard::adapters::{AdminApi, SessionStore};
use license_control_suite::modules::admin_dashboard::auth::{login_with_challenge, AuthSession};
use license_control_suite::modules::shared_contracts::dto::{
    AdminAuthChallengeRequest, AdminAuthChallengeResponse, AdminAuthVerifyRequest,
    AdminAuthVerifyResponse,
};
use license_control_suite::modules::shared_contracts::errors::ApiError;

#[derive(Default)]
struct MemStore {
    session: Option<AuthSession>,
}

impl SessionStore for MemStore {
    fn save_session(&mut self, session: AuthSession) -> Result<(), String> {
        self.session = Some(session);
        Ok(())
    }

    fn load_session(&self) -> Option<AuthSession> {
        self.session.clone()
    }
}

struct FakeApi;

impl AdminApi for FakeApi {
    fn get_challenge(
        &self,
        _req: &AdminAuthChallengeRequest,
    ) -> Result<AdminAuthChallengeResponse, ApiError> {
        Ok(AdminAuthChallengeResponse {
            challenge_id: "ch_1".into(),
            nonce: "n".into(),
            expires_in_sec: 60,
        })
    }

    fn verify_challenge(
        &self,
        _req: &AdminAuthVerifyRequest,
    ) -> Result<AdminAuthVerifyResponse, ApiError> {
        Ok(AdminAuthVerifyResponse {
            admin_access_token: "tok".into(),
            expires_in_sec: 600,
            scopes: vec!["admin:read".into(), "admin:reset:write".into()],
        })
    }

    fn list_pending_resets(
        &self,
    ) -> Result<Vec<license_control_suite::modules::shared_contracts::dto::DeviceResetStatusResponse>, ApiError>
    {
        unimplemented!()
    }

    fn approve_reset(
        &self,
        _id: &str,
        _idempotency_key: &str,
    ) -> Result<license_control_suite::modules::shared_contracts::dto::AdminResetDecisionResponse, ApiError>
    {
        unimplemented!()
    }

    fn reject_reset(
        &self,
        _id: &str,
        _idempotency_key: &str,
    ) -> Result<license_control_suite::modules::shared_contracts::dto::AdminResetDecisionResponse, ApiError>
    {
        unimplemented!()
    }
}

#[test]
fn login_challenge_verify_persists_session() {
    let api = FakeApi;
    let mut store = MemStore::default();
    let session = login_with_challenge(&api, &mut store, "op_1", "sig").unwrap();
    assert_eq!(session.operator_id, "op_1");
    assert!(store.load_session().is_some());
}

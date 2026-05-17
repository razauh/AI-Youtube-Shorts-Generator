//! Local/native reference backend for licensing flows.
//!
//! This module is useful for local testing and route-contract reasoning. It is
//! not a verified Cloudflare runtime target, does not implement Gumroad
//! provider integration, and does not provide durable production backend
//! storage.

use crate::modules::user_reg::auth_licensing_core::{
    AccessToken, ActivationOutcome, ActivationRequest, AdminResetDecision, AuthError,
    BoundDeviceSummary, DeviceId, DevicePublicKey, DeviceResetRequest, DeviceResetStatus,
    EntitlementStatus, LicenseKey, MaskedLicenseKey, ResetRequestId, ValidationOutcome,
};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaserIdentity {
    pub email: String,
    pub gumroad_sale_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseRecord {
    pub license_key_hash: String,
    pub masked_license_key: MaskedLicenseKey,
    pub purchaser: PurchaserIdentity,
    pub current_binding: Option<DeviceBinding>,
    pub entitlement: EntitlementStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceBinding {
    pub device_id: DeviceId,
    pub public_key: DevicePublicKey,
    pub fingerprint: crate::modules::user_reg::auth_licensing_core::DeviceFingerprint,
    pub activated_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredResetRequest {
    pub request_id: ResetRequestId,
    pub license_key_hash: Option<String>,
    pub masked_license_key: Option<MaskedLicenseKey>,
    pub purchaser_email: String,
    pub device_public_key: DevicePublicKey,
    pub fingerprint: crate::modules::user_reg::auth_licensing_core::DeviceFingerprint,
    pub app_version: String,
    pub receipt_reference: Option<String>,
    pub status: DeviceResetStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub kind: String,
    pub license_key_hash: Option<String>,
    pub reset_request_id: Option<String>,
    pub timestamp_ms: i64,
}

#[derive(Clone, Default)]
pub struct InMemoryWorkerStore {
    licenses: Arc<Mutex<HashMap<String, LicenseRecord>>>,
    resets: Arc<Mutex<HashMap<String, StoredResetRequest>>>,
    tokens: Arc<Mutex<HashMap<String, String>>>,
    audit: Arc<Mutex<Vec<AuditEvent>>>,
}

impl InMemoryWorkerStore {
    pub fn insert_license(&self, license_key: &LicenseKey, purchaser_email: &str) {
        let hash = hash_secret(license_key.expose_secret());
        self.licenses.lock().unwrap().insert(
            hash.clone(),
            LicenseRecord {
                license_key_hash: hash,
                masked_license_key: license_key.masked(),
                purchaser: PurchaserIdentity {
                    email: purchaser_email.to_string(),
                    gumroad_sale_id: None,
                },
                current_binding: None,
                entitlement: EntitlementStatus::Active,
            },
        );
    }

    pub fn license(&self, license_key: &LicenseKey) -> Option<LicenseRecord> {
        self.licenses
            .lock()
            .unwrap()
            .get(&hash_secret(license_key.expose_secret()))
            .cloned()
    }

    pub fn audit_events(&self) -> Vec<AuditEvent> {
        self.audit.lock().unwrap().clone()
    }

    fn append_audit(&self, event: AuditEvent) {
        self.audit.lock().unwrap().push(event);
    }
}

#[derive(Clone)]
pub struct WorkerApp {
    store: InMemoryWorkerStore,
    admin_token: String,
}

impl WorkerApp {
    pub fn new(store: InMemoryWorkerStore, admin_token: impl Into<String>) -> Self {
        Self {
            store,
            admin_token: admin_token.into(),
        }
    }

    pub fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        let license_hash = hash_secret(request.license_key.expose_secret());
        let mut licenses = self.store.licenses.lock().unwrap();
        let record = licenses
            .get_mut(&license_hash)
            .ok_or(AuthError::InvalidLicenseKey)?;

        if record.entitlement != EntitlementStatus::Active {
            return Err(AuthError::InvalidLicenseKey);
        }

        let incoming_device_id = DeviceId::from_public_key(&request.device_public_key);
        if let Some(binding) = &record.current_binding {
            if binding.device_id != incoming_device_id {
                self.store.append_audit(AuditEvent {
                    kind: "activation_device_bound_failure".into(),
                    license_key_hash: Some(license_hash),
                    reset_request_id: None,
                    timestamp_ms: request.timestamp_ms,
                });
                return Err(AuthError::DeviceAlreadyBound);
            }
        }

        let binding = DeviceBinding {
            device_id: incoming_device_id.clone(),
            public_key: request.device_public_key.clone(),
            fingerprint: request.fingerprint.clone(),
            activated_at_ms: request.timestamp_ms,
        };
        record.current_binding = Some(binding.clone());
        let token = issue_token(
            &license_hash,
            binding.device_id.as_str(),
            request.timestamp_ms,
        );
        self.store
            .tokens
            .lock()
            .unwrap()
            .insert(token.expose_secret().to_string(), license_hash.clone());
        let outcome = ActivationOutcome {
            access_token: token,
            masked_license_key: record.masked_license_key.clone(),
            bound_device: BoundDeviceSummary {
                device_id: binding.device_id,
                public_key: binding.public_key,
                fingerprint: binding.fingerprint,
            },
            entitlement: EntitlementStatus::Active,
            token_expires_at_ms: request.timestamp_ms + 3_600_000,
        };
        drop(licenses);
        self.store.append_audit(AuditEvent {
            kind: "activation_success".into(),
            license_key_hash: Some(license_hash),
            reset_request_id: None,
            timestamp_ms: request.timestamp_ms,
        });
        Ok(outcome)
    }

    pub fn validate_session(&self, token: AccessToken) -> Result<ValidationOutcome, AuthError> {
        let token_value = token.expose_secret().to_string();
        let Some(license_hash) = self.store.tokens.lock().unwrap().get(&token_value).cloned()
        else {
            self.store.append_audit(AuditEvent {
                kind: "validation_failure".into(),
                license_key_hash: None,
                reset_request_id: None,
                timestamp_ms: 0,
            });
            return Ok(ValidationOutcome::ReauthRequired);
        };
        let licenses = self.store.licenses.lock().unwrap();
        let Some(record) = licenses.get(&license_hash) else {
            return Ok(ValidationOutcome::ReauthRequired);
        };
        let Some(binding) = &record.current_binding else {
            return Ok(ValidationOutcome::Revoked);
        };
        Ok(ValidationOutcome::Active {
            masked_license_key: record.masked_license_key.clone(),
            bound_device: BoundDeviceSummary {
                device_id: binding.device_id.clone(),
                public_key: binding.public_key.clone(),
                fingerprint: binding.fingerprint.clone(),
            },
            token_expires_at_ms: 3_600_000,
        })
    }

    pub fn request_device_reset(
        &self,
        request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        let request_id = reset_id(&request);
        let license_hash = request
            .license_key
            .as_ref()
            .map(|key| hash_secret(key.expose_secret()));
        let status = DeviceResetStatus::Pending {
            request_id: request_id.clone(),
            created_at_ms: request.timestamp_ms,
        };
        let stored = StoredResetRequest {
            request_id: request_id.clone(),
            license_key_hash: license_hash.clone(),
            masked_license_key: request.masked_license_key.or_else(|| {
                request
                    .license_key
                    .as_ref()
                    .map(|license_key| license_key.masked())
            }),
            purchaser_email: request.purchaser_email.as_str().to_string(),
            device_public_key: request.device_public_key,
            fingerprint: request.fingerprint,
            app_version: request.app_version,
            receipt_reference: request.receipt_reference,
            status: status.clone(),
        };
        self.store
            .resets
            .lock()
            .unwrap()
            .insert(request_id.as_str().to_string(), stored);
        self.store.append_audit(AuditEvent {
            kind: "reset_request_created".into(),
            license_key_hash: license_hash,
            reset_request_id: Some(request_id.as_str().to_string()),
            timestamp_ms: request.timestamp_ms,
        });
        Ok(status)
    }

    pub fn get_reset_status(&self, request_id: ResetRequestId) -> DeviceResetStatus {
        self.store
            .resets
            .lock()
            .unwrap()
            .get(request_id.as_str())
            .map(|request| request.status.clone())
            .unwrap_or(DeviceResetStatus::NotFound { request_id })
    }

    pub fn decide_reset(
        &self,
        admin_token: &str,
        request_id: ResetRequestId,
        decision: AdminResetDecision,
        timestamp_ms: i64,
    ) -> Result<DeviceResetStatus, AuthError> {
        if admin_token != self.admin_token {
            return Err(AuthError::Unauthorized);
        }

        let mut resets = self.store.resets.lock().unwrap();
        let stored = resets
            .get_mut(request_id.as_str())
            .ok_or(AuthError::ResetRequestNotFound)?;
        let status = match decision {
            AdminResetDecision::Approve => {
                if let Some(hash) = &stored.license_key_hash {
                    if let Some(record) = self.store.licenses.lock().unwrap().get_mut(hash) {
                        record.current_binding = None;
                    }
                    self.store
                        .tokens
                        .lock()
                        .unwrap()
                        .retain(|_, token_hash| token_hash != hash);
                }
                DeviceResetStatus::Approved {
                    request_id: request_id.clone(),
                    decided_at_ms: timestamp_ms,
                }
            }
            AdminResetDecision::Reject => DeviceResetStatus::Rejected {
                request_id: request_id.clone(),
                decided_at_ms: timestamp_ms,
                reason: None,
            },
        };
        stored.status = status.clone();
        drop(resets);
        self.store.append_audit(AuditEvent {
            kind: match decision {
                AdminResetDecision::Approve => "reset_approved",
                AdminResetDecision::Reject => "reset_rejected",
            }
            .into(),
            license_key_hash: None,
            reset_request_id: Some(request_id.as_str().to_string()),
            timestamp_ms,
        });
        Ok(status)
    }
}

fn hash_secret(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    STANDARD_NO_PAD.encode(hasher.finalize())
}

fn issue_token(license_hash: &str, device_id: &str, timestamp_ms: i64) -> AccessToken {
    AccessToken::new(format!(
        "tok_{}",
        hash_secret(&format!("{license_hash}:{device_id}:{timestamp_ms}"))
    ))
    .unwrap()
}

fn reset_id(request: &DeviceResetRequest) -> ResetRequestId {
    let key = request
        .license_key
        .as_ref()
        .map(|key| key.expose_secret().to_string())
        .or_else(|| {
            request
                .masked_license_key
                .as_ref()
                .map(|key| key.as_str().to_string())
        })
        .unwrap_or_else(|| "unknown".into());
    ResetRequestId::new(format!(
        "reset_{}",
        &hash_secret(&format!(
            "{}:{}:{}",
            key,
            request.purchaser_email.as_str(),
            request.timestamp_ms
        ))[..16]
    ))
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user_reg::auth_licensing_core::{DeviceFingerprint, PurchaseEmail};

    fn license() -> LicenseKey {
        LicenseKey::new("LICENSE-1234").unwrap()
    }

    fn activation_request(public_key: &str) -> ActivationRequest {
        ActivationRequest {
            license_key: license(),
            device_public_key: DevicePublicKey::new(public_key).unwrap(),
            fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
            app_version: "1.0.0".into(),
            timestamp_ms: 10,
        }
    }

    fn reset_request() -> DeviceResetRequest {
        DeviceResetRequest {
            license_key: Some(license()),
            masked_license_key: None,
            purchaser_email: PurchaseEmail::new("buyer@example.com").unwrap(),
            device_public_key: DevicePublicKey::new("public").unwrap(),
            fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
            app_version: "1.0.0".into(),
            timestamp_ms: 20,
            receipt_reference: Some("receipt".into()),
        }
    }

    fn app() -> WorkerApp {
        let store = InMemoryWorkerStore::default();
        store.insert_license(&license(), "buyer@example.com");
        WorkerApp::new(store, "admin-secret")
    }

    #[test]
    fn storage_creates_and_reads_license_record() {
        let store = InMemoryWorkerStore::default();
        store.insert_license(&license(), "buyer@example.com");
        assert_eq!(
            store.license(&license()).unwrap().purchaser.email,
            "buyer@example.com"
        );
    }

    #[test]
    fn valid_unbound_activation_binds_device() {
        let app = app();
        let outcome = app.activate(activation_request("public")).unwrap();
        assert_eq!(outcome.masked_license_key.as_str(), "••••-1234");
        assert_eq!(
            app.store
                .license(&license())
                .unwrap()
                .current_binding
                .unwrap()
                .public_key
                .as_str(),
            "public"
        );
    }

    #[test]
    fn same_device_reactivation_refreshes_token() {
        let app = app();
        app.activate(activation_request("public")).unwrap();
        let second = app.activate(activation_request("public")).unwrap();
        assert_eq!(second.bound_device.public_key.as_str(), "public");
    }

    #[test]
    fn different_device_activation_fails_while_bound() {
        let app = app();
        app.activate(activation_request("public")).unwrap();
        assert_eq!(
            app.activate(activation_request("other-public"))
                .unwrap_err(),
            AuthError::DeviceAlreadyBound
        );
    }

    #[test]
    fn validation_succeeds_for_active_token_and_fails_after_reset_approval() {
        let app = app();
        let outcome = app.activate(activation_request("public")).unwrap();
        assert!(matches!(
            app.validate_session(outcome.access_token.clone()).unwrap(),
            ValidationOutcome::Active { .. }
        ));
        let status = app.request_device_reset(reset_request()).unwrap();
        let request_id = status.request_id().clone();
        app.decide_reset("admin-secret", request_id, AdminResetDecision::Approve, 30)
            .unwrap();
        assert!(matches!(
            app.validate_session(outcome.access_token).unwrap(),
            ValidationOutcome::ReauthRequired
        ));
    }

    #[test]
    fn reset_request_creates_pending_record_with_metadata() {
        let app = app();
        let status = app.request_device_reset(reset_request()).unwrap();
        assert!(matches!(status, DeviceResetStatus::Pending { .. }));
        let stored = app
            .store
            .resets
            .lock()
            .unwrap()
            .get(status.request_id().as_str())
            .cloned()
            .unwrap();
        assert_eq!(stored.purchaser_email, "buyer@example.com");
        assert_eq!(stored.receipt_reference.as_deref(), Some("receipt"));
    }

    #[test]
    fn reset_status_returns_not_found_for_missing_request() {
        let app = app();
        assert!(matches!(
            app.get_reset_status(ResetRequestId::new("missing").unwrap()),
            DeviceResetStatus::NotFound { .. }
        ));
    }

    #[test]
    fn admin_approval_unbinds_license_and_revokes_token() {
        let app = app();
        let outcome = app.activate(activation_request("public")).unwrap();
        let status = app.request_device_reset(reset_request()).unwrap();
        app.decide_reset(
            "admin-secret",
            status.request_id().clone(),
            AdminResetDecision::Approve,
            30,
        )
        .unwrap();
        assert!(app
            .store
            .license(&license())
            .unwrap()
            .current_binding
            .is_none());
        assert!(matches!(
            app.validate_session(outcome.access_token).unwrap(),
            ValidationOutcome::ReauthRequired
        ));
    }

    #[test]
    fn admin_rejection_preserves_binding() {
        let app = app();
        app.activate(activation_request("public")).unwrap();
        let status = app.request_device_reset(reset_request()).unwrap();
        app.decide_reset(
            "admin-secret",
            status.request_id().clone(),
            AdminResetDecision::Reject,
            30,
        )
        .unwrap();
        assert!(app
            .store
            .license(&license())
            .unwrap()
            .current_binding
            .is_some());
    }

    #[test]
    fn admin_routes_require_authorization() {
        let app = app();
        let status = app.request_device_reset(reset_request()).unwrap();
        assert_eq!(
            app.decide_reset(
                "wrong",
                status.request_id().clone(),
                AdminResetDecision::Approve,
                30
            )
            .unwrap_err(),
            AuthError::Unauthorized
        );
    }

    #[test]
    fn audit_events_are_written() {
        let app = app();
        app.activate(activation_request("public")).unwrap();
        let status = app.request_device_reset(reset_request()).unwrap();
        app.decide_reset(
            "admin-secret",
            status.request_id().clone(),
            AdminResetDecision::Reject,
            30,
        )
        .unwrap();
        let kinds = app
            .store
            .audit_events()
            .into_iter()
            .map(|event| event.kind)
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"activation_success".to_string()));
        assert!(kinds.contains(&"reset_request_created".to_string()));
        assert!(kinds.contains(&"reset_rejected".to_string()));
    }

    #[test]
    fn worker_errors_do_not_echo_raw_secrets() {
        let error = app()
            .activate(activation_request("other"))
            .unwrap()
            .access_token;
        assert!(!format!("{error:?}").contains("tok_"));
    }

    #[test]
    fn audit_events_do_not_store_raw_access_tokens() {
        let app = app();
        let token = app
            .activate(activation_request("public"))
            .unwrap()
            .access_token;
        let audit_json = serde_json::to_string(&app.store.audit_events()).unwrap();
        assert!(!audit_json.contains(token.expose_secret()));
    }
}

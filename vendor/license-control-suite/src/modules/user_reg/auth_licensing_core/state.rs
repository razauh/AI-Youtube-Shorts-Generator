use super::domain::{AuthError, BoundDeviceSummary, MaskedLicenseKey, ResetRequestId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Unauthenticated,
    Licensed {
        masked_license_key: MaskedLicenseKey,
        bound_device: BoundDeviceSummary,
        token_expires_at_ms: i64,
    },
    ReauthRequired {
        masked_license_key: Option<MaskedLicenseKey>,
    },
    ResetPending {
        request_id: ResetRequestId,
        masked_license_key: Option<MaskedLicenseKey>,
    },
    ResetApprovedUnbound {
        request_id: ResetRequestId,
        masked_license_key: Option<MaskedLicenseKey>,
    },
    ResetRejected {
        request_id: ResetRequestId,
        masked_license_key: Option<MaskedLicenseKey>,
    },
    ResetExpired {
        request_id: ResetRequestId,
        masked_license_key: Option<MaskedLicenseKey>,
    },
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Unauthenticated
    }
}

impl SessionState {
    pub fn after_activation(
        masked_license_key: MaskedLicenseKey,
        bound_device: BoundDeviceSummary,
        token_expires_at_ms: i64,
    ) -> Self {
        Self::Licensed {
            masked_license_key,
            bound_device,
            token_expires_at_ms,
        }
    }

    pub fn require_reauth(masked_license_key: Option<MaskedLicenseKey>) -> Self {
        Self::ReauthRequired { masked_license_key }
    }

    pub fn after_reset_status(
        status: &DeviceResetStatus,
        masked_license_key: Option<MaskedLicenseKey>,
    ) -> Result<Self, AuthError> {
        match status {
            DeviceResetStatus::Pending { request_id, .. } => Ok(Self::ResetPending {
                request_id: request_id.clone(),
                masked_license_key,
            }),
            DeviceResetStatus::Approved { request_id, .. } => Ok(Self::ResetApprovedUnbound {
                request_id: request_id.clone(),
                masked_license_key,
            }),
            DeviceResetStatus::Rejected { request_id, .. } => Ok(Self::ResetRejected {
                request_id: request_id.clone(),
                masked_license_key,
            }),
            DeviceResetStatus::Expired { request_id, .. } => Ok(Self::ResetExpired {
                request_id: request_id.clone(),
                masked_license_key,
            }),
            DeviceResetStatus::NotFound { .. } => Err(AuthError::ResetRequestNotFound),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceResetStatus {
    Pending {
        request_id: ResetRequestId,
        created_at_ms: i64,
    },
    Approved {
        request_id: ResetRequestId,
        decided_at_ms: i64,
    },
    Rejected {
        request_id: ResetRequestId,
        decided_at_ms: i64,
        reason: Option<String>,
    },
    Expired {
        request_id: ResetRequestId,
        expired_at_ms: i64,
    },
    NotFound {
        request_id: ResetRequestId,
    },
}

impl DeviceResetStatus {
    pub fn request_id(&self) -> &ResetRequestId {
        match self {
            Self::Pending { request_id, .. }
            | Self::Approved { request_id, .. }
            | Self::Rejected { request_id, .. }
            | Self::Expired { request_id, .. }
            | Self::NotFound { request_id } => request_id,
        }
    }

    pub fn approved_message() -> &'static str {
        "Device reset approved. You can now use this license key to activate a device."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::domain::{DeviceFingerprint, DeviceId, DevicePublicKey};

    fn masked() -> MaskedLicenseKey {
        MaskedLicenseKey::new("••••-1234").unwrap()
    }

    fn bound_device() -> BoundDeviceSummary {
        let public_key = DevicePublicKey::new("public").unwrap();
        BoundDeviceSummary {
            device_id: DeviceId::from_public_key(&public_key),
            public_key,
            fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
        }
    }

    #[test]
    fn default_state_is_unauthenticated() {
        assert_eq!(SessionState::default(), SessionState::Unauthenticated);
    }

    #[test]
    fn activation_moves_to_licensed() {
        let state = SessionState::after_activation(masked(), bound_device(), 100);
        assert!(matches!(state, SessionState::Licensed { .. }));
    }

    #[test]
    fn missing_or_expired_token_requires_reauth() {
        let state = SessionState::require_reauth(Some(masked()));
        assert!(matches!(state, SessionState::ReauthRequired { .. }));
    }

    #[test]
    fn reset_approval_is_unbound_not_current_device_activation() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let status = DeviceResetStatus::Approved {
            request_id: request_id.clone(),
            decided_at_ms: 10,
        };
        let state = SessionState::after_reset_status(&status, Some(masked())).unwrap();
        assert_eq!(
            state,
            SessionState::ResetApprovedUnbound {
                request_id,
                masked_license_key: Some(masked())
            }
        );
        assert!(!DeviceResetStatus::approved_message().contains("this device is activated"));
    }

    #[test]
    fn rejected_reset_does_not_mark_unbound() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let status = DeviceResetStatus::Rejected {
            request_id,
            decided_at_ms: 10,
            reason: None,
        };
        let state = SessionState::after_reset_status(&status, Some(masked())).unwrap();
        assert!(matches!(state, SessionState::ResetRejected { .. }));
    }

    #[test]
    fn not_found_status_is_recoverable_error() {
        let request_id = ResetRequestId::new("reset-1").unwrap();
        let status = DeviceResetStatus::NotFound { request_id };
        assert_eq!(
            SessionState::after_reset_status(&status, None).unwrap_err(),
            AuthError::ResetRequestNotFound
        );
    }
}

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("invalid license key")]
    InvalidLicenseKey,
    #[error("invalid purchase email")]
    InvalidPurchaseEmail,
    #[error("invalid device identity")]
    InvalidDeviceIdentity,
    #[error("invalid reset request")]
    InvalidResetRequest,
    #[error("license is already bound to another device")]
    DeviceAlreadyBound,
    #[error("session requires re-authentication")]
    ReauthRequired,
    #[error("worker is unreachable")]
    WorkerUnreachable,
    #[error("reset request was not found")]
    ResetRequestNotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("storage error: {0}")]
    Storage(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid state transition: {0}")]
    InvalidTransition(String),
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidLicenseKey => "invalid_license_key",
            Self::InvalidPurchaseEmail => "invalid_purchase_email",
            Self::InvalidDeviceIdentity => "invalid_device_identity",
            Self::InvalidResetRequest => "invalid_reset_request",
            Self::DeviceAlreadyBound => "device_already_bound",
            Self::ReauthRequired => "reauth_required",
            Self::WorkerUnreachable => "worker_unreachable",
            Self::ResetRequestNotFound => "reset_request_not_found",
            Self::Unauthorized => "unauthorized",
            Self::Storage(_) => "storage",
            Self::Serialization(_) => "serialization",
            Self::InvalidTransition(_) => "invalid_transition",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseKey(String);

impl LicenseKey {
    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let normalized = value.into().trim().to_string();
        if normalized.is_empty() {
            return Err(AuthError::InvalidLicenseKey);
        }
        Ok(Self(normalized))
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub fn masked(&self) -> MaskedLicenseKey {
        MaskedLicenseKey::from_license_key(self)
    }
}

impl fmt::Debug for LicenseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LicenseKey").field(&"<redacted>").finish()
    }
}

impl fmt::Display for LicenseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted-license-key>")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskedLicenseKey(String);

impl MaskedLicenseKey {
    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(AuthError::InvalidLicenseKey);
        }
        Ok(Self(value))
    }

    pub fn from_license_key(key: &LicenseKey) -> Self {
        let raw = key.expose_secret();
        let visible = raw.chars().rev().take(4).collect::<Vec<_>>();
        let suffix = visible.into_iter().rev().collect::<String>();
        Self(format!("••••-{}", suffix))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MaskedLicenseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseEmail(String);

impl PurchaseEmail {
    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let email = value.into().trim().to_ascii_lowercase();
        let has_one_at = email.matches('@').count() == 1;
        let has_domain_dot = email
            .split('@')
            .nth(1)
            .map(|domain| {
                domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
            })
            .unwrap_or(false);
        if email.is_empty() || !has_one_at || !has_domain_dot {
            return Err(AuthError::InvalidPurchaseEmail);
        }
        Ok(Self(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PurchaseEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(AuthError::ReauthRequired);
        }
        Ok(Self(value))
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccessToken").field(&"<redacted>").finish()
    }
}

impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted-access-token>")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevicePublicKey(String);

impl DevicePublicKey {
    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(AuthError::InvalidDeviceIdentity);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceKeyPair {
    public_key: DevicePublicKey,
    private_key_material: String,
}

impl DeviceKeyPair {
    pub fn new(
        public_key: DevicePublicKey,
        private_key_material: impl Into<String>,
    ) -> Result<Self, AuthError> {
        let private_key_material = private_key_material.into();
        if private_key_material.trim().is_empty() {
            return Err(AuthError::InvalidDeviceIdentity);
        }
        Ok(Self {
            public_key,
            private_key_material,
        })
    }

    pub fn public_key(&self) -> &DevicePublicKey {
        &self.public_key
    }

    pub fn expose_private_key_material(&self) -> &str {
        &self.private_key_material
    }
}

impl fmt::Debug for DeviceKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceKeyPair")
            .field("public_key", &self.public_key)
            .field("private_key_material", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn from_public_key(public_key: &DevicePublicKey) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_str().as_bytes());
        let digest = hasher.finalize();
        Self(STANDARD_NO_PAD.encode(&digest[..18]))
    }

    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(AuthError::InvalidDeviceIdentity);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResetRequestId(String);

impl ResetRequestId {
    pub fn new(value: impl Into<String>) -> Result<Self, AuthError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(AuthError::InvalidResetRequest);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub platform: String,
    pub os: String,
    pub arch: String,
    pub hostname_hash: Option<String>,
}

impl DeviceFingerprint {
    pub fn new(
        platform: impl Into<String>,
        os: impl Into<String>,
        arch: impl Into<String>,
        hostname_hash: Option<String>,
    ) -> Result<Self, AuthError> {
        let fingerprint = Self {
            platform: platform.into().trim().to_string(),
            os: os.into().trim().to_string(),
            arch: arch.into().trim().to_string(),
            hostname_hash,
        };
        if fingerprint.platform.is_empty()
            || fingerprint.os.is_empty()
            || fingerprint.arch.is_empty()
        {
            return Err(AuthError::InvalidDeviceIdentity);
        }
        Ok(fingerprint)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundDeviceSummary {
    pub device_id: DeviceId,
    pub public_key: DevicePublicKey,
    pub fingerprint: DeviceFingerprint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationRequest {
    pub license_key: LicenseKey,
    pub device_public_key: DevicePublicKey,
    pub fingerprint: DeviceFingerprint,
    pub app_version: String,
    pub timestamp_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationOutcome {
    pub access_token: AccessToken,
    pub masked_license_key: MaskedLicenseKey,
    pub bound_device: BoundDeviceSummary,
    pub entitlement: EntitlementStatus,
    pub token_expires_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntitlementStatus {
    Active,
    Disabled,
    Refunded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationOutcome {
    Active {
        masked_license_key: MaskedLicenseKey,
        bound_device: BoundDeviceSummary,
        token_expires_at_ms: i64,
    },
    ReauthRequired,
    Revoked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceResetRequest {
    pub license_key: Option<LicenseKey>,
    pub masked_license_key: Option<MaskedLicenseKey>,
    pub purchaser_email: PurchaseEmail,
    pub device_public_key: DevicePublicKey,
    pub fingerprint: DeviceFingerprint,
    pub app_version: String,
    pub timestamp_ms: i64,
    pub receipt_reference: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminResetDecision {
    Approve,
    Reject,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_key_normalizes_and_masks() {
        let key = LicenseKey::new("  ABCD-1234  ").unwrap();
        assert_eq!(key.expose_secret(), "ABCD-1234");
        assert_eq!(key.masked().as_str(), "••••-1234");
    }

    #[test]
    fn license_key_rejects_empty_values() {
        assert_eq!(
            LicenseKey::new("   ").unwrap_err(),
            AuthError::InvalidLicenseKey
        );
    }

    #[test]
    fn secret_formatting_is_redacted() {
        let key = LicenseKey::new("SECRET-LICENSE-KEY").unwrap();
        let token = AccessToken::new("SECRET-TOKEN").unwrap();
        let private =
            DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "PRIVATE").unwrap();

        assert!(!format!("{key:?}").contains("SECRET-LICENSE-KEY"));
        assert!(!format!("{key}").contains("SECRET-LICENSE-KEY"));
        assert!(!format!("{token:?}").contains("SECRET-TOKEN"));
        assert!(!format!("{token}").contains("SECRET-TOKEN"));
        assert!(!format!("{private:?}").contains("PRIVATE"));
    }

    #[test]
    fn purchase_email_validates_and_normalizes() {
        let email = PurchaseEmail::new(" Buyer@Example.COM ").unwrap();
        assert_eq!(email.as_str(), "buyer@example.com");
        assert_eq!(
            PurchaseEmail::new("not-an-email").unwrap_err(),
            AuthError::InvalidPurchaseEmail
        );
    }

    #[test]
    fn public_safe_types_serialize() {
        let masked = MaskedLicenseKey::new("••••-1234").unwrap();
        let json = serde_json::to_string(&masked).unwrap();
        assert!(json.contains("1234"));
    }
}

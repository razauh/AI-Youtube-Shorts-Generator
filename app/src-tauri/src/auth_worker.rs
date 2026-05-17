use async_trait::async_trait;
use license_control_suite::core::{
    AccessToken, ActivationOutcome, ActivationRequest, AuthError, BoundDeviceSummary,
    DeviceResetRequest, DeviceResetStatus, EntitlementStatus, LicenseKey, MaskedLicenseKey,
    PurchaseEmail, ResetRequestId, ValidationOutcome, WorkerClient,
};
use license_control_suite::modules::user_reg::auth_licensing_tauri::HttpWorkerClient;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use crate::core::config::{LicenseBackendMode, LicenseWorkerConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerCircuitBreakerPolicy {
    pub failure_threshold: u32,
    pub cooldown_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerClientPolicy {
    pub timeout_ms: u64,
    pub retry: WorkerRetryPolicy,
    pub circuit_breaker: WorkerCircuitBreakerPolicy,
}

impl WorkerClientPolicy {
    pub fn from_config(config: &LicenseWorkerConfig) -> Self {
        Self {
            timeout_ms: config.timeout_ms,
            retry: WorkerRetryPolicy {
                max_attempts: config.retry_attempts,
                backoff_ms: config.retry_backoff_ms,
            },
            circuit_breaker: WorkerCircuitBreakerPolicy {
                failure_threshold: config.circuit_breaker_failure_threshold,
                cooldown_ms: config.circuit_breaker_cooldown_ms,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerContractErrorCode {
    InvalidLicenseKey,
    InvalidPurchaseEmail,
    InvalidDeviceIdentity,
    InvalidResetRequest,
    DeviceAlreadyBound,
    ReauthRequired,
    WorkerUnreachable,
    ResetRequestNotFound,
    Unauthorized,
    Storage,
    Serialization,
    InvalidTransition,
}

impl WorkerContractErrorCode {
    pub fn from_auth_error(error: &AuthError) -> Self {
        match error {
            AuthError::InvalidLicenseKey => Self::InvalidLicenseKey,
            AuthError::InvalidPurchaseEmail => Self::InvalidPurchaseEmail,
            AuthError::InvalidDeviceIdentity => Self::InvalidDeviceIdentity,
            AuthError::InvalidResetRequest => Self::InvalidResetRequest,
            AuthError::DeviceAlreadyBound => Self::DeviceAlreadyBound,
            AuthError::ReauthRequired => Self::ReauthRequired,
            AuthError::WorkerUnreachable => Self::WorkerUnreachable,
            AuthError::ResetRequestNotFound => Self::ResetRequestNotFound,
            AuthError::Unauthorized => Self::Unauthorized,
            AuthError::Storage(_) => Self::Storage,
            AuthError::Serialization(_) => Self::Serialization,
            AuthError::InvalidTransition(_) => Self::InvalidTransition,
        }
    }

    pub fn as_str(&self) -> &'static str {
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
            Self::Storage => "storage",
            Self::Serialization => "serialization",
            Self::InvalidTransition => "invalid_transition",
        }
    }
}

#[derive(Clone)]
pub struct WorkerActivationContract {
    pub idempotency_key: String,
    pub request: ActivationRequest,
}

impl fmt::Debug for WorkerActivationContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerActivationContract")
            .field("idempotency_key", &self.idempotency_key)
            .field("license_key", &"<redacted>")
            .field("device_public_key", &self.request.device_public_key)
            .field("fingerprint", &self.request.fingerprint)
            .field("app_version", &self.request.app_version)
            .field("timestamp_ms", &self.request.timestamp_ms)
            .finish()
    }
}

#[derive(Clone)]
pub struct WorkerResetContract {
    pub idempotency_key: String,
    pub request: DeviceResetRequest,
}

impl fmt::Debug for WorkerResetContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerResetContract")
            .field("idempotency_key", &self.idempotency_key)
            .field(
                "license_key",
                &self.request.license_key.as_ref().map(|_| "<redacted>"),
            )
            .field("masked_license_key", &self.request.masked_license_key)
            .field("purchaser_email", &self.request.purchaser_email)
            .field("device_public_key", &self.request.device_public_key)
            .field("fingerprint", &self.request.fingerprint)
            .field("app_version", &self.request.app_version)
            .field("timestamp_ms", &self.request.timestamp_ms)
            .field("receipt_reference", &self.request.receipt_reference)
            .finish()
    }
}

pub trait WebhookVerifier: Send + Sync {
    fn verify(&self, payload: &[u8], signature: &str) -> Result<(), AuthError>;
}

#[async_trait]
pub trait PurchaseProvider: Send + Sync {
    async fn verify_purchase(
        &self,
        license_key: &LicenseKey,
        purchaser_email: &PurchaseEmail,
    ) -> Result<PurchaseVerification, AuthError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurchaseVerification {
    pub provider: String,
    pub provider_sale_id: Option<String>,
    pub entitlement: EntitlementStatus,
}

#[async_trait]
pub trait LicenseStore: Send + Sync {
    async fn persist_activation(
        &self,
        contract: &WorkerActivationContract,
        outcome: &ActivationOutcome,
    ) -> Result<(), AuthError>;

    async fn persist_reset_request(
        &self,
        contract: &WorkerResetContract,
        status: &DeviceResetStatus,
    ) -> Result<(), AuthError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseWorkerReadiness {
    pub backend_mode: LicenseBackendMode,
    pub contract_adapter_enabled: bool,
    pub provider_adapter_enabled: bool,
    pub webhook_verifier_enabled: bool,
    pub idempotency_enabled: bool,
    pub durable_store_enabled: bool,
}

impl LicenseWorkerReadiness {
    pub fn local(config: &LicenseWorkerConfig) -> Self {
        Self {
            backend_mode: config.backend_mode,
            contract_adapter_enabled: true,
            provider_adapter_enabled: false,
            webhook_verifier_enabled: false,
            idempotency_enabled: true,
            durable_store_enabled: false,
        }
    }
}

pub fn build_worker_client(
    config: &LicenseWorkerConfig,
) -> Result<Arc<dyn WorkerClient>, AuthError> {
    match config.backend_mode {
        LicenseBackendMode::Reference | LicenseBackendMode::Hosted => {
            let http = HttpWorkerClient::with_timeout(
                &config.base_url,
                Duration::from_millis(config.timeout_ms),
            )?;
            Ok(Arc::new(PolicyWorkerClient::new(
                Arc::new(http),
                WorkerClientPolicy::from_config(config),
            )))
        }
        LicenseBackendMode::Mock => Ok(Arc::new(PolicyWorkerClient::new(
            Arc::new(MockLicenseWorkerClient),
            WorkerClientPolicy::from_config(config),
        ))),
    }
}

#[derive(Clone)]
pub struct PolicyWorkerClient {
    inner: Arc<dyn WorkerClient>,
    policy: WorkerClientPolicy,
    circuit: Arc<Mutex<CircuitState>>,
}

#[derive(Debug, Default)]
struct CircuitState {
    consecutive_failures: u32,
    opened_at_ms: Option<i64>,
}

impl PolicyWorkerClient {
    pub fn new(inner: Arc<dyn WorkerClient>, policy: WorkerClientPolicy) -> Self {
        Self {
            inner,
            policy,
            circuit: Arc::new(Mutex::new(CircuitState::default())),
        }
    }

    fn before_request(&self) -> Result<(), AuthError> {
        let now_ms = current_time_ms();
        let mut circuit = self
            .circuit
            .lock()
            .map_err(|err| AuthError::Storage(err.to_string()))?;
        if let Some(opened_at_ms) = circuit.opened_at_ms {
            let elapsed = now_ms.saturating_sub(opened_at_ms);
            if elapsed < self.policy.circuit_breaker.cooldown_ms as i64 {
                return Err(AuthError::WorkerUnreachable);
            }
            circuit.opened_at_ms = None;
            circuit.consecutive_failures = 0;
        }
        Ok(())
    }

    fn after_request(&self, result: &Result<(), AuthError>) {
        let Ok(mut circuit) = self.circuit.lock() else {
            return;
        };
        match result {
            Ok(()) => {
                circuit.consecutive_failures = 0;
                circuit.opened_at_ms = None;
            }
            Err(AuthError::WorkerUnreachable) => {
                circuit.consecutive_failures = circuit.consecutive_failures.saturating_add(1);
                if circuit.consecutive_failures >= self.policy.circuit_breaker.failure_threshold {
                    circuit.opened_at_ms = Some(current_time_ms());
                }
            }
            Err(_) => {}
        }
    }

    async fn retry<T, Fut, Op>(&self, mut operation: Op) -> Result<T, AuthError>
    where
        Op: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, AuthError>>,
    {
        self.before_request()?;
        let attempts = self.policy.retry.max_attempts.max(1);
        let mut last_error = AuthError::WorkerUnreachable;
        for attempt in 1..=attempts {
            match operation().await {
                Ok(value) => {
                    self.after_request(&Ok(()));
                    return Ok(value);
                }
                Err(AuthError::WorkerUnreachable) if attempt < attempts => {
                    last_error = AuthError::WorkerUnreachable;
                    if self.policy.retry.backoff_ms > 0 {
                        sleep(Duration::from_millis(self.policy.retry.backoff_ms)).await;
                    }
                }
                Err(error) => {
                    let result = Err(error);
                    self.after_request(&result.as_ref().map(|_| ()).map_err(Clone::clone));
                    return result;
                }
            }
        }
        let result = Err(last_error);
        self.after_request(&result.as_ref().map(|_| ()).map_err(Clone::clone));
        result
    }
}

#[async_trait]
impl WorkerClient for PolicyWorkerClient {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        let contract = WorkerActivationContract {
            idempotency_key: activation_idempotency_key(&request),
            request,
        };
        self.retry(|| {
            let inner = Arc::clone(&self.inner);
            let request = contract.request.clone();
            async move { inner.activate(request).await }
        })
        .await
    }

    async fn validate_session(&self, token: AccessToken) -> Result<ValidationOutcome, AuthError> {
        self.retry(|| {
            let inner = Arc::clone(&self.inner);
            let token = token.clone();
            async move { inner.validate_session(token).await }
        })
        .await
    }

    async fn request_device_reset(
        &self,
        request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        let contract = WorkerResetContract {
            idempotency_key: reset_idempotency_key(&request),
            request,
        };
        self.retry(|| {
            let inner = Arc::clone(&self.inner);
            let request = contract.request.clone();
            async move { inner.request_device_reset(request).await }
        })
        .await
    }

    async fn get_device_reset_status(
        &self,
        request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        self.retry(|| {
            let inner = Arc::clone(&self.inner);
            let request_id = request_id.clone();
            async move { inner.get_device_reset_status(request_id).await }
        })
        .await
    }
}

#[derive(Clone)]
struct MockLicenseWorkerClient;

#[async_trait]
impl WorkerClient for MockLicenseWorkerClient {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        let bound_device = BoundDeviceSummary {
            device_id: license_control_suite::core::DeviceId::from_public_key(
                &request.device_public_key,
            ),
            public_key: request.device_public_key,
            fingerprint: request.fingerprint,
        };
        Ok(ActivationOutcome {
            access_token: AccessToken::new(format!("mock-token-{}", request.timestamp_ms))?,
            masked_license_key: request.license_key.masked(),
            bound_device,
            entitlement: EntitlementStatus::Active,
            token_expires_at_ms: request.timestamp_ms + 86_400_000,
        })
    }

    async fn validate_session(&self, _token: AccessToken) -> Result<ValidationOutcome, AuthError> {
        Ok(ValidationOutcome::ReauthRequired)
    }

    async fn request_device_reset(
        &self,
        request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        Ok(DeviceResetStatus::Pending {
            request_id: ResetRequestId::new(format!("mock-reset-{}", request.timestamp_ms))?,
            created_at_ms: request.timestamp_ms,
        })
    }

    async fn get_device_reset_status(
        &self,
        request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        Ok(DeviceResetStatus::Pending {
            request_id,
            created_at_ms: current_time_ms(),
        })
    }
}

pub fn activation_idempotency_key(request: &ActivationRequest) -> String {
    format!(
        "activate:{}:{}:{}",
        request.device_public_key.as_str(),
        request.app_version,
        request.timestamp_ms
    )
}

pub fn reset_idempotency_key(request: &DeviceResetRequest) -> String {
    let license_marker = request
        .masked_license_key
        .as_ref()
        .map(MaskedLicenseKey::as_str)
        .unwrap_or("unmasked");
    format!(
        "reset:{}:{}:{}",
        request.device_public_key.as_str(),
        license_marker,
        request.timestamp_ms
    )
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

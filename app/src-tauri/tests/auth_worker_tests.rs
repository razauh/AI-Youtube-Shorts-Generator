use async_trait::async_trait;
use sha2::Digest;
use license_control_suite::core::{
    AccessToken, ActivationOutcome, ActivationRequest, AuthError, BoundDeviceSummary,
    DeviceFingerprint, DeviceId, DevicePublicKey, DeviceResetRequest, DeviceResetStatus,
    EntitlementStatus, LicenseKey, PurchaseEmail, ResetRequestId, ValidationOutcome, WorkerClient,
};
use shorts_tauri_app::auth_worker::{
    activation_idempotency_key, build_worker_client, reset_idempotency_key, DevolensWorkerClient,
    LicenseStore, PolicyWorkerClient, WorkerActivationContract, WorkerCircuitBreakerPolicy,
    WorkerClientPolicy, WorkerContractErrorCode, WorkerResetContract, WorkerRetryPolicy,
};
use shorts_tauri_app::core::config::{
    LicenseBackendMode, LicenseWorkerConfig, DEFAULT_DEVOLENS_BASE_URL,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn fingerprint() -> DeviceFingerprint {
    DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap()
}

fn activation_request() -> ActivationRequest {
    ActivationRequest {
        license_key: LicenseKey::new("SECRET-LICENSE").unwrap(),
        device_public_key: DevicePublicKey::new("public").unwrap(),
        fingerprint: fingerprint(),
        app_version: "0.1.0".to_string(),
        timestamp_ms: 123,
    }
}

fn reset_request() -> DeviceResetRequest {
    DeviceResetRequest {
        license_key: Some(LicenseKey::new("SECRET-LICENSE").unwrap()),
        masked_license_key: Some(LicenseKey::new("SECRET-LICENSE").unwrap().masked()),
        purchaser_email: Some(PurchaseEmail::new("buyer@example.com").unwrap()),
        device_public_key: DevicePublicKey::new("public").unwrap(),
        fingerprint: fingerprint(),
        app_version: "0.1.0".to_string(),
        timestamp_ms: 456,
    }
}

fn policy(max_attempts: u32, failure_threshold: u32) -> WorkerClientPolicy {
    WorkerClientPolicy {
        timeout_ms: 1000,
        retry: WorkerRetryPolicy {
            max_attempts,
            backoff_ms: 0,
        },
        circuit_breaker: WorkerCircuitBreakerPolicy {
            failure_threshold,
            cooldown_ms: 60_000,
        },
    }
}

fn worker_config(mode: LicenseBackendMode, base_url: String) -> LicenseWorkerConfig {
    LicenseWorkerConfig {
        backend_mode: mode,
        base_url,
        storage_namespace: "desktop-client-test".to_string(),
        keychain_service: "shorts-test".to_string(),
        timeout_ms: 1000,
        retry_attempts: 1,
        retry_backoff_ms: 0,
        circuit_breaker_failure_threshold: 3,
        circuit_breaker_cooldown_ms: 1000,
        devolens_base_url: DEFAULT_DEVOLENS_BASE_URL.to_string(),
        devolens_access_token: String::new(),
        devolens_product_id: String::new(),
    }
}

fn local_http_server(response_body: String) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    let addr = listener.local_addr().expect("local server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let read = stream.read(&mut buffer).expect("read request");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let header_end = request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|pos| pos + 4)
            .unwrap_or(request.len());
        let headers = String::from_utf8_lossy(&request[..header_end]).to_string();
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        while request.len().saturating_sub(header_end) < content_length {
            let read = stream.read(&mut buffer).expect("read request body");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
        }
        let request_text = String::from_utf8_lossy(&request).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request_text
    });
    (format!("http://{addr}"), handle)
}

#[derive(Default)]
struct FlakyWorker {
    attempts: AtomicUsize,
    fail_until: usize,
}

#[derive(Default)]
struct CaptureLicenseStore {
    activations: Mutex<Vec<String>>,
    resets: Mutex<Vec<String>>,
}

#[async_trait]
impl shorts_tauri_app::auth_worker::LicenseStore for CaptureLicenseStore {
    async fn persist_activation(
        &self,
        contract: &shorts_tauri_app::auth_worker::WorkerActivationContract,
        _outcome: &ActivationOutcome,
    ) -> Result<(), AuthError> {
        self.activations
            .lock()
            .expect("lock activations")
            .push(contract.idempotency_key.clone());
        Ok(())
    }

    async fn persist_reset_request(
        &self,
        contract: &shorts_tauri_app::auth_worker::WorkerResetContract,
        _status: &DeviceResetStatus,
    ) -> Result<(), AuthError> {
        self.resets
            .lock()
            .expect("lock resets")
            .push(contract.idempotency_key.clone());
        Ok(())
    }
}

#[async_trait]
impl WorkerClient for FlakyWorker {
    async fn activate(&self, request: ActivationRequest) -> Result<ActivationOutcome, AuthError> {
        let attempt = self.attempts.fetch_add(1, Ordering::SeqCst) + 1;
        if attempt <= self.fail_until {
            return Err(AuthError::WorkerUnreachable);
        }
        let public_key = request.device_public_key.clone();
        Ok(ActivationOutcome {
            access_token: AccessToken::new("TOKEN").unwrap(),
            masked_license_key: request.license_key.masked(),
            bound_device: BoundDeviceSummary {
                device_id: DeviceId::from_public_key(&public_key),
                public_key,
                fingerprint: request.fingerprint,
            },
            entitlement: EntitlementStatus::Active,
            token_expires_at_ms: request.timestamp_ms + 1000,
        })
    }

    async fn validate_session(&self, _token: AccessToken) -> Result<ValidationOutcome, AuthError> {
        Err(AuthError::WorkerUnreachable)
    }

    async fn request_device_reset(
        &self,
        _request: DeviceResetRequest,
    ) -> Result<DeviceResetStatus, AuthError> {
        Err(AuthError::WorkerUnreachable)
    }

    async fn get_device_reset_status(
        &self,
        _request_id: ResetRequestId,
    ) -> Result<DeviceResetStatus, AuthError> {
        Err(AuthError::WorkerUnreachable)
    }
}

#[test]
fn local_contract_idempotency_keys_are_stable_and_secret_free() {
    let activation = activation_request();
    let reset = reset_request();

    let activation_key = activation_idempotency_key(&activation);
    let reset_key = reset_idempotency_key(&reset);

    assert_eq!(activation_key, activation_idempotency_key(&activation));
    assert_eq!(reset_key, reset_idempotency_key(&reset));
    assert!(!activation_key.contains("SECRET-LICENSE"));
    assert!(!reset_key.contains("SECRET-LICENSE"));
}

#[test]
fn auth_error_mapping_uses_local_error_taxonomy() {
    let mapped = WorkerContractErrorCode::from_auth_error(&AuthError::DeviceAlreadyBound);
    assert_eq!(mapped.as_str(), "device_already_bound");
}

#[tokio::test]
async fn policy_worker_retries_transient_worker_unreachable_errors() {
    let inner = Arc::new(FlakyWorker {
        attempts: AtomicUsize::new(0),
        fail_until: 1,
    });
    let worker = PolicyWorkerClient::new(inner.clone(), policy(2, 3));

    let outcome = worker
        .activate(activation_request())
        .await
        .expect("second attempt should succeed");

    assert_eq!(outcome.masked_license_key.as_str(), "••••-ENSE");
    assert_eq!(inner.attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn policy_worker_opens_circuit_after_threshold_failures() {
    let inner = Arc::new(FlakyWorker {
        attempts: AtomicUsize::new(0),
        fail_until: 10,
    });
    let worker = PolicyWorkerClient::new(inner.clone(), policy(1, 1));

    let first = worker
        .activate(activation_request())
        .await
        .expect_err("first request should fail and open circuit");
    let second = worker
        .activate(activation_request())
        .await
        .expect_err("second request should be short-circuited");

    assert_eq!(first, AuthError::WorkerUnreachable);
    assert_eq!(second, AuthError::WorkerUnreachable);
    assert_eq!(inner.attempts.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn mock_backend_mode_routes_through_local_worker_switch() {
    let cfg = worker_config(LicenseBackendMode::Mock, String::new());
    let worker = build_worker_client(&cfg).expect("mock worker should build");

    let outcome = worker
        .activate(activation_request())
        .await
        .expect("mock activation should succeed");

    assert_eq!(outcome.entitlement, EntitlementStatus::Active);
}

#[tokio::test]
async fn local_license_store_interface_accepts_contract_metadata() {
    let store = CaptureLicenseStore::default();
    let activation = activation_request();
    let reset = reset_request();
    let outcome = ActivationOutcome {
        access_token: AccessToken::new("TOKEN").unwrap(),
        masked_license_key: activation.license_key.masked(),
        bound_device: BoundDeviceSummary {
            device_id: DeviceId::from_public_key(&activation.device_public_key),
            public_key: activation.device_public_key.clone(),
            fingerprint: activation.fingerprint.clone(),
        },
        entitlement: EntitlementStatus::Active,
        token_expires_at_ms: 9999,
    };

    store
        .persist_activation(
            &WorkerActivationContract {
                idempotency_key: activation_idempotency_key(&activation),
                request: activation,
            },
            &outcome,
        )
        .await
        .expect("activation contract should persist");
    store
        .persist_reset_request(
            &WorkerResetContract {
                idempotency_key: reset_idempotency_key(&reset),
                request: reset,
            },
            &DeviceResetStatus::Pending {
                request_id: ResetRequestId::new("reset-1").unwrap(),
                created_at_ms: 1,
            },
        )
        .await
        .expect("reset contract should persist");

    assert_eq!(store.activations.lock().unwrap().len(), 1);
    assert_eq!(store.resets.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn hosted_worker_mode_hits_real_local_http_route_contract() {
    let public_key = DevicePublicKey::new("public").unwrap();
    let device_id = DeviceId::from_public_key(&public_key);
    let body = format!(
        r#"{{
            "ok":true,
            "data":{{
                "access_token":"TOKEN",
                "masked_license_key":"****-ENSE",
                "bound_device":{{
                    "device_id":"{}",
                    "public_key":"public",
                    "fingerprint":{{
                        "platform":"linux",
                        "os":"linux",
                        "arch":"x86_64",
                        "hostname_hash":null
                    }}
                }},
                "entitlement":"active",
                "token_expires_at_ms":9999
                }}
        }}"#,
        device_id.as_str()
    );
    let (base_url, handle) = local_http_server(body);
    let cfg = worker_config(LicenseBackendMode::Hosted, base_url);
    let worker = build_worker_client(&cfg).expect("hosted worker should build");

    let outcome = worker
        .activate(activation_request())
        .await
        .expect("local HTTP worker should activate");
    let request = handle.join().expect("server thread should finish");

    assert!(request.starts_with("POST /v1/license/activate "));
    assert!(request.contains(r#""license_key":"SECRET-LICENSE""#));
    assert_eq!(outcome.entitlement, EntitlementStatus::Active);
}

#[tokio::test]
async fn devolens_backend_mode_activates_against_provider_api_shape() {
    let (base_url, handle) = local_http_server(
        r#"{"result":0,"licenseKey":{"blocked":false,"expired":false}}"#.to_string(),
    );
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_base_url = base_url;
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker = build_worker_client(&cfg).expect("devolens worker should build");

    let outcome = worker
        .activate(activation_request())
        .await
        .expect("devolens activation should succeed");
    let request = handle.join().expect("server thread should finish");

    assert!(request.starts_with("POST /api/key/Activate "));
    assert!(request
        .to_ascii_lowercase()
        .contains("content-type: application/x-www-form-urlencoded"));
    assert!(request.contains("ProductId=1234"));
    assert!(request.contains("MachineCode="));
    assert_eq!(outcome.entitlement, EntitlementStatus::Active);
    assert_eq!(outcome.masked_license_key.as_str(), "••••-ENSE");
}

#[tokio::test]
async fn devolens_client_reauths_existing_sessions_instead_of_trusting_local_token() {
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker =
        DevolensWorkerClient::with_timeout(&cfg, std::time::Duration::from_millis(1000))
            .expect("devolens worker should build");

    let status = worker
        .validate_session(AccessToken::new("local-provider-marker").unwrap())
        .await
        .expect("devolens validation should request reauth");

    assert_eq!(status, ValidationOutcome::ReauthRequired);
}

#[test]
fn test_devolens_parser_handles_all_fixtures() {
    let base_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/license-control-suite/fixtures/devolens");

    // Helper to read and deserialize a fixture
    let read_fixture = |filename: &str| -> shorts_tauri_app::auth_worker::DevolensActivationResponse {
        let path = base_path.join(filename);
        let content = std::fs::read_to_string(path).expect("failed to read fixture file");
        serde_json::from_str(&content).expect("failed to deserialize fixture JSON")
    };

    // 1. Success fixture
    let success = read_fixture("activation_success.json");
    assert!(success.is_success());
    assert!(success.license_is_active());

    // 2. Invalid key fixture
    let invalid = read_fixture("activation_invalid_key.json");
    assert!(!invalid.is_success());
    assert_eq!(invalid.message(), Some("The key is invalid."));

    // 3. Expired key fixture
    let expired = read_fixture("activation_expired_key.json");
    assert!(expired.is_success());
    assert!(!expired.license_is_active());

    // 4. Blocked key fixture
    let blocked = read_fixture("activation_blocked_key.json");
    assert!(blocked.is_success());
    assert!(!blocked.license_is_active());

    // 5. Machine limit reached fixture
    let limit = read_fixture("activation_machine_limit.json");
    assert!(!limit.is_success());
    assert!(limit.message().unwrap().to_ascii_lowercase().contains("machine"));
}

#[tokio::test]
async fn devolens_session_validation_returns_active() {
    let response_body = r#"{
      "result": 0,
      "licenseKey": {
        "blocked": false,
        "expired": false
      },
      "maskedLicenseKey": "••••-ENSE",
      "boundDevice": {
        "device_id": "test-device-id",
        "public_key": "test-public-key",
        "fingerprint": {
          "platform": "linux",
          "os": "ubuntu",
          "arch": "x86_64",
          "hostname_hash": null
        }
      },
      "tokenExpiresAtMs": 1813833600000
    }"#.to_string();

    let (base_url, handle) = local_http_server(response_body);
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_base_url = base_url;
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker = build_worker_client(&cfg).expect("devolens worker should build");

    let payload = "test-device-id:123456";
    let mut hasher = sha2::Sha256::new();
    hasher.update(payload.as_bytes());
    hasher.update(b":");
    hasher.update(b"devolens-token");
    let sig = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let signed_token = format!("devolens:{}:{}", payload, sig);

    let status = worker
        .validate_session(AccessToken::new(signed_token.clone()).unwrap())
        .await
        .expect("devolens validation should succeed");

    let request = handle.join().expect("server thread should finish");

    assert!(request.starts_with("POST /api/key/Validate "));
    assert!(request
        .to_ascii_lowercase()
        .contains("content-type: application/x-www-form-urlencoded"));
    assert!(request.contains("ProductId=1234"));
    assert!(request.contains(&format!("SessionToken={}", signed_token.replace(":", "%3A"))));

    match status {
        ValidationOutcome::Active { masked_license_key, bound_device, token_expires_at_ms } => {
            assert_eq!(masked_license_key.as_str(), "••••-ENSE");
            assert_eq!(bound_device.device_id.as_str(), "test-device-id");
            assert_eq!(token_expires_at_ms, 1813833600000);
        }
        _ => panic!("Expected ValidationOutcome::Active, got {:?}", status),
    }
}

#[tokio::test]
async fn devolens_session_validation_returns_revoked() {
    let response_body = r#"{
      "result": 1,
      "message": "Key is blocked or invalid"
    }"#.to_string();

    let (base_url, handle) = local_http_server(response_body);
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_base_url = base_url;
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker = build_worker_client(&cfg).expect("devolens worker should build");

    let payload = "test-device-id:123456";
    let mut hasher = sha2::Sha256::new();
    hasher.update(payload.as_bytes());
    hasher.update(b":");
    hasher.update(b"devolens-token");
    let sig = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let signed_token = format!("devolens:{}:{}", payload, sig);

    let status = worker
        .validate_session(AccessToken::new(signed_token).unwrap())
        .await
        .expect("devolens validation should succeed");

    let _request = handle.join().expect("server thread should finish");

    assert_eq!(status, ValidationOutcome::Revoked);
}

#[tokio::test]
async fn test_devolens_access_token_integrity() {
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker = build_worker_client(&cfg).expect("devolens worker should build");

    // 1. Forged token with invalid signature
    let forged_token_1 = AccessToken::new("devolens:test-device-id:123456:invalid-signature").unwrap();
    let result_1 = worker.validate_session(forged_token_1).await;
    assert_eq!(result_1.unwrap_err(), AuthError::ReauthRequired);

    // 2. Token missing devolens prefix
    let forged_token_2 = AccessToken::new("test-device-id:123456").unwrap();
    let status_2 = worker.validate_session(forged_token_2).await.unwrap();
    assert_eq!(status_2, ValidationOutcome::ReauthRequired);

    // 3. Forged token with altered device id or invalid format
    let forged_token_3 = AccessToken::new("devolens:other-device:123456:some-sig").unwrap();
    let result_3 = worker.validate_session(forged_token_3).await;
    assert_eq!(result_3.unwrap_err(), AuthError::ReauthRequired);
}

#[tokio::test]
async fn devolens_activation_machine_limit_returns_device_already_bound() {
    let response_body = r#"{
      "result": 1,
      "message": "Machine limit reached."
    }"#.to_string();

    let (base_url, handle) = local_http_server(response_body);
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_base_url = base_url;
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker = build_worker_client(&cfg).expect("devolens worker should build");

    let result = worker.activate(activation_request()).await;
    let _request = handle.join().expect("server thread should finish");

    assert_eq!(result.unwrap_err(), AuthError::DeviceAlreadyBound);

    // Verify command mapping matches the frontend's expected error code
    let cmd_err = license_control_suite::modules::user_reg::auth_licensing_tauri::AuthCommandError::from(AuthError::DeviceAlreadyBound);
    assert_eq!(cmd_err.code, "device_already_bound");
}

#[tokio::test]
async fn test_request_device_reset_calls_devolens() {
    let response_body = r#"{
      "result": 0,
      "message": ""
    }"#.to_string();

    let (base_url, handle) = local_http_server(response_body);
    let mut cfg = worker_config(LicenseBackendMode::Devolens, String::new());
    cfg.devolens_base_url = base_url;
    cfg.devolens_access_token = "devolens-token".to_string();
    cfg.devolens_product_id = "1234".to_string();
    let worker = build_worker_client(&cfg).expect("devolens worker should build");

    let req = DeviceResetRequest {
        license_key: Some(LicenseKey::new("SECRET-LICENSE").unwrap()),
        masked_license_key: None,
        purchaser_email: None,
        device_public_key: DevicePublicKey::new("public").unwrap(),
        fingerprint: fingerprint(),
        app_version: "0.1.0".to_string(),
        timestamp_ms: 123456,
    };

    let status = worker.request_device_reset(req).await.expect("deactivate should succeed");
    let request = handle.join().expect("server thread should finish");

    assert!(request.starts_with("POST /api/key/Deactivate "));
    assert!(request.contains("ProductId=1234"));
    assert!(request.contains("Key=SECRET-LICENSE"));
    assert!(request.contains("MachineCode="));

    match status {
        DeviceResetStatus::Approved { request_id, decided_at_ms } => {
            assert_eq!(request_id.as_str(), "reset-123456");
            assert_eq!(decided_at_ms, 123456);
        }
        _ => panic!("Expected Approved reset status"),
    }
}

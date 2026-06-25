use license_control_suite::core::test_support::{FakeWorkerClient, TestService};
use license_control_suite::core::{
    AccessToken, BoundDeviceSummary, DeviceFingerprint, DeviceId, DevicePublicKey,
    DeviceResetStatus, LicenseKey, LocalStateStore, MaskedLicenseKey, ResetRequestId,
    SecretStore, SessionState,
};
use license_control_suite::desktop::tauri::{AuthAppState, AuthStateView};
use license_control_suite::desktop::tauri::deactivate_current_device_with_service;
use serde_json::Value;
use shorts_tauri_app::commands::{
    generate::{
        run_generate, run_generate_authorized, GenerateCommandArgs, GenerateEnvelope,
        GenerateShortsCommand,
    },
    health::health_check,
};
use std::fs;
use std::sync::Arc;

fn base_req() -> GenerateShortsCommand {
    GenerateShortsCommand {
        run_id: None,
        youtube_url: "https://youtube.com/watch?v=abc".into(),
        num_clips: 1,
        aspect_ratio: "9:16".into(),
        download_format: "720".into(),
        language: None,
        output_json: None,
        mode: "api".into(),
    }
}

fn unauthenticated_auth_state() -> AuthAppState {
    AuthAppState {
        service: Arc::new(TestService::new(FakeWorkerClient::new()).service),
    }
}

async fn licensed_auth_state() -> AuthAppState {
    let harness = TestService::new(FakeWorkerClient::new());
    let public_key = DevicePublicKey::new("public").expect("public key");
    harness
        .state
        .save_session_state(SessionState::Licensed {
            masked_license_key: MaskedLicenseKey::new("****-1234").expect("masked key"),
            bound_device: BoundDeviceSummary {
                device_id: DeviceId::from_public_key(&public_key),
                public_key,
                fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None)
                    .expect("fingerprint"),
            },
            token_expires_at_ms: 1_700_000_000_000,
            last_validated_at_ms: 1_700_000_000_000,
            next_validation_due_ms: 1_700_086_400_000,
        })
        .await
        .expect("licensed state should be saved");
    AuthAppState {
        service: Arc::new(harness.service),
    }
}

#[tokio::test]
async fn deactivate_current_device_returns_unauthenticated_state_on_success() {
    let request_id = ResetRequestId::new("reset-1").expect("request id");
    let harness = TestService::new(FakeWorkerClient::new().with_reset_request(Ok(
        DeviceResetStatus::Approved {
            request_id,
            decided_at_ms: 1,
        },
    )));
    harness
        .secrets
        .put_license_key(LicenseKey::new("LICENSE-1234").expect("license key"))
        .await
        .expect("license should be stored");
    harness
        .secrets
        .put_access_token(AccessToken::new("token").expect("access token"))
        .await
        .expect("access token should be stored");
    harness
        .state
        .save_session_state(SessionState::Licensed {
            masked_license_key: MaskedLicenseKey::new("••••-1234").expect("masked key"),
            bound_device: BoundDeviceSummary {
                device_id: DeviceId::new("device-id").expect("device id"),
                public_key: DevicePublicKey::new("public").expect("public key"),
                fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None)
                    .expect("fingerprint"),
            },
            token_expires_at_ms: 10,
            last_validated_at_ms: 1,
            next_validation_due_ms: 2,
        })
        .await
        .expect("licensed state should be stored");

    let view = deactivate_current_device_with_service(&harness.service)
        .await
        .expect("deactivate should succeed");

    assert!(matches!(view.auth_state, AuthStateView::Unauthenticated));
}

#[test]
fn generate_success_returns_envelope() {
    let (out, _) = run_generate(GenerateCommandArgs {
        request: base_req(),
        test_mode: Some("success".to_string()),
    });

    match out {
        GenerateEnvelope::Success { ok, result } => {
            assert!(ok);
            assert_eq!(result.mode, "api");
        }
        GenerateEnvelope::Failure { .. } => panic!("expected success envelope"),
    }
}

#[test]
fn generate_failure_returns_stable_error_envelope() {
    let (out, _) = run_generate(GenerateCommandArgs {
        request: base_req(),
        test_mode: Some("fail_transcribe".to_string()),
    });

    match out {
        GenerateEnvelope::Failure { ok, error } => {
            assert!(!ok);
            assert!(!error.error.is_empty());
            assert!(error.mode.is_some());
            assert!(error.source_video_url.is_some());
        }
        GenerateEnvelope::Success { .. } => panic!("expected failure envelope"),
    }
}

#[tokio::test]
async fn authorized_generate_rejects_unauthenticated_state() {
    let (out, events) = run_generate_authorized(
        GenerateCommandArgs {
            request: base_req(),
            test_mode: Some("success".to_string()),
        },
        &unauthenticated_auth_state(),
    )
    .await;

    match out {
        GenerateEnvelope::Failure { ok, error } => {
            assert!(!ok);
            assert_eq!(error.error, "license required");
            assert_eq!(error.mode.as_deref(), Some("api"));
            assert_eq!(
                error
                    .details
                    .as_ref()
                    .and_then(|details| details.get("code"))
                    .and_then(Value::as_str),
                Some("E_LICENSE_REQUIRED")
            );
        }
        GenerateEnvelope::Success { .. } => panic!("expected license failure"),
    }
    assert!(events.is_empty());
}

#[tokio::test]
async fn authorized_generate_allows_licensed_state() {
    let auth_state = licensed_auth_state().await;
    let (out, events) = run_generate_authorized(
        GenerateCommandArgs {
            request: base_req(),
            test_mode: Some("success".to_string()),
        },
        &auth_state,
    )
    .await;

    match out {
        GenerateEnvelope::Success { ok, result } => {
            assert!(ok);
            assert_eq!(result.mode, "api");
        }
        GenerateEnvelope::Failure { .. } => panic!("expected success envelope"),
    }
    assert!(!events.is_empty());
}

#[test]
fn event_order_includes_stage_start_end_and_status_changes() {
    let (_envelope, events) = run_generate(GenerateCommandArgs {
        request: base_req(),
        test_mode: Some("success_with_status".to_string()),
    });

    let events_json = serde_json::to_value(events).unwrap();
    let stages: Vec<&str> = events_json
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.get("stage").and_then(Value::as_str))
        .collect();

    assert!(stages
        .windows(2)
        .any(|w| w == ["download:start", "download:end"]));
    assert!(stages.iter().any(|s| *s == "muapi_poll:queued"));
    assert!(stages.iter().any(|s| *s == "muapi_poll:completed"));
}

#[test]
fn health_command_callable() {
    let health = health_check();
    assert_eq!(health.status, "ok");
}

#[test]
fn generate_writes_output_json_when_path_provided() {
    let out_path = std::env::temp_dir().join("shorts_tauri_output_test.json");
    let _ = fs::remove_file(&out_path);

    let mut req = base_req();
    req.output_json = Some(out_path.display().to_string());

    let (out, _) = run_generate(GenerateCommandArgs {
        request: req,
        test_mode: Some("success".to_string()),
    });

    match out {
        GenerateEnvelope::Success { .. } => {}
        GenerateEnvelope::Failure { .. } => panic!("expected success envelope"),
    }

    let written = fs::read_to_string(&out_path).expect("output json file should exist");
    let parsed: Value = serde_json::from_str(&written).expect("valid json");
    assert!(parsed.get("shorts").is_some());
    let _ = fs::remove_file(&out_path);
}

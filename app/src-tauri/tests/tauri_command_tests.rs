use serde_json::Value;
use std::fs;
use shorts_tauri_app::commands::{
    generate::{
        generate_shorts, run_generate, GenerateCommandArgs, GenerateEnvelope, GenerateShortsCommand,
    },
    health::{health_check, validate_runtime},
};

fn base_req() -> GenerateShortsCommand {
    GenerateShortsCommand {
        youtube_url: "https://youtube.com/watch?v=abc".into(),
        num_clips: 1,
        aspect_ratio: "9:16".into(),
        download_format: "720".into(),
        language: None,
        output_json: None,
        mode: "api".into(),
    }
}

#[test]
fn generate_success_returns_envelope() {
    let out = generate_shorts(GenerateCommandArgs {
        request: base_req(),
        test_mode: Some("success".to_string()),
    })
    .expect("generate success");

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
    let out = generate_shorts(GenerateCommandArgs {
        request: base_req(),
        test_mode: Some("fail_transcribe".to_string()),
    })
    .expect("command should resolve with envelope");

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
fn health_and_runtime_validation_commands_callable() {
    let health = health_check();
    assert_eq!(health.status, "ok");

    let validate = validate_runtime();
    assert!(validate.runtime.contains("python"));
}

#[test]
fn generate_writes_output_json_when_path_provided() {
    let out_path = std::env::temp_dir().join("shorts_tauri_output_test.json");
    let _ = fs::remove_file(&out_path);

    let mut req = base_req();
    req.output_json = Some(out_path.display().to_string());

    let out = generate_shorts(GenerateCommandArgs {
        request: req,
        test_mode: Some("success".to_string()),
    })
    .expect("generate success");

    match out {
        GenerateEnvelope::Success { .. } => {}
        GenerateEnvelope::Failure { .. } => panic!("expected success envelope"),
    }

    let written = fs::read_to_string(&out_path).expect("output json file should exist");
    let parsed: Value = serde_json::from_str(&written).expect("valid json");
    assert!(parsed.get("shorts").is_some());
    let _ = fs::remove_file(&out_path);
}

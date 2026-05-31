use serde_json::Value;
use shorts_tauri_app::commands::generate::{
    run_generate, GenerateCommandArgs, GenerateEnvelope, GenerateShortsCommand,
};
use shorts_tauri_app::core::observability::events::status_change_payload;

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

#[test]
fn fail_transcribe_maps_stage_code_retryable() {
    let (out, _events) = run_generate(GenerateCommandArgs {
        request: base_req(),
        test_mode: Some("fail_transcribe".to_string()),
    });

    match out {
        GenerateEnvelope::Failure { error, .. } => {
            let details = error.details.expect("details required");
            assert_eq!(
                details.get("stage").and_then(Value::as_str),
                Some("transcribe")
            );
            assert_eq!(
                details.get("code").and_then(Value::as_str),
                Some("E_TRANSCRIBE_FAILED")
            );
            assert_eq!(
                details.get("retryable").and_then(Value::as_bool),
                Some(false)
            );
        }
        _ => panic!("expected failure"),
    }
}

#[test]
fn muapi_status_payload_contains_structured_metadata() {
    let payload = status_change_payload("transcribe", "queued");
    assert_eq!(
        payload.get("event").and_then(Value::as_str),
        Some("progress")
    );
    assert_eq!(
        payload.get("stage").and_then(Value::as_str),
        Some("muapi_poll")
    );
    assert_eq!(
        payload.get("message").and_then(Value::as_str),
        Some("[muapi] transcribe: queued")
    );
    assert_eq!(
        payload.get("code").and_then(Value::as_str),
        Some("E_MUAPI_STATUS")
    );
    assert_eq!(
        payload.get("retryable").and_then(Value::as_bool),
        Some(true)
    );
}

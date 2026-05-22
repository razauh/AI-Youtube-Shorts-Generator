use serde_json::{json, Value};
use shorts_tauri_app::core::local_mode::bridge::{
    run_local_bridge, BridgeConfig, BridgeErrorCode, BridgeRequest,
};
use shorts_tauri_app::runtime::process_supervisor::{run_supervised, ProcessSpec};

fn bridge_script() -> String {
    "../../python_legacy/bridge_entry.py".to_string()
}

fn python_bin() -> String {
    std::env::var("PYTHON_BRIDGE_BIN").unwrap_or_else(|_| "../../.venv/bin/python".to_string())
}

fn base_request() -> BridgeRequest {
    BridgeRequest {
        version: "1".to_string(),
        action: "run_local".to_string(),
        payload: json!({
            "youtube_url": "https://youtube.com/watch?v=abc",
            "num_clips": 1,
            "aspect_ratio": "9:16",
            "download_format": "720",
            "language": Value::Null,
        }),
    }
}

fn base_config() -> BridgeConfig {
    BridgeConfig {
        python_bin: python_bin(),
        entry_script: bridge_script(),
        env: Vec::new(),
        timeout_ms: 5_000,
    }
}

#[test]
fn valid_json_roundtrip() {
    let req = base_request();
    let out = run_local_bridge(req, base_config()).expect("bridge should return json envelope");

    assert_eq!(out.version, "1");
    assert_eq!(out.ok, true);
    assert_eq!(
        out.result.get("mode").and_then(|v| v.as_str()),
        Some("local")
    );
}

#[test]
fn malformed_json_from_python_maps_error() {
    let mut req = base_request();
    req.action = "emit_malformed".to_string();

    let err = run_local_bridge(req, base_config()).expect_err("malformed json must fail");

    assert_eq!(err.code, BridgeErrorCode::MalformedJson);
}

#[test]
fn hang_timeout_then_terminate() {
    let mut req = base_request();
    req.action = "sleep".to_string();
    req.payload = json!({"seconds": 5});

    let err = run_local_bridge(
        req,
        BridgeConfig {
            env: Vec::new(),
            timeout_ms: 150,
            ..base_config()
        },
    )
    .expect_err("hung process must timeout");

    assert_eq!(err.code, BridgeErrorCode::Timeout);
}

#[test]
fn non_zero_exit_mapping() {
    let mut req = base_request();
    req.action = "exit_nonzero".to_string();

    let err = run_local_bridge(req, base_config()).expect_err("non-zero must error");

    assert_eq!(err.code, BridgeErrorCode::ProcessExit);
    assert_eq!(err.exit_code, Some(17));
}

#[test]
fn stderr_capture_propagation() {
    let mut req = base_request();
    req.action = "stderr_then_fail".to_string();

    let err = run_local_bridge(req, base_config()).expect_err("must include stderr");

    assert_eq!(err.code, BridgeErrorCode::PythonError);
    assert!(err.stderr.contains("bridge stderr test"));
}

#[test]
fn parity_bridge_local_output_matches_fixture_shape() {
    let req = base_request();
    let out = run_local_bridge(req, base_config()).expect("bridge success expected");

    let got = out.result;
    assert!(got.get("source_video_url").is_some());
    assert!(got.get("transcript").is_some());
    assert!(got.get("highlights").is_some());
    assert!(got.get("shorts").is_some());
}

#[test]
fn process_env_overrides_reach_child_without_debug_value_leak() {
    let out = run_supervised(ProcessSpec {
        program: python_bin(),
        args: vec![
            "-c".to_string(),
            "import os; print(os.environ.get('LOCAL_WHISPER_MODEL', '') + ':' + os.environ.get('OPENAI_API_KEY', ''))".to_string(),
        ],
        env: vec![
            ("LOCAL_WHISPER_MODEL".to_string(), "small".to_string()),
            ("OPENAI_API_KEY".to_string(), "test-openai-secret".to_string()),
        ],
        stdin_bytes: Vec::new(),
        timeout_ms: 5_000,
    })
    .expect("process should run");

    assert_eq!(out.status_code, Some(0));
    assert_eq!(out.stdout.trim(), "small:test-openai-secret");

    let debug = format!(
        "{:?}",
        BridgeConfig {
            python_bin: python_bin(),
            entry_script: bridge_script(),
            env: vec![(
                "OPENAI_API_KEY".to_string(),
                "test-openai-secret".to_string()
            )],
            timeout_ms: 5_000,
        }
    );
    assert!(debug.contains("OPENAI_API_KEY"));
    assert!(!debug.contains("test-openai-secret"));
}

use serde_json::json;
use shorts_tauri_app::commands::generate::{
    run_generate, GenerateCommandArgs, GenerateEnvelope, GenerateShortsCommand,
};
use shorts_tauri_app::runtime::fs_output::{write_result_json_atomic, FsOutputErrorCode};
use shorts_tauri_app::runtime::tool_resolver::{
    resolve_tool, validate_runtime_tools, ResolveConfig, ToolKind,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("shorts_tauri_{tag}_{nanos}"));
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

#[test]
fn output_json_written_and_valid() {
    let dir = unique_temp_dir("json_ok");
    let out = dir.join("result.json");
    let value = json!({"mode":"local","shorts":[{"clip_url":"x.mp4"}]});

    write_result_json_atomic(&out, &value).expect("write json");

    let raw = fs::read_to_string(&out).expect("read result");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert_eq!(parsed, value);
}

#[test]
fn output_json_invalid_path_returns_structured_error() {
    let dir = unique_temp_dir("json_err");
    let invalid = dir.join("missing").join("result.json");
    let value = json!({"ok":true});

    let err = write_result_json_atomic(&invalid, &value).expect_err("must fail");
    assert_eq!(err.code, FsOutputErrorCode::ParentMissing);
    assert!(err.message.contains("parent directory"));
}

#[test]
fn runtime_validator_reports_missing_tool_with_actionable_message() {
    let result = validate_runtime_tools(ResolveConfig {
        bundled_dir: None,
        allow_system_path: false,
        python_bin: "python3".to_string(),
        required_tools: vec![ToolKind::Ffmpeg],
    });

    assert_eq!(result.tools.len(), 1);
    assert!(!result.tools[0].ok);
    assert_eq!(result.tools[0].tool, "ffmpeg");
    assert!(result.tools[0].message.contains("Install ffmpeg"));
}

#[test]
fn resolver_prefers_bundled_then_system_path() {
    let dir = unique_temp_dir("resolve_order");
    let bundled = dir.join("bin");
    fs::create_dir_all(&bundled).expect("mkdir bundled");
    let bundled_ffmpeg = bundled.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });
    fs::write(&bundled_ffmpeg, b"fake").expect("write fake tool");

    let resolved = resolve_tool(
        ToolKind::Ffmpeg,
        ResolveConfig {
            bundled_dir: Some(bundled.clone()),
            allow_system_path: true,
            python_bin: "python3".to_string(),
            required_tools: vec![],
        },
    )
    .expect("resolver should find bundled");

    assert_eq!(resolved.path, bundled_ffmpeg);
    assert_eq!(resolved.source, "bundled");
}

#[test]
fn output_json_matches_direct_pipeline_result_structure() {
    let (envelope, _) = run_generate(GenerateCommandArgs {
        request: GenerateShortsCommand {
            youtube_url: "https://youtube.com/watch?v=abc".to_string(),
            num_clips: 1,
            aspect_ratio: "9:16".to_string(),
            download_format: "720".to_string(),
            language: None,
            output_json: None,
            mode: "api".to_string(),
        },
        test_mode: Some("success".to_string()),
    });

    let result_value = match envelope {
        GenerateEnvelope::Success { result, .. } => serde_json::to_value(result).expect("to value"),
        GenerateEnvelope::Failure { .. } => panic!("expected success"),
    };

    let dir = unique_temp_dir("json_parity");
    let out = dir.join("result.json");
    write_result_json_atomic(&out, &result_value).expect("write json");
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out).expect("read")).expect("valid json");
    assert_eq!(parsed, result_value);
}

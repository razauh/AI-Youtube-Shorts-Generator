use crate::core::config::Config;
use crate::core::contracts::{ErrorEnvelope, PipelineSuccess};
use crate::runtime::python_runtime::{
    invoke_python, resolve_python_bridge_paths, with_python_runtime_env, PythonInvokeRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub version: String,
    pub action: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub version: String,
    pub ok: bool,
    #[serde(default)]
    pub result: Value,
    #[serde(default)]
    pub error: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeErrorCode {
    ContractVersion,
    Timeout,
    ProcessExit,
    MalformedJson,
    PythonError,
    Runtime,
}

#[derive(Debug, Clone)]
pub struct BridgeError {
    pub code: BridgeErrorCode,
    pub message: String,
    pub exit_code: Option<i32>,
    pub stderr: String,
}

#[derive(Clone)]
pub struct BridgeConfig {
    pub python_bin: String,
    pub entry_script: String,
    pub env: Vec<(String, String)>,
    pub timeout_ms: u64,
}

impl fmt::Debug for BridgeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BridgeConfig")
            .field("python_bin", &self.python_bin)
            .field("entry_script", &self.entry_script)
            .field(
                "env_keys",
                &self.env.iter().map(|(key, _)| key).collect::<Vec<_>>(),
            )
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

pub fn run_local_bridge(
    req: BridgeRequest,
    cfg: BridgeConfig,
) -> Result<BridgeResponse, BridgeError> {
    if req.version != "1" {
        return Err(BridgeError {
            code: BridgeErrorCode::ContractVersion,
            message: format!("unsupported bridge contract version: {}", req.version),
            exit_code: None,
            stderr: String::new(),
        });
    }

    let stdin_json = serde_json::to_vec(&req).map_err(|e| BridgeError {
        code: BridgeErrorCode::Runtime,
        message: e.to_string(),
        exit_code: None,
        stderr: String::new(),
    })?;

    let proc = invoke_python(PythonInvokeRequest {
        python_bin: cfg.python_bin,
        entry_script: cfg.entry_script,
        env: cfg.env,
        stdin_json,
        timeout_ms: cfg.timeout_ms,
    })
    .map_err(|e| BridgeError {
        code: BridgeErrorCode::Runtime,
        message: format!("python invoke failed: {e:?}"),
        exit_code: None,
        stderr: String::new(),
    })?;

    if proc.timed_out {
        return Err(BridgeError {
            code: BridgeErrorCode::Timeout,
            message: "python bridge timed out".to_string(),
            exit_code: proc.status_code,
            stderr: proc.stderr,
        });
    }

    if proc.status_code.unwrap_or(1) != 0 {
        return Err(BridgeError {
            code: BridgeErrorCode::ProcessExit,
            message: "python bridge exited non-zero".to_string(),
            exit_code: proc.status_code,
            stderr: proc.stderr,
        });
    }

    let parsed: BridgeResponse =
        serde_json::from_str(proc.stdout.trim()).map_err(|e| BridgeError {
            code: BridgeErrorCode::MalformedJson,
            message: format!("invalid bridge json: {e}"),
            exit_code: proc.status_code,
            stderr: proc.stderr.clone(),
        })?;

    if parsed.version != "1" {
        return Err(BridgeError {
            code: BridgeErrorCode::ContractVersion,
            message: format!("unsupported response version: {}", parsed.version),
            exit_code: proc.status_code,
            stderr: proc.stderr,
        });
    }

    if !parsed.ok {
        return Err(BridgeError {
            code: BridgeErrorCode::PythonError,
            message: parsed
                .error
                .as_ref()
                .and_then(|v| v.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("python bridge returned error")
                .to_string(),
            exit_code: proc.status_code,
            stderr: proc.stderr,
        });
    }

    Ok(parsed)
}

pub fn run_local_pipeline_bridge(
    youtube_url: String,
    num_clips: usize,
    aspect_ratio: String,
    download_format: String,
    language: Option<String>,
) -> Result<PipelineSuccess, ErrorEnvelope> {
    let req = BridgeRequest {
        version: "1".to_string(),
        action: "run_local".to_string(),
        payload: json!({
            "youtube_url": youtube_url,
            "num_clips": num_clips,
            "aspect_ratio": aspect_ratio,
            "download_format": download_format,
            "language": language,
        }),
    };

    let bridge = resolve_python_bridge_paths();
    let env = local_bridge_env().map_err(|e| ErrorEnvelope {
        mode: Some("local".to_string()),
        source_video_url: None,
        error: e.to_string(),
        details: Some(json!({"stage": "local_bridge_config"})),
    })?;
    let cfg = BridgeConfig {
        python_bin: bridge.python_bin.clone(),
        entry_script: bridge.entry_script.clone(),
        env: with_python_runtime_env(env, &bridge),
        timeout_ms: std::env::var("PYTHON_BRIDGE_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300_000),
    };

    let out = run_local_bridge(req, cfg).map_err(|e| ErrorEnvelope {
        mode: Some("local".to_string()),
        source_video_url: None,
        error: e.message,
        details: Some(json!({
            "stage": "local_bridge",
            "code": format!("{:?}", e.code),
            "exit_code": e.exit_code,
            "stderr": e.stderr,
        })),
    })?;

    serde_json::from_value(out.result).map_err(|e| ErrorEnvelope {
        mode: Some("local".to_string()),
        source_video_url: None,
        error: format!("invalid local bridge payload: {e}"),
        details: Some(json!({"stage": "local_bridge"})),
    })
}

fn local_bridge_env() -> Result<Vec<(String, String)>, crate::core::errors::ConfigError> {
    let config = Config::from_env()?;
    let mut env = vec![
        (
            "LOCAL_WHISPER_MODEL".to_string(),
            config.local_whisper_model,
        ),
        (
            "LOCAL_WHISPER_DEVICE".to_string(),
            config.local_whisper_device,
        ),
    ];

    if let Ok(root) = std::env::var("LOCAL_MODEL_CACHE_DIR") {
        if !root.trim().is_empty() {
            env.push(("LOCAL_MODEL_CACHE_DIR".to_string(), root));
        }
    } else if let Some(cache_dir) = default_local_model_cache_dir() {
        env.push(("LOCAL_MODEL_CACHE_DIR".to_string(), cache_dir));
    }

    if !config.openai_api_key.is_empty() {
        env.push(("OPENAI_API_KEY".to_string(), config.openai_api_key));
    }

    Ok(env)
}

fn default_local_model_cache_dir() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return std::env::var("APPDATA")
            .ok()
            .map(|root| format!("{root}\\ai-youtube-shorts-generator\\models\\huggingface"));
    }

    #[cfg(target_os = "macos")]
    {
        return std::env::var("HOME").ok().map(|home| {
            format!(
                "{home}/Library/Application Support/ai-youtube-shorts-generator/models/huggingface"
            )
        });
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var("HOME").ok().map(|home| {
            format!("{home}/.local/share/ai-youtube-shorts-generator/models/huggingface")
        })
    }
}

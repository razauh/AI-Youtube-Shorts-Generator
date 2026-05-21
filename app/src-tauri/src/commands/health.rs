use crate::core::config::Config;
use crate::runtime::tool_resolver::{validate_runtime_tools, ResolveConfig, ToolKind, ToolStatus};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeValidation {
    pub runtime: String,
    pub bridge_entry: String,
    pub bridge_entry_exists: bool,
    pub ok: bool,
    pub tools: Vec<ToolStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigSummary {
    pub license_backend_mode: String,
    pub license_worker_endpoint: String,
    pub license_worker_endpoint_kind: String,
    pub muapi_configured: bool,
    pub openai_configured: bool,
    pub local_whisper_model: String,
    pub local_whisper_device: String,
    pub license_worker_timeout_ms: u64,
    pub license_worker_retry_attempts: u32,
}

#[tauri::command]
pub fn health_check() -> HealthCheck {
    HealthCheck {
        status: "ok".to_string(),
    }
}

#[tauri::command]
pub fn validate_runtime() -> RuntimeValidation {
    let python = std::env::var("PYTHON_BRIDGE_BIN").unwrap_or_else(|_| "python3".to_string());
    let entry = std::env::var("PYTHON_BRIDGE_ENTRY")
        .unwrap_or_else(|_| "../../python_legacy/bridge_entry.py".to_string());
    let bundled_dir = std::env::var("BUNDLED_RUNTIME_DIR").ok().map(Into::into);

    let _ = Config::from_env();
    let validation = validate_runtime_tools(ResolveConfig {
        bundled_dir,
        allow_system_path: true,
        python_bin: python.clone(),
        required_tools: vec![ToolKind::Python, ToolKind::Ffmpeg, ToolKind::YtDlp],
    });

    RuntimeValidation {
        runtime: format!("python:{python}"),
        bridge_entry_exists: Path::new(&entry).exists(),
        bridge_entry: entry,
        ok: validation.ok,
        tools: validation.tools,
    }
}

#[tauri::command]
pub fn app_config_summary() -> Result<AppConfigSummary, String> {
    let cfg = Config::from_env().map_err(|err| err.to_string())?;
    let (license_worker_endpoint, license_worker_endpoint_kind) =
        summarize_worker_endpoint(&cfg.license_worker_base_url);

    Ok(AppConfigSummary {
        license_backend_mode: cfg.license_backend_mode.as_str().to_string(),
        license_worker_endpoint,
        license_worker_endpoint_kind,
        muapi_configured: !cfg.muapi_api_key.is_empty(),
        openai_configured: !cfg.openai_api_key.is_empty(),
        local_whisper_model: cfg.local_whisper_model,
        local_whisper_device: cfg.local_whisper_device,
        license_worker_timeout_ms: cfg.license_worker_timeout_ms,
        license_worker_retry_attempts: cfg.license_worker_retry_attempts,
    })
}

fn summarize_worker_endpoint(raw: &str) -> (String, String) {
    let without_scheme = raw
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(raw)
        .trim();
    let authority = without_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .trim();
    let host = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority)
        .split(':')
        .next()
        .unwrap_or("")
        .trim_matches(['[', ']']);
    let normalized = host.to_ascii_lowercase();

    if normalized.is_empty() {
        return ("not configured".to_string(), "unknown".to_string());
    }

    if matches!(
        normalized.as_str(),
        "localhost" | "127.0.0.1" | "0.0.0.0" | "::1"
    ) || normalized.starts_with("192.168.")
        || normalized.starts_with("10.")
        || normalized.starts_with("172.16.")
        || normalized.starts_with("172.17.")
        || normalized.starts_with("172.18.")
        || normalized.starts_with("172.19.")
        || normalized.starts_with("172.20.")
        || normalized.starts_with("172.21.")
        || normalized.starts_with("172.22.")
        || normalized.starts_with("172.23.")
        || normalized.starts_with("172.24.")
        || normalized.starts_with("172.25.")
        || normalized.starts_with("172.26.")
        || normalized.starts_with("172.27.")
        || normalized.starts_with("172.28.")
        || normalized.starts_with("172.29.")
        || normalized.starts_with("172.30.")
        || normalized.starts_with("172.31.")
    {
        return ("local/private worker".to_string(), "local".to_string());
    }

    (host.to_string(), "remote".to_string())
}

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

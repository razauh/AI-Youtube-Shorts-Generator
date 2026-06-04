use crate::core::config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigSummary {
    pub license_backend_mode: String,
    pub license_worker_endpoint: String,
    pub license_worker_endpoint_kind: String,
    pub muapi_configured: bool,
    pub openai_configured: bool,
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

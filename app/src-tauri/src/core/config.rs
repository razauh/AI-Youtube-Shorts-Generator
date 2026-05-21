use crate::core::errors::ConfigError;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub muapi_api_key: String,
    pub muapi_base_url: String,
    pub muapi_poll_interval_seconds: f64,
    pub muapi_poll_timeout_seconds: f64,
    pub openai_api_key: String,
    pub openai_model: String,
    pub local_whisper_model: String,
    pub local_whisper_device: String,
    pub local_output_dir: String,
    pub license_worker_base_url: String,
    pub license_storage_namespace: String,
    pub license_keychain_service: String,
    pub license_backend_mode: LicenseBackendMode,
    pub license_worker_timeout_ms: u64,
    pub license_worker_retry_attempts: u32,
    pub license_worker_retry_backoff_ms: u64,
    pub license_worker_circuit_breaker_failure_threshold: u32,
    pub license_worker_circuit_breaker_cooldown_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LicenseWorkerConfig {
    pub backend_mode: LicenseBackendMode,
    pub base_url: String,
    pub storage_namespace: String,
    pub keychain_service: String,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_backoff_ms: u64,
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_cooldown_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseBackendMode {
    Reference,
    Hosted,
    Mock,
}

impl LicenseBackendMode {
    pub fn parse(value: &str) -> Result<Self, ConfigError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "reference" => Ok(Self::Reference),
            "hosted" => Ok(Self::Hosted),
            "mock" => Ok(Self::Mock),
            _ => Err(ConfigError::InvalidLicenseBackendMode {
                value: value.to_string(),
            }),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Hosted => "hosted",
            Self::Mock => "mock",
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let muapi_api_key = read_env_trimmed("MUAPI_API_KEY", "");
        let muapi_base_url = read_env_trimmed("MUAPI_BASE_URL", "https://api.muapi.ai/api/v1")
            .trim_end_matches('/')
            .to_string();
        let muapi_poll_interval_seconds = parse_float_env("MUAPI_POLL_INTERVAL", "5")?;
        let muapi_poll_timeout_seconds = parse_float_env("MUAPI_POLL_TIMEOUT", "600")?;
        let openai_api_key = read_env_trimmed("OPENAI_API_KEY", "");
        let openai_model = read_env_trimmed("OPENAI_MODEL", "gpt-4o-mini");
        let local_whisper_model = read_env_trimmed("LOCAL_WHISPER_MODEL", "base");
        let local_whisper_device = read_env_trimmed("LOCAL_WHISPER_DEVICE", "auto");
        let local_output_dir = read_env_trimmed("LOCAL_OUTPUT_DIR", "output");
        let license_worker_base_url =
            read_env_trimmed("LICENSE_WORKER_BASE_URL", "http://127.0.0.1:8787")
                .trim_end_matches('/')
                .to_string();
        let license_storage_namespace =
            read_env_trimmed("LICENSE_STORAGE_NAMESPACE", "desktop-client");
        let license_keychain_service =
            read_env_trimmed("LICENSE_KEYCHAIN_SERVICE", "ai-youtube-shorts-generator");
        let license_backend_mode =
            LicenseBackendMode::parse(&read_env_trimmed("LICENSE_BACKEND_MODE", "reference"))?;
        let license_worker_timeout_ms = parse_u64_env("LICENSE_WORKER_TIMEOUT_MS", "10000")?;
        let license_worker_retry_attempts = parse_u32_env("LICENSE_WORKER_RETRY_ATTEMPTS", "2")?;
        let license_worker_retry_backoff_ms =
            parse_u64_env("LICENSE_WORKER_RETRY_BACKOFF_MS", "150")?;
        let license_worker_circuit_breaker_failure_threshold =
            parse_u32_env("LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD", "3")?;
        let license_worker_circuit_breaker_cooldown_ms =
            parse_u64_env("LICENSE_WORKER_CIRCUIT_COOLDOWN_MS", "30000")?;

        validate_license_worker_config(
            license_backend_mode,
            &license_worker_base_url,
            &license_storage_namespace,
            &license_keychain_service,
            license_worker_timeout_ms,
            license_worker_retry_attempts,
            license_worker_circuit_breaker_failure_threshold,
        )?;

        Ok(Self {
            muapi_api_key,
            muapi_base_url,
            muapi_poll_interval_seconds,
            muapi_poll_timeout_seconds,
            openai_api_key,
            openai_model,
            local_whisper_model,
            local_whisper_device,
            local_output_dir,
            license_worker_base_url,
            license_storage_namespace,
            license_keychain_service,
            license_backend_mode,
            license_worker_timeout_ms,
            license_worker_retry_attempts,
            license_worker_retry_backoff_ms,
            license_worker_circuit_breaker_failure_threshold,
            license_worker_circuit_breaker_cooldown_ms,
        })
    }

    pub fn license_worker_config(&self) -> LicenseWorkerConfig {
        LicenseWorkerConfig {
            backend_mode: self.license_backend_mode,
            base_url: self.license_worker_base_url.clone(),
            storage_namespace: self.license_storage_namespace.clone(),
            keychain_service: self.license_keychain_service.clone(),
            timeout_ms: self.license_worker_timeout_ms,
            retry_attempts: self.license_worker_retry_attempts,
            retry_backoff_ms: self.license_worker_retry_backoff_ms,
            circuit_breaker_failure_threshold: self
                .license_worker_circuit_breaker_failure_threshold,
            circuit_breaker_cooldown_ms: self.license_worker_circuit_breaker_cooldown_ms,
        }
    }

    pub fn require_api_key(&self) -> Result<&str, ConfigError> {
        if self.muapi_api_key.is_empty() {
            return Err(ConfigError::MissingApiKey);
        }
        Ok(self.muapi_api_key.as_str())
    }

    pub fn require_openai_key(&self) -> Result<&str, ConfigError> {
        if self.openai_api_key.is_empty() {
            return Err(ConfigError::MissingOpenAiKey);
        }
        Ok(self.openai_api_key.as_str())
    }
}

fn read_env_trimmed(key: &str, default: &str) -> String {
    std::env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .trim()
        .to_string()
}

pub fn load_env_files_near_current_dir() {
    if let Ok(current_dir) = std::env::current_dir() {
        load_env_files_from(current_dir);
    }
}

pub fn load_env_files_from(start_dir: impl AsRef<Path>) {
    let mut candidates = Vec::new();
    for dir in start_dir.as_ref().ancestors() {
        candidates.push(dir.join(".env"));
    }
    candidates.reverse();

    let existing_env: HashSet<String> = std::env::vars().map(|(key, _)| key).collect();
    for path in candidates {
        load_env_file(&path, &existing_env);
    }
}

fn load_env_file(path: &Path, existing_env: &HashSet<String>) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return;
    };

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || existing_env.contains(key) {
            continue;
        }
        let value = unquote_env_value(value.trim());
        unsafe {
            std::env::set_var(key, value);
        }
    }
}

fn unquote_env_value(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}

fn parse_float_env(var_name: &'static str, default: &'static str) -> Result<f64, ConfigError> {
    let raw = std::env::var(var_name).unwrap_or_else(|_| default.to_string());
    raw.parse::<f64>().map_err(|_| ConfigError::InvalidFloat {
        var_name,
        value: raw,
    })
}

fn parse_u64_env(var_name: &'static str, default: &'static str) -> Result<u64, ConfigError> {
    let raw = std::env::var(var_name).unwrap_or_else(|_| default.to_string());
    raw.parse::<u64>().map_err(|_| ConfigError::InvalidInteger {
        var_name,
        value: raw,
    })
}

fn parse_u32_env(var_name: &'static str, default: &'static str) -> Result<u32, ConfigError> {
    let raw = std::env::var(var_name).unwrap_or_else(|_| default.to_string());
    raw.parse::<u32>().map_err(|_| ConfigError::InvalidInteger {
        var_name,
        value: raw,
    })
}

fn validate_license_worker_config(
    mode: LicenseBackendMode,
    base_url: &str,
    namespace: &str,
    keychain_service: &str,
    timeout_ms: u64,
    retry_attempts: u32,
    circuit_failure_threshold: u32,
) -> Result<(), ConfigError> {
    if mode != LicenseBackendMode::Mock
        && !(base_url.starts_with("http://") || base_url.starts_with("https://"))
    {
        return Err(ConfigError::InvalidConfigValue {
            var_name: "LICENSE_WORKER_BASE_URL",
            reason: "must start with http:// or https://",
        });
    }
    if namespace.trim().is_empty() {
        return Err(ConfigError::InvalidConfigValue {
            var_name: "LICENSE_STORAGE_NAMESPACE",
            reason: "must not be empty",
        });
    }
    if keychain_service.trim().is_empty() {
        return Err(ConfigError::InvalidConfigValue {
            var_name: "LICENSE_KEYCHAIN_SERVICE",
            reason: "must not be empty",
        });
    }
    if timeout_ms == 0 {
        return Err(ConfigError::InvalidConfigValue {
            var_name: "LICENSE_WORKER_TIMEOUT_MS",
            reason: "must be greater than zero",
        });
    }
    if retry_attempts == 0 {
        return Err(ConfigError::InvalidConfigValue {
            var_name: "LICENSE_WORKER_RETRY_ATTEMPTS",
            reason: "must be greater than zero",
        });
    }
    if circuit_failure_threshold == 0 {
        return Err(ConfigError::InvalidConfigValue {
            var_name: "LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD",
            reason: "must be greater than zero",
        });
    }
    Ok(())
}

use crate::core::errors::ConfigError;

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
        })
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

fn parse_float_env(var_name: &'static str, default: &'static str) -> Result<f64, ConfigError> {
    let raw = std::env::var(var_name).unwrap_or_else(|_| default.to_string());
    raw.parse::<f64>().map_err(|_| ConfigError::InvalidFloat {
        var_name,
        value: raw,
    })
}

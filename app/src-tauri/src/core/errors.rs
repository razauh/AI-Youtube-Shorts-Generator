use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    MissingApiKey,
    MissingOpenAiKey,
    InvalidFloat {
        var_name: &'static str,
        value: String,
    },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingApiKey => write!(
                f,
                "MUAPI_API_KEY is not set. Add it to your .env file or export it as an env var."
            ),
            Self::MissingOpenAiKey => write!(
                f,
                "OPENAI_API_KEY is not set. Local mode needs an OpenAI key for highlight ranking. Add it to your .env or export it, or switch back to --mode api."
            ),
            Self::InvalidFloat { var_name, value } => {
                write!(f, "invalid float for {var_name}: {value}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStage {
    Config,
    Download,
    Transcribe,
    Highlights,
    Clip,
    Poll,
    Submit,
    Runtime,
}

impl ErrorStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Download => "download",
            Self::Transcribe => "transcribe",
            Self::Highlights => "highlights",
            Self::Clip => "clip",
            Self::Poll => "poll",
            Self::Submit => "submit",
            Self::Runtime => "runtime",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ConfigMissingApiKey,
    ConfigMissingOpenAiKey,
    ConfigInvalidFloat,
    DownloadFailed,
    TranscribeFailed,
    NoTranscriptSegments,
    HighlightsFailed,
    NoHighlights,
    ClipFailed,
    MuApiStatus,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConfigMissingApiKey => "E_CONFIG_MISSING_API_KEY",
            Self::ConfigMissingOpenAiKey => "E_CONFIG_MISSING_OPENAI_KEY",
            Self::ConfigInvalidFloat => "E_CONFIG_INVALID_FLOAT",
            Self::DownloadFailed => "E_DOWNLOAD_FAILED",
            Self::TranscribeFailed => "E_TRANSCRIBE_FAILED",
            Self::NoTranscriptSegments => "E_NO_TRANSCRIPT_SEGMENTS",
            Self::HighlightsFailed => "E_HIGHLIGHTS_FAILED",
            Self::NoHighlights => "E_NO_HIGHLIGHTS",
            Self::ClipFailed => "E_CLIP_FAILED",
            Self::MuApiStatus => "E_MUAPI_STATUS",
        }
    }
}

pub fn redact_message(message: &str) -> String {
    // Keep payload context, but strip obvious provider payload blobs from logs.
    message.replace(
        "\"provider_payload\":",
        "\"provider_payload\":\"[REDACTED]\"",
    )
}

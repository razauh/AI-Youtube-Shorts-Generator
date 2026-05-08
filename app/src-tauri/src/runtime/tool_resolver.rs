use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Python,
    Ffmpeg,
    YtDlp,
}

impl ToolKind {
    fn executable_name(self) -> &'static str {
        match self {
            ToolKind::Python => {
                if cfg!(windows) {
                    "python.exe"
                } else {
                    "python3"
                }
            }
            ToolKind::Ffmpeg => {
                if cfg!(windows) {
                    "ffmpeg.exe"
                } else {
                    "ffmpeg"
                }
            }
            ToolKind::YtDlp => {
                if cfg!(windows) {
                    "yt-dlp.exe"
                } else {
                    "yt-dlp"
                }
            }
        }
    }

    fn name(self) -> &'static str {
        match self {
            ToolKind::Python => "python",
            ToolKind::Ffmpeg => "ffmpeg",
            ToolKind::YtDlp => "yt-dlp",
        }
    }

    fn install_hint(self) -> &'static str {
        match self {
            ToolKind::Python => "Install Python 3 and set PYTHON_BRIDGE_BIN if needed.",
            ToolKind::Ffmpeg => {
                "Install ffmpeg and ensure binary available in bundled runtime or PATH."
            }
            ToolKind::YtDlp => {
                "Install yt-dlp (pip install yt-dlp) and ensure command is available."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolveConfig {
    pub bundled_dir: Option<PathBuf>,
    pub allow_system_path: bool,
    pub python_bin: String,
    pub required_tools: Vec<ToolKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTool {
    pub tool: String,
    pub path: PathBuf,
    pub source: String,
    pub version: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub tool: String,
    pub ok: bool,
    pub path: Option<String>,
    pub source: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeValidationResult {
    pub ok: bool,
    pub tools: Vec<ToolStatus>,
}

pub fn resolve_tool(kind: ToolKind, cfg: ResolveConfig) -> Option<ResolvedTool> {
    if kind == ToolKind::Python {
        let path = PathBuf::from(&cfg.python_bin);
        if path.components().count() > 1 && path.exists() {
            return Some(ResolvedTool {
                tool: kind.name().to_string(),
                path,
                source: "configured".to_string(),
                version: None,
                checksum: None,
            });
        }
    }

    let exe = match kind {
        ToolKind::Python => cfg.python_bin.as_str(),
        _ => kind.executable_name(),
    };

    if let Some(dir) = cfg.bundled_dir.as_ref() {
        let candidate = dir.join(exe);
        if candidate.exists() {
            return Some(ResolvedTool {
                tool: kind.name().to_string(),
                path: normalize_path(candidate),
                source: "bundled".to_string(),
                version: None,
                checksum: None,
            });
        }
    }

    if cfg.allow_system_path {
        if let Some(path) = resolve_from_path(exe) {
            return Some(ResolvedTool {
                tool: kind.name().to_string(),
                path: normalize_path(path),
                source: "path".to_string(),
                version: None,
                checksum: None,
            });
        }
    }

    None
}

pub fn validate_runtime_tools(cfg: ResolveConfig) -> RuntimeValidationResult {
    let mut tools = Vec::new();
    for kind in cfg.required_tools.iter().copied() {
        let found = resolve_tool(kind, cfg.clone());
        match found {
            Some(t) => tools.push(ToolStatus {
                tool: t.tool,
                ok: true,
                path: Some(t.path.display().to_string()),
                source: Some(t.source),
                message: "ok".to_string(),
            }),
            None => tools.push(ToolStatus {
                tool: kind.name().to_string(),
                ok: false,
                path: None,
                source: None,
                message: format!("Missing tool '{}'. {}", kind.name(), kind.install_hint()),
            }),
        }
    }
    let ok = tools.iter().all(|t| t.ok);
    RuntimeValidationResult { ok, tools }
}

fn resolve_from_path(exe: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;
    for dir in env::split_paths(&path_env) {
        let candidate = dir.join(exe);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        Path::new(".").join(path)
    }
}

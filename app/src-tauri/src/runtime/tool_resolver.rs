use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

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
            ToolKind::Python => "The packaged runtime is missing Python. Reinstall the app.",
            ToolKind::Ffmpeg => "The packaged runtime is missing ffmpeg. Reinstall the app.",
            ToolKind::YtDlp => "The packaged runtime is missing yt-dlp. Reinstall the app.",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolveConfig {
    pub bundled_dir: Option<PathBuf>,
    pub allow_system_path: bool,
    pub python_bin: String,
    pub required_tools: Vec<ToolKind>,
    pub required_python_modules: Vec<String>,
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
    pub python_packages: Vec<ToolStatus>,
    pub local_runtime_ready: bool,
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
    let mut python_packages = Vec::new();
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
                message: if cfg.bundled_dir.is_some() {
                    format!(
                        "Missing tool '{}'. {}",
                        kind.name(),
                        kind.install_hint()
                    )
                } else {
                    format!(
                        "Missing tool '{}'. {}",
                        kind.name(),
                        match kind {
                            ToolKind::Python => {
                                "Install Python locally for development or rebuild the packaged runtime."
                            }
                            ToolKind::Ffmpeg => "Install ffmpeg locally for development or rebuild the packaged runtime.",
                            ToolKind::YtDlp => "Install yt-dlp locally for development or rebuild the packaged runtime.",
                        }
                    )
                },
            }),
        }
    }
    let python_ok = tools
        .iter()
        .find(|tool| tool.tool == ToolKind::Python.name())
        .map(|tool| tool.ok)
        .unwrap_or(false);

    if python_ok {
        for module in cfg.required_python_modules {
            let probe = check_python_module(&cfg.python_bin, &module);
            python_packages.push(ToolStatus {
                tool: module.clone(),
                ok: probe.is_ok(),
                path: None,
                source: None,
                message: probe.unwrap_or_else(|message| message),
            });
        }
    } else {
        for module in cfg.required_python_modules {
            python_packages.push(ToolStatus {
                tool: module,
                ok: false,
                path: None,
                source: None,
                message: "Python runtime is unavailable; cannot verify module.".to_string(),
            });
        }
    }

    let tools_ok = tools.iter().all(|t| t.ok);
    let python_packages_ok = python_packages.iter().all(|p| p.ok);
    let ok = tools_ok && python_packages_ok;
    RuntimeValidationResult {
        ok,
        tools,
        python_packages,
        local_runtime_ready: ok,
    }
}

fn check_python_module(python_bin: &str, module: &str) -> Result<String, String> {
    let output = Command::new(python_bin)
        .arg("-c")
        .arg(format!("import {module}; print('ok')"))
        .output()
        .map_err(|_| format!("Python module '{module}' is missing or Python is not executable."))?;
    if output.status.success() {
        Ok("ok".to_string())
    } else {
        Err(format!(
            "Python module '{module}' is unavailable. Ensure bundled runtime is complete."
        ))
    }
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

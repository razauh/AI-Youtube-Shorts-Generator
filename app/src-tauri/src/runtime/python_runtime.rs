use crate::runtime::process_supervisor::{
    run_supervised, ProcessError, ProcessOutput, ProcessSpec,
};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone)]
pub struct PythonInvokeRequest {
    pub python_bin: String,
    pub entry_script: String,
    pub env: Vec<(String, String)>,
    pub stdin_json: Vec<u8>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PythonBridgePaths {
    pub python_bin: String,
    pub entry_script: String,
    pub bundled_runtime_dir: Option<PathBuf>,
}

impl fmt::Debug for PythonInvokeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PythonInvokeRequest")
            .field("python_bin", &self.python_bin)
            .field("entry_script", &self.entry_script)
            .field(
                "env_keys",
                &self.env.iter().map(|(key, _)| key).collect::<Vec<_>>(),
            )
            .field("stdin_json_len", &self.stdin_json.len())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

pub fn invoke_python(req: PythonInvokeRequest) -> Result<ProcessOutput, ProcessError> {
    run_supervised(ProcessSpec {
        program: req.python_bin,
        args: vec![req.entry_script],
        env: req.env,
        stdin_bytes: req.stdin_json,
        timeout_ms: req.timeout_ms,
    })
}

pub fn resolve_python_bridge_paths() -> PythonBridgePaths {
    if cfg!(debug_assertions) {
        if let Some((python, entry)) = detect_dev_bridge_paths() {
            return PythonBridgePaths {
                python_bin: python.display().to_string(),
                entry_script: entry.display().to_string(),
                bundled_runtime_dir: None,
            };
        }
    }

    if let Some(dir) = detect_optional_runtime_pack_dir() {
        let python = if cfg!(windows) {
            dir.join("python.exe")
        } else {
            dir.join("python3")
        };
        let entry = dir.join("python_legacy").join("bridge_entry.py");
        if python.exists() && entry.exists() {
            return PythonBridgePaths {
                python_bin: python.display().to_string(),
                entry_script: entry.display().to_string(),
                bundled_runtime_dir: Some(dir),
            };
        }
    }

    if let Some(dir) = detect_bundled_runtime_dir() {
        let python = if cfg!(windows) {
            dir.join("python.exe")
        } else {
            dir.join("python3")
        };
        let entry = dir.join("python_legacy").join("bridge_entry.py");
        if python.exists() && entry.exists() {
            return PythonBridgePaths {
                python_bin: python.display().to_string(),
                entry_script: entry.display().to_string(),
                bundled_runtime_dir: Some(dir),
            };
        }
    }

    // In release/packaged builds, avoid falling back to PATH/relative bridge paths.
    // A packaged app must carry a complete bundled runtime.
    if !cfg!(debug_assertions) {
        return PythonBridgePaths {
            python_bin: "__missing_bundled_python__".to_string(),
            entry_script: "__missing_bundled_bridge_entry__".to_string(),
            bundled_runtime_dir: None,
        };
    }

    PythonBridgePaths {
        python_bin: "python3".to_string(),
        entry_script: "../../python_legacy/bridge_entry.py".to_string(),
        bundled_runtime_dir: None,
    }
}

fn detect_optional_runtime_pack_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA").ok().map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Library").join("Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".local").join("share"));
    let root = base?.join("ai-youtube-shorts-generator").join("runtime-pack").join("current");
    if root.exists() {
        Some(root)
    } else {
        None
    }
}

pub fn with_python_runtime_env(
    base_env: Vec<(String, String)>,
    bridge: &PythonBridgePaths,
) -> Vec<(String, String)> {
    let mut merged: BTreeMap<String, String> = base_env.into_iter().collect();
    if let Some(dir) = bridge.bundled_runtime_dir.as_ref() {
        let path_sep = if cfg!(windows) { ";" } else { ":" };
        let site_packages = dir.join("site-packages");
        if site_packages.exists() {
            let existing = merged.get("PYTHONPATH").cloned().unwrap_or_default();
            let value = if existing.trim().is_empty() {
                site_packages.display().to_string()
            } else {
                format!("{}{}{}", site_packages.display(), path_sep, existing)
            };
            merged.insert("PYTHONPATH".to_string(), value);
        }

        let existing_path = merged
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();
        let runtime_bin = dir.display().to_string();
        let path_value = if existing_path.trim().is_empty() {
            runtime_bin
        } else {
            format!("{runtime_bin}{path_sep}{existing_path}")
        };
        merged.insert("PATH".to_string(), path_value);
    }
    merged.into_iter().collect()
}

fn detect_bundled_runtime_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = vec![
        exe_dir.join("../lib").join("bundled-runtime"),
        exe_dir.join("../resources").join("bundled-runtime"),
        exe_dir.join("resources").join("bundled-runtime"),
        manifest_dir.join("bundled-runtime"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

fn detect_dev_bridge_paths() -> Option<(PathBuf, PathBuf)> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent()?.parent()?.to_path_buf();
    let entry = repo_root.join("python_legacy").join("bridge_entry.py");
    if !entry.exists() {
        return None;
    }
    let py_candidates = if cfg!(windows) {
        vec![
            repo_root.join(".venv").join("Scripts").join("python.exe"),
            repo_root.join(".venv").join("Scripts").join("python3.exe"),
        ]
    } else {
        vec![
            repo_root.join(".venv").join("bin").join("python3"),
            repo_root.join(".venv").join("bin").join("python"),
        ]
    };
    let python = py_candidates.into_iter().find(|p| p.exists())?;
    Some((python, entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_runtime_env_adds_site_packages_to_pythonpath() {
        let temp = std::env::temp_dir().join(format!(
            "shorts_pyenv_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let site_packages = temp.join("site-packages");
        std::fs::create_dir_all(&site_packages).expect("mkdir site-packages");

        let bridge = PythonBridgePaths {
            python_bin: "python3".to_string(),
            entry_script: "bridge_entry.py".to_string(),
            bundled_runtime_dir: Some(temp),
        };
        let env = with_python_runtime_env(vec![], &bridge);
        let map: BTreeMap<String, String> = env.into_iter().collect();
        let pythonpath = map.get("PYTHONPATH").expect("pythonpath");
        assert!(pythonpath.contains("site-packages"));
        assert!(map.get("PATH").is_some());
    }
}

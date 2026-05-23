use crate::runtime::process_supervisor::{
    run_supervised, ProcessError, ProcessOutput, ProcessSpec,
};
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

    if let Some((python, entry)) = detect_dev_bridge_paths() {
        return PythonBridgePaths {
            python_bin: python.display().to_string(),
            entry_script: entry.display().to_string(),
            bundled_runtime_dir: None,
        };
    }

    PythonBridgePaths {
        python_bin: "python3".to_string(),
        entry_script: "../../python_legacy/bridge_entry.py".to_string(),
        bundled_runtime_dir: None,
    }
}

fn detect_bundled_runtime_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidates = vec![
        exe_dir.join("../lib").join("bundled-runtime"),
        exe_dir.join("../resources").join("bundled-runtime"),
        exe_dir.join("resources").join("bundled-runtime"),
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

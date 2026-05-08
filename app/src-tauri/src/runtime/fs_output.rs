use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FsOutputErrorCode {
    ParentMissing,
    SerializeFailed,
    WriteFailed,
    RenameFailed,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsOutputError {
    pub code: FsOutputErrorCode,
    pub message: String,
    pub path: PathBuf,
}

impl Display for FsOutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:?}): {}",
            self.path.display(),
            self.code,
            self.message
        )
    }
}

impl std::error::Error for FsOutputError {}

pub fn write_result_json_atomic(
    path: &Path,
    value: &serde_json::Value,
) -> Result<(), FsOutputError> {
    let parent = path.parent().ok_or_else(|| FsOutputError {
        code: FsOutputErrorCode::ParentMissing,
        message: "output path must include parent directory".to_string(),
        path: path.to_path_buf(),
    })?;

    if !parent.exists() {
        return Err(FsOutputError {
            code: FsOutputErrorCode::ParentMissing,
            message: "output parent directory does not exist".to_string(),
            path: parent.to_path_buf(),
        });
    }

    let content = serde_json::to_vec_pretty(value).map_err(|e| FsOutputError {
        code: FsOutputErrorCode::SerializeFailed,
        message: e.to_string(),
        path: path.to_path_buf(),
    })?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(".tmp_result_{stamp}.json"));

    fs::write(&tmp, content).map_err(|e| FsOutputError {
        code: FsOutputErrorCode::WriteFailed,
        message: e.to_string(),
        path: tmp.clone(),
    })?;

    fs::rename(&tmp, path).map_err(|e| FsOutputError {
        code: FsOutputErrorCode::RenameFailed,
        message: e.to_string(),
        path: path.to_path_buf(),
    })?;

    Ok(())
}

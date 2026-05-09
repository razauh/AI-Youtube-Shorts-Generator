#[tauri::command]
pub fn pick_local_video_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Video", &["mp4", "mov", "mkv", "webm", "avi", "m4v"])
        .pick_file()
        .map(|path| path.display().to_string())
}

#[tauri::command]
pub fn pick_output_json_path() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("shorts-report.json")
        .save_file()
        .map(|path| path.display().to_string())
}

#[tauri::command]
pub fn open_in_file_manager(path: String) -> Result<(), String> {
    use std::path::PathBuf;
    use std::process::Command;

    let input = PathBuf::from(path);
    let target = if input.is_file() {
        input
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Could not resolve parent directory".to_string())?
    } else {
        input
    };

    if !target.exists() {
        return Err("Target path does not exist".to_string());
    }

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(&target);
        c
    };

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(&target);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("explorer");
        c.arg(&target);
        c
    };

    cmd.spawn()
        .map_err(|e| format!("Failed to open file manager: {e}"))?;
    Ok(())
}

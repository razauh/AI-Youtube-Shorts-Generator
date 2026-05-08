#[tauri::command]
pub fn pick_local_video_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Video", &["mp4", "mov", "mkv", "webm", "avi", "m4v"])
        .pick_file()
        .map(|path| path.display().to_string())
}


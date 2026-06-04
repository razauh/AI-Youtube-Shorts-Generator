#[tauri::command]
pub fn pick_output_json_path() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("shorts-report.json")
        .save_file()
        .map(|path| path.display().to_string())
}

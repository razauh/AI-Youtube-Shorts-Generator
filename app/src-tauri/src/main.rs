#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            shorts_tauri_app::commands::files::pick_local_video_file,
            shorts_tauri_app::commands::generate::generate_shorts,
            shorts_tauri_app::commands::generate::generate_shorts_with_events,
            shorts_tauri_app::commands::generate::generate_shorts_stream,
            shorts_tauri_app::commands::health::health_check,
            shorts_tauri_app::commands::health::validate_runtime,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let auth_state = shorts_tauri_app::auth::build_auth_state(app.handle())
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
            app.manage(auth_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            shorts_tauri_app::commands::auth::activate_license,
            shorts_tauri_app::commands::auth::validate_session,
            shorts_tauri_app::commands::auth::request_device_reset,
            shorts_tauri_app::commands::auth::get_device_reset_status,
            shorts_tauri_app::commands::auth::clear_local_session,
            shorts_tauri_app::commands::auth::get_auth_state,
            shorts_tauri_app::commands::files::pick_local_video_file,
            shorts_tauri_app::commands::files::pick_output_json_path,
            shorts_tauri_app::commands::files::open_in_file_manager,
            shorts_tauri_app::commands::generate::generate_shorts,
            shorts_tauri_app::commands::generate::generate_shorts_with_events,
            shorts_tauri_app::commands::generate::generate_shorts_stream,
            shorts_tauri_app::commands::health::health_check,
            shorts_tauri_app::commands::health::validate_runtime,
            shorts_tauri_app::commands::health::app_config_summary,
            shorts_tauri_app::commands::runtime::runtime_context,
            shorts_tauri_app::commands::runtime::runtime_machine_secret,
            shorts_tauri_app::commands::runtime::runtime_fs_read_text,
            shorts_tauri_app::commands::runtime::runtime_fs_write_text,
            shorts_tauri_app::commands::runtime::runtime_fs_append_line,
            shorts_tauri_app::commands::runtime::runtime_fs_remove,
            shorts_tauri_app::commands::runtime::runtime_fs_exists,
            shorts_tauri_app::commands::runtime::runtime_fs_list,
            shorts_tauri_app::commands::runtime::runtime_fs_rename,
            shorts_tauri_app::commands::runtime::runtime_fs_chmod_readonly,
            shorts_tauri_app::commands::runtime::runtime_fs_size,
            shorts_tauri_app::commands::runtime::secure_store_save,
            shorts_tauri_app::commands::runtime::secure_store_load,
            shorts_tauri_app::commands::runtime::secure_store_delete,
            shorts_tauri_app::commands::runtime::secure_store_exists,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

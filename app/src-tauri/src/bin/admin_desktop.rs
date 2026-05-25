#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            shorts_tauri_app::commands::admin::admin_config_load,
            shorts_tauri_app::commands::admin::admin_config_save,
            shorts_tauri_app::commands::admin::admin_config_clear,
            shorts_tauri_app::commands::admin::admin_test_connection,
            shorts_tauri_app::commands::admin::admin_list_reset_requests,
            shorts_tauri_app::commands::admin::admin_approve_reset_request,
            shorts_tauri_app::commands::admin::admin_reject_reset_request,
        ])
        .run(tauri::generate_context!("tauri.admin.conf.json"))
        .expect("error while running admin desktop application");
}

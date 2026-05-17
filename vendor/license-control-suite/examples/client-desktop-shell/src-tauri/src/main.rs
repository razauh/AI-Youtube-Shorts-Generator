fn main() {
    tauri::Builder::default()
        .invoke_handler(
            license_control_suite::desktop::tauri::auth_command_handler::<tauri::Wry>(),
        )
        .run(tauri::generate_context!())
        .expect("client desktop shell should start");
}

use license_control_suite::app_command_names;
use license_control_suite::desktop::tauri::{
    auth_command_handler, command_names, register_auth_commands,
};

#[test]
fn crate_owned_auth_handler_remains_available_for_host_shells() {
    fn host_auth_handler<R>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
    where
        R: tauri::Runtime,
    {
        auth_command_handler::<R>()
    }

    let _ = host_auth_handler::<tauri::Wry>();
    assert_eq!(command_names(), app_command_names());
}

#[test]
fn legacy_convenience_registration_helper_remains_available_if_retained() {
    let _builder_fn: fn(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> =
        register_auth_commands::<tauri::Wry>;
}

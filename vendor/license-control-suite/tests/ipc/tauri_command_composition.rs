use license_control_suite::app_command_names;
use license_control_suite::desktop::tauri::{
    activate_license, auth_command_handler, clear_local_session, command_names, get_auth_state,
    get_device_reset_status, register_auth_commands, request_device_reset, validate_session,
};

#[test]
fn host_composition_example_compiles_without_forcing_crate_owned_handler() {
    fn host_owned_handler<R>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
    where
        R: tauri::Runtime,
    {
        tauri::generate_handler![
            activate_license,
            validate_session,
            request_device_reset,
            get_device_reset_status,
            clear_local_session,
            get_auth_state
        ]
    }

    let _ = host_owned_handler::<tauri::Wry>();
    let _ = auth_command_handler::<tauri::Wry>();
    assert_eq!(command_names(), app_command_names());
}

#[test]
fn legacy_convenience_registration_helper_remains_available_if_retained() {
    let _builder_fn: fn(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> =
        register_auth_commands::<tauri::Wry>;
}

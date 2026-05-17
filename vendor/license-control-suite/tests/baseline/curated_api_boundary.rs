use license_control_suite::app_command_names;
use license_control_suite::modules;

#[test]
fn curated_core_imports_are_available_from_crate_root() {
    use license_control_suite::core::{
        AuthService, Clock, DeviceIdentityProvider, LocalStateStore, SecretStore, WorkerClient,
    };

    let _ = core::mem::size_of::<AuthService>();
    let _ = core::mem::size_of::<Option<&'static dyn WorkerClient>>();
    let _ = core::mem::size_of::<Option<&'static dyn SecretStore>>();
    let _ = core::mem::size_of::<Option<&'static dyn LocalStateStore>>();
    let _ = core::mem::size_of::<Option<&'static dyn DeviceIdentityProvider>>();
    let _ = core::mem::size_of::<Option<&'static dyn Clock>>();
}

#[test]
fn curated_desktop_tauri_imports_are_available() {
    use license_control_suite::desktop::tauri::{
        command_handler, command_names, register_auth_commands, ActivationView, AuthAppState,
        AuthCommandError, AuthStateView, DeviceResetInput, DeviceResetView, SessionView,
    };

    let _ = core::mem::size_of::<ActivationView>();
    let _ = core::mem::size_of::<AuthAppState>();
    let _ = core::mem::size_of::<AuthCommandError>();
    let _ = core::mem::size_of::<AuthStateView>();
    let _ = core::mem::size_of::<DeviceResetInput>();
    let _ = core::mem::size_of::<DeviceResetView>();
    let _ = core::mem::size_of::<SessionView>();
    assert_eq!(command_names(), app_command_names());
    let _ = command_handler::<tauri::Wry>();
    let _builder_fn: fn(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> =
        register_auth_commands::<tauri::Wry>;
}

#[test]
fn curated_desktop_persistence_imports_are_available() {
    use license_control_suite::desktop::persistence::{
        AppDataStateStore, InMemorySecretStore, KeychainSecretStore,
    };

    let _ = core::mem::size_of::<AppDataStateStore>();
    let _ = core::mem::size_of::<InMemorySecretStore>();
    let _ = core::mem::size_of::<KeychainSecretStore>();
}

#[test]
fn legacy_compatibility_imports_remain_available() {
    let _ = core::mem::size_of_val(&modules::shared_contracts::NAMESPACE);
    let _ = core::mem::size_of_val(&modules::admin_dashboard::NAMESPACE);
    let _ = core::mem::size_of_val(&modules::auth_core::NAMESPACE);
    let _ = core::mem::size_of_val(&modules::user_reg::NAMESPACE);
}

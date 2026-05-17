pub mod modules;

#[cfg(feature = "core")]
pub mod core;
#[cfg(any(feature = "desktop-tauri", feature = "desktop-persistence"))]
pub mod desktop;
#[cfg(feature = "reference-worker")]
pub mod reference_worker;

#[cfg(feature = "desktop-tauri")]
pub const APP_COMMAND_NAMES: [&str; 6] = [
    "activate_license",
    "validate_session",
    "request_device_reset",
    "get_device_reset_status",
    "clear_local_session",
    "get_auth_state",
];

#[cfg(feature = "desktop-tauri")]
pub fn app_command_names() -> &'static [&'static str] {
    &APP_COMMAND_NAMES
}

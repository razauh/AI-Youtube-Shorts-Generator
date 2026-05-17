//! Curated Tauri adapter facade for desktop consumers.

pub use crate::modules::user_reg::auth_licensing_tauri::{
    activate_license, activate_license_with_service, auth_command_handler,
    clear_local_session, clear_local_session_with_service, command_handler, command_names,
    get_auth_state, get_auth_state_with_service, get_device_reset_status,
    get_device_reset_status_with_service, register_auth_commands, request_device_reset,
    request_device_reset_with_service, validate_session, validate_session_with_service,
    ActivationView, AuthAppState, AuthCommandError, AuthStateView, DeviceResetInput,
    DeviceResetView, SessionView,
};

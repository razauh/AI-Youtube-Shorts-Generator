use license_control_suite::desktop::tauri::{
    self as auth_tauri, ActivationView, AuthAppState, AuthCommandError, AuthStateView,
    DeviceResetInput, DeviceResetView, SessionView,
};
use tauri::State;

#[tauri::command]
pub async fn activate_license(
    license_key: String,
    state: State<'_, AuthAppState>,
) -> Result<ActivationView, AuthCommandError> {
    auth_tauri::activate_license_with_service(license_key, &state.service).await
}

#[tauri::command]
pub async fn validate_session(
    state: State<'_, AuthAppState>,
) -> Result<SessionView, AuthCommandError> {
    auth_tauri::validate_session_with_service(&state.service).await
}

#[tauri::command]
pub async fn request_device_reset(
    input: DeviceResetInput,
    state: State<'_, AuthAppState>,
) -> Result<DeviceResetView, AuthCommandError> {
    auth_tauri::request_device_reset_with_service(input, &state.service).await
}

#[tauri::command]
pub async fn get_device_reset_status(
    request_id: String,
    state: State<'_, AuthAppState>,
) -> Result<DeviceResetView, AuthCommandError> {
    auth_tauri::get_device_reset_status_with_service(request_id, &state.service).await
}

#[tauri::command]
pub async fn clear_local_session(state: State<'_, AuthAppState>) -> Result<(), AuthCommandError> {
    auth_tauri::clear_local_session_with_service(&state.service).await
}

#[tauri::command]
pub async fn get_auth_state(
    state: State<'_, AuthAppState>,
) -> Result<AuthStateView, AuthCommandError> {
    auth_tauri::get_auth_state_with_service(&state.service).await
}

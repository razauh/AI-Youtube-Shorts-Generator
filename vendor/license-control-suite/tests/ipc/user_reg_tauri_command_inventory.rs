use license_control_suite::modules::user_reg::auth_licensing_tauri::command_names;

#[test]
fn command_names_return_expected_six_commands() {
    assert_eq!(
        command_names(),
        &[
            "activate_license",
            "validate_session",
            "request_device_reset",
            "get_device_reset_status",
            "clear_local_session",
            "get_auth_state",
        ]
    );
}

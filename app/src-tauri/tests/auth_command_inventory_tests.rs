#[test]
fn license_control_suite_command_inventory_is_registered_by_name() {
    let names = license_control_suite::desktop::tauri::command_names();

    assert_eq!(
        names,
        [
            "activate_license",
            "validate_session",
            "deactivate_current_device",
            "request_device_reset",
            "get_device_reset_status",
            "clear_local_session",
            "get_auth_state",
        ]
    );
}

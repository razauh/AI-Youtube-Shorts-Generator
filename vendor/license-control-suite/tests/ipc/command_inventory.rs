use license_control_suite::{app_command_names, APP_COMMAND_NAMES};
use std::collections::BTreeSet;

const EXPECTED_COMMAND_NAMES: [&str; 6] = [
    "activate_license",
    "validate_session",
    "request_device_reset",
    "get_device_reset_status",
    "clear_local_session",
    "get_auth_state",
];

#[test]
fn command_inventory_has_exactly_six_known_commands() {
    let names = app_command_names();
    assert_eq!(names.len(), 6);
    assert_eq!(names, &EXPECTED_COMMAND_NAMES);
}

#[test]
fn command_inventory_has_no_duplicates() {
    let names = app_command_names();
    let unique: BTreeSet<_> = names.iter().copied().collect();
    assert_eq!(unique.len(), names.len());
}

#[test]
fn baseline_command_inventory_still_matches_root_constant() {
    let names = app_command_names();
    assert_eq!(names, &EXPECTED_COMMAND_NAMES);
    assert_eq!(APP_COMMAND_NAMES, EXPECTED_COMMAND_NAMES);
}

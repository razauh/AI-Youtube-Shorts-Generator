use license_control_suite::app_command_names;
use std::fs;
use std::path::Path;

#[test]
fn six_commands_remain_reserved_for_tauri_capability_mapping() {
    assert_eq!(app_command_names().len(), 6);
}

#[test]
fn client_example_shell_exists_with_tauri_config_and_capabilities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shell = root.join("examples/client-desktop-shell/src-tauri");
    assert!(shell.exists());
    assert!(shell.join("Cargo.toml").exists());
    assert!(shell.join("tauri.conf.json").exists());
    assert!(shell.join("capabilities").exists());
}

#[test]
fn admin_example_shell_exists_with_tauri_manifest_and_capabilities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shell = root.join("examples/admin-desktop-shell/src-tauri");
    assert!(shell.exists());
    assert!(shell.join("Cargo.toml").exists());
    assert!(shell.join("tauri.conf.json").exists());
    assert!(shell.join("capabilities").exists());
}

#[test]
fn example_shells_ship_icon_assets_for_tauri_context_generation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        root.join("examples/client-desktop-shell/src-tauri/icons/icon.png")
            .exists()
    );
    assert!(
        root.join("examples/admin-desktop-shell/src-tauri/icons/icon.png")
            .exists()
    );
}

#[test]
fn client_shell_capabilities_map_the_six_user_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let caps = fs::read_to_string(
        root.join("examples/client-desktop-shell/src-tauri/capabilities/default.json"),
    )
    .expect("client shell capability file should exist");

    for command in app_command_names() {
        assert!(caps.contains(command));
    }
}

#[test]
fn tauri_smoke_and_capability_scripts_target_example_shells() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let smoke = fs::read_to_string(root.join("scripts/tauri_smoke.sh"))
        .expect("tauri smoke script should exist");
    let caps = fs::read_to_string(root.join("scripts/check_tauri_capabilities.sh"))
        .expect("tauri capability script should exist");

    assert!(smoke.contains("examples/client-desktop-shell/src-tauri"));
    assert!(smoke.contains("examples/admin-desktop-shell/src-tauri"));
    assert!(caps.contains("examples/client-desktop-shell/src-tauri"));
    assert!(caps.contains("command -v rg"));
    assert!(caps.contains("grep -q"));
}

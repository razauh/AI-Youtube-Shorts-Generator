use serde_json::Value;
use std::fs;

#[test]
fn updater_plugin_dependency_is_declared() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml should be readable");

    assert!(cargo_toml.contains("tauri-plugin-updater"));
}

#[test]
fn updater_plugin_is_registered_in_tauri_builder() {
    let main_rs = fs::read_to_string("src/main.rs").expect("main.rs should be readable");

    assert!(main_rs.contains("tauri_plugin_updater::Builder::new().build()"));
}

#[test]
fn updater_capability_uses_off_the_shelf_permission() {
    let capability = fs::read_to_string("capabilities/default.json")
        .expect("default capability should be readable");
    let parsed: Value = serde_json::from_str(&capability).expect("capability should be valid json");
    let permissions = parsed
        .get("permissions")
        .and_then(Value::as_array)
        .expect("permissions should be an array");

    assert!(permissions
        .iter()
        .any(|p| p.as_str() == Some("updater:default")));
}

#[test]
fn updater_config_creates_tauri_updater_artifacts() {
    let config = fs::read_to_string("tauri.conf.json").expect("tauri config should be readable");
    let parsed: Value = serde_json::from_str(&config).expect("tauri config should be valid json");

    assert_eq!(
        parsed
            .pointer("/bundle/createUpdaterArtifacts")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert!(parsed.pointer("/plugins/updater/endpoints").is_some());
    assert!(parsed.pointer("/plugins/updater/pubkey").is_some());
}

#[test]
fn custom_updater_engine_is_not_present() {
    assert!(!std::path::Path::new("src/core/updater").exists());
    assert!(!std::path::Path::new("src/commands/updater.rs").exists());
}

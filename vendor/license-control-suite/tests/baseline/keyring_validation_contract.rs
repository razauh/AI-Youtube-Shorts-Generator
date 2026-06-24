use std::fs;
use std::path::Path;

#[test]
fn default_suite_remains_fake_store_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let persistence_rs = fs::read_to_string(
        root.join("src/modules/user_reg/auth_licensing_tauri/persistence.rs"),
    )
    .expect("persistence.rs should exist");

    assert!(persistence_rs.contains("LICENSE_CONTROL_SUITE_RUN_REAL_KEYRING_SMOKE"));
    assert!(persistence_rs.contains("#[ignore = \"opt-in real keyring smoke; set LICENSE_CONTROL_SUITE_RUN_REAL_KEYRING_SMOKE=1 and run explicitly\"]"));
}

#[test]
fn platform_notes_exist_for_linux_macos_windows_keyring_validation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let note = fs::read_to_string(root.join("docs/migration/keyring_validation.md"))
        .expect("keyring validation note should exist");

    assert!(note.contains("Linux"));
    assert!(note.contains("macOS"));
    assert!(note.contains("Windows"));
    assert!(note.contains("LICENSE_CONTROL_SUITE_RUN_REAL_KEYRING_SMOKE=1"));
    assert!(note.contains("disposable namespace"));
}

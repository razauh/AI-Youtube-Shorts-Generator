use std::fs;
use std::path::Path;

#[test]
fn packaging_smoke_artifacts_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("docs/migration/packaging_smoke.md").exists());
    assert!(root.join("scripts/tauri_smoke.sh").exists());
}

#[test]
fn tauri_smoke_script_has_blocked_signal_for_missing_shell() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content = fs::read_to_string(root.join("scripts/tauri_smoke.sh"))
        .expect("tauri_smoke.sh should be readable");
    assert!(content.contains("BLOCKED"));
    assert!(content.contains("examples/client-desktop-shell/src-tauri"));
    assert!(content.contains("examples/admin-desktop-shell/src-tauri"));
}

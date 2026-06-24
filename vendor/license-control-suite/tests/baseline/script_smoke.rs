use std::path::Path;

#[test]
fn required_check_scripts_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("scripts/check_all.sh").exists());
    assert!(root.join("scripts/baseline.sh").exists());
    assert!(root.join("scripts/tauri_smoke.sh").exists());
}

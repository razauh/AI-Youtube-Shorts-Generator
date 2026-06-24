use std::fs;
use std::path::Path;

#[test]
fn main_shell_uses_curated_auth_command_handler_helper() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let main_rs = fs::read_to_string(root.join("src/main.rs")).expect("src/main.rs should exist");

    assert!(main_rs.contains("auth_command_handler::<R>()"));
    assert!(!main_rs.contains("tauri::generate_handler!["));
}

use license_control_suite::{app_command_names, modules, APP_COMMAND_NAMES};
use std::fs;
use std::path::Path;

#[test]
fn module_namespaces_are_exposed() {
    let _ = core::mem::size_of_val(&modules::shared_contracts::NAMESPACE);
    let _ = core::mem::size_of_val(&modules::admin_dashboard::NAMESPACE);
    let _ = core::mem::size_of_val(&modules::auth_core::NAMESPACE);
    let _ = core::mem::size_of_val(&modules::user_reg::NAMESPACE);
}

#[test]
fn curated_root_exports_are_available_after_api_curation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = fs::read_to_string(root.join("src/lib.rs")).expect("src/lib.rs should exist");

    let _ = app_command_names();
    let _ = APP_COMMAND_NAMES;
    let _ = core::mem::size_of_val(&modules::user_reg::NAMESPACE);

    assert!(lib_rs.contains("pub mod modules;"));
    assert!(lib_rs.contains("pub const APP_COMMAND_NAMES"));
    assert!(lib_rs.contains("pub fn app_command_names()"));
    assert!(lib_rs.contains("pub mod core;"));
    assert!(lib_rs.contains("pub mod desktop;"));
}

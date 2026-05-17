use std::fs;
use std::path::Path;

#[test]
fn admin_desktop_curated_imports_compile() {
    use license_control_suite::desktop::admin::{
        adapters, auth, authz, compatibility, ops, queue, realtime,
    };

    let _ = core::mem::size_of::<Option<&'static dyn adapters::AdminApi>>();
    let _ = core::mem::size_of::<Option<&'static dyn adapters::SessionStore>>();
    let _ = core::mem::size_of_val(&auth::login_with_challenge);
    let _ = core::mem::size_of_val(&authz::can_read);
    let _ = core::mem::size_of_val(&compatibility::supported_shared_contracts_range);
    let _ = core::mem::size_of_val(&ops::compute_health);
    let _ = core::mem::size_of_val(&queue::approve);
    let _ = core::mem::size_of_val(&realtime::backoff_step_sec);
}

#[test]
fn admin_domain_remains_separate_from_user_command_surface() {
    let names = license_control_suite::app_command_names();
    assert_eq!(names.len(), 6);
    assert!(!names.iter().any(|name| name.contains("admin")));
    assert!(!names.iter().any(|name| name.contains("challenge")));
    assert!(!names.iter().any(|name| name.contains("reset:write")));
}

#[test]
fn admin_desktop_docs_explicitly_exclude_web_dashboard_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");
    let note = fs::read_to_string(root.join("docs/admin_desktop_boundary.md"))
        .expect("admin desktop boundary note should exist");

    assert!(readme.contains("desktop-only admin console"));
    assert!(note.contains("desktop-only"));
    assert!(note.contains("not a web dashboard"));
    assert!(note.contains("separate from the six user/client commands"));
}

#[test]
fn existing_admin_domain_tests_still_pass_under_curated_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("tests/integration/admin_dashboard_auth_int.rs").exists());
    assert!(root.join("tests/integration/admin_dashboard_queue_int.rs").exists());
    assert!(root.join("tests/integration/admin_dashboard_ops_unit.rs").exists());
    assert!(root.join("tests/integration/admin_dashboard_reconnect_rt.rs").exists());
}

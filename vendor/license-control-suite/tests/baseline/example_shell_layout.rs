use std::fs;
use std::path::Path;

#[test]
fn admin_example_shell_exists_and_is_separate_from_client_shell() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let client = root.join("examples/client-desktop-shell");
    let admin = root.join("examples/admin-desktop-shell");

    assert!(client.exists());
    assert!(admin.exists());
    assert_ne!(client, admin);

    let admin_readme =
        fs::read_to_string(admin.join("README.md")).expect("admin shell README should exist");
    assert!(admin_readme.contains("desktop admin console"));
    assert!(admin_readme.contains("not a web dashboard"));
}

#[test]
fn no_web_or_mobile_shell_target_is_introduced() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!root.join("examples/web-dashboard-shell").exists());
    assert!(!root.join("examples/mobile-shell").exists());
    assert!(!root.join("app/src").exists());
}

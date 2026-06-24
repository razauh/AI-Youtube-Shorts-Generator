use std::fs;
use std::path::Path;

#[test]
fn downstream_consumer_harnesses_live_under_fixtures_not_crate_components() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("fixtures/downstream_consumers");

    assert!(fixtures.exists());
    assert!(fixtures.join("core_only_consumer").exists());
    assert!(fixtures.join("tauri_host_consumer").exists());
    assert!(fixtures.join("git_dependency_consumer").exists());
    assert!(!root.join("src/downstream_consumers").exists());
    assert!(!root.join("examples/downstream_consumers").exists());
}

#[test]
fn core_only_consumer_uses_curated_core_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(
        root.join("fixtures/downstream_consumers/core_only_consumer/Cargo.toml"),
    )
    .expect("core-only consumer manifest should exist");
    let main_rs = fs::read_to_string(
        root.join("fixtures/downstream_consumers/core_only_consumer/src/main.rs"),
    )
    .expect("core-only consumer main should exist");

    assert!(manifest.contains("license-control-suite"));
    assert!(manifest.contains("default-features = false"));
    assert!(manifest.contains("features = [\"core\"]"));
    assert!(main_rs.contains("license_control_suite::core"));
}

#[test]
fn tauri_host_consumer_uses_curated_desktop_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(
        root.join("fixtures/downstream_consumers/tauri_host_consumer/Cargo.toml"),
    )
    .expect("tauri host consumer manifest should exist");
    let main_rs = fs::read_to_string(
        root.join("fixtures/downstream_consumers/tauri_host_consumer/src/main.rs"),
    )
    .expect("tauri host consumer main should exist");

    assert!(manifest.contains("features = [\"core\", \"desktop-tauri\", \"desktop-persistence\"]"));
    assert!(main_rs.contains("license_control_suite::desktop::tauri"));
    assert!(main_rs.contains("auth_command_handler::<R>()"));
}

#[test]
fn downstream_consumer_verification_script_covers_packaged_validation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("scripts/verify_downstream_consumers.sh"))
        .expect("downstream consumer verification script should exist");
    let readme = fs::read_to_string(root.join("fixtures/downstream_consumers/README.md"))
        .expect("downstream consumer README should exist");

    assert!(script.contains("cargo package"));
    assert!(script.contains("mktemp -d"));
    assert!(script.contains("git clone --bare"));
    assert!(script.contains("git rev-parse HEAD"));
    assert!(script.contains("fixtures/downstream_consumers/core_only_consumer"));
    assert!(script.contains("fixtures/downstream_consumers/tauri_host_consumer"));
    assert!(readme.contains("packaged-consumer"));
}

#[test]
fn git_dependency_consumer_template_uses_git_and_rev_placeholders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(
        root.join("fixtures/downstream_consumers/git_dependency_consumer/Cargo.toml.template"),
    )
    .expect("git dependency consumer manifest template should exist");
    let main_rs = fs::read_to_string(
        root.join("fixtures/downstream_consumers/git_dependency_consumer/src/main.rs"),
    )
    .expect("git dependency consumer main should exist");
    let readme = fs::read_to_string(root.join("fixtures/downstream_consumers/README.md"))
        .expect("downstream consumer README should exist");

    assert!(manifest.contains("git = \"__REPO_URL__\""));
    assert!(manifest.contains("rev = \"__REPO_REV__\""));
    assert!(main_rs.contains("license_control_suite::core"));
    assert!(readme.contains("git-dependency consumer"));
}

use std::fs;
use std::path::Path;

#[test]
fn release_readiness_checklist_exists_and_covers_external_consumption() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checklist = fs::read_to_string(root.join("docs/release_readiness_checklist.md"))
        .expect("release readiness checklist should exist");

    for required in [
        "downstream consumer",
        "canary app",
        "cargo publish --dry-run",
        "desktop-only",
        "keyring",
        "Tauri",
    ] {
        assert!(
            checklist.contains(required),
            "release readiness checklist should mention {required}"
        );
    }
}

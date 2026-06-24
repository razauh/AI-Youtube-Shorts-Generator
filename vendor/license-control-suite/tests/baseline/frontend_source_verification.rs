use std::fs;
use std::path::Path;

#[test]
fn frontend_verification_artifacts_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("docs/migration/frontend_asset_inventory.md").exists());
    assert!(root.join("frontend/README.md").exists());
}

#[test]
fn frontend_readme_documents_placeholder_and_graphify_exclusion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content = fs::read_to_string(root.join("frontend/README.md"))
        .expect("frontend README should be readable");
    assert!(content.contains("placeholder"));
    assert!(content.contains("Do not copy Graphify graph.html"));
}

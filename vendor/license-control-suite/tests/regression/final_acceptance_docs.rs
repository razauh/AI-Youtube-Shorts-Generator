use std::path::Path;

#[test]
fn required_final_handoff_docs_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("docs/migration/final_regression_report.md").exists());
    assert!(root.join("docs/migration/final_acceptance_checklist.md").exists());
    assert!(root.join("docs/migration/handoff_summary.md").exists());
    assert!(Path::new("/home/pc/Downloads/inf/plan/docs/unified_merge_unresolved_issues.md").exists());
}

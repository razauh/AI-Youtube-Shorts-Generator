use std::fs;
use std::path::Path;

#[test]
fn github_actions_ci_workflow_exists_and_covers_release_gates() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("GitHub Actions CI workflow should exist");

    for required in [
        "verify",
        "downstream-consumers",
        "publish-dry-run",
        "tauri-debug-build",
        "scripts/run_ci_sequence_logged.sh verify",
        "scripts/run_ci_sequence_logged.sh downstream-consumers",
        "scripts/run_ci_sequence_logged.sh publish-dry-run",
        "scripts/run_ci_sequence_logged.sh tauri-debug-build",
    ] {
        assert!(
            workflow.contains(required),
            "CI workflow should mention {required}"
        );
    }
}

#[test]
fn ci_sequence_doc_exists_and_references_workflow_and_release_checklist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let doc = fs::read_to_string(root.join("docs/ci_job_sequence.md"))
        .expect("CI job sequence doc should exist");

    assert!(doc.contains(".github/workflows/ci.yml"));
    assert!(doc.contains("docs/release_readiness_checklist.md"));
    assert!(doc.contains("downstream consumers"));
    assert!(doc.contains("publish --dry-run"));
}

#[test]
fn unified_logged_ci_runner_exists_and_writes_log_root() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = fs::read_to_string(root.join("scripts/run_ci_sequence_logged.sh"))
        .expect("unified CI runner script should exist");

    for required in [
        "MODE=",
        "RUN_DIR=",
        "SUMMARY=",
        "logs/ci_",
        "check_all.sh",
        "verify_downstream_consumers.sh",
        "cargo publish --dry-run --allow-dirty",
        "cargo tauri build --debug",
    ] {
        assert!(
            script.contains(required),
            "unified CI runner should mention {required}"
        );
    }
}

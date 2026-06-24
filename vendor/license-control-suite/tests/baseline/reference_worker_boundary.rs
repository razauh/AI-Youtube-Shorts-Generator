use std::fs;
use std::path::Path;

#[test]
fn reference_worker_curated_export_path_exists_if_adopted() {
    use license_control_suite::reference_worker::{InMemoryWorkerStore, WorkerApp};

    let _ = core::mem::size_of::<InMemoryWorkerStore>();
    let _ = core::mem::size_of::<WorkerApp>();
}

#[test]
fn docs_do_not_claim_cloudflare_or_gumroad_support() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");
    let note = fs::read_to_string(root.join("docs/reference_worker_boundary.md"))
        .expect("reference worker boundary note should exist");

    assert!(readme.contains("reference backend"));
    assert!(note.contains("Cloudflare runtime adapter"));
    assert!(note.contains("Gumroad provider adapter"));
    assert!(note.contains("payment verification"));
    assert!(note.contains("webhook ingestion"));
    assert!(note.contains("durable backend storage"));
    assert!(note.contains("deferred"));
}

use license_control_suite::modules::user_reg::auth_licensing_core::{
    ActivationRequest, DeviceFingerprint, DevicePublicKey, DeviceResetRequest, LicenseKey,
    PurchaseEmail, ResetRequestId,
};
use license_control_suite::modules::user_reg::licensing_worker::{InMemoryWorkerStore, WorkerApp};

fn license() -> LicenseKey {
    LicenseKey::new("LICENSE-1234").unwrap()
}

fn activation_request(public_key: &str, ts: i64) -> ActivationRequest {
    ActivationRequest {
        license_key: license(),
        device_public_key: DevicePublicKey::new(public_key).unwrap(),
        fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
        app_version: "1.0.0".into(),
        timestamp_ms: ts,
    }
}

fn reset_request(ts: i64) -> DeviceResetRequest {
    DeviceResetRequest {
        license_key: Some(license()),
        masked_license_key: None,
        purchaser_email: PurchaseEmail::new("buyer@example.com").unwrap(),
        device_public_key: DevicePublicKey::new("public").unwrap(),
        fingerprint: DeviceFingerprint::new("linux", "linux", "x86_64", None).unwrap(),
        app_version: "1.0.0".into(),
        timestamp_ms: ts,
        receipt_reference: Some("receipt".into()),
    }
}

fn app() -> WorkerApp {
    let store = InMemoryWorkerStore::default();
    store.insert_license(&license(), "buyer@example.com");
    WorkerApp::new(store, "admin-secret")
}

#[tokio::test]
async fn concurrent_activation_attempts_against_store_do_not_panic() {
    let app = app();
    let a = async { app.activate(activation_request("public", 10)) };
    let b = async { app.activate(activation_request("public", 11)) };
    let c = async { app.activate(activation_request("public", 12)) };

    let (r1, r2, r3) = tokio::join!(a, b, c);
    let ok_count = [r1.is_ok(), r2.is_ok(), r3.is_ok()]
        .into_iter()
        .filter(|v| *v)
        .count();
    assert!(ok_count >= 1);
}

#[tokio::test]
async fn concurrent_reset_status_reads_are_consistent() {
    let app = app();
    let pending = app.request_device_reset(reset_request(20)).unwrap();
    let id: ResetRequestId = pending.request_id().clone();

    let a = async { app.get_reset_status(id.clone()) };
    let b = async { app.get_reset_status(id.clone()) };
    let c = async { app.get_reset_status(id) };
    let (s1, s2, s3) = tokio::join!(a, b, c);

    assert_eq!(s1, s2);
    assert_eq!(s2, s3);
}

#[tokio::test]
async fn worker_store_allows_concurrent_audit_reads_and_writes() {
    let app = app();
    let write = async {
        let _ = app.activate(activation_request("public", 10));
        let _ = app.request_device_reset(reset_request(20));
    };
    let read1 = async { app.store.audit_events() };
    let read2 = async { app.store.audit_events() };
    let (_, a1, a2) = tokio::join!(write, read1, read2);

    assert!(a1.len() <= app.store.audit_events().len());
    assert!(a2.len() <= app.store.audit_events().len());
}

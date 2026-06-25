use serde_json::json;
use shorts_tauri_app::core::api_mode::muapi::{MuApiClient, MuApiError};
use shorts_tauri_app::core::config::Config;
use shorts_tauri_app::core::observability::events::ProgressEmitter;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct MockResponse {
    status: u16,
    body: &'static str,
}

fn start_server(responses: Vec<MockResponse>) -> (SocketAddr, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    listener
        .set_nonblocking(true)
        .expect("set nonblocking listener");
    let addr = listener.local_addr().expect("local addr");
    let handle = thread::spawn(move || {
        let started = Instant::now();
        let max_runtime = Duration::from_secs(2);
        let mut sent = 0usize;
        while sent < responses.len() && started.elapsed() < max_runtime {
            let accepted = listener.accept();
            let (mut stream, _) = match accepted {
                Ok(ok) => ok,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2));
                    continue;
                }
                Err(err) => panic!("accept failed: {err}"),
            };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = &responses[sent];
            let status_line = match response.status {
                200 => "HTTP/1.1 200 OK",
                400 => "HTTP/1.1 400 Bad Request",
                500 => "HTTP/1.1 500 Internal Server Error",
                _ => "HTTP/1.1 200 OK",
            };
            let body = response.body;
            let data = format!(
                "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(data.as_bytes()).expect("write response");
            sent += 1;
        }
    });
    (addr, handle)
}

fn cfg(base: String) -> Config {
    Config {
        muapi_api_key: "k".to_string(),
        muapi_base_url: base,
        muapi_poll_interval_seconds: 0.01,
        muapi_poll_timeout_seconds: 0.05,
        openai_api_key: String::new(),
        openai_model: "gpt-4o-mini".to_string(),
        license_worker_base_url: "http://127.0.0.1:8787".to_string(),
        license_storage_namespace: "desktop-client-test".to_string(),
        license_keychain_service: "shorts-test".to_string(),
        license_backend_mode: shorts_tauri_app::core::config::LicenseBackendMode::Devolens,
        license_worker_timeout_ms: 10_000,
        license_worker_retry_attempts: 2,
        license_worker_retry_backoff_ms: 150,
        license_worker_circuit_breaker_failure_threshold: 3,
        license_worker_circuit_breaker_cooldown_ms: 30_000,
        devolens_base_url: shorts_tauri_app::core::config::DEFAULT_DEVOLENS_BASE_URL.to_string(),
        devolens_access_token: "client-token".to_string(),
        devolens_product_id: "1234".to_string(),
        devolens_offline_grace_period_ms: 86400000,
    }
}

#[derive(Default)]
struct CaptureEmitter {
    statuses: Mutex<Vec<String>>,
}

impl CaptureEmitter {
    fn snapshot(&self) -> Vec<String> {
        self.statuses.lock().expect("lock statuses").clone()
    }
}

impl ProgressEmitter for CaptureEmitter {
    fn emit_status_change(&self, label: &str, status: &str) {
        self.statuses
            .lock()
            .expect("lock statuses")
            .push(format!("{label}:{status}"));
    }
}

#[tokio::test]
async fn submit_success_returns_request_id() {
    let body = include_str!("../../../tests/fixtures/muapi/submit_success.json");
    let (addr, handle) = start_server(vec![MockResponse { status: 200, body }]);
    let client = MuApiClient::new(cfg(format!("http://{addr}/api/v1")));

    let id = client
        .submit("youtube-download", &json!({"youtube_url": "u"}), 1)
        .await
        .expect("submit ok");
    assert_eq!(id, "req_123");
    handle.join().expect("join server");
}

#[tokio::test]
async fn submit_missing_request_id_returns_error() {
    let body = include_str!("../../../tests/fixtures/muapi/submit_missing_request_id.json");
    let (addr, handle) = start_server(vec![MockResponse { status: 200, body }]);
    let client = MuApiClient::new(cfg(format!("http://{addr}/api/v1")));

    let err = client
        .submit("youtube-download", &json!({"youtube_url": "u"}), 1)
        .await
        .expect_err("missing id must fail");
    match err {
        MuApiError::Api { stage, message } => {
            assert_eq!(stage, "submit");
            assert!(message.contains("response had no request_id"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    handle.join().expect("join server");
}

#[tokio::test]
async fn poll_terminal_success_returns_payload() {
    let s1 = include_str!("../../../tests/fixtures/muapi/poll_processing.json");
    let s2 = include_str!("../../../tests/fixtures/muapi/poll_completed.json");
    let (addr, handle) = start_server(vec![
        MockResponse {
            status: 200,
            body: s1,
        },
        MockResponse {
            status: 200,
            body: s2,
        },
    ]);
    let client = MuApiClient::new(cfg(format!("http://{addr}/api/v1")));

    let out = client
        .poll("req_123", 0.01, 0.2, Some("download"))
        .await
        .expect("poll success");
    assert_eq!(out["status"], "completed");
    assert_eq!(out["result"]["video_url"], "https://cdn/video.mp4");
    handle.join().expect("join server");
}

#[tokio::test]
async fn poll_emits_status_change_events_in_order() {
    let s1 = include_str!("../../../tests/fixtures/muapi/poll_processing.json");
    let s2 = include_str!("../../../tests/fixtures/muapi/poll_completed.json");
    let (addr, handle) = start_server(vec![
        MockResponse {
            status: 200,
            body: s1,
        },
        MockResponse {
            status: 200,
            body: s2,
        },
    ]);

    let emitter = Arc::new(CaptureEmitter::default());
    let client = MuApiClient::with_emitter(
        cfg(format!("http://{addr}/api/v1")),
        emitter.clone() as Arc<dyn ProgressEmitter>,
    );

    let out = client
        .poll("req_123", 0.01, 0.2, Some("download"))
        .await
        .expect("poll success");
    assert_eq!(out["status"], "completed");

    let statuses = emitter.snapshot();
    assert!(
        statuses.len() >= 2,
        "expected at least 2 status emissions, got {statuses:?}"
    );
    assert_eq!(statuses[0], "download:processing");
    assert_eq!(statuses[1], "download:completed");

    handle.join().expect("join server");
}

#[tokio::test]
async fn poll_timeout_returns_error() {
    let body = include_str!("../../../tests/fixtures/muapi/poll_processing.json");
    let (addr, handle) = start_server(vec![
        MockResponse { status: 200, body },
        MockResponse { status: 200, body },
        MockResponse { status: 200, body },
        MockResponse { status: 200, body },
        MockResponse { status: 200, body },
    ]);
    let client = MuApiClient::new(cfg(format!("http://{addr}/api/v1")));

    let err = client
        .poll("req_123", 0.01, 0.02, Some("download"))
        .await
        .expect_err("must timeout");
    match err {
        MuApiError::Api { stage, message } => {
            assert_eq!(stage, "poll");
            assert!(message.contains("timed out after 0.02s"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    handle.join().expect("join server");
}

#[tokio::test]
async fn poll_failure_status_returns_error() {
    let body = include_str!("../../../tests/fixtures/muapi/poll_failed.json");
    let (addr, handle) = start_server(vec![MockResponse { status: 200, body }]);
    let client = MuApiClient::new(cfg(format!("http://{addr}/api/v1")));

    let err = client
        .poll("req_123", 0.01, 0.1, Some("download"))
        .await
        .expect_err("failed status must fail");
    match err {
        MuApiError::Api { stage, message } => {
            assert_eq!(stage, "poll");
            assert!(message.contains("download failed:"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    handle.join().expect("join server");
}

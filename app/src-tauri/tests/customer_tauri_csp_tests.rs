use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

fn customer_tauri_config() -> Value {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let config = fs::read_to_string(config_path).expect("customer tauri config should be readable");

    serde_json::from_str(&config).expect("customer tauri config should be valid json")
}

fn csp_sources(csp: &str) -> BTreeSet<&str> {
    csp.split(';')
        .flat_map(|directive| directive.split_whitespace().skip(1))
        .collect()
}

#[test]
fn customer_tauri_csp_is_restrictive() {
    let parsed = customer_tauri_config();
    let csp = parsed
        .pointer("/app/security/csp")
        .and_then(Value::as_str)
        .expect("customer CSP should be a non-null string");

    assert!(!csp.trim().is_empty(), "customer CSP should not be empty");

    let directives = csp
        .split(';')
        .filter_map(|directive| directive.split_whitespace().next())
        .collect::<BTreeSet<_>>();
    let sources = csp_sources(csp);

    for required_directive in [
        "default-src",
        "script-src",
        "style-src",
        "connect-src",
        "img-src",
        "font-src",
        "object-src",
        "base-uri",
        "frame-src",
        "worker-src",
        "form-action",
    ] {
        assert!(
            directives.contains(required_directive),
            "customer CSP should include {required_directive}"
        );
    }

    assert!(
        sources.contains("ipc:"),
        "customer CSP should allow Tauri IPC"
    );
    assert!(
        sources.contains("http://ipc.localhost"),
        "customer CSP should allow Tauri IPC localhost transport"
    );

    for dangerous_source in ["'unsafe-eval'", "*", "http:", "https:"] {
        assert!(
            !sources.contains(dangerous_source),
            "customer CSP should not allow {dangerous_source}"
        );
    }
}

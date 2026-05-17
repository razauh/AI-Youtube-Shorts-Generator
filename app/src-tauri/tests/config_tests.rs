use shorts_tauri_app::core::config::Config;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn defaults_match_python_when_env_missing() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    for key in [
        "MUAPI_API_KEY",
        "MUAPI_BASE_URL",
        "MUAPI_POLL_INTERVAL",
        "MUAPI_POLL_TIMEOUT",
        "OPENAI_API_KEY",
        "OPENAI_MODEL",
        "LOCAL_WHISPER_MODEL",
        "LOCAL_WHISPER_DEVICE",
        "LOCAL_OUTPUT_DIR",
        "LICENSE_WORKER_BASE_URL",
        "LICENSE_STORAGE_NAMESPACE",
        "LICENSE_KEYCHAIN_SERVICE",
        "LICENSE_BACKEND_MODE",
        "LICENSE_WORKER_TIMEOUT_MS",
        "LICENSE_WORKER_RETRY_ATTEMPTS",
        "LICENSE_WORKER_RETRY_BACKOFF_MS",
        "LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD",
        "LICENSE_WORKER_CIRCUIT_COOLDOWN_MS",
    ] {
        unsafe { std::env::remove_var(key) };
    }

    let cfg = Config::from_env().expect("config should load from defaults");
    assert_eq!(cfg.muapi_api_key, "");
    assert_eq!(cfg.muapi_base_url, "https://api.muapi.ai/api/v1");
    assert_eq!(cfg.muapi_poll_interval_seconds, 5.0);
    assert_eq!(cfg.muapi_poll_timeout_seconds, 600.0);
    assert_eq!(cfg.openai_api_key, "");
    assert_eq!(cfg.openai_model, "gpt-4o-mini");
    assert_eq!(cfg.local_whisper_model, "base");
    assert_eq!(cfg.local_whisper_device, "auto");
    assert_eq!(cfg.local_output_dir, "output");
    assert_eq!(cfg.license_worker_base_url, "http://127.0.0.1:8787");
    assert_eq!(cfg.license_storage_namespace, "desktop-client");
    assert_eq!(cfg.license_keychain_service, "ai-youtube-shorts-generator");
    assert_eq!(
        cfg.license_backend_mode,
        shorts_tauri_app::core::config::LicenseBackendMode::Reference
    );
    assert_eq!(cfg.license_worker_timeout_ms, 10_000);
    assert_eq!(cfg.license_worker_retry_attempts, 2);
    assert_eq!(cfg.license_worker_retry_backoff_ms, 150);
    assert_eq!(cfg.license_worker_circuit_breaker_failure_threshold, 3);
    assert_eq!(cfg.license_worker_circuit_breaker_cooldown_ms, 30_000);
}

#[test]
fn license_config_env_overrides_are_trimmed_and_normalized() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    unsafe {
        std::env::set_var(
            "LICENSE_WORKER_BASE_URL",
            " https://licenses.example.test/ ",
        );
        std::env::set_var("LICENSE_STORAGE_NAMESPACE", " desktop-client-test ");
        std::env::set_var("LICENSE_KEYCHAIN_SERVICE", " shorts-test ");
        std::env::set_var("LICENSE_BACKEND_MODE", " hosted ");
        std::env::set_var("LICENSE_WORKER_TIMEOUT_MS", "2500");
        std::env::set_var("LICENSE_WORKER_RETRY_ATTEMPTS", "4");
        std::env::set_var("LICENSE_WORKER_RETRY_BACKOFF_MS", "25");
        std::env::set_var("LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD", "5");
        std::env::set_var("LICENSE_WORKER_CIRCUIT_COOLDOWN_MS", "5000");
    }

    let cfg = Config::from_env().expect("config should load");
    assert_eq!(cfg.license_worker_base_url, "https://licenses.example.test");
    assert_eq!(cfg.license_storage_namespace, "desktop-client-test");
    assert_eq!(cfg.license_keychain_service, "shorts-test");
    assert_eq!(
        cfg.license_backend_mode,
        shorts_tauri_app::core::config::LicenseBackendMode::Hosted
    );
    assert_eq!(cfg.license_worker_timeout_ms, 2500);
    assert_eq!(cfg.license_worker_retry_attempts, 4);
    assert_eq!(cfg.license_worker_retry_backoff_ms, 25);
    assert_eq!(cfg.license_worker_circuit_breaker_failure_threshold, 5);
    assert_eq!(cfg.license_worker_circuit_breaker_cooldown_ms, 5000);

    unsafe {
        std::env::remove_var("LICENSE_WORKER_BASE_URL");
        std::env::remove_var("LICENSE_STORAGE_NAMESPACE");
        std::env::remove_var("LICENSE_KEYCHAIN_SERVICE");
        std::env::remove_var("LICENSE_BACKEND_MODE");
        std::env::remove_var("LICENSE_WORKER_TIMEOUT_MS");
        std::env::remove_var("LICENSE_WORKER_RETRY_ATTEMPTS");
        std::env::remove_var("LICENSE_WORKER_RETRY_BACKOFF_MS");
        std::env::remove_var("LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD");
        std::env::remove_var("LICENSE_WORKER_CIRCUIT_COOLDOWN_MS");
    }
}

#[test]
fn missing_required_keys_return_deterministic_prefixes() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    unsafe {
        std::env::remove_var("MUAPI_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
    }

    let cfg = Config::from_env().expect("config should still load");
    let api_err = cfg
        .require_api_key()
        .expect_err("must fail when MUAPI_API_KEY missing");
    assert!(api_err.to_string().starts_with("MUAPI_API_KEY is not set."));

    let openai_err = cfg
        .require_openai_key()
        .expect_err("must fail when OPENAI_API_KEY missing");
    assert!(openai_err
        .to_string()
        .starts_with("OPENAI_API_KEY is not set."));
}

#[test]
fn invalid_float_env_returns_parse_error_with_var_name() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    unsafe {
        std::env::set_var("MUAPI_POLL_INTERVAL", "abc");
    }

    let err = Config::from_env().expect_err("invalid float env should fail");
    assert!(err
        .to_string()
        .starts_with("invalid float for MUAPI_POLL_INTERVAL:"));

    unsafe {
        std::env::remove_var("MUAPI_POLL_INTERVAL");
    }
}

#[test]
fn invalid_license_backend_mode_is_rejected() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    unsafe {
        std::env::set_var("LICENSE_BACKEND_MODE", "cloudflare");
    }

    let err = Config::from_env().expect_err("unknown backend mode should fail");
    assert!(err.to_string().starts_with("invalid LICENSE_BACKEND_MODE:"));

    unsafe {
        std::env::remove_var("LICENSE_BACKEND_MODE");
    }
}

#[test]
fn invalid_license_worker_integer_is_rejected() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    unsafe {
        std::env::set_var("LICENSE_WORKER_TIMEOUT_MS", "soon");
    }

    let err = Config::from_env().expect_err("invalid integer env should fail");
    assert!(err
        .to_string()
        .starts_with("invalid integer for LICENSE_WORKER_TIMEOUT_MS:"));

    unsafe {
        std::env::remove_var("LICENSE_WORKER_TIMEOUT_MS");
    }
}

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

use shorts_tauri_app::commands::health::app_config_summary;
use shorts_tauri_app::core::config::{
    load_env_files_from, Config, DEFAULT_DEVOLENS_BASE_URL, PRODUCTION_LICENSE_WORKER_BASE_URL,
};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

const CONFIG_ENV_KEYS: &[&str] = &[
    "MUAPI_API_KEY",
    "MUAPI_BASE_URL",
    "MUAPI_POLL_INTERVAL",
    "MUAPI_POLL_TIMEOUT",
    "OPENAI_API_KEY",
    "OPENAI_MODEL",
    "LICENSE_WORKER_BASE_URL",
    "LICENSE_STORAGE_NAMESPACE",
    "LICENSE_KEYCHAIN_SERVICE",
    "LICENSE_BACKEND_MODE",
    "LICENSE_WORKER_TIMEOUT_MS",
    "LICENSE_WORKER_RETRY_ATTEMPTS",
    "LICENSE_WORKER_RETRY_BACKOFF_MS",
    "LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD",
    "LICENSE_WORKER_CIRCUIT_COOLDOWN_MS",
    "DEVOLENS_BASE_URL",
    "DEVOLENS_ACCESS_TOKEN",
    "DEVOLENS_PRODUCT_ID",
    "DEVOLENS_OFFLINE_GRACE_PERIOD_MS",
    "SKIP_DEVOLENS_TOKEN_SAFETY_CHECK",
];

const ISOLATED_ENV_KEYS: &[&str] = &["HOME", "PATH", "APPDATA"];

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct IsolatedConfigEnv {
    _guard: MutexGuard<'static, ()>,
    saved_env: Vec<(&'static str, Option<String>)>,
    root: PathBuf,
}

impl IsolatedConfigEnv {
    fn new(tag: &str) -> Self {
        let guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = std::env::temp_dir().join(format!(
            "shorts-config-test-{tag}-{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).expect("isolated config test root should be created");

        let saved_env = CONFIG_ENV_KEYS
            .iter()
            .chain(ISOLATED_ENV_KEYS.iter())
            .map(|key| (*key, std::env::var(key).ok()))
            .collect();

        unsafe {
            for key in CONFIG_ENV_KEYS {
                std::env::remove_var(key);
            }
            std::env::set_var("HOME", &root);
            std::env::set_var("PATH", &root);
            std::env::set_var("APPDATA", &root);
        }

        Self {
            _guard: guard,
            saved_env,
            root,
        }
    }
}

impl Drop for IsolatedConfigEnv {
    fn drop(&mut self) {
        unsafe {
            for (key, value) in &self.saved_env {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
        std::fs::remove_dir_all(&self.root).ok();
    }
}

fn configure_required_devolens_env() {
    unsafe {
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", "client-token");
        std::env::set_var("DEVOLENS_PRODUCT_ID", "1234");
        std::env::set_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK", "1");
    }
}

#[test]
fn defaults_use_devolens_when_required_credentials_are_set() {
    let _env = IsolatedConfigEnv::new("defaults");
    configure_required_devolens_env();

    let cfg = Config::from_env().expect("config should load from defaults");
    assert_eq!(cfg.muapi_api_key, "");
    assert_eq!(cfg.muapi_base_url, "https://api.muapi.ai/api/v1");
    assert_eq!(cfg.muapi_poll_interval_seconds, 5.0);
    assert_eq!(cfg.muapi_poll_timeout_seconds, 600.0);
    assert_eq!(cfg.openai_api_key, "");
    assert_eq!(cfg.openai_model, "gpt-4o-mini");
    assert_eq!(cfg.license_worker_base_url, PRODUCTION_LICENSE_WORKER_BASE_URL);
    assert_eq!(cfg.license_storage_namespace, "desktop-client");
    assert_eq!(cfg.license_keychain_service, "ai-youtube-shorts-generator");
    assert_eq!(
        cfg.license_backend_mode,
        shorts_tauri_app::core::config::LicenseBackendMode::Devolens
    );
    assert_eq!(cfg.license_worker_timeout_ms, 10_000);
    assert_eq!(cfg.license_worker_retry_attempts, 2);
    assert_eq!(cfg.license_worker_retry_backoff_ms, 150);
    assert_eq!(cfg.license_worker_circuit_breaker_failure_threshold, 3);
    assert_eq!(cfg.license_worker_circuit_breaker_cooldown_ms, 30_000);
    assert_eq!(cfg.devolens_base_url, DEFAULT_DEVOLENS_BASE_URL);
    assert_eq!(cfg.devolens_access_token, "client-token");
    assert_eq!(cfg.devolens_product_id, "1234");
}

#[test]
fn default_devolens_config_requires_provider_credentials() {
    let _env = IsolatedConfigEnv::new("devolens-required-by-default");

    let err = Config::from_env().expect_err("devolens token should be required");
    assert!(err.to_string().contains("DEVOLENS_ACCESS_TOKEN"));
}

#[test]
fn app_config_summary_reports_status_without_secret_values() {
    let _env = IsolatedConfigEnv::new("summary");
    unsafe {
        std::env::set_var("MUAPI_API_KEY", "muapi-secret-value");
        std::env::set_var("OPENAI_API_KEY", "openai-secret-value");
        std::env::set_var(
            "LICENSE_WORKER_BASE_URL",
            "http://127.0.0.1:8787/license-secret-route",
        );
        configure_required_devolens_env();
    }

    let summary = app_config_summary().expect("summary should load");
    assert_eq!(summary.license_backend_mode, "devolens");
    assert_eq!(summary.license_worker_endpoint, "local/private worker");
    assert_eq!(summary.license_worker_endpoint_kind, "local");
    assert!(summary.muapi_configured);
    assert!(summary.openai_configured);

    let encoded = serde_json::to_string(&summary).expect("summary should serialize");
    assert!(!encoded.contains("muapi-secret-value"));
    assert!(!encoded.contains("openai-secret-value"));
    assert!(!encoded.contains("127.0.0.1"));
    assert!(!encoded.contains("8787"));
    assert!(!encoded.contains("license-secret-route"));

    unsafe {
        std::env::remove_var("MUAPI_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("LICENSE_WORKER_BASE_URL");
        std::env::remove_var("DEVOLENS_ACCESS_TOKEN");
        std::env::remove_var("DEVOLENS_PRODUCT_ID");
        std::env::remove_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK");
    }
}

#[test]
fn dotenv_loader_walks_parent_dirs_without_overriding_existing_env() {
    let _env = IsolatedConfigEnv::new("dotenv");
    let root = std::env::temp_dir().join(format!("shorts-dotenv-test-{}", std::process::id()));
    let app_dir = root.join("app");
    std::fs::create_dir_all(&app_dir).expect("test app dir should be created");
    std::fs::write(
        root.join(".env"),
        "LICENSE_WORKER_BASE_URL=https://licenses.example.test\nLICENSE_BACKEND_MODE=devolens\nLICENSE_STORAGE_NAMESPACE=root-env\n",
    )
    .expect("root dotenv should be written");
    std::fs::write(
        app_dir.join(".env"),
        "LICENSE_STORAGE_NAMESPACE=app-env\nLICENSE_KEYCHAIN_SERVICE='quoted-service'\n",
    )
    .expect("app dotenv should be written");

    unsafe {
        std::env::remove_var("LICENSE_WORKER_BASE_URL");
        std::env::remove_var("LICENSE_BACKEND_MODE");
        std::env::remove_var("LICENSE_STORAGE_NAMESPACE");
        std::env::remove_var("LICENSE_KEYCHAIN_SERVICE");
        std::env::set_var("LICENSE_BACKEND_MODE", "mock");
    }

    load_env_files_from(&app_dir);

    assert_eq!(
        std::env::var("LICENSE_WORKER_BASE_URL").unwrap(),
        "https://licenses.example.test"
    );
    assert_eq!(std::env::var("LICENSE_BACKEND_MODE").unwrap(), "mock");
    assert_eq!(
        std::env::var("LICENSE_STORAGE_NAMESPACE").unwrap(),
        "app-env"
    );
    assert_eq!(
        std::env::var("LICENSE_KEYCHAIN_SERVICE").unwrap(),
        "quoted-service"
    );

    unsafe {
        std::env::remove_var("LICENSE_WORKER_BASE_URL");
        std::env::remove_var("LICENSE_BACKEND_MODE");
        std::env::remove_var("LICENSE_STORAGE_NAMESPACE");
        std::env::remove_var("LICENSE_KEYCHAIN_SERVICE");
    }
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn license_config_env_overrides_are_trimmed_and_normalized() {
    let _env = IsolatedConfigEnv::new("overrides");
    unsafe {
        std::env::set_var(
            "LICENSE_WORKER_BASE_URL",
            " https://licenses.example.test/ ",
        );
        std::env::set_var("LICENSE_STORAGE_NAMESPACE", " desktop-client-test ");
        std::env::set_var("LICENSE_KEYCHAIN_SERVICE", " shorts-test ");
        std::env::set_var("LICENSE_BACKEND_MODE", " devolens ");
        std::env::set_var("LICENSE_WORKER_TIMEOUT_MS", "2500");
        std::env::set_var("LICENSE_WORKER_RETRY_ATTEMPTS", "4");
        std::env::set_var("LICENSE_WORKER_RETRY_BACKOFF_MS", "25");
        std::env::set_var("LICENSE_WORKER_CIRCUIT_FAILURE_THRESHOLD", "5");
        std::env::set_var("LICENSE_WORKER_CIRCUIT_COOLDOWN_MS", "5000");
        std::env::set_var("DEVOLENS_BASE_URL", " https://devolens.example.test/ ");
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", " devolens-secret ");
        std::env::set_var("DEVOLENS_PRODUCT_ID", " 1234 ");
        std::env::set_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK", "1");
    }

    let cfg = Config::from_env().expect("config should load");
    assert_eq!(cfg.license_worker_base_url, "https://licenses.example.test");
    assert_eq!(cfg.license_storage_namespace, "desktop-client-test");
    assert_eq!(cfg.license_keychain_service, "shorts-test");
    assert_eq!(
        cfg.license_backend_mode,
        shorts_tauri_app::core::config::LicenseBackendMode::Devolens
    );
    assert_eq!(cfg.license_worker_timeout_ms, 2500);
    assert_eq!(cfg.license_worker_retry_attempts, 4);
    assert_eq!(cfg.license_worker_retry_backoff_ms, 25);
    assert_eq!(cfg.license_worker_circuit_breaker_failure_threshold, 5);
    assert_eq!(cfg.license_worker_circuit_breaker_cooldown_ms, 5000);
    assert_eq!(cfg.devolens_base_url, "https://devolens.example.test");
    assert_eq!(cfg.devolens_access_token, "devolens-secret");
    assert_eq!(cfg.devolens_product_id, "1234");

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
        std::env::remove_var("DEVOLENS_BASE_URL");
        std::env::remove_var("DEVOLENS_ACCESS_TOKEN");
        std::env::remove_var("DEVOLENS_PRODUCT_ID");
        std::env::remove_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK");
    }
}

#[test]
fn devolens_backend_requires_provider_credentials() {
    let _env = IsolatedConfigEnv::new("devolens-required");
    unsafe {
        std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
        std::env::set_var("DEVOLENS_PRODUCT_ID", "1234");
    }

    let err = Config::from_env().expect_err("devolens token should be required");
    assert!(err.to_string().contains("DEVOLENS_ACCESS_TOKEN"));

    unsafe {
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", "token");
        std::env::remove_var("DEVOLENS_PRODUCT_ID");
    }

    let err = Config::from_env().expect_err("devolens product id should be required");
    assert!(err.to_string().contains("DEVOLENS_PRODUCT_ID"));
}

#[test]
fn devolens_backend_mode_loads_provider_config() {
    let _env = IsolatedConfigEnv::new("devolens-mode");
    unsafe {
        std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
        std::env::set_var("DEVOLENS_BASE_URL", "https://api.cryptolens.io/");
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", "devolens-secret-value");
        std::env::set_var("DEVOLENS_PRODUCT_ID", "1234");
        std::env::set_var("SKIP_DEVOLENS_TOKEN_SAFETY_CHECK", "1");
    }

    let cfg = Config::from_env().expect("devolens config should load");

    assert_eq!(
        cfg.license_backend_mode,
        shorts_tauri_app::core::config::LicenseBackendMode::Devolens
    );
    assert_eq!(cfg.devolens_base_url, "https://api.cryptolens.io");
    assert_eq!(cfg.devolens_access_token, "devolens-secret-value");
    assert_eq!(cfg.devolens_product_id, "1234");
}

#[test]
fn missing_required_keys_return_deterministic_prefixes() {
    let _env = IsolatedConfigEnv::new("missing-keys");
    configure_required_devolens_env();

    let cfg = Config::from_env().expect("config should still load");
    let api_err = cfg
        .require_api_key()
        .expect_err("must fail when MUAPI_API_KEY missing");
    assert!(api_err.to_string().starts_with("MUAPI_API_KEY is not set."));

}

#[test]
fn invalid_float_env_returns_parse_error_with_var_name() {
    let _env = IsolatedConfigEnv::new("invalid-float");
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
    let _env = IsolatedConfigEnv::new("invalid-backend");
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
fn legacy_license_backend_modes_are_rejected() {
    let _env = IsolatedConfigEnv::new("legacy-backends");
    for mode in ["reference", "hosted"] {
        unsafe {
            std::env::set_var("LICENSE_BACKEND_MODE", mode);
        }

        let err = Config::from_env().expect_err("legacy backend mode should fail");
        assert!(err.to_string().starts_with("invalid LICENSE_BACKEND_MODE:"));
    }

    unsafe {
        std::env::remove_var("LICENSE_BACKEND_MODE");
    }
}

#[test]
fn invalid_license_worker_integer_is_rejected() {
    let _env = IsolatedConfigEnv::new("invalid-integer");
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

#[test]
fn devolens_token_safety_check_rejects_privileged_token() {
    let _env = IsolatedConfigEnv::new("devolens-safety-fail");
    
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let mock_url = format!("http://127.0.0.1:{}", port);
    
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buffer);
            
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"result\":0,\"products\":[]}";
            let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
        }
    });

    unsafe {
        std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
        std::env::set_var("DEVOLENS_BASE_URL", &mock_url);
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", "privileged-token");
        std::env::set_var("DEVOLENS_PRODUCT_ID", "1234");
    }

    let err = Config::from_env().expect_err("privileged token must be rejected");
    assert!(err.to_string().contains("DEVOLENS_ACCESS_TOKEN"));
    assert!(err.to_string().contains("management scopes"));
}

#[test]
fn devolens_token_safety_check_accepts_client_only_token() {
    let _env = IsolatedConfigEnv::new("devolens-safety-pass");
    
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let mock_url = format!("http://127.0.0.1:{}", port);
    
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buffer);
            
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"result\":1,\"message\":\"Access denied.\"}";
            let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
        }
    });

    unsafe {
        std::env::set_var("LICENSE_BACKEND_MODE", "devolens");
        std::env::set_var("DEVOLENS_BASE_URL", &mock_url);
        std::env::set_var("DEVOLENS_ACCESS_TOKEN", "client-token");
        std::env::set_var("DEVOLENS_PRODUCT_ID", "1234");
    }

    let cfg = Config::from_env().expect("client-only token must be accepted");
    assert_eq!(cfg.devolens_access_token, "client-token");
}

use crate::auth_worker::build_worker_client;
use crate::core::config::{Config, LicenseWorkerConfig};
use async_trait::async_trait;
use license_control_suite::core::{
    AuthError, AuthService, Clock, DeviceFingerprint, DeviceIdentityProvider, DeviceKeyPair,
    DevicePublicKey,
};
use license_control_suite::desktop::persistence::{AppDataStateStore, KeychainSecretStore};
use license_control_suite::desktop::tauri::AuthAppState;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

const DEVICE_IDENTITY_FILE: &str = "device_identity.json";

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0)
    }
}

#[derive(Clone)]
pub struct RuntimeDeviceIdentityProvider {
    identity_path: PathBuf,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StoredDeviceIdentity {
    public_key: String,
    private_key_material: String,
}

impl RuntimeDeviceIdentityProvider {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            identity_path: root.into().join(DEVICE_IDENTITY_FILE),
        }
    }

    pub fn identity_path(&self) -> &Path {
        &self.identity_path
    }

    fn ensure_parent(&self) -> Result<(), AuthError> {
        if let Some(parent) = self.identity_path.parent() {
            fs::create_dir_all(parent).map_err(|err| AuthError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn read_stored(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
        if !self.identity_path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&self.identity_path)
            .map_err(|err| AuthError::Storage(err.to_string()))?;
        let stored: StoredDeviceIdentity =
            serde_json::from_str(&raw).map_err(|err| AuthError::Serialization(err.to_string()))?;
        DeviceKeyPair::new(
            DevicePublicKey::new(stored.public_key)?,
            stored.private_key_material,
        )
        .map(Some)
    }

    fn write_new(&self) -> Result<DeviceKeyPair, AuthError> {
        self.ensure_parent()?;
        let private_key_material = generate_secret_material();
        let public_key = bytes_to_hex(&Sha256::digest(private_key_material.as_bytes()));
        let keypair = DeviceKeyPair::new(
            DevicePublicKey::new(public_key.clone())?,
            private_key_material.clone(),
        )?;
        let stored = StoredDeviceIdentity {
            public_key,
            private_key_material,
        };
        let raw = serde_json::to_string_pretty(&stored)
            .map_err(|err| AuthError::Serialization(err.to_string()))?;
        fs::write(&self.identity_path, raw).map_err(|err| AuthError::Storage(err.to_string()))?;
        Ok(keypair)
    }
}

#[async_trait]
impl DeviceIdentityProvider for RuntimeDeviceIdentityProvider {
    async fn get_or_create_keypair(&self) -> Result<DeviceKeyPair, AuthError> {
        match self.read_stored()? {
            Some(keypair) => Ok(keypair),
            None => self.write_new(),
        }
    }

    async fn collect_fingerprint(&self) -> Result<DeviceFingerprint, AuthError> {
        DeviceFingerprint::new(
            std::env::consts::OS,
            std::env::consts::OS,
            std::env::consts::ARCH,
            hostname_hash(),
        )
    }
}

pub fn build_auth_state<R>(app: &tauri::AppHandle<R>) -> Result<AuthAppState, String>
where
    R: tauri::Runtime,
{
    let cfg = Config::from_env().map_err(|err| err.to_string())?;
    let app_data_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    build_auth_state_from_parts(
        &cfg.license_worker_config(),
        app_data_dir,
        env!("CARGO_PKG_VERSION"),
    )
}

pub fn build_auth_state_from_parts(
    worker_config: &LicenseWorkerConfig,
    app_data_dir: impl Into<PathBuf>,
    app_version: &str,
) -> Result<AuthAppState, String> {
    let app_data_dir = app_data_dir.into();
    let worker = build_worker_client(worker_config).map_err(|err| err.to_string())?;
    let state = AppDataStateStore::with_namespace(&app_data_dir, &worker_config.storage_namespace);
    let secrets = KeychainSecretStore::with_namespace(
        &worker_config.keychain_service,
        &worker_config.storage_namespace,
    );
    let identity = RuntimeDeviceIdentityProvider::new(app_data_dir.join("auth"));
    let service = AuthService::new(
        worker,
        Arc::new(secrets),
        Arc::new(state),
        Arc::new(identity),
        Arc::new(SystemClock),
        app_version,
    );
    Ok(AuthAppState {
        service: Arc::new(service),
    })
}

fn generate_secret_material() -> String {
    let seed = format!(
        "{}:{}:{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0),
        std::env::var("HOME").unwrap_or_default()
    );
    bytes_to_hex(&Sha256::digest(seed.as_bytes()))
}

fn hostname_hash() -> Option<String> {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| bytes_to_hex(&Sha256::digest(value.trim().as_bytes())))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "shorts-auth-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root should be created");
        root
    }

    #[test]
    fn system_clock_returns_positive_epoch_ms() {
        assert!(SystemClock.now_ms() > 0);
    }

    #[tokio::test]
    async fn runtime_identity_provider_creates_and_reuses_keypair() {
        let root = temp_root("stable");
        let provider = RuntimeDeviceIdentityProvider::new(&root);

        let first = provider
            .get_or_create_keypair()
            .await
            .expect("first keypair should be created");
        let second = provider
            .get_or_create_keypair()
            .await
            .expect("second keypair should load");

        assert_eq!(first.public_key().as_str(), second.public_key().as_str());
        assert!(!first.public_key().as_str().is_empty());
        assert!(provider.identity_path().exists());
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn runtime_identity_provider_reports_corrupt_identity_as_error() {
        let root = temp_root("corrupt");
        let provider = RuntimeDeviceIdentityProvider::new(&root);
        fs::write(provider.identity_path(), "not-json")
            .expect("corrupt identity should be written");

        let err = provider
            .get_or_create_keypair()
            .await
            .expect_err("corrupt identity should not be silently accepted");
        assert_eq!(err.code(), "serialization");
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn runtime_identity_provider_collects_platform_fingerprint() {
        let root = temp_root("fingerprint");
        let provider = RuntimeDeviceIdentityProvider::new(&root);
        let fingerprint = provider
            .collect_fingerprint()
            .await
            .expect("fingerprint should be collected");

        assert_eq!(fingerprint.platform, std::env::consts::OS);
        assert_eq!(fingerprint.os, std::env::consts::OS);
        assert_eq!(fingerprint.arch, std::env::consts::ARCH);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn auth_state_builder_accepts_basic_parts_without_touching_keyring() {
        let root = temp_root("builder");
        let worker_config = LicenseWorkerConfig {
            backend_mode: crate::core::config::LicenseBackendMode::Reference,
            base_url: "http://127.0.0.1:8787".to_string(),
            storage_namespace: "desktop-client-test".to_string(),
            keychain_service: "shorts-test".to_string(),
            timeout_ms: 10_000,
            retry_attempts: 1,
            retry_backoff_ms: 0,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_cooldown_ms: 30_000,
        };
        let state = build_auth_state_from_parts(&worker_config, &root, "0.1.0");

        assert!(state.is_ok());
        let _ = fs::remove_dir_all(root);
    }
}

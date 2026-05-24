use crate::auth_worker::build_worker_client;
use crate::core::config::{Config, LicenseWorkerConfig};
use async_trait::async_trait;
use license_control_suite::core::{
    AccessToken, AuthError, AuthService, Clock, DeviceFingerprint, DeviceIdentityProvider,
    DeviceKeyPair, DevicePublicKey, LicenseKey, SecretStore,
};
use license_control_suite::desktop::persistence::{AppDataStateStore, KeychainSecretStore};
use license_control_suite::desktop::tauri::AuthAppState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

const DEVICE_IDENTITY_FILE: &str = "device_identity.json";
const SESSION_SECRETS_FALLBACK_FILE: &str = "session_secrets_fallback.json";

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

#[derive(Clone)]
struct ResilientSecretStore {
    primary: Arc<dyn SecretStore>,
    fallback_path: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FallbackSecrets {
    license_key: Option<String>,
    access_token: Option<String>,
    device_keypair: Option<DeviceKeyPair>,
}

impl ResilientSecretStore {
    fn new(primary: Arc<dyn SecretStore>, fallback_path: PathBuf) -> Self {
        Self {
            primary,
            fallback_path,
        }
    }

    fn ensure_parent(&self) -> Result<(), AuthError> {
        if let Some(parent) = self.fallback_path.parent() {
            fs::create_dir_all(parent).map_err(|err| AuthError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn load_fallback(&self) -> Result<FallbackSecrets, AuthError> {
        if !self.fallback_path.exists() {
            return Ok(FallbackSecrets::default());
        }
        let raw = fs::read_to_string(&self.fallback_path)
            .map_err(|err| AuthError::Storage(err.to_string()))?;
        serde_json::from_str(&raw).map_err(|err| AuthError::Serialization(err.to_string()))
    }

    fn save_fallback(&self, secrets: &FallbackSecrets) -> Result<(), AuthError> {
        self.ensure_parent()?;
        let raw = serde_json::to_string_pretty(secrets)
            .map_err(|err| AuthError::Serialization(err.to_string()))?;
        fs::write(&self.fallback_path, raw).map_err(|err| AuthError::Storage(err.to_string()))
    }

    fn update_fallback<F>(&self, mutator: F) -> Result<(), AuthError>
    where
        F: FnOnce(&mut FallbackSecrets),
    {
        let mut secrets = self.load_fallback()?;
        mutator(&mut secrets);
        self.save_fallback(&secrets)
    }
}

#[async_trait]
impl SecretStore for ResilientSecretStore {
    async fn put_license_key(&self, value: LicenseKey) -> Result<(), AuthError> {
        let primary_result = self.primary.put_license_key(value.clone()).await;
        self.update_fallback(|secrets| {
            secrets.license_key = Some(value.expose_secret().to_string());
        })?;
        match primary_result {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError> {
        match self.primary.get_license_key().await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) | Err(_) => self
                .load_fallback()?
                .license_key
                .map(LicenseKey::new)
                .transpose(),
        }
    }

    async fn put_access_token(&self, value: AccessToken) -> Result<(), AuthError> {
        let primary_result = self.primary.put_access_token(value.clone()).await;
        self.update_fallback(|secrets| {
            secrets.access_token = Some(value.expose_secret().to_string());
        })?;
        match primary_result {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
        match self.primary.get_access_token().await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) | Err(_) => self
                .load_fallback()?
                .access_token
                .map(AccessToken::new)
                .transpose(),
        }
    }

    async fn put_device_keypair(&self, value: DeviceKeyPair) -> Result<(), AuthError> {
        let primary_result = self.primary.put_device_keypair(value.clone()).await;
        self.update_fallback(|secrets| {
            secrets.device_keypair = Some(value);
        })?;
        match primary_result {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
        match self.primary.get_device_keypair().await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) | Err(_) => Ok(self.load_fallback()?.device_keypair),
        }
    }

    async fn clear_session_secrets(&self) -> Result<(), AuthError> {
        let _ = self.primary.clear_session_secrets().await;
        self.update_fallback(|secrets| {
            secrets.license_key = None;
            secrets.access_token = None;
        })
    }
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
    crate::core::config::load_env_files_near_current_dir();
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
    let keychain = KeychainSecretStore::with_namespace(
        &worker_config.keychain_service,
        &worker_config.storage_namespace,
    );
    let secrets: Arc<dyn SecretStore> = Arc::new(ResilientSecretStore::new(
        Arc::new(keychain),
        app_data_dir
            .join("auth")
            .join(format!("{}.{}", worker_config.storage_namespace, SESSION_SECRETS_FALLBACK_FILE)),
    ));
    let identity = RuntimeDeviceIdentityProvider::new(app_data_dir.join("auth"));
    let service = AuthService::new(
        worker,
        secrets,
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
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct AlwaysFailSecretStore;

    #[async_trait]
    impl SecretStore for AlwaysFailSecretStore {
        async fn put_license_key(&self, _value: LicenseKey) -> Result<(), AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
        async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
        async fn put_access_token(&self, _value: AccessToken) -> Result<(), AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
        async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
        async fn put_device_keypair(&self, _value: DeviceKeyPair) -> Result<(), AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
        async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
        async fn clear_session_secrets(&self) -> Result<(), AuthError> {
            Err(AuthError::Storage("keychain unavailable".to_string()))
        }
    }

    #[derive(Default)]
    struct MemoryPrimarySecretStore {
        license_key: Mutex<Option<LicenseKey>>,
        access_token: Mutex<Option<AccessToken>>,
        device_keypair: Mutex<Option<DeviceKeyPair>>,
    }

    #[async_trait]
    impl SecretStore for MemoryPrimarySecretStore {
        async fn put_license_key(&self, value: LicenseKey) -> Result<(), AuthError> {
            *self.license_key.lock().expect("lock license key") = Some(value);
            Ok(())
        }
        async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError> {
            Ok(self
                .license_key
                .lock()
                .expect("lock license key")
                .clone())
        }
        async fn put_access_token(&self, value: AccessToken) -> Result<(), AuthError> {
            *self.access_token.lock().expect("lock access token") = Some(value);
            Ok(())
        }
        async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
            Ok(self
                .access_token
                .lock()
                .expect("lock access token")
                .clone())
        }
        async fn put_device_keypair(&self, value: DeviceKeyPair) -> Result<(), AuthError> {
            *self
                .device_keypair
                .lock()
                .expect("lock device keypair") = Some(value);
            Ok(())
        }
        async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
            Ok(self
                .device_keypair
                .lock()
                .expect("lock device keypair")
                .clone())
        }
        async fn clear_session_secrets(&self) -> Result<(), AuthError> {
            *self.license_key.lock().expect("lock license key") = None;
            *self.access_token.lock().expect("lock access token") = None;
            Ok(())
        }
    }

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

    #[tokio::test]
    async fn resilient_secret_store_uses_fallback_when_primary_unavailable() {
        let root = temp_root("resilient-fallback");
        let store = ResilientSecretStore::new(
            Arc::new(AlwaysFailSecretStore),
            root.join("auth").join("secrets_fallback.json"),
        );
        let keypair =
            DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "private").unwrap();

        store
            .put_license_key(LicenseKey::new("LICENSE-1234").unwrap())
            .await
            .expect("fallback license should persist");
        store
            .put_access_token(AccessToken::new("token-123").unwrap())
            .await
            .expect("fallback token should persist");
        store
            .put_device_keypair(keypair.clone())
            .await
            .expect("fallback keypair should persist");

        assert!(store.get_license_key().await.unwrap().is_some());
        assert!(store.get_access_token().await.unwrap().is_some());
        assert_eq!(
            store
                .get_device_keypair()
                .await
                .unwrap()
                .expect("device keypair expected")
                .public_key()
                .as_str(),
            keypair.public_key().as_str()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn resilient_secret_store_prefers_primary_reads() {
        let root = temp_root("resilient-primary");
        let primary = Arc::new(MemoryPrimarySecretStore::default());
        primary
            .put_access_token(AccessToken::new("token-primary").unwrap())
            .await
            .expect("primary token should persist");
        let store = ResilientSecretStore::new(
            primary,
            root.join("auth").join("secrets_fallback.json"),
        );

        assert_eq!(
            store
                .get_access_token()
                .await
                .unwrap()
                .expect("primary token expected")
                .expose_secret(),
            "token-primary"
        );
        let _ = fs::remove_dir_all(root);
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

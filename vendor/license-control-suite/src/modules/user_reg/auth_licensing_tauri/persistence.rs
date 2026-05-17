use async_trait::async_trait;
use crate::modules::user_reg::auth_licensing_core::{
    AccessToken, AuthError, DeviceKeyPair, DeviceResetStatus, LicenseKey, LocalStateStore,
    SecretStore, SessionState,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "desktop-tauri")]
use tauri::Manager;

#[cfg(test)]
const REAL_KEYRING_SMOKE_ENV: &str = "LICENSE_CONTROL_SUITE_RUN_REAL_KEYRING_SMOKE";

#[derive(Clone, Default)]
pub struct InMemorySecretStore {
    license_key: Arc<Mutex<Option<LicenseKey>>>,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    device_keypair: Arc<Mutex<Option<DeviceKeyPair>>>,
}

#[async_trait]
impl SecretStore for InMemorySecretStore {
    async fn put_license_key(&self, value: LicenseKey) -> Result<(), AuthError> {
        *self.license_key.lock().unwrap() = Some(value);
        Ok(())
    }

    async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError> {
        Ok(self.license_key.lock().unwrap().clone())
    }

    async fn put_access_token(&self, value: AccessToken) -> Result<(), AuthError> {
        *self.access_token.lock().unwrap() = Some(value);
        Ok(())
    }

    async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
        Ok(self.access_token.lock().unwrap().clone())
    }

    async fn put_device_keypair(&self, value: DeviceKeyPair) -> Result<(), AuthError> {
        *self.device_keypair.lock().unwrap() = Some(value);
        Ok(())
    }

    async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
        Ok(self.device_keypair.lock().unwrap().clone())
    }

    async fn clear_session_secrets(&self) -> Result<(), AuthError> {
        *self.license_key.lock().unwrap() = None;
        *self.access_token.lock().unwrap() = None;
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppDataStateStore {
    root: PathBuf,
    session_filename: String,
    reset_filename: String,
}

impl AppDataStateStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_file_names(root, "session_state.json", "reset_status.json")
    }

    pub fn with_namespace(root: impl Into<PathBuf>, namespace: impl AsRef<str>) -> Self {
        let namespace = namespace.as_ref();
        Self::with_file_names(
            root,
            format!("{namespace}.session_state.json"),
            format!("{namespace}.reset_status.json"),
        )
    }

    pub fn with_file_names(
        root: impl Into<PathBuf>,
        session_filename: impl Into<String>,
        reset_filename: impl Into<String>,
    ) -> Self {
        Self {
            root: root.into(),
            session_filename: session_filename.into(),
            reset_filename: reset_filename.into(),
        }
    }

    #[cfg(feature = "desktop-tauri")]
    pub fn from_app_handle<R>(app: &tauri::AppHandle<R>) -> Result<Self, AuthError>
    where
        R: tauri::Runtime,
    {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|err| AuthError::Storage(err.to_string()))?;
        Ok(Self::new(root))
    }

    #[cfg(feature = "desktop-tauri")]
    pub fn from_app_handle_with_namespace<R>(
        app: &tauri::AppHandle<R>,
        namespace: impl AsRef<str>,
    ) -> Result<Self, AuthError>
    where
        R: tauri::Runtime,
    {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|err| AuthError::Storage(err.to_string()))?;
        Ok(Self::with_namespace(root, namespace))
    }

    pub fn session_state_path(&self) -> PathBuf {
        self.root.join(&self.session_filename)
    }

    pub fn reset_status_path(&self) -> PathBuf {
        self.root.join(&self.reset_filename)
    }

    fn read_json<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Result<Option<T>, AuthError> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path).map_err(|err| AuthError::Storage(err.to_string()))?;
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|err| AuthError::Serialization(err.to_string()))
    }

    fn write_json<T: serde::Serialize>(&self, path: &Path, value: &T) -> Result<(), AuthError> {
        std::fs::create_dir_all(&self.root).map_err(|err| AuthError::Storage(err.to_string()))?;
        let bytes = serde_json::to_vec_pretty(value)
            .map_err(|err| AuthError::Serialization(err.to_string()))?;
        std::fs::write(path, bytes).map_err(|err| AuthError::Storage(err.to_string()))
    }
}

#[async_trait]
impl LocalStateStore for AppDataStateStore {
    async fn save_session_state(&self, state: SessionState) -> Result<(), AuthError> {
        self.write_json(&self.session_state_path(), &state)
    }

    async fn load_session_state(&self) -> Result<SessionState, AuthError> {
        Ok(Self::read_json(&self.session_state_path())?.unwrap_or_default())
    }

    async fn save_reset_status(&self, status: DeviceResetStatus) -> Result<(), AuthError> {
        self.write_json(&self.reset_status_path(), &status)
    }

    async fn load_reset_status(&self) -> Result<Option<DeviceResetStatus>, AuthError> {
        Self::read_json(&self.reset_status_path())
    }
}

#[derive(Clone, Default)]
pub struct KeychainSecretStore {
    service_name: String,
    license_key_name: String,
    access_token_name: String,
    device_keypair_name: String,
}

impl KeychainSecretStore {
    pub fn new_for_app(service_name: impl Into<String>) -> Self {
        Self::with_entry_names(
            service_name,
            "license_key",
            "access_token",
            "device_keypair",
        )
    }

    pub fn with_namespace(
        service_name: impl Into<String>,
        namespace: impl AsRef<str>,
    ) -> Self {
        let namespace = namespace.as_ref();
        Self::with_entry_names(
            service_name,
            format!("{namespace}.license_key"),
            format!("{namespace}.access_token"),
            format!("{namespace}.device_keypair"),
        )
    }

    pub fn with_entry_names(
        service_name: impl Into<String>,
        license_key_name: impl Into<String>,
        access_token_name: impl Into<String>,
        device_keypair_name: impl Into<String>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            license_key_name: license_key_name.into(),
            access_token_name: access_token_name.into(),
            device_keypair_name: device_keypair_name.into(),
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn license_key_entry_name(&self) -> &str {
        &self.license_key_name
    }

    pub fn access_token_entry_name(&self) -> &str {
        &self.access_token_name
    }

    pub fn device_keypair_entry_name(&self) -> &str {
        &self.device_keypair_name
    }

    fn entry(&self, key: &str) -> Result<keyring::Entry, AuthError> {
        keyring::Entry::new(&self.service_name, key)
            .map_err(|err| AuthError::Storage(err.to_string()))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), AuthError> {
        self.entry(key)?
            .set_password(value)
            .map_err(|err| AuthError::Storage(err.to_string()))
    }

    fn get(&self, key: &str) -> Result<Option<String>, AuthError> {
        match self.entry(key)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(AuthError::Storage(err.to_string())),
        }
    }

    fn delete(&self, key: &str) -> Result<(), AuthError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(AuthError::Storage(err.to_string())),
        }
    }
}

#[async_trait]
impl SecretStore for KeychainSecretStore {
    async fn put_license_key(&self, value: LicenseKey) -> Result<(), AuthError> {
        self.set(&self.license_key_name, value.expose_secret())
    }

    async fn get_license_key(&self) -> Result<Option<LicenseKey>, AuthError> {
        self.get(&self.license_key_name)?
            .map(LicenseKey::new)
            .transpose()
    }

    async fn put_access_token(&self, value: AccessToken) -> Result<(), AuthError> {
        self.set(&self.access_token_name, value.expose_secret())
    }

    async fn get_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
        self.get(&self.access_token_name)?
            .map(AccessToken::new)
            .transpose()
    }

    async fn put_device_keypair(&self, value: DeviceKeyPair) -> Result<(), AuthError> {
        let json = serde_json::to_string(&value)
            .map_err(|err| AuthError::Serialization(err.to_string()))?;
        self.set(&self.device_keypair_name, &json)
    }

    async fn get_device_keypair(&self) -> Result<Option<DeviceKeyPair>, AuthError> {
        self.get(&self.device_keypair_name)?
            .map(|json| {
                serde_json::from_str(&json).map_err(|err| AuthError::Serialization(err.to_string()))
            })
            .transpose()
    }

    async fn clear_session_secrets(&self) -> Result<(), AuthError> {
        self.delete(&self.license_key_name)?;
        self.delete(&self.access_token_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user_reg::auth_licensing_core::{DevicePublicKey, MaskedLicenseKey, ResetRequestId};

    fn real_keyring_smoke_is_enabled_value(value: Option<&str>) -> bool {
        matches!(
            value,
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
        )
    }

    fn real_keyring_smoke_is_enabled() -> bool {
        real_keyring_smoke_is_enabled_value(std::env::var(REAL_KEYRING_SMOKE_ENV).ok().as_deref())
    }

    fn disposable_keyring_test_namespace() -> String {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_millis();
        format!("license-control-suite.keyring-smoke.{stamp}.{}", std::process::id())
    }

    fn disposable_keyring_test_store() -> KeychainSecretStore {
        KeychainSecretStore::with_namespace(
            "license-control-suite.real-keyring-smoke",
            disposable_keyring_test_namespace(),
        )
    }

    #[tokio::test]
    async fn app_data_store_missing_state_defaults_to_unauthenticated() {
        let dir = tempfile::tempdir().unwrap();
        let store = AppDataStateStore::new(dir.path());
        assert_eq!(
            store.load_session_state().await.unwrap(),
            SessionState::Unauthenticated
        );
    }

    #[tokio::test]
    async fn app_data_store_persists_reset_status() {
        let dir = tempfile::tempdir().unwrap();
        let store = AppDataStateStore::new(dir.path());
        let status = DeviceResetStatus::Pending {
            request_id: ResetRequestId::new("reset-1").unwrap(),
            created_at_ms: 10,
        };
        store.save_reset_status(status.clone()).await.unwrap();
        assert_eq!(store.load_reset_status().await.unwrap(), Some(status));
    }

    #[tokio::test]
    async fn corrupt_app_data_state_is_recoverable_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = AppDataStateStore::new(dir.path());
        std::fs::write(dir.path().join("session_state.json"), b"not-json").unwrap();
        assert!(matches!(
            store.load_session_state().await.unwrap_err(),
            AuthError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn clearing_session_removes_secrets() {
        let store = InMemorySecretStore::default();
        store
            .put_license_key(LicenseKey::new("key").unwrap())
            .await
            .unwrap();
        store
            .put_access_token(AccessToken::new("token").unwrap())
            .await
            .unwrap();
        store
            .put_device_keypair(
                DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "private").unwrap(),
            )
            .await
            .unwrap();
        store.clear_session_secrets().await.unwrap();
        assert!(store.get_license_key().await.unwrap().is_none());
        assert!(store.get_access_token().await.unwrap().is_none());
        assert!(store.get_device_keypair().await.unwrap().is_some());
    }

    #[test]
    fn real_keyring_round_trip_runs_only_when_explicitly_enabled() {
        assert!(!real_keyring_smoke_is_enabled_value(None));
        assert!(!real_keyring_smoke_is_enabled_value(Some("0")));
        assert!(real_keyring_smoke_is_enabled_value(Some("1")));
    }

    #[test]
    fn real_keyring_test_uses_disposable_namespace() {
        let store = disposable_keyring_test_store();
        assert_eq!(store.service_name(), "license-control-suite.real-keyring-smoke");
        assert!(store.license_key_entry_name().starts_with("license-control-suite.keyring-smoke."));
        assert!(store.access_token_entry_name().contains(".access_token"));
        assert!(store.device_keypair_entry_name().contains(".device_keypair"));
    }

    #[tokio::test]
    #[ignore = "opt-in real keyring smoke; set LICENSE_CONTROL_SUITE_RUN_REAL_KEYRING_SMOKE=1 and run explicitly"]
    async fn real_keychain_round_trip() {
        if !real_keyring_smoke_is_enabled() {
            return;
        }

        let store = disposable_keyring_test_store();
        store
            .put_license_key(LicenseKey::new("key").unwrap())
            .await
            .unwrap();
        assert!(store.get_license_key().await.unwrap().is_some());
        store.clear_session_secrets().await.unwrap();
    }

    #[tokio::test]
    async fn app_data_store_persists_non_secret_session_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = AppDataStateStore::new(dir.path());
        let state = SessionState::ReauthRequired {
            masked_license_key: Some(MaskedLicenseKey::new("••••-1234").unwrap()),
        };
        store.save_session_state(state.clone()).await.unwrap();
        assert_eq!(store.load_session_state().await.unwrap(), state);
    }
}

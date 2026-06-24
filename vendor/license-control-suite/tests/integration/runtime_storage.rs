use license_control_suite::modules::user_reg::auth_licensing_core::{
    AccessToken, DeviceKeyPair, DevicePublicKey, LicenseKey, LocalStateStore, MaskedLicenseKey,
    ResetRequestId, SecretStore, SessionState,
};
use license_control_suite::modules::user_reg::auth_licensing_tauri::{
    AppDataStateStore, InMemorySecretStore,
};
use license_control_suite::modules::user_reg::auth_licensing_core::DeviceResetStatus;
#[cfg(feature = "desktop-persistence")]
use license_control_suite::modules::user_reg::auth_licensing_tauri::KeychainSecretStore;

#[tokio::test]
async fn session_state_writes_to_expected_temp_path() {
    let dir = tempfile::tempdir().unwrap();
    let store = AppDataStateStore::new(dir.path());

    let state = SessionState::ReauthRequired {
        masked_license_key: Some(MaskedLicenseKey::new("••••-1234").unwrap()),
    };
    store.save_session_state(state).await.unwrap();

    let path = dir.path().join("session_state.json");
    assert!(path.exists());
}

#[tokio::test]
async fn reset_status_writes_to_expected_temp_path() {
    let dir = tempfile::tempdir().unwrap();
    let store = AppDataStateStore::new(dir.path());
    let status = DeviceResetStatus::Pending {
        request_id: ResetRequestId::new("reset-1").unwrap(),
        created_at_ms: 10,
    };
    store.save_reset_status(status).await.unwrap();

    let path = dir.path().join("reset_status.json");
    assert!(path.exists());
}

#[tokio::test]
async fn clearing_session_removes_license_and_access_but_keeps_device_keypair() {
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

#[tokio::test]
async fn baseline_storage_paths_and_clear_session_semantics_remain_current() {
    let dir = tempfile::tempdir().unwrap();
    let app_data_store = AppDataStateStore::new(dir.path());
    let session_state = SessionState::ReauthRequired {
        masked_license_key: Some(MaskedLicenseKey::new("••••-1234").unwrap()),
    };
    let reset_status = DeviceResetStatus::Pending {
        request_id: ResetRequestId::new("reset-1").unwrap(),
        created_at_ms: 10,
    };

    app_data_store
        .save_session_state(session_state.clone())
        .await
        .unwrap();
    app_data_store
        .save_reset_status(reset_status.clone())
        .await
        .unwrap();

    let session_path = dir.path().join("session_state.json");
    let reset_path = dir.path().join("reset_status.json");
    assert!(session_path.exists());
    assert!(reset_path.exists());
    assert_eq!(app_data_store.load_session_state().await.unwrap(), session_state);
    assert_eq!(app_data_store.load_reset_status().await.unwrap(), Some(reset_status));

    let secret_store = InMemorySecretStore::default();
    secret_store
        .put_license_key(LicenseKey::new("key").unwrap())
        .await
        .unwrap();
    secret_store
        .put_access_token(AccessToken::new("token").unwrap())
        .await
        .unwrap();
    secret_store
        .put_device_keypair(
            DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "private").unwrap(),
        )
        .await
        .unwrap();

    secret_store.clear_session_secrets().await.unwrap();

    assert!(secret_store.get_license_key().await.unwrap().is_none());
    assert!(secret_store.get_access_token().await.unwrap().is_none());
    assert!(secret_store.get_device_keypair().await.unwrap().is_some());
}

#[tokio::test]
async fn namespaced_session_state_writes_to_namespace_specific_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = AppDataStateStore::with_namespace(dir.path(), "desktop-client");
    let state = SessionState::ReauthRequired {
        masked_license_key: Some(MaskedLicenseKey::new("••••-1234").unwrap()),
    };

    store.save_session_state(state.clone()).await.unwrap();

    assert_eq!(
        store.session_state_path(),
        dir.path().join("desktop-client.session_state.json")
    );
    assert!(store.session_state_path().exists());
    assert_eq!(store.load_session_state().await.unwrap(), state);
}

#[tokio::test]
async fn namespaced_reset_status_writes_to_namespace_specific_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = AppDataStateStore::with_namespace(dir.path(), "desktop-client");
    let status = DeviceResetStatus::Pending {
        request_id: ResetRequestId::new("reset-1").unwrap(),
        created_at_ms: 10,
    };

    store.save_reset_status(status.clone()).await.unwrap();

    assert_eq!(
        store.reset_status_path(),
        dir.path().join("desktop-client.reset_status.json")
    );
    assert!(store.reset_status_path().exists());
    assert_eq!(store.load_reset_status().await.unwrap(), Some(status));
}

#[tokio::test]
async fn two_namespaces_do_not_collide_in_local_state() {
    let dir = tempfile::tempdir().unwrap();
    let alpha = AppDataStateStore::with_namespace(dir.path(), "alpha");
    let beta = AppDataStateStore::with_namespace(dir.path(), "beta");
    let alpha_state = SessionState::ReauthRequired {
        masked_license_key: Some(MaskedLicenseKey::new("••••-1111").unwrap()),
    };
    let beta_state = SessionState::ReauthRequired {
        masked_license_key: Some(MaskedLicenseKey::new("••••-2222").unwrap()),
    };
    let alpha_reset = DeviceResetStatus::Pending {
        request_id: ResetRequestId::new("alpha-reset").unwrap(),
        created_at_ms: 10,
    };
    let beta_reset = DeviceResetStatus::Pending {
        request_id: ResetRequestId::new("beta-reset").unwrap(),
        created_at_ms: 20,
    };

    alpha.save_session_state(alpha_state.clone()).await.unwrap();
    beta.save_session_state(beta_state.clone()).await.unwrap();
    alpha.save_reset_status(alpha_reset.clone()).await.unwrap();
    beta.save_reset_status(beta_reset.clone()).await.unwrap();

    assert_ne!(alpha.session_state_path(), beta.session_state_path());
    assert_ne!(alpha.reset_status_path(), beta.reset_status_path());
    assert_eq!(alpha.load_session_state().await.unwrap(), alpha_state);
    assert_eq!(beta.load_session_state().await.unwrap(), beta_state);
    assert_eq!(alpha.load_reset_status().await.unwrap(), Some(alpha_reset));
    assert_eq!(beta.load_reset_status().await.unwrap(), Some(beta_reset));
}

#[test]
#[cfg(feature = "desktop-persistence")]
fn keychain_secret_store_uses_namespaced_service_and_item_names() {
    let store = KeychainSecretStore::with_namespace("com.example.desktop", "desktop-client");

    assert_eq!(store.service_name(), "com.example.desktop");
    assert_eq!(store.license_key_entry_name(), "desktop-client.license_key");
    assert_eq!(store.access_token_entry_name(), "desktop-client.access_token");
    assert_eq!(store.device_keypair_entry_name(), "desktop-client.device_keypair");
}

#[tokio::test]
async fn default_compatibility_path_is_preserved_or_migration_behavior_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let legacy_store = AppDataStateStore::new(dir.path());
    let namespaced_store = AppDataStateStore::with_namespace(dir.path(), "desktop-client");
    let state = SessionState::ReauthRequired {
        masked_license_key: Some(MaskedLicenseKey::new("••••-1234").unwrap()),
    };

    legacy_store.save_session_state(state.clone()).await.unwrap();

    assert!(dir.path().join("session_state.json").exists());
    assert_eq!(legacy_store.session_state_path(), dir.path().join("session_state.json"));
    assert_eq!(namespaced_store.session_state_path(), dir.path().join("desktop-client.session_state.json"));
    assert_eq!(legacy_store.load_session_state().await.unwrap(), state);
}

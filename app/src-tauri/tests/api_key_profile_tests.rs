use shorts_tauri_app::commands::runtime::{
    api_key_profile_activate, api_key_profile_add, api_key_profile_delete, api_key_profiles,
    secure_store_load,
};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn isolate_runtime_home(tag: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "shorts-api-key-profile-test-{tag}-{}",
        std::process::id()
    ));
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(&root).expect("test runtime home should be created");
    unsafe {
        std::env::set_var("HOME", &root);
        std::env::set_var("PATH", &root);
        std::env::remove_var("MUAPI_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
    }
    root
}

#[test]
fn api_key_profiles_keep_metadata_secret_free_and_switch_active_key() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let root = isolate_runtime_home("switch");

    let first = api_key_profile_add(
        "muapi".to_string(),
        "Personal MuAPI".to_string(),
        "muapi-secret-one".to_string(),
        true,
    )
    .expect("first profile should save");
    assert_eq!(first.profiles.len(), 1);
    assert_eq!(first.profiles[0].label, "Personal MuAPI");
    assert_eq!(first.profiles[0].last_four, "-one");
    assert!(first.profiles[0].active);

    let second = api_key_profile_add(
        "muapi".to_string(),
        "Client MuAPI".to_string(),
        "muapi-secret-two".to_string(),
        false,
    )
    .expect("second profile should save");
    assert_eq!(second.profiles.len(), 2);
    let client = second
        .profiles
        .iter()
        .find(|profile| profile.label == "Client MuAPI")
        .expect("client profile should be listed");
    assert!(!client.active);
    let client_id = client.id.clone();

    let encoded = std::fs::read_to_string(
        root.join(".local")
            .join("share")
            .join("ai-youtube-shorts-generator")
            .join("config")
            .join("api-key-profiles.json"),
    )
    .expect("metadata should be written");
    assert!(!encoded.contains("muapi-secret-one"));
    assert!(!encoded.contains("muapi-secret-two"));

    let activated = api_key_profile_activate("muapi".to_string(), client_id.clone())
        .expect("client profile should activate");
    assert!(activated
        .profiles
        .iter()
        .any(|profile| profile.id == client_id && profile.active));
    assert_eq!(
        secure_store_load("MUAPI_API_KEY".to_string()).expect("legacy key should load"),
        Some("muapi-secret-two".to_string())
    );

    let deleted = api_key_profile_delete("muapi".to_string(), client_id)
        .expect("active profile should delete");
    assert_eq!(deleted.profiles.len(), 1);
    assert!(deleted.profiles[0].active);
    assert_eq!(
        secure_store_load("MUAPI_API_KEY".to_string()).expect("fallback key should load"),
        Some("muapi-secret-one".to_string())
    );

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn legacy_key_is_migrated_to_active_profile() {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let root = isolate_runtime_home("migrate");

    shorts_tauri_app::commands::runtime::secure_store_save(
        "OPENAI_API_KEY".to_string(),
        "openai-legacy-secret".to_string(),
    )
    .expect("legacy key should save");

    let profiles = api_key_profiles("openai".to_string()).expect("profiles should load");
    assert_eq!(profiles.profiles.len(), 1);
    assert_eq!(profiles.profiles[0].label, "Current OpenAI key");
    assert_eq!(profiles.profiles[0].last_four, "cret");
    assert!(profiles.profiles[0].active);

    let encoded = std::fs::read_to_string(
        root.join(".local")
            .join("share")
            .join("ai-youtube-shorts-generator")
            .join("config")
            .join("api-key-profiles.json"),
    )
    .expect("metadata should be written");
    assert!(!encoded.contains("openai-legacy-secret"));

    std::fs::remove_dir_all(root).ok();
}

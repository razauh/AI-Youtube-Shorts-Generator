use crate::runtime::python_runtime::{invoke_python, PythonInvokeRequest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

const APP_DIR_NAME: &str = "ai-youtube-shorts-generator";
const KEYCHAIN_SERVICE: &str = "ai-youtube-shorts-generator";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeContext {
    pub app_version: String,
    pub platform: String,
    pub runtime_root: String,
    pub log_dir: String,
    pub log_path: String,
    pub crash_log_path: String,
    pub config_path: String,
    pub protected_base_path: String,
    pub secure_fallback_base_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyProfileView {
    pub id: String,
    pub label: String,
    pub last_four: String,
    pub active: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyProfilesView {
    pub provider: String,
    pub env_override: bool,
    pub profiles: Vec<ApiKeyProfileView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelProfileView {
    pub id: String,
    pub label: String,
    pub model: String,
    pub device: String,
    pub active: bool,
    pub download_status: String,
    pub error: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelProfilesView {
    pub env_override: bool,
    pub active_profile_id: Option<String>,
    pub profiles: Vec<LocalModelProfileView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelDownloadStatus {
    pub active: bool,
    pub profile_id: Option<String>,
    pub model: Option<String>,
    pub device: Option<String>,
    pub phase: String,
    pub progress: f64,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyProfileRecord {
    id: String,
    provider: String,
    label: String,
    last_four: String,
    created_at_ms: u64,
    updated_at_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyProfileStore {
    profiles: Vec<ApiKeyProfileRecord>,
    active: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalModelProfileRecord {
    id: String,
    label: String,
    model: String,
    device: String,
    download_status: String,
    error: Option<String>,
    created_at_ms: u64,
    updated_at_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalModelProfileStore {
    profiles: Vec<LocalModelProfileRecord>,
    active: Option<String>,
}

#[derive(Debug)]
pub struct LocalModelDownloadState {
    current: Arc<Mutex<LocalModelDownloadStatus>>,
}

impl Default for LocalModelDownloadState {
    fn default() -> Self {
        Self {
            current: Arc::new(Mutex::new(default_download_status())),
        }
    }
}

fn home_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(v) = std::env::var("APPDATA") {
            return Ok(PathBuf::from(v));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(v) = std::env::var("HOME") {
            return Ok(PathBuf::from(v).join("Library").join("Application Support"));
        }
    }

    if let Ok(v) = std::env::var("HOME") {
        return Ok(PathBuf::from(v).join(".local").join("share"));
    }

    Err("unable to resolve runtime home directory".to_string())
}

fn runtime_root() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(APP_DIR_NAME))
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_runtime_dirs(root: &Path) -> Result<(), String> {
    for path in [
        root.to_path_buf(),
        root.join("logs"),
        root.join("protected"),
        root.join("secure-fallback"),
        root.join("config"),
    ] {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn canonicalize_within_runtime(input: &str) -> Result<PathBuf, String> {
    let root = runtime_root()?;
    ensure_runtime_dirs(&root)?;

    let requested = PathBuf::from(input);
    let candidate = if requested.is_absolute() {
        requested
    } else {
        root.join(requested)
    };

    ensure_parent(&candidate)?;
    let normalized = candidate
        .components()
        .fold(PathBuf::new(), |mut acc, component| {
            match component {
                std::path::Component::ParentDir => {
                    acc.pop();
                }
                std::path::Component::CurDir => {}
                _ => acc.push(component.as_os_str()),
            }
            acc
        });

    if !normalized.starts_with(&root) {
        return Err("path escapes runtime root".to_string());
    }

    Ok(normalized)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn generate_machine_secret() -> String {
    #[cfg(unix)]
    {
        if let Ok(mut file) = fs::File::open("/dev/urandom") {
            let mut out = [0_u8; 32];
            if file.read_exact(&mut out).is_ok() {
                return bytes_to_hex(&out);
            }
        }
    }

    let seed = format!(
        "{}:{}:{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        std::env::var("HOME").unwrap_or_default()
    );
    bytes_to_hex(&Sha256::digest(seed.as_bytes()))
}

fn machine_secret_path() -> Result<PathBuf, String> {
    Ok(runtime_root()?.join("config").join("machine-secret.txt"))
}

fn api_key_profiles_path() -> Result<PathBuf, String> {
    Ok(runtime_root()?.join("config").join("api-key-profiles.json"))
}

fn local_model_profiles_path() -> Result<PathBuf, String> {
    Ok(runtime_root()?
        .join("config")
        .join("local-model-profiles.json"))
}

fn local_model_cache_dir() -> Result<PathBuf, String> {
    let root = runtime_root()?;
    ensure_runtime_dirs(&root)?;
    Ok(root.join("models").join("huggingface"))
}

fn get_or_create_machine_secret() -> Result<String, String> {
    let path = machine_secret_path()?;
    ensure_parent(&path)?;
    if path.exists() {
        return fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| e.to_string());
    }
    let secret = generate_machine_secret();
    fs::write(&path, format!("{secret}\n")).map_err(|e| e.to_string())?;
    Ok(secret)
}

fn secure_fallback_path(key: &str) -> Result<PathBuf, String> {
    Ok(runtime_root()?
        .join("secure-fallback")
        .join(format!("{key}.secret")))
}

fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn normalize_api_provider(provider: &str) -> Result<&'static str, String> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "muapi" => Ok("muapi"),
        "openai" => Ok("openai"),
        _ => Err("unsupported API key provider".to_string()),
    }
}

fn legacy_api_key_name(provider: &str) -> Result<&'static str, String> {
    match normalize_api_provider(provider)? {
        "muapi" => Ok("MUAPI_API_KEY"),
        "openai" => Ok("OPENAI_API_KEY"),
        _ => Err("unsupported API key provider".to_string()),
    }
}

fn default_api_profile_label(provider: &str) -> Result<&'static str, String> {
    match normalize_api_provider(provider)? {
        "muapi" => Ok("Current MuAPI key"),
        "openai" => Ok("Current OpenAI key"),
        _ => Err("unsupported API key provider".to_string()),
    }
}

fn api_profile_secret_key(provider: &str, profile_id: &str) -> Result<String, String> {
    Ok(format!(
        "API_KEY_PROFILE_{}_{}",
        normalize_api_provider(provider)?.to_ascii_uppercase(),
        profile_id
    ))
}

fn key_last_four(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    chars.iter().skip(chars.len().saturating_sub(4)).collect()
}

fn create_api_profile_id(provider: &str, label: &str, key: &str, created_at_ms: u64) -> String {
    let seed = format!("{provider}:{label}:{key}:{created_at_ms}");
    bytes_to_hex(&Sha256::digest(seed.as_bytes()))[..16].to_string()
}

fn create_local_model_profile_id(
    label: &str,
    model: &str,
    device: &str,
    created_at_ms: u64,
) -> String {
    let seed = format!("local-model:{label}:{model}:{device}:{created_at_ms}");
    bytes_to_hex(&Sha256::digest(seed.as_bytes()))[..16].to_string()
}

fn load_api_profile_store() -> Result<ApiKeyProfileStore, String> {
    let path = api_key_profiles_path()?;
    if !path.exists() {
        return Ok(ApiKeyProfileStore::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_api_profile_store(store: &ApiKeyProfileStore) -> Result<(), String> {
    let path = api_key_profiles_path()?;
    ensure_parent(&path)?;
    let encoded = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, encoded).map_err(|e| e.to_string())
}

fn load_local_model_profile_store() -> Result<LocalModelProfileStore, String> {
    let path = local_model_profiles_path()?;
    if !path.exists() {
        return Ok(LocalModelProfileStore::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_local_model_profile_store(store: &LocalModelProfileStore) -> Result<(), String> {
    let path = local_model_profiles_path()?;
    ensure_parent(&path)?;
    let encoded = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, encoded).map_err(|e| e.to_string())
}

fn api_provider_has_profiles(store: &ApiKeyProfileStore, provider: &str) -> bool {
    store
        .profiles
        .iter()
        .any(|profile| profile.provider == provider)
}

fn view_api_profiles(provider: &str, store: &ApiKeyProfileStore) -> ApiKeyProfilesView {
    let active_id = store.active.get(provider);
    let env_override = legacy_api_key_name(provider)
        .ok()
        .and_then(|key| std::env::var(key).ok())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let mut profiles: Vec<ApiKeyProfileView> = store
        .profiles
        .iter()
        .filter(|profile| profile.provider == provider)
        .map(|profile| ApiKeyProfileView {
            id: profile.id.clone(),
            label: profile.label.clone(),
            last_four: profile.last_four.clone(),
            active: active_id == Some(&profile.id),
            created_at_ms: profile.created_at_ms,
            updated_at_ms: profile.updated_at_ms,
        })
        .collect();
    profiles.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));

    ApiKeyProfilesView {
        provider: provider.to_string(),
        env_override,
        profiles,
    }
}

fn mirror_active_api_profile(provider: &str, store: &ApiKeyProfileStore) -> Result<(), String> {
    let legacy_key = legacy_api_key_name(provider)?;
    if let Some(active_id) = store.active.get(provider) {
        let secret_key = api_profile_secret_key(provider, active_id)?;
        if let Some(value) = secure_store_load(secret_key)? {
            return secure_store_save(legacy_key.to_string(), value);
        }
    }
    secure_store_delete(legacy_key.to_string())
}

fn normalize_local_model(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("model is required".to_string());
    }
    if trimmed.len() > 120
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || trimmed.starts_with('/')
    {
        return Err("unsupported model name".to_string());
    }
    Ok(trimmed.to_string())
}

fn normalize_local_device(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok("auto".to_string()),
        "cpu" => Ok("cpu".to_string()),
        "cuda" => Ok("cuda".to_string()),
        _ => Err("unsupported processing device".to_string()),
    }
}

fn local_model_env_override() -> bool {
    ["LOCAL_WHISPER_MODEL", "LOCAL_WHISPER_DEVICE"]
        .iter()
        .any(|key| {
            std::env::var(key)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        })
}

fn mirror_active_local_model_profile(store: &LocalModelProfileStore) -> Result<(), String> {
    if local_model_env_override() {
        return Ok(());
    }

    if let Some(active_id) = store.active.as_ref() {
        if let Some(profile) = store
            .profiles
            .iter()
            .find(|profile| &profile.id == active_id)
        {
            secure_store_save("LOCAL_WHISPER_MODEL".to_string(), profile.model.clone())?;
            secure_store_save("LOCAL_WHISPER_DEVICE".to_string(), profile.device.clone())?;
            return Ok(());
        }
    }

    secure_store_delete("LOCAL_WHISPER_MODEL".to_string())?;
    secure_store_delete("LOCAL_WHISPER_DEVICE".to_string())
}

fn view_local_model_profiles(store: &LocalModelProfileStore) -> LocalModelProfilesView {
    let active_id = store.active.as_ref();
    let mut profiles: Vec<LocalModelProfileView> = store
        .profiles
        .iter()
        .map(|profile| LocalModelProfileView {
            id: profile.id.clone(),
            label: profile.label.clone(),
            model: profile.model.clone(),
            device: profile.device.clone(),
            active: active_id == Some(&profile.id),
            download_status: profile.download_status.clone(),
            error: profile.error.clone(),
            created_at_ms: profile.created_at_ms,
            updated_at_ms: profile.updated_at_ms,
        })
        .collect();
    profiles.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));

    LocalModelProfilesView {
        env_override: local_model_env_override(),
        active_profile_id: store.active.clone(),
        profiles,
    }
}

fn set_local_model_profile_download_state(
    profile_id: &str,
    status: &str,
    error: Option<String>,
) -> Result<(), String> {
    let mut store = load_local_model_profile_store()?;
    if let Some(profile) = store
        .profiles
        .iter_mut()
        .find(|profile| profile.id == profile_id)
    {
        profile.download_status = status.to_string();
        profile.error = error;
        profile.updated_at_ms = now_epoch_ms();
        save_local_model_profile_store(&store)?;
    }
    Ok(())
}

fn default_download_status() -> LocalModelDownloadStatus {
    LocalModelDownloadStatus {
        active: false,
        profile_id: None,
        model: None,
        device: None,
        phase: "idle".to_string(),
        progress: 0.0,
        message: "No local model download is running.".to_string(),
        error: None,
    }
}

fn safe_download_error(value: &str) -> String {
    let first_line = value
        .lines()
        .next()
        .unwrap_or("model download failed")
        .trim();
    if first_line.is_empty() {
        "model download failed".to_string()
    } else {
        let redacted = match std::env::var("HOME") {
            Ok(home) if !home.trim().is_empty() => first_line.replace(&home, "[home]"),
            _ => first_line.to_string(),
        };
        redacted.chars().take(220).collect()
    }
}

fn migrate_legacy_api_key(provider: &str, store: &mut ApiKeyProfileStore) -> Result<bool, String> {
    if api_provider_has_profiles(store, provider) {
        return Ok(false);
    }

    let legacy_key = legacy_api_key_name(provider)?;
    let Some(value) = secure_store_load(legacy_key.to_string())? else {
        return Ok(false);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }

    let now = now_epoch_ms();
    let label = default_api_profile_label(provider)?.to_string();
    let id = create_api_profile_id(provider, &label, trimmed, now);
    let secret_key = api_profile_secret_key(provider, &id)?;
    secure_store_save(secret_key, trimmed.to_string())?;
    store.profiles.push(ApiKeyProfileRecord {
        id: id.clone(),
        provider: provider.to_string(),
        label,
        last_four: key_last_four(trimmed),
        created_at_ms: now,
        updated_at_ms: now,
    });
    store.active.insert(provider.to_string(), id);
    Ok(true)
}

fn load_migrated_api_profile_store(provider: &str) -> Result<ApiKeyProfileStore, String> {
    let mut store = load_api_profile_store()?;
    if migrate_legacy_api_key(provider, &mut store)? {
        save_api_profile_store(&store)?;
    }
    Ok(store)
}

fn save_secure_fallback(key: &str, value: &str) -> Result<(), String> {
    let path = secure_fallback_path(key)?;
    ensure_parent(&path)?;
    fs::write(path, value).map_err(|e| e.to_string())
}

fn load_secure_fallback(key: &str) -> Result<Option<String>, String> {
    let path = secure_fallback_path(key)?;
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path)
        .map(|v| Some(v))
        .map_err(|e| e.to_string())
}

fn delete_secure_fallback(key: &str) -> Result<(), String> {
    let path = secure_fallback_path(key)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn keychain_save(key: &str, value: &str) -> Result<(), String> {
    let mut child = Command::new("secret-tool")
        .args([
            "store",
            "--label",
            KEYCHAIN_SERVICE,
            "service",
            KEYCHAIN_SERVICE,
            "account",
            key,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(value.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(target_os = "linux")]
fn keychain_load(key: &str) -> Result<Option<String>, String> {
    let output = Command::new("secret-tool")
        .args(["lookup", "service", KEYCHAIN_SERVICE, "account", key])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout)
                .trim_end_matches('\n')
                .to_string(),
        ))
    } else {
        Ok(None)
    }
}

#[cfg(target_os = "linux")]
fn keychain_delete(key: &str) -> Result<(), String> {
    let output = Command::new("secret-tool")
        .args(["clear", "service", KEYCHAIN_SERVICE, "account", key])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(target_os = "macos")]
fn keychain_save(key: &str, value: &str) -> Result<(), String> {
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-a",
            key,
            "-s",
            KEYCHAIN_SERVICE,
            "-U",
            "-w",
            value,
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(target_os = "macos")]
fn keychain_load(key: &str) -> Result<Option<String>, String> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-a",
            key,
            "-s",
            KEYCHAIN_SERVICE,
            "-w",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout)
                .trim_end_matches('\n')
                .to_string(),
        ))
    } else {
        Ok(None)
    }
}

#[cfg(target_os = "macos")]
fn keychain_delete(key: &str) -> Result<(), String> {
    let output = Command::new("security")
        .args(["delete-generic-password", "-a", key, "-s", KEYCHAIN_SERVICE])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(target_os = "windows")]
fn keychain_save(key: &str, value: &str) -> Result<(), String> {
    use std::ffi::c_void;
    use std::ptr;

    #[repr(C)]
    struct FileTime {
        dw_low_date_time: u32,
        dw_high_date_time: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        r#type: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "Advapi32")]
    unsafe extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;

    let target = wide(&format!("{KEYCHAIN_SERVICE}:{key}"));
    let user_name = wide(KEYCHAIN_SERVICE);
    let mut blob = value.as_bytes().to_vec();

    let credential = CredentialW {
        flags: 0,
        r#type: CRED_TYPE_GENERIC,
        target_name: target.as_ptr() as *mut u16,
        comment: ptr::null_mut(),
        last_written: FileTime {
            dw_low_date_time: 0,
            dw_high_date_time: 0,
        },
        credential_blob_size: blob.len() as u32,
        credential_blob: blob.as_mut_ptr(),
        persist: CRED_PERSIST_LOCAL_MACHINE,
        attribute_count: 0,
        attributes: ptr::null_mut(),
        target_alias: ptr::null_mut(),
        user_name: user_name.as_ptr() as *mut u16,
    };

    // SAFETY: We pass a valid, initialized CREDENTIALW with stable pointers for the call duration.
    let wrote = unsafe { CredWriteW(&credential, 0) };
    if wrote == 0 {
        return Err("CredWriteW failed".to_string());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn keychain_load(key: &str) -> Result<Option<String>, String> {
    use std::ffi::c_void;
    use std::ptr;
    use std::slice;

    #[repr(C)]
    struct FileTime {
        dw_low_date_time: u32,
        dw_high_date_time: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        r#type: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "Advapi32")]
    unsafe extern "system" {
        fn CredReadW(
            target_name: *const u16,
            r#type: u32,
            flags: u32,
            credential: *mut *mut CredentialW,
        ) -> i32;
        fn CredFree(buffer: *mut c_void);
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    const CRED_TYPE_GENERIC: u32 = 1;
    let target = wide(&format!("{KEYCHAIN_SERVICE}:{key}"));
    let mut credential: *mut CredentialW = ptr::null_mut();

    // SAFETY: Windows API initializes `credential` on success. Inputs are valid NUL-terminated UTF-16.
    let read = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    if read == 0 || credential.is_null() {
        return Ok(None);
    }

    // SAFETY: `credential` is valid for reads until freed via CredFree.
    let value = unsafe {
        let blob_ptr = (*credential).credential_blob;
        let blob_len = (*credential).credential_blob_size as usize;
        let bytes = slice::from_raw_parts(blob_ptr, blob_len);
        String::from_utf8_lossy(bytes).to_string()
    };

    // SAFETY: Pointer was returned by CredReadW and must be released by CredFree.
    unsafe { CredFree(credential as *mut c_void) };
    Ok(Some(value))
}

#[cfg(target_os = "windows")]
fn keychain_delete(key: &str) -> Result<(), String> {
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[link(name = "Advapi32")]
    unsafe extern "system" {
        fn CredDeleteW(target_name: *const u16, r#type: u32, flags: u32) -> i32;
    }

    const CRED_TYPE_GENERIC: u32 = 1;
    let target = wide(&format!("{KEYCHAIN_SERVICE}:{key}"));
    // SAFETY: We pass a valid NUL-terminated UTF-16 target name to the Win32 API.
    let deleted = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
    if deleted == 0 {
        return Ok(());
    }
    Ok(())
}

#[tauri::command]
pub fn runtime_context() -> Result<RuntimeContext, String> {
    let root = runtime_root()?;
    ensure_runtime_dirs(&root)?;
    let _ = get_or_create_machine_secret()?;

    Ok(RuntimeContext {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        runtime_root: root.display().to_string(),
        log_dir: root.join("logs").display().to_string(),
        log_path: root.join("logs").join("app.log").display().to_string(),
        crash_log_path: root.join("logs").join("crash.log").display().to_string(),
        config_path: root
            .join("config")
            .join("config.json")
            .display()
            .to_string(),
        protected_base_path: root.join("protected").display().to_string(),
        secure_fallback_base_path: root.join("secure-fallback").display().to_string(),
    })
}

#[tauri::command]
pub fn runtime_machine_secret() -> Result<String, String> {
    get_or_create_machine_secret()
}

#[tauri::command]
pub fn runtime_fs_read_text(path: String) -> Result<Option<String>, String> {
    let path = canonicalize_within_runtime(&path)?;
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn runtime_fs_write_text(path: String, value: String) -> Result<(), String> {
    let path = canonicalize_within_runtime(&path)?;
    ensure_parent(&path)?;
    fs::write(path, value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn runtime_fs_append_line(path: String, line: String) -> Result<(), String> {
    let path = canonicalize_within_runtime(&path)?;
    ensure_parent(&path)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn runtime_fs_remove(path: String) -> Result<(), String> {
    let path = canonicalize_within_runtime(&path)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn runtime_fs_exists(path: String) -> Result<bool, String> {
    let path = canonicalize_within_runtime(&path)?;
    Ok(path.exists())
}

#[tauri::command]
pub fn runtime_fs_list(prefix: String) -> Result<Vec<String>, String> {
    let prefix = canonicalize_within_runtime(&prefix)?;
    if !prefix.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(prefix).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        out.push(entry.path().display().to_string());
    }
    Ok(out)
}

#[tauri::command]
pub fn runtime_fs_rename(from: String, to: String) -> Result<(), String> {
    let from = canonicalize_within_runtime(&from)?;
    let to = canonicalize_within_runtime(&to)?;
    ensure_parent(&to)?;
    fs::rename(from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn runtime_fs_chmod_readonly(path: String) -> Result<(), String> {
    let path = canonicalize_within_runtime(&path)?;
    let mut perms = fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .permissions();
    perms.set_readonly(true);
    fs::set_permissions(path, perms).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn runtime_fs_size(path: String) -> Result<u64, String> {
    let path = canonicalize_within_runtime(&path)?;
    if !path.exists() {
        return Ok(0);
    }
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn secure_store_save(key: String, value: String) -> Result<(), String> {
    match keychain_save(&key, &value) {
        Ok(()) => Ok(()),
        Err(_) => save_secure_fallback(&key, &value),
    }
}

#[tauri::command]
pub fn secure_store_load(key: String) -> Result<Option<String>, String> {
    match keychain_load(&key) {
        Ok(Some(value)) => Ok(Some(value)),
        Ok(None) | Err(_) => load_secure_fallback(&key),
    }
}

#[tauri::command]
pub fn secure_store_delete(key: String) -> Result<(), String> {
    let _ = keychain_delete(&key);
    delete_secure_fallback(&key)
}

#[tauri::command]
pub fn secure_store_exists(key: String) -> Result<bool, String> {
    Ok(secure_store_load(key)?.is_some())
}

#[tauri::command]
pub fn api_key_profiles(provider: String) -> Result<ApiKeyProfilesView, String> {
    let provider = normalize_api_provider(&provider)?;
    let store = load_migrated_api_profile_store(provider)?;
    Ok(view_api_profiles(provider, &store))
}

#[tauri::command]
pub fn api_key_profile_add(
    provider: String,
    label: String,
    key: String,
    activate: bool,
) -> Result<ApiKeyProfilesView, String> {
    let provider = normalize_api_provider(&provider)?;
    let label = label.trim();
    let key = key.trim();
    if label.is_empty() {
        return Err("profile name is required".to_string());
    }
    if key.is_empty() {
        return Err("API key is required".to_string());
    }

    let mut store = load_migrated_api_profile_store(provider)?;
    let now = now_epoch_ms();
    let id = create_api_profile_id(provider, label, key, now);
    let secret_key = api_profile_secret_key(provider, &id)?;
    secure_store_save(secret_key, key.to_string())?;

    let had_active = store.active.contains_key(provider);
    store.profiles.push(ApiKeyProfileRecord {
        id: id.clone(),
        provider: provider.to_string(),
        label: label.to_string(),
        last_four: key_last_four(key),
        created_at_ms: now,
        updated_at_ms: now,
    });
    if activate || !had_active {
        store.active.insert(provider.to_string(), id);
    }
    save_api_profile_store(&store)?;
    mirror_active_api_profile(provider, &store)?;
    Ok(view_api_profiles(provider, &store))
}

#[tauri::command]
pub fn api_key_profile_activate(
    provider: String,
    profile_id: String,
) -> Result<ApiKeyProfilesView, String> {
    let provider = normalize_api_provider(&provider)?;
    let profile_id = profile_id.trim();
    let mut store = load_migrated_api_profile_store(provider)?;
    let exists = store
        .profiles
        .iter()
        .any(|profile| profile.provider == provider && profile.id == profile_id);
    if !exists {
        return Err("API key profile not found".to_string());
    }

    store
        .active
        .insert(provider.to_string(), profile_id.to_string());
    save_api_profile_store(&store)?;
    mirror_active_api_profile(provider, &store)?;
    Ok(view_api_profiles(provider, &store))
}

#[tauri::command]
pub fn api_key_profile_delete(
    provider: String,
    profile_id: String,
) -> Result<ApiKeyProfilesView, String> {
    let provider = normalize_api_provider(&provider)?;
    let profile_id = profile_id.trim();
    let mut store = load_migrated_api_profile_store(provider)?;
    let before_len = store.profiles.len();
    store
        .profiles
        .retain(|profile| !(profile.provider == provider && profile.id == profile_id));
    if store.profiles.len() == before_len {
        return Err("API key profile not found".to_string());
    }

    let secret_key = api_profile_secret_key(provider, profile_id)?;
    secure_store_delete(secret_key)?;

    if store.active.get(provider).map(String::as_str) == Some(profile_id) {
        if let Some(next) = store
            .profiles
            .iter()
            .filter(|profile| profile.provider == provider)
            .max_by_key(|profile| profile.created_at_ms)
        {
            store.active.insert(provider.to_string(), next.id.clone());
        } else {
            store.active.remove(provider);
        }
    }

    save_api_profile_store(&store)?;
    mirror_active_api_profile(provider, &store)?;
    Ok(view_api_profiles(provider, &store))
}

#[tauri::command]
pub fn local_model_profiles() -> Result<LocalModelProfilesView, String> {
    let store = load_local_model_profile_store()?;
    Ok(view_local_model_profiles(&store))
}

#[tauri::command]
pub fn local_model_download_status(
    state: tauri::State<'_, LocalModelDownloadState>,
) -> Result<LocalModelDownloadStatus, String> {
    Ok(state
        .current
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|_| default_download_status()))
}

#[tauri::command]
pub fn local_model_profile_add(
    app: tauri::AppHandle,
    state: tauri::State<'_, LocalModelDownloadState>,
    label: String,
    model: String,
    device: String,
    activate: bool,
) -> Result<LocalModelProfilesView, String> {
    let label = label.trim();
    if label.is_empty() {
        return Err("profile name is required".to_string());
    }
    let model = normalize_local_model(&model)?;
    let device = normalize_local_device(&device)?;

    let mut store = load_local_model_profile_store()?;
    let now = now_epoch_ms();
    let id = create_local_model_profile_id(label, &model, &device, now);
    let had_active = store.active.is_some();
    store.profiles.push(LocalModelProfileRecord {
        id: id.clone(),
        label: label.to_string(),
        model: model.clone(),
        device: device.clone(),
        download_status: "queued".to_string(),
        error: None,
        created_at_ms: now,
        updated_at_ms: now,
    });
    if activate || !had_active {
        store.active = Some(id.clone());
    }
    save_local_model_profile_store(&store)?;
    mirror_active_local_model_profile(&store)?;
    start_local_model_download(app, &state, id, model, device)?;
    Ok(view_local_model_profiles(&load_local_model_profile_store()?))
}

#[tauri::command]
pub fn local_model_profile_activate(
    app: tauri::AppHandle,
    state: tauri::State<'_, LocalModelDownloadState>,
    profile_id: String,
) -> Result<LocalModelProfilesView, String> {
    let profile_id = profile_id.trim();
    let mut store = load_local_model_profile_store()?;
    let profile = store
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
        .ok_or_else(|| "local model profile not found".to_string())?;

    store.active = Some(profile.id.clone());
    save_local_model_profile_store(&store)?;
    mirror_active_local_model_profile(&store)?;
    if profile.download_status != "ready" {
        start_local_model_download(app, &state, profile.id, profile.model, profile.device)?;
    }
    Ok(view_local_model_profiles(&load_local_model_profile_store()?))
}

#[tauri::command]
pub fn local_model_profile_delete(profile_id: String) -> Result<LocalModelProfilesView, String> {
    let profile_id = profile_id.trim();
    let mut store = load_local_model_profile_store()?;
    let before_len = store.profiles.len();
    store.profiles.retain(|profile| profile.id != profile_id);
    if store.profiles.len() == before_len {
        return Err("local model profile not found".to_string());
    }

    if store.active.as_deref() == Some(profile_id) {
        store.active = store
            .profiles
            .iter()
            .max_by_key(|profile| profile.created_at_ms)
            .map(|profile| profile.id.clone());
    }
    save_local_model_profile_store(&store)?;
    mirror_active_local_model_profile(&store)?;
    Ok(view_local_model_profiles(&store))
}

#[tauri::command]
pub fn local_model_profile_retry_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, LocalModelDownloadState>,
    profile_id: String,
) -> Result<LocalModelProfilesView, String> {
    let profile_id = profile_id.trim();
    let store = load_local_model_profile_store()?;
    let profile = store
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
        .ok_or_else(|| "local model profile not found".to_string())?;
    start_local_model_download(app, &state, profile.id, profile.model, profile.device)?;
    Ok(view_local_model_profiles(&load_local_model_profile_store()?))
}

fn publish_download_status(
    app: &tauri::AppHandle,
    state: &LocalModelDownloadState,
    next: LocalModelDownloadStatus,
) {
    if let Ok(mut current) = state.current.lock() {
        *current = next.clone();
    }
    let _ = app.emit("local-model-download-progress", next);
}

fn start_local_model_download(
    app: tauri::AppHandle,
    state: &LocalModelDownloadState,
    profile_id: String,
    model: String,
    device: String,
) -> Result<(), String> {
    let initial = LocalModelDownloadStatus {
        active: true,
        profile_id: Some(profile_id.clone()),
        model: Some(model.clone()),
        device: Some(device.clone()),
        phase: "checking".to_string(),
        progress: 0.05,
        message: format!("Checking local model {model}..."),
        error: None,
    };

    {
        let mut current = state
            .current
            .lock()
            .map_err(|_| "download state is unavailable".to_string())?;
        if current.active {
            if current.profile_id.as_deref() == Some(profile_id.as_str()) {
                return Ok(());
            }
            return Err("another local model download is already running".to_string());
        }
        *current = initial.clone();
    }

    let _ = set_local_model_profile_download_state(&profile_id, "downloading", None);
    let _ = app.emit("local-model-download-progress", initial);

    let state_for_thread = LocalModelDownloadState {
        current: Arc::clone(&state.current),
    };
    std::thread::spawn(move || {
        publish_download_status(
            &app,
            &state_for_thread,
            LocalModelDownloadStatus {
                active: true,
                profile_id: Some(profile_id.clone()),
                model: Some(model.clone()),
                device: Some(device.clone()),
                phase: "downloading".to_string(),
                progress: 0.35,
                message: format!("Downloading local model {model}..."),
                error: None,
            },
        );

        let result = prefetch_local_model(&model, &device);
        match result {
            Ok(()) => {
                let _ = set_local_model_profile_download_state(&profile_id, "ready", None);
                publish_download_status(
                    &app,
                    &state_for_thread,
                    LocalModelDownloadStatus {
                        active: true,
                        profile_id: Some(profile_id.clone()),
                        model: Some(model.clone()),
                        device: Some(device.clone()),
                        phase: "verifying".to_string(),
                        progress: 0.92,
                        message: format!("Verifying local model {model}..."),
                        error: None,
                    },
                );
                publish_download_status(
                    &app,
                    &state_for_thread,
                    LocalModelDownloadStatus {
                        active: false,
                        profile_id: Some(profile_id),
                        model: Some(model.clone()),
                        device: Some(device),
                        phase: "ready".to_string(),
                        progress: 1.0,
                        message: format!("Local model {model} is ready."),
                        error: None,
                    },
                );
            }
            Err(err) => {
                let safe = safe_download_error(&err);
                let _ = set_local_model_profile_download_state(
                    &profile_id,
                    "failed",
                    Some(safe.clone()),
                );
                publish_download_status(
                    &app,
                    &state_for_thread,
                    LocalModelDownloadStatus {
                        active: false,
                        profile_id: Some(profile_id),
                        model: Some(model),
                        device: Some(device),
                        phase: "failed".to_string(),
                        progress: 1.0,
                        message: "Local model download failed.".to_string(),
                        error: Some(safe),
                    },
                );
            }
        }
    });

    Ok(())
}

fn prefetch_local_model(model: &str, device: &str) -> Result<(), String> {
    let cache_dir = local_model_cache_dir()?;
    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    let req = serde_json::json!({
        "version": "1",
        "action": "prefetch_local_model",
        "payload": {
            "model": model,
            "device": device,
            "cache_dir": cache_dir.display().to_string()
        }
    });
    let stdin_json = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
    let env = vec![
        (
            "LOCAL_MODEL_CACHE_DIR".to_string(),
            cache_dir.display().to_string(),
        ),
        ("LOCAL_WHISPER_MODEL".to_string(), model.to_string()),
        ("LOCAL_WHISPER_DEVICE".to_string(), device.to_string()),
    ];
    let proc = invoke_python(PythonInvokeRequest {
        python_bin: std::env::var("PYTHON_BRIDGE_BIN").unwrap_or_else(|_| "python3".to_string()),
        entry_script: std::env::var("PYTHON_BRIDGE_ENTRY")
            .unwrap_or_else(|_| "../../python_legacy/bridge_entry.py".to_string()),
        env,
        stdin_json,
        timeout_ms: std::env::var("PYTHON_BRIDGE_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300_000),
    })
    .map_err(|e| format!("python invoke failed: {e:?}"))?;

    if proc.timed_out {
        return Err("python bridge timed out while downloading local model".to_string());
    }
    if proc.status_code.unwrap_or(1) != 0 {
        return Err(if proc.stderr.trim().is_empty() {
            "python bridge exited non-zero while downloading local model".to_string()
        } else {
            proc.stderr
        });
    }
    let parsed: serde_json::Value = serde_json::from_str(proc.stdout.trim())
        .map_err(|e| format!("invalid bridge json: {e}"))?;
    if parsed.get("ok").and_then(|value| value.as_bool()) == Some(true) {
        Ok(())
    } else {
        Err(parsed
            .get("error")
            .and_then(|value| value.get("message"))
            .and_then(|value| value.as_str())
            .unwrap_or("python bridge returned error while downloading local model")
            .to_string())
    }
}

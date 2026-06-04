use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
        .map(Some)
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

    let read = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    if read == 0 || credential.is_null() {
        return Ok(None);
    }

    let value = unsafe {
        let blob_ptr = (*credential).credential_blob;
        let blob_len = (*credential).credential_blob_size as usize;
        let bytes = slice::from_raw_parts(blob_ptr, blob_len);
        String::from_utf8_lossy(bytes).to_string()
    };

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

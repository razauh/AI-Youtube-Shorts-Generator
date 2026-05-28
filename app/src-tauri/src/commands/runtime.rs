use crate::runtime::python_runtime::{
    invoke_python, resolve_python_bridge_paths, with_python_runtime_env, PythonInvokeRequest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
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
    pub error_code: Option<String>,
    pub debug_ref: Option<String>,
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
    pub error_code: Option<String>,
    pub debug_ref: Option<String>,
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
    error_code: Option<String>,
    debug_ref: Option<String>,
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
    cancel_token: Arc<Mutex<Option<Arc<AtomicBool>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePackStatusKind {
    NotInstalled,
    Downloading,
    Installing,
    Installed,
    Ready,
    Failed,
    Corrupted,
    IncompatiblePlatform,
    MissingFiles,
    MissingDependency,
    NativeImportFailure,
    PermissionError,
    NetworkError,
    ValidationFailed,
    UnknownError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalRuntimePackStatus {
    pub status: RuntimePackStatusKind,
    pub version: Option<String>,
    pub platform: String,
    pub arch: String,
    pub install_dir: String,
    pub manifest_url: String,
    pub required_size_bytes: Option<u64>,
    pub message: String,
    pub error_code: Option<String>,
    pub debug_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePackProgressEvent {
    pub phase: String,
    pub progress: f64,
    pub message: String,
    pub status: RuntimePackStatusKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalRuntimePackStateStore {
    status: Option<RuntimePackStatusKind>,
    version: Option<String>,
    installed_at_ms: Option<u64>,
    last_error_code: Option<String>,
    last_debug_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePackManifest {
    version: String,
    app_compatibility: Option<String>,
    assets: Vec<RuntimePackAsset>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePackAsset {
    platform: String,
    arch: String,
    filename: String,
    url: String,
    size: Option<u64>,
    sha256: String,
    required_modules: Option<Vec<String>>,
}

impl Default for LocalModelDownloadState {
    fn default() -> Self {
        Self {
            current: Arc::new(Mutex::new(default_download_status())),
            cancel_token: Arc::new(Mutex::new(None)),
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

fn local_runtime_pack_root() -> Result<PathBuf, String> {
    let root = runtime_root()?;
    ensure_runtime_dirs(&root)?;
    Ok(root.join("runtime-pack"))
}

fn local_runtime_pack_current_dir() -> Result<PathBuf, String> {
    Ok(local_runtime_pack_root()?.join("current"))
}

fn local_runtime_pack_state_path() -> Result<PathBuf, String> {
    Ok(runtime_root()?.join("config").join("local-runtime-pack-state.json"))
}

fn local_runtime_pack_manifest_url() -> Option<String> {
    std::env::var("LOCAL_RUNTIME_PACK_MANIFEST_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn local_runtime_pack_manifest_local_path() -> Result<PathBuf, String> {
    Ok(local_runtime_pack_root()?.join("manifest.json"))
}

fn load_runtime_pack_state_store() -> Result<LocalRuntimePackStateStore, String> {
    let path = local_runtime_pack_state_path()?;
    if !path.exists() {
        return Ok(LocalRuntimePackStateStore::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_runtime_pack_state_store(store: &LocalRuntimePackStateStore) -> Result<(), String> {
    let path = local_runtime_pack_state_path()?;
    ensure_parent(&path)?;
    let encoded = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, encoded).map_err(|e| e.to_string())
}

fn runtime_pack_platform() -> String {
    std::env::consts::OS.to_string()
}

fn runtime_pack_arch() -> String {
    std::env::consts::ARCH.to_string()
}

fn resolve_runtime_pack_asset<'a>(
    manifest: &'a RuntimePackManifest,
) -> Result<&'a RuntimePackAsset, String> {
    let platform = runtime_pack_platform();
    let arch = runtime_pack_arch();
    manifest
        .assets
        .iter()
        .find(|asset| asset.platform == platform && asset.arch == arch)
        .ok_or_else(|| "incompatible_platform".to_string())
}

fn runtime_pack_bridge_paths(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let python = if cfg!(windows) {
        root.join("python.exe")
    } else {
        root.join("python3")
    };
    let entry = root.join("python_legacy").join("bridge_entry.py");
    let site_packages = root.join("site-packages");
    (python, entry, site_packages)
}

fn validate_runtime_pack_install(root: &Path, required_modules: &[String]) -> Result<(), String> {
    let (python, entry, _site_packages) = runtime_pack_bridge_paths(root);
    if !python.exists() || !entry.exists() {
        return Err("missing_files".to_string());
    }
    let mut imports = String::new();
    for module in required_modules {
        imports.push_str(&format!("import {module}\n"));
    }
    let snippet = format!("{imports}print('ok')");
    let output = Command::new(&python)
        .args(["-c", snippet.as_str()])
        .output()
        .map_err(|_| "validation_failed".to_string())?;
    if !output.status.success() {
        return Err("validation_failed".to_string());
    }
    Ok(())
}

fn emit_runtime_pack_progress(
    app: &tauri::AppHandle,
    phase: &str,
    progress: f64,
    message: &str,
    status: RuntimePackStatusKind,
) {
    let _ = app.emit(
        "local-runtime-pack-progress",
        RuntimePackProgressEvent {
            phase: phase.to_string(),
            progress,
            message: message.to_string(),
            status,
        },
    );
}

fn sanitize_runtime_error(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "runtime operation failed".to_string();
    }
    let mut output = trimmed.to_string();
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            output = output.replace(&home, "[home]");
        }
    }
    output.chars().take(2000).collect()
}

fn manifest_url_host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn sanitize_body_preview(body: &str) -> String {
    let collapsed = body.replace('\n', " ").replace('\r', " ");
    sanitize_runtime_error(collapsed.trim())
        .chars()
        .take(200)
        .collect()
}

fn append_runtime_setup_log(debug_ref: &str, stage: &str, category: &str, message: &str) {
    let line = format!(
        "[{}] [{}] local_runtime_setup stage={} category={} message=\"{}\"",
        now_epoch_ms(),
        debug_ref,
        stage,
        category,
        sanitize_runtime_error(message).replace('\n', "\\n")
    );
    let _ = runtime_fs_append_line("logs/app.log".to_string(), line);
}

fn log_runtime_setup(stage: &str, category: &str, message: &str) {
    eprintln!(
        "local_runtime_setup stage={} category={} message=\"{}\" timestamp_ms={}",
        stage,
        category,
        sanitize_runtime_error(message).replace('\n', "\\n"),
        now_epoch_ms()
    );
}

fn log_runtime_setup_manifest(
    stage: &str,
    category: &str,
    manifest_source: &str,
    manifest_url: &str,
    http_status: Option<u16>,
    content_type: Option<&str>,
    body_preview: Option<&str>,
    parse_error: Option<&str>,
    schema_error: Option<&str>,
) {
    eprintln!(
        "local_runtime_setup stage={} category={} manifest_source={} manifest_url_host={} http_status={} content_type={} body_preview=\"{}\" parse_error=\"{}\" schema_error=\"{}\" platform={} app_version={} timestamp_ms={}",
        stage,
        category,
        manifest_source,
        manifest_url_host(manifest_url),
        http_status.map(|v| v.to_string()).unwrap_or_else(|| "not_applicable".to_string()),
        content_type.unwrap_or("unknown"),
        body_preview.map(sanitize_body_preview).unwrap_or_default(),
        parse_error.map(sanitize_runtime_error).unwrap_or_default(),
        schema_error.map(sanitize_runtime_error).unwrap_or_default(),
        format!("{}-{}", runtime_pack_platform(), runtime_pack_arch()),
        env!("CARGO_PKG_VERSION"),
        now_epoch_ms()
    );
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(bytes_to_hex(&hasher.finalize()))
}

fn extract_runtime_pack_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Expand-Archive",
                "-Force",
                "-Path",
                &zip_path.display().to_string(),
                "-DestinationPath",
                &target_dir.display().to_string(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("failed to extract runtime pack".to_string());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status = Command::new("unzip")
            .args([
                "-q",
                &zip_path.display().to_string(),
                "-d",
                &target_dir.display().to_string(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("failed to extract runtime pack".to_string());
        }
    }
    Ok(())
}

fn read_runtime_pack_status() -> Result<LocalRuntimePackStatus, String> {
    let store = load_runtime_pack_state_store()?;
    let install_dir = local_runtime_pack_current_dir()?;
    let manifest_url = local_runtime_pack_manifest_url()
        .unwrap_or_else(|| "local://runtime-pack/manifest.json".to_string());

    let bridge = resolve_python_bridge_paths();
    if let Some(dir) = bridge.bundled_runtime_dir {
        let required_modules = vec!["faster_whisper".to_string()];
        if validate_runtime_pack_install(&dir, &required_modules).is_ok() {
            return Ok(LocalRuntimePackStatus {
                status: RuntimePackStatusKind::Ready,
                version: store.version,
                platform: runtime_pack_platform(),
                arch: runtime_pack_arch(),
                install_dir: dir.display().to_string(),
                manifest_url,
                required_size_bytes: None,
                message: "Local processing runtime is ready.".to_string(),
                error_code: None,
                debug_ref: None,
            });
        }
    }

    let status = store.status.unwrap_or(RuntimePackStatusKind::NotInstalled);
    let message = match status {
        RuntimePackStatusKind::NotInstalled => "Local processing runtime is not installed.",
        RuntimePackStatusKind::Downloading => "Local processing runtime is downloading.",
        RuntimePackStatusKind::Installing => "Local processing runtime is installing.",
        RuntimePackStatusKind::Installed => "Local processing runtime is installed.",
        RuntimePackStatusKind::Ready => "Local processing runtime is ready.",
        RuntimePackStatusKind::Failed => "Local processing runtime failed.",
        RuntimePackStatusKind::Corrupted => "Local processing runtime archive is corrupted.",
        RuntimePackStatusKind::IncompatiblePlatform => {
            "No compatible local processing runtime is available for this platform."
        }
        RuntimePackStatusKind::MissingFiles => "Local processing runtime is missing required files.",
        RuntimePackStatusKind::MissingDependency => {
            "Local processing runtime is missing required dependencies."
        }
        RuntimePackStatusKind::NativeImportFailure => {
            "Local processing runtime failed to load a native dependency."
        }
        RuntimePackStatusKind::PermissionError => {
            "Local processing runtime does not have required permissions."
        }
        RuntimePackStatusKind::NetworkError => "Could not download local processing runtime.",
        RuntimePackStatusKind::ValidationFailed => {
            "Local processing runtime validation failed."
        }
        RuntimePackStatusKind::UnknownError => "Local processing runtime failed unexpectedly.",
    };
    Ok(LocalRuntimePackStatus {
        status,
        version: store.version,
        platform: runtime_pack_platform(),
        arch: runtime_pack_arch(),
        install_dir: install_dir.display().to_string(),
        manifest_url,
        required_size_bytes: None,
        message: message.to_string(),
        error_code: store.last_error_code,
        debug_ref: store.last_debug_ref,
    })
}

pub fn is_local_runtime_pack_ready() -> bool {
    read_runtime_pack_status()
        .map(|status| matches!(status.status, RuntimePackStatusKind::Ready))
        .unwrap_or(false)
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
            error_code: profile.error_code.clone(),
            debug_ref: profile.debug_ref.clone(),
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
    error_code: Option<String>,
    debug_ref: Option<String>,
) -> Result<(), String> {
    let mut store = load_local_model_profile_store()?;
    if let Some(profile) = store
        .profiles
        .iter_mut()
        .find(|profile| profile.id == profile_id)
    {
        profile.download_status = status.to_string();
        profile.error = error;
        profile.error_code = error_code;
        profile.debug_ref = debug_ref;
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
        error_code: None,
        debug_ref: None,
    }
}

#[derive(Debug, Clone)]
struct LocalModelErrorSummary {
    code: String,
    user_message: String,
}

fn classify_local_model_download_error(raw: &str) -> LocalModelErrorSummary {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("timed out") {
        return LocalModelErrorSummary {
            code: "timeout".to_string(),
            user_message: "The model download timed out. Please try again.".to_string(),
        };
    }
    if lower.contains("runtime_pack_setup_required") {
        return LocalModelErrorSummary {
            code: "runtime_pack_setup_required".to_string(),
            user_message: "Local processing setup failed. Please try again.".to_string(),
        };
    }
    if lower.contains("runtime pack setup failed")
        || lower.contains("runtime pack validation failed")
    {
        if lower.contains("network_error") {
            return LocalModelErrorSummary {
                code: "network_error".to_string(),
                user_message: "Could not download the required local processing components. Check your internet connection and try again.".to_string(),
            };
        }
        if lower.contains("permission_error") {
            return LocalModelErrorSummary {
                code: "permission_error".to_string(),
                user_message:
                    "Local processing setup failed because the app could not write required files."
                        .to_string(),
            };
        }
        return LocalModelErrorSummary {
            code: "validation_failed".to_string(),
            user_message: "The local processing setup appears incomplete. Please retry setup."
                .to_string(),
        };
    }
    if lower.contains("runtime pack manifest fetch failed")
        || lower.contains("runtime pack download failed")
    {
        return LocalModelErrorSummary {
            code: "network_error".to_string(),
            user_message: "Could not download the required local processing components. Check your internet connection and try again.".to_string(),
        };
    }
    if lower.contains("runtime pack manifest is empty")
        || lower.contains("runtime pack manifest is not json")
        || lower.contains("runtime pack manifest parse failed")
        || lower.contains("runtime pack manifest schema failed")
        || lower.contains("runtime pack manifest read failed")
        || lower.contains("runtime pack manifest url not configured")
    {
        return LocalModelErrorSummary {
            code: "runtime_manifest_parse_error".to_string(),
            user_message:
                "Could not read the local processing setup manifest. Please try again later."
                    .to_string(),
        };
    }
    if lower.contains("failed to install python dependency") {
        return LocalModelErrorSummary {
            code: "dependency_install_failed".to_string(),
            user_message:
                "Local setup could not download a required Python component. Check your internet connection and try again."
                    .to_string(),
        };
    }
    if lower.contains("faster-whisper is required")
        || lower.contains("no module named 'faster_whisper'")
        || lower.contains("no module named faster_whisper")
        || lower.contains("module not found: faster_whisper")
        || lower.contains("no module named 'requests'")
        || lower.contains("no module named requests")
        || lower.contains("no module named 'yt_dlp'")
        || lower.contains("no module named yt_dlp")
        || lower.contains("no module named 'openai'")
        || lower.contains("no module named openai")
        || lower.contains("no module named 'cv2'")
        || lower.contains("no module named cv2")
    {
        return LocalModelErrorSummary {
            code: "missing_dependency".to_string(),
            user_message:
                "Local model setup is incomplete. The app could not find a required Python package."
                .to_string(),
        };
    }
    if lower.contains("failed to import")
        || lower.contains("could not be loaded")
        || lower.contains("cannot open shared object file")
        || lower.contains("undefined symbol")
        || lower.contains("importerror")
    {
        return LocalModelErrorSummary {
            code: "dependency_load_failed".to_string(),
            user_message: "Local model setup failed because one required Python dependency could not load. Open logs for technical details.".to_string(),
        };
    }
    if lower.contains("unsupported model name") || lower.contains("model is required") {
        return LocalModelErrorSummary {
            code: "invalid_model".to_string(),
            user_message: "The selected model name is not supported. Choose another model."
                .to_string(),
        };
    }
    if lower.contains("permission denied") || lower.contains("read-only file system") {
        return LocalModelErrorSummary {
            code: "permission_path".to_string(),
            user_message: "The app cannot save model files to the target folder.".to_string(),
        };
    }
    if lower.contains("no space left on device") {
        return LocalModelErrorSummary {
            code: "disk_space".to_string(),
            user_message: "There is not enough disk space to download this model.".to_string(),
        };
    }
    if lower.contains("certificate") || lower.contains("ssl") || lower.contains("connection") {
        return LocalModelErrorSummary {
            code: "network".to_string(),
            user_message: "The model server could not be reached. Check your internet connection."
                .to_string(),
        };
    }
    LocalModelErrorSummary {
        code: "unknown".to_string(),
        user_message: "Model download failed. Please try again or open logs for details."
            .to_string(),
    }
}

fn redact_local_model_error(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "model download failed".to_string();
    }
    let mut output = trimmed.to_string();
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            output = output.replace(&home, "[home]");
        }
    }
    output.chars().take(6000).collect()
}

fn local_model_debug_ref() -> String {
    format!("lm-{}", now_epoch_ms())
}

fn append_local_model_download_log(
    debug_ref: &str,
    model: &str,
    device: &str,
    error_code: &str,
    raw_error: &str,
) {
    let bridge = resolve_python_bridge_paths();
    let python_source = if bridge.python_bin.contains("runtime-pack") {
        "runtime_pack"
    } else {
        "system_path"
    };
    let line = format!(
        "[{}] [{}] local_model_download_failed code={} model={} device={} cache_dir_class=models/huggingface python_source={} bridge_entry_file={} error=\"{}\"",
        now_epoch_ms(),
        debug_ref,
        error_code,
        model,
        device,
        python_source,
        Path::new(&bridge.entry_script)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("bridge_entry.py"),
        redact_local_model_error(raw_error).replace('\n', "\\n")
    );
    eprintln!("{line}");
    let _ = runtime_fs_append_line("logs/app.log".to_string(), line);
}

fn ensure_runtime_pack_for_model_download(
    app: &tauri::AppHandle,
    state: &LocalModelDownloadState,
    profile_id: &str,
    model: &str,
    device: &str,
) -> Result<(), String> {
    publish_download_status(
        app,
        state,
        LocalModelDownloadStatus {
            active: true,
            profile_id: Some(profile_id.to_string()),
            model: Some(model.to_string()),
            device: Some(device.to_string()),
            phase: "checking_runtime".to_string(),
            progress: 0.1,
            message: "Checking local processing setup...".to_string(),
            error: None,
            error_code: None,
            debug_ref: None,
        },
    );
    log_runtime_setup("checking_runtime", "start", "Starting runtime status check before model download.");
    let status = read_runtime_pack_status()?;
    if matches!(status.status, RuntimePackStatusKind::Ready) {
        log_runtime_setup("checking_runtime", "already_ready", "Runtime is already ready.");
        return Ok(());
    }

    publish_download_status(
        app,
        state,
        LocalModelDownloadStatus {
            active: true,
            profile_id: Some(profile_id.to_string()),
            model: Some(model.to_string()),
            device: Some(device.to_string()),
            phase: "installing_runtime".to_string(),
            progress: 0.25,
            message: "Installing required local processing components...".to_string(),
            error: None,
            error_code: None,
            debug_ref: None,
        },
    );
    log_runtime_setup("installing_runtime", "start", "Runtime setup/repair started.");
    let prepared = match tauri::async_runtime::block_on(local_runtime_pack_prepare(app.clone())) {
        Ok(status) => status,
        Err(err) => {
            if err.trim() == "runtime pack manifest url not configured"
                && can_use_dev_python_bridge_without_runtime_pack()
            {
                log_runtime_setup(
                    "installing_runtime",
                    "dev_fallback",
                    "Runtime pack manifest missing; using development bridge fallback.",
                );
                return Ok(());
            }
            return Err(err);
        }
    };
    if !matches!(prepared.status, RuntimePackStatusKind::Ready) {
        return Err(format!(
            "runtime pack setup failed status={:?} error_code={}",
            prepared.status,
            prepared
                .error_code
                .as_deref()
                .unwrap_or("unknown_error")
        ));
    }
    publish_download_status(
        app,
        state,
        LocalModelDownloadStatus {
            active: true,
            profile_id: Some(profile_id.to_string()),
            model: Some(model.to_string()),
            device: Some(device.to_string()),
            phase: "validating_runtime".to_string(),
            progress: 0.4,
            message: "Checking local processing setup...".to_string(),
            error: None,
            error_code: None,
            debug_ref: None,
        },
    );
    let validated = read_runtime_pack_status()?;
    if !matches!(validated.status, RuntimePackStatusKind::Ready) {
        return Err(format!(
            "runtime pack validation failed status={:?} error_code={}",
            validated.status,
            validated
                .error_code
                .as_deref()
                .unwrap_or("unknown_error")
        ));
    }
    log_runtime_setup("validating_runtime", "success", "Runtime validation succeeded before model download.");
    Ok(())
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
        error_code: None,
        debug_ref: None,
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
pub fn local_model_profile_delete(
    app: tauri::AppHandle,
    state: tauri::State<'_, LocalModelDownloadState>,
    profile_id: String,
) -> Result<LocalModelProfilesView, String> {
    let profile_id = profile_id.trim();
    cancel_active_local_model_download(&app, &state, profile_id);
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

fn cancel_active_local_model_download(
    app: &tauri::AppHandle,
    state: &LocalModelDownloadState,
    profile_id: &str,
) {
    let should_cancel = state
        .current
        .lock()
        .map(|current| current.active && current.profile_id.as_deref() == Some(profile_id))
        .unwrap_or(false);
    if !should_cancel {
        return;
    }
    if let Ok(token) = state.cancel_token.lock() {
        if let Some(token) = token.as_ref() {
            token.store(true, Ordering::SeqCst);
        }
    }
    publish_download_status(app, state, default_download_status());
}

fn local_model_download_was_cancelled(cancel_token: &Arc<AtomicBool>) -> bool {
    cancel_token.load(Ordering::SeqCst)
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
        error_code: None,
        debug_ref: None,
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
    let cancel_token = Arc::new(AtomicBool::new(false));
    if let Ok(mut current_token) = state.cancel_token.lock() {
        *current_token = Some(Arc::clone(&cancel_token));
    }

    let _ = set_local_model_profile_download_state(&profile_id, "downloading", None, None, None);
    let _ = app.emit("local-model-download-progress", initial);

    let state_for_thread = LocalModelDownloadState {
        current: Arc::clone(&state.current),
        cancel_token: Arc::clone(&state.cancel_token),
    };
    std::thread::spawn(move || {
        eprintln!(
            "local_model_setup stage=start category=start message=\"Starting one-step local model setup.\" model={} device={} timestamp_ms={}",
            model,
            device,
            now_epoch_ms()
        );
        let runtime_ready = ensure_runtime_pack_for_model_download(
            &app,
            &state_for_thread,
            &profile_id,
            &model,
            &device,
        );
        if local_model_download_was_cancelled(&cancel_token) {
            log_runtime_setup("cancelled", "cancelled", "Local model setup cancelled.");
            return;
        }
        if let Err(err) = runtime_ready {
            let summary = classify_local_model_download_error(&err);
            let debug_ref = local_model_debug_ref();
            log_runtime_setup(
                "failed",
                summary.code.as_str(),
                &format!("Combined setup failed before model download: {err}"),
            );
            append_runtime_setup_log(&debug_ref, "failed", summary.code.as_str(), &err);
            let _ = set_local_model_profile_download_state(
                &profile_id,
                "failed",
                Some(summary.user_message.clone()),
                Some(summary.code.clone()),
                Some(debug_ref.clone()),
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
                    message: "Setup failed. Please try again.".to_string(),
                    error: Some(summary.user_message),
                    error_code: Some(summary.code),
                    debug_ref: Some(debug_ref),
                },
            );
            return;
        }

        publish_download_status(
            &app,
            &state_for_thread,
            LocalModelDownloadStatus {
                active: true,
                profile_id: Some(profile_id.clone()),
                model: Some(model.clone()),
                device: Some(device.clone()),
                phase: "downloading_model".to_string(),
                progress: 0.65,
                message: "Downloading model...".to_string(),
                error: None,
                error_code: None,
                debug_ref: None,
            },
        );

        let result = prefetch_local_model(
            &app,
            &state_for_thread,
            Arc::clone(&cancel_token),
            &profile_id,
            &model,
            &device,
        );
        if local_model_download_was_cancelled(&cancel_token) {
            log_runtime_setup("cancelled", "cancelled", "Local model download cancelled.");
            return;
        }
        match result {
            Ok(()) => {
                eprintln!(
                    "local_model_setup stage=validating_model category=success message=\"Model download and validation succeeded.\" model={} device={} timestamp_ms={}",
                    model,
                    device,
                    now_epoch_ms()
                );
                let _ = set_local_model_profile_download_state(
                    &profile_id,
                    "ready",
                    None,
                    None,
                    None,
                );
                publish_download_status(
                    &app,
                    &state_for_thread,
                    LocalModelDownloadStatus {
                        active: true,
                        profile_id: Some(profile_id.clone()),
                        model: Some(model.clone()),
                        device: Some(device.clone()),
                        phase: "validating_model".to_string(),
                        progress: 0.92,
                        message: "Validating model...".to_string(),
                        error: None,
                        error_code: None,
                        debug_ref: None,
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
                        message: "Model ready.".to_string(),
                        error: None,
                        error_code: None,
                        debug_ref: None,
                    },
                );
            }
            Err(err) => {
                let summary = classify_local_model_download_error(&err);
                let debug_ref = local_model_debug_ref();
                eprintln!(
                    "local_model_setup stage=failed category={} message=\"{}\" model={} device={} timestamp_ms={}",
                    summary.code,
                    redact_local_model_error(&err).replace('\n', "\\n"),
                    model,
                    device,
                    now_epoch_ms()
                );
                append_local_model_download_log(
                    &debug_ref,
                    &model,
                    &device,
                    &summary.code,
                    &err,
                );
                let _ = set_local_model_profile_download_state(
                    &profile_id,
                    "failed",
                    Some(summary.user_message.clone()),
                    Some(summary.code.clone()),
                    Some(debug_ref.clone()),
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
                        message: "Download failed. Please check your connection and try again.".to_string(),
                        error: Some(summary.user_message),
                        error_code: Some(summary.code),
                        debug_ref: Some(debug_ref),
                    },
                );
            }
        }
    });

    Ok(())
}

fn publish_bridge_progress_line(
    app: &tauri::AppHandle,
    state: &LocalModelDownloadState,
    profile_id: &str,
    model: &str,
    device: &str,
    line: &str,
) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
        return;
    };
    if value.get("event").and_then(|event| event.as_str()) != Some("local_model_setup_progress") {
        return;
    }
    let phase = value
        .get("phase")
        .and_then(|phase| phase.as_str())
        .unwrap_or("downloading_model")
        .to_string();
    let progress = value
        .get("progress")
        .and_then(|progress| progress.as_f64())
        .unwrap_or(0.65);
    let message = value
        .get("message")
        .and_then(|message| message.as_str())
        .unwrap_or("Preparing local model...")
        .to_string();
    eprintln!(
        "local_model_setup stage={} category=progress message=\"{}\" model={} device={} timestamp_ms={}",
        phase,
        sanitize_runtime_error(&message).replace('\n', "\\n"),
        model,
        device,
        now_epoch_ms()
    );
    publish_download_status(
        app,
        state,
        LocalModelDownloadStatus {
            active: true,
            profile_id: Some(profile_id.to_string()),
            model: Some(model.to_string()),
            device: Some(device.to_string()),
            phase,
            progress,
            message,
            error: None,
            error_code: None,
            debug_ref: None,
        },
    );
}

fn invoke_python_for_local_model_prefetch(
    app: &tauri::AppHandle,
    state: &LocalModelDownloadState,
    cancel_token: Arc<AtomicBool>,
    profile_id: &str,
    model: &str,
    device: &str,
    req: PythonInvokeRequest,
) -> Result<crate::runtime::process_supervisor::ProcessOutput, String> {
    let mut command = Command::new(&req.python_bin);
    command
        .args([req.entry_script.as_str()])
        .envs(req.env.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|err| format!("python invoke failed: {}", err))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(&req.stdin_json)
            .map_err(|err| format!("python invoke failed: {}", err))?;
        stdin
            .flush()
            .map_err(|err| format!("python invoke failed: {}", err))?;
    }
    drop(child.stdin.take());

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "python invoke failed: missing stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "python invoke failed: missing stderr".to_string())?;
    let stdout_handle = thread::spawn(move || {
        let mut output = String::new();
        let mut reader = BufReader::new(stdout);
        let _ = reader.read_to_string(&mut output);
        output
    });
    let (line_tx, line_rx) = mpsc::channel::<String>();
    let stderr_handle = thread::spawn(move || {
        let mut raw = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let _ = line_tx.send(line.clone());
                    raw.push_str(&line);
                    raw.push('\n');
                }
                Err(_) => break,
            }
        }
        raw
    });

    let deadline = Instant::now() + Duration::from_millis(req.timeout_ms);
    let mut timed_out = false;
    loop {
        while let Ok(line) = line_rx.try_recv() {
            publish_bridge_progress_line(app, state, profile_id, model, device, &line);
        }
        if cancel_token.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("local model download cancelled".to_string());
        }
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => return Err(format!("python invoke failed: {}", err)),
        }
    }
    while let Ok(line) = line_rx.try_recv() {
        publish_bridge_progress_line(app, state, profile_id, model, device, &line);
    }
    let status_code = child
        .try_wait()
        .map_err(|err| format!("python invoke failed: {}", err))?
        .and_then(|status| status.code());
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();
    Ok(crate::runtime::process_supervisor::ProcessOutput {
        status_code,
        stdout,
        stderr,
        timed_out,
    })
}

fn prefetch_local_model(
    app: &tauri::AppHandle,
    state: &LocalModelDownloadState,
    cancel_token: Arc<AtomicBool>,
    profile_id: &str,
    model: &str,
    device: &str,
) -> Result<(), String> {
    let runtime_pack = read_runtime_pack_status()?;
    if !matches!(runtime_pack.status, RuntimePackStatusKind::Ready) {
        if !can_use_dev_python_bridge_without_runtime_pack() {
            return Err("runtime_pack_setup_required".to_string());
        }
        log_runtime_setup(
            "checking_runtime",
            "dev_fallback",
            "Runtime pack is not ready; proceeding with development bridge fallback.",
        );
    }
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
    let bridge = resolve_python_bridge_paths();
    let env = with_python_runtime_env(env, &bridge);
    let proc = invoke_python_for_local_model_prefetch(
        app,
        state,
        cancel_token,
        profile_id,
        model,
        device,
        PythonInvokeRequest {
        python_bin: bridge.python_bin,
        entry_script: bridge.entry_script,
        env,
        stdin_json,
        timeout_ms: std::env::var("PYTHON_BRIDGE_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300_000),
        },
    )?;

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
    let bridge_stdout = proc
        .stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .unwrap_or_else(|| proc.stdout.trim());
    let parsed: serde_json::Value = serde_json::from_str(bridge_stdout.trim())
        .map_err(|e| format!("invalid bridge json: {e}"))?;
    if parsed.get("ok").and_then(|value| value.as_bool()) == Some(true) {
        Ok(())
    } else {
        let error = parsed.get("error");
        let message = error
            .and_then(|value| value.get("message"))
            .and_then(|value| value.as_str())
            .unwrap_or("python bridge returned error while downloading local model");
        let code = error
            .and_then(|value| value.get("code"))
            .and_then(|value| value.as_str())
            .unwrap_or("PYTHON_ERROR");
        let details = error
            .and_then(|value| value.get("details"))
            .map(|value| value.to_string())
            .unwrap_or_else(|| "{}".to_string());
        Err(format!(
            "{message} [bridge_code={code}] [bridge_details={details}]"
        ))
    }
}

fn can_use_dev_python_bridge_without_runtime_pack() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    let bridge = resolve_python_bridge_paths();
    bridge.bundled_runtime_dir.is_none()
}

#[tauri::command]
pub fn local_runtime_pack_status() -> Result<LocalRuntimePackStatus, String> {
    read_runtime_pack_status()
}

#[tauri::command]
pub async fn local_runtime_pack_prepare(app: tauri::AppHandle) -> Result<LocalRuntimePackStatus, String> {
    let manifest_url = local_runtime_pack_manifest_url();
    log_runtime_setup("checking_runtime", "start", "Starting local runtime status check.");
    emit_runtime_pack_progress(
        &app,
        "manifest",
        0.05,
        "Checking runtime-pack manifest...",
        RuntimePackStatusKind::Downloading,
    );
    let (manifest_source, manifest_url_label, status, content_type, body) = if let Some(url) = manifest_url {
        log_runtime_setup_manifest(
            "runtime_manifest_load",
            "start",
            "remote",
            &url,
            None,
            None,
            None,
            None,
            None,
        );
        let manifest_response = reqwest::get(&url).await.map_err(|err| {
            log_runtime_setup_manifest(
                "runtime_manifest_load",
                "runtime_manifest_http_error",
                "remote",
                &url,
                None,
                None,
                None,
                Some(&err.to_string()),
                None,
            );
            "runtime pack manifest fetch failed".to_string()
        })?;
        let status = manifest_response.status();
        let content_type = manifest_response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let body = manifest_response.text().await.map_err(|err| {
            log_runtime_setup_manifest(
                "runtime_manifest_load",
                "runtime_manifest_read_failed",
                "remote",
                &url,
                Some(status.as_u16()),
                content_type.as_deref(),
                None,
                Some(&err.to_string()),
                None,
            );
            "runtime pack manifest read failed".to_string()
        })?;
        ("remote".to_string(), url, status, content_type, body)
    } else {
        let manifest_path = local_runtime_pack_manifest_local_path()?;
        let manifest_label = "local://runtime-pack/manifest.json".to_string();
        log_runtime_setup_manifest(
            "runtime_manifest_load",
            "start",
            "local",
            &manifest_label,
            None,
            Some("application/json"),
            None,
            None,
            None,
        );
        if !manifest_path.exists() {
            log_runtime_setup_manifest(
                "runtime_manifest_load",
                "runtime_manifest_missing",
                "local",
                &manifest_label,
                None,
                Some("application/json"),
                None,
                None,
                Some("set LOCAL_RUNTIME_PACK_MANIFEST_URL or place manifest at runtime-pack/manifest.json"),
            );
            return Err("runtime pack manifest url not configured".to_string());
        }
        let body = fs::read_to_string(&manifest_path).map_err(|err| {
            log_runtime_setup_manifest(
                "runtime_manifest_load",
                "runtime_manifest_read_failed",
                "local",
                &manifest_label,
                None,
                Some("application/json"),
                None,
                Some(&err.to_string()),
                None,
            );
            "runtime pack manifest read failed".to_string()
        })?;
        (
            "local".to_string(),
            manifest_label,
            reqwest::StatusCode::OK,
            Some("application/json".to_string()),
            body,
        )
    };
    if manifest_source == "remote" && !status.is_success() {
        log_runtime_setup_manifest(
            "runtime_manifest_load",
            "runtime_manifest_http_error",
            &manifest_source,
            &manifest_url_label,
            Some(status.as_u16()),
            content_type.as_deref(),
            Some(&body),
            None,
            None,
        );
        return Err("runtime pack manifest fetch failed".to_string());
    }
    if body.trim().is_empty() {
        log_runtime_setup_manifest(
            "runtime_manifest_parse",
            "runtime_manifest_empty",
            &manifest_source,
            &manifest_url_label,
            Some(status.as_u16()),
            content_type.as_deref(),
            Some(&body),
            None,
            None,
        );
        return Err("runtime pack manifest is empty".to_string());
    }
    if content_type
        .as_deref()
        .map(|value| value.contains("json"))
        .unwrap_or(false)
        == false
        && body.trim_start().starts_with('<')
    {
        log_runtime_setup_manifest(
            "runtime_manifest_parse",
            "runtime_manifest_not_json",
            &manifest_source,
            &manifest_url_label,
            Some(status.as_u16()),
            content_type.as_deref(),
            Some(&body),
            None,
            None,
        );
        return Err("runtime pack manifest is not json".to_string());
    }
    let manifest_value: serde_json::Value = serde_json::from_str(&body).map_err(|err| {
        log_runtime_setup_manifest(
            "runtime_manifest_parse",
            "runtime_manifest_parse_error",
            &manifest_source,
            &manifest_url_label,
            Some(status.as_u16()),
            content_type.as_deref(),
            Some(&body),
            Some(&err.to_string()),
            None,
        );
        "runtime pack manifest parse failed".to_string()
    })?;
    let manifest: RuntimePackManifest = serde_json::from_value(manifest_value).map_err(|err| {
        log_runtime_setup_manifest(
            "runtime_manifest_parse",
            "runtime_manifest_schema_error",
            &manifest_source,
            &manifest_url_label,
            Some(status.as_u16()),
            content_type.as_deref(),
            Some(&body),
            None,
            Some(&err.to_string()),
        );
        "runtime pack manifest schema failed".to_string()
    })?;
    if let Some(required_app_version) = manifest
        .app_compatibility
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        if required_app_version != env!("CARGO_PKG_VERSION") {
            let store = LocalRuntimePackStateStore {
                status: Some(RuntimePackStatusKind::ValidationFailed),
                version: Some(manifest.version),
                installed_at_ms: None,
                last_error_code: Some("incompatible_app_version".to_string()),
                last_debug_ref: Some(format!("rp-{}", now_epoch_ms())),
            };
            let _ = save_runtime_pack_state_store(&store);
            log_runtime_setup(
                "validating_runtime",
                "runtime_manifest_incompatible_version",
                "Runtime app version compatibility check failed.",
            );
            return read_runtime_pack_status();
        }
    }
    let asset = match resolve_runtime_pack_asset(&manifest) {
        Ok(asset) => asset,
        Err(_) => {
            let store = LocalRuntimePackStateStore {
                status: Some(RuntimePackStatusKind::IncompatiblePlatform),
                version: Some(manifest.version),
                installed_at_ms: None,
                last_error_code: Some("incompatible_platform".to_string()),
                last_debug_ref: Some(format!("rp-{}", now_epoch_ms())),
            };
            let _ = save_runtime_pack_state_store(&store);
            log_runtime_setup_manifest(
                "runtime_manifest_parse",
                "runtime_manifest_platform_missing",
                &manifest_source,
                &manifest_url_label,
                Some(status.as_u16()),
                content_type.as_deref(),
                Some(&body),
                None,
                Some("no matching platform/arch asset in manifest"),
            );
            return read_runtime_pack_status();
        }
    };

    let runtime_root = local_runtime_pack_root()?;
    fs::create_dir_all(&runtime_root).map_err(|e| e.to_string())?;
    let tmp_zip = runtime_root.join(format!("{}.zip", asset.filename));
    emit_runtime_pack_progress(
        &app,
        "download",
        0.25,
        "Downloading runtime-pack...",
        RuntimePackStatusKind::Downloading,
    );
    let bytes = reqwest::get(&asset.url)
        .await
        .map_err(|_| {
            log_runtime_setup("downloading_runtime", "network_error", "Runtime pack download request failed.");
            "runtime pack download failed".to_string()
        })?
        .bytes()
        .await
        .map_err(|_| {
            log_runtime_setup("downloading_runtime", "network_error", "Runtime pack download body read failed.");
            "runtime pack download failed".to_string()
        })?;
    if let Some(expected_size) = asset.size {
        if bytes.len() as u64 != expected_size {
            let store = LocalRuntimePackStateStore {
                status: Some(RuntimePackStatusKind::Corrupted),
                version: Some(manifest.version),
                installed_at_ms: None,
                last_error_code: Some("size_mismatch".to_string()),
                last_debug_ref: Some(format!("rp-{}", now_epoch_ms())),
            };
            let _ = save_runtime_pack_state_store(&store);
            log_runtime_setup("validating_runtime", "validation_failed", "Runtime pack size did not match manifest.");
            return read_runtime_pack_status();
        }
    }
    fs::write(&tmp_zip, &bytes).map_err(|e| {
        log_runtime_setup("installing_runtime", "permission_error", "Failed to write runtime archive.");
        e.to_string()
    })?;
    let actual = sha256_file(&tmp_zip)?;
    if actual != asset.sha256.to_ascii_lowercase() {
        let store = LocalRuntimePackStateStore {
            status: Some(RuntimePackStatusKind::Corrupted),
            version: Some(manifest.version),
            installed_at_ms: None,
            last_error_code: Some("checksum_mismatch".to_string()),
            last_debug_ref: Some(format!("rp-{}", now_epoch_ms())),
        };
        let _ = save_runtime_pack_state_store(&store);
        log_runtime_setup("validating_runtime", "corrupted", "Runtime pack checksum mismatch.");
        return read_runtime_pack_status();
    }

    emit_runtime_pack_progress(
        &app,
        "install",
        0.65,
        "Installing runtime-pack...",
        RuntimePackStatusKind::Installing,
    );
    let staging_dir = runtime_root.join(format!("staging-{}", now_epoch_ms()));
    extract_runtime_pack_zip(&tmp_zip, &staging_dir)?;
    let required_modules = asset.required_modules.clone().unwrap_or_default();
    if let Err(code) = validate_runtime_pack_install(&staging_dir, &required_modules) {
        let status = if code == "missing_files" {
            RuntimePackStatusKind::MissingFiles
        } else if code == "validation_failed" {
            RuntimePackStatusKind::NativeImportFailure
        } else {
            RuntimePackStatusKind::ValidationFailed
        };
        let store = LocalRuntimePackStateStore {
            status: Some(status),
            version: Some(manifest.version),
            installed_at_ms: None,
            last_error_code: Some(code),
            last_debug_ref: Some(format!("rp-{}", now_epoch_ms())),
        };
        let _ = save_runtime_pack_state_store(&store);
        let _ = fs::remove_dir_all(&staging_dir);
        log_runtime_setup("validating_runtime", "validation_failed", "Runtime dependency validation failed.");
        return read_runtime_pack_status();
    }

    let current_dir = local_runtime_pack_current_dir()?;
    let backup_dir = runtime_root.join("previous");
    if backup_dir.exists() {
        let _ = fs::remove_dir_all(&backup_dir);
    }
    if current_dir.exists() {
        let _ = fs::rename(&current_dir, &backup_dir);
    }
    fs::rename(&staging_dir, &current_dir).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&tmp_zip);
    let _ = fs::remove_dir_all(&backup_dir);

    let store = LocalRuntimePackStateStore {
        status: Some(RuntimePackStatusKind::Ready),
        version: Some(manifest.version),
        installed_at_ms: Some(now_epoch_ms()),
        last_error_code: None,
        last_debug_ref: None,
    };
    save_runtime_pack_state_store(&store)?;
    emit_runtime_pack_progress(
        &app,
        "ready",
        1.0,
        "Runtime-pack ready.",
        RuntimePackStatusKind::Ready,
    );
    log_runtime_setup("validating_runtime", "success", "Runtime validation succeeded.");
    read_runtime_pack_status()
}

#[tauri::command]
pub async fn local_runtime_pack_retry(app: tauri::AppHandle) -> Result<LocalRuntimePackStatus, String> {
    local_runtime_pack_prepare(app).await
}

#[tauri::command]
pub async fn local_runtime_pack_repair(app: tauri::AppHandle) -> Result<LocalRuntimePackStatus, String> {
    let root = local_runtime_pack_root()?;
    let _ = fs::remove_dir_all(root.join("current"));
    local_runtime_pack_prepare(app).await
}

#[cfg(test)]
mod tests {
    use super::{classify_local_model_download_error, resolve_runtime_pack_asset, sanitize_runtime_error, RuntimePackManifest};

    #[test]
    fn local_model_error_missing_dependency_for_top_level_module_only() {
        let result =
            classify_local_model_download_error("No module named 'faster_whisper'");
        assert_eq!(result.code, "missing_dependency");
    }

    #[test]
    fn local_model_error_transitive_dependency_load_failure() {
        let result = classify_local_model_download_error(
            "faster-whisper could not be loaded because a required local dependency failed to import. ImportError: cannot open shared object file",
        );
        assert_eq!(result.code, "dependency_load_failed");
    }

    #[test]
    fn local_model_error_runtime_pack_required() {
        let result = classify_local_model_download_error("runtime_pack_setup_required");
        assert_eq!(result.code, "runtime_pack_setup_required");
    }

    #[test]
    fn local_model_error_runtime_setup_network_failure() {
        let result = classify_local_model_download_error(
            "runtime pack setup failed status=network_error error_code=network_error",
        );
        assert_eq!(result.code, "network_error");
    }

    #[test]
    fn local_model_error_runtime_setup_permission_failure() {
        let result = classify_local_model_download_error(
            "runtime pack setup failed status=permission_error error_code=permission_error",
        );
        assert_eq!(result.code, "permission_error");
    }

    #[test]
    fn local_model_error_runtime_manifest_parse_failure() {
        let result = classify_local_model_download_error("runtime pack manifest parse failed");
        assert_eq!(result.code, "runtime_manifest_parse_error");
    }

    #[test]
    fn local_model_error_runtime_manifest_not_json_failure() {
        let result = classify_local_model_download_error("runtime pack manifest is not json");
        assert_eq!(result.code, "runtime_manifest_parse_error");
    }

    #[test]
    fn local_model_error_runtime_manifest_missing_configuration() {
        let result =
            classify_local_model_download_error("runtime pack manifest url not configured");
        assert_eq!(result.code, "runtime_manifest_parse_error");
    }

    #[test]
    fn local_model_error_dependency_install_failure() {
        let result = classify_local_model_download_error(
            "failed to install Python dependency requests>=2.31: network is unreachable",
        );
        assert_eq!(result.code, "dependency_install_failed");
    }

    #[test]
    fn local_model_error_missing_requests_dependency() {
        let result = classify_local_model_download_error("No module named 'requests'");
        assert_eq!(result.code, "missing_dependency");
    }

    #[test]
    fn runtime_error_sanitizer_redacts_home_path() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp/home".to_string());
        let raw = format!("failed at {home}/secret-path");
        let redacted = sanitize_runtime_error(&raw);
        assert!(!redacted.contains(&home));
        assert!(redacted.contains("[home]"));
    }

    #[test]
    fn runtime_pack_manifest_selector_matches_platform() {
        let manifest: RuntimePackManifest = serde_json::from_str(
            r#"{
                "version":"1.2.3",
                "assets":[
                    {"platform":"linux","arch":"x86_64","filename":"linux.zip","url":"https://example.test/linux.zip","sha256":"abc","requiredModules":["faster_whisper"]},
                    {"platform":"macos","arch":"aarch64","filename":"mac.zip","url":"https://example.test/mac.zip","sha256":"def","requiredModules":["faster_whisper"]}
                ]
            }"#,
        )
        .expect("manifest parse");
        if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            let asset = resolve_runtime_pack_asset(&manifest).expect("asset");
            assert_eq!(asset.filename, "linux.zip");
        }
    }
}

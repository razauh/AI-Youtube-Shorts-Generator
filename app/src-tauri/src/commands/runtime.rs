use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

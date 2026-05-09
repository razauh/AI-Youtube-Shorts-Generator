# Shared Secure Storage Abstraction (Language-Agnostic Guide)

This document specifies the secure local storage abstraction used by all desktop application modules to persist sensitive data (tokens, credentials, metadata).

---

## Purpose

Desktop applications must persist certain data locally:
- License activation tokens (JWT)
- Device fingerprint and install ID
- Grace period timestamps
- Application configuration
- Update state

This data has varying sensitivity levels and requires appropriate storage mechanisms.

---

## Storage Tiers

| Tier | Security Level | Backed By | Use For |
|---|---|---|---|
| **Secure** | High | OS keychain / credential store | Tokens, secrets, signing keys |
| **Protected** | Medium | Encrypted file in app data directory | License metadata, grace timestamps, device fingerprint hash |
| **Standard** | Low | Plain file in app data directory | Non-sensitive preferences, UI state, log configuration |

---

## Secure Tier — OS Keychain

### Interface

```
SecureStore {
  save(key: string, value: string) → Result<void, StorageError>
  load(key: string) → Result<string | null, StorageError>
  delete(key: string) → Result<void, StorageError>
  exists(key: string) → Result<boolean, StorageError>
}
```

### Platform Mapping

| Platform | Backend | Notes |
|---|---|---|
| **Windows** | Windows Credential Manager (DPAPI) | Per-user, encrypted at rest |
| **macOS** | Keychain Services | Per-app, protected by user login |
| **Linux** | `libsecret` / Secret Service API / `kwallet` | Requires a running keyring daemon |
| **Fallback** | Encrypted file with machine-derived key | When OS keychain is unavailable |

### Stored Items

| Key | Value | Module |
|---|---|---|
| `license_token` | JWT string | Licensing |
| `license_install_id` | UUID string | Licensing |

### Rules
- Never store secrets in plaintext files, environment variables, or application logs.
- If the OS keychain is unavailable, fall back to the Protected tier with encrypted file storage.
- Log `STOR_KEYCHAIN_UNAVAILABLE` warning when falling back.
- On read failure (corruption, permission denied), treat as empty and require re-authentication.

---

## Protected Tier — Encrypted File

### Interface

```
ProtectedStore {
  save(key: string, value: any) → Result<void, StorageError>
  load(key: string) → Result<any | null, StorageError>
  delete(key: string) → Result<void, StorageError>
  clear() → Result<void, StorageError>
}
```

### Encryption
- Algorithm: AES-256-GCM (or platform-native equivalent).
- Key derivation: Derive the encryption key from a combination of machine-specific identifiers (machine ID, app installation path) using HKDF or PBKDF2.
- The derived key must not be stored — it is recomputed on each access.
- Each value is encrypted independently with a unique nonce/IV.

### File Layout
```
<app_data_dir>/
  protected/
    license_meta.enc     # Grace period, last check timestamps
    device_info.enc      # Device fingerprint hash, install ID
```

### Stored Items

| Key | Value | Module |
|---|---|---|
| `last_validated_at` | ISO 8601 timestamp | Licensing |
| `grace_until` | ISO 8601 timestamp | Licensing |
| `last_check_at` | ISO 8601 timestamp | Licensing |
| `device_fingerprint_hash` | SHA-256 hex string | Licensing |
| `update_pending_version` | Semver string | Updater |
| `update_channel` | `stable` / `beta` / `nightly` | Updater |

### Rules
- If decryption fails (key changed, corruption), delete the file and treat as first launch.
- Log `STOR_CORRUPT` when decryption fails.
- Set file permissions to owner-read-write only (e.g., `0600` on Unix).

---

## Standard Tier — Plain Configuration

### Interface

```
ConfigStore {
  get(key: string) → string | null
  set(key: string, value: string) → void
  getAll() → Map<string, string>
  reset() → void
}
```

### File Format
- Use a standard format: JSON, TOML, or YAML.
- File location: `<app_data_dir>/config.<ext>`

### Stored Items

| Key | Value | Module |
|---|---|---|
| `log_level` | `TRACE` / `DEBUG` / `INFO` / `WARN` / `ERROR` / `FATAL` | Logging |
| `update_mode` | `manual` / `notify` / `auto-download` | Updater |
| `update_check_cooldown_ms` | Integer | Updater |
| `admin_base_url` | URL string | Admin Console |
| `admin_default_key_id` | String | Admin Console |

### Rules
- No sensitive data in this tier — ever.
- File should be human-readable for debugging.
- Missing file: create with defaults on first launch.
- Corrupt file: log warning and reset to defaults.

---

## Error Handling

All storage operations must handle these failures:

| Failure | Error Code | Recovery |
|---|---|---|
| Keychain unavailable | `STOR_KEYCHAIN_UNAVAILABLE` | Fall back to Protected tier |
| File read permission denied | `STOR_READ_FAILURE` | Log and treat as empty |
| File write permission denied | `STOR_WRITE_FAILURE` | Log and surface to user |
| Disk full | `STOR_DISK_FULL` | Log and surface to user |
| Decryption failure | `STOR_CORRUPT` | Delete corrupted file, reset to defaults |
| Unexpected error | `SYS_INTERNAL` | Log with full context |

---

## Platform Data Directories

| Platform | App Data Directory |
|---|---|
| **Windows** | `%APPDATA%\<AppName>\` |
| **macOS** | `~/Library/Application Support/<AppName>/` |
| **Linux** | `~/.local/share/<AppName>/` or `$XDG_DATA_HOME/<AppName>/` |

---

## Testing Requirements

- Verify secure storage round-trip: save → load → matches original.
- Verify secure storage deletion: delete → load returns null.
- Verify protected tier encryption: saved file is not readable as plaintext.
- Verify protected tier corruption handling: corrupt file → reset to defaults.
- Verify standard tier missing file: creates with defaults.
- Verify standard tier corrupt file: resets to defaults.
- Verify keychain fallback: mock unavailable keychain → falls back to encrypted file.
- Verify file permissions: created files have correct restricted permissions.

# Shared Error Taxonomy (Language-Agnostic Guide)

This document defines the unified error classification system used across all modules (Updater, Licensing Worker, Admin Console, Enhanced Logging). All modules must use this taxonomy for consistent error handling, logging, and user messaging.

---

## Error Categories

| Category | Code Range | Description |
|---|---|---|
| `VALIDATION` | `VAL_*` | Input validation failures (client-side correctable) |
| `AUTHENTICATION` | `AUTH_*` | Identity/credential verification failures |
| `AUTHORIZATION` | `AUTHZ_*` | Permission/entitlement denials |
| `NETWORK` | `NET_*` | Network connectivity and transport failures |
| `PROVIDER` | `PROV_*` | External provider/service failures |
| `STORAGE` | `STOR_*` | Local or remote storage failures |
| `SECURITY` | `SEC_*` | Security verification failures (signatures, tamper detection) |
| `SYSTEM` | `SYS_*` | Internal system/runtime errors |
| `RATE_LIMIT` | `RATE_*` | Rate limiting enforcement |

---

## Error Codes

### Validation Errors

| Code | Description | User Message Example |
|---|---|---|
| `VAL_MISSING_FIELD` | Required field not provided | "Please fill in all required fields." |
| `VAL_INVALID_FORMAT` | Field format is incorrect | "The value entered is not in the expected format." |
| `VAL_OUT_OF_RANGE` | Numeric/date value outside allowed range | "The value is outside the allowed range." |
| `VAL_INVALID_JSON` | Request body is not valid JSON | "The request could not be processed." |
| `VAL_INVALID_VERSION` | Version string is not valid semver | "Invalid version format." |

### Authentication Errors

| Code | Description | User Message Example |
|---|---|---|
| `AUTH_TOKEN_MISSING` | Bearer token not provided | "Authentication required." |
| `AUTH_TOKEN_EXPIRED` | Token has expired | "Your session has expired. Please re-authenticate." |
| `AUTH_TOKEN_INVALID` | Token signature or format is invalid | "Authentication failed." |
| `AUTH_TOKEN_FUTURE` | Token issued-at is in the future beyond skew | "Authentication failed." |
| `AUTH_ADMIN_SIGNATURE_INVALID` | Admin HMAC signature mismatch | "Admin authentication failed. Check your credentials." |
| `AUTH_ADMIN_TIMESTAMP_STALE` | Admin timestamp outside freshness window | "Request expired. Please try again." |
| `AUTH_ADMIN_NONCE_REPLAYED` | Admin nonce has been used before | "Request rejected (duplicate)." |
| `AUTH_ADMIN_KEY_UNKNOWN` | Admin key ID not recognized | "Unknown admin key. Check your key ID." |

### Authorization Errors

| Code | Description | User Message Example |
|---|---|---|
| `AUTHZ_LICENSE_INVALID` | License key is not valid | "Invalid license key." |
| `AUTHZ_LICENSE_REVOKED` | License has been revoked | "This license has been revoked." |
| `AUTHZ_LICENSE_EXPIRED` | License has expired | "This license has expired." |
| `AUTHZ_SUBSCRIPTION_INACTIVE` | Subscription is cancelled/refunded/past_due | "Your subscription is no longer active." |
| `AUTHZ_DEVICE_MISMATCH` | Device fingerprint does not match bound device | "This license is bound to another device." |
| `AUTHZ_GRACE_EXPIRED` | Offline grace period has expired | "Please connect to the internet to verify your license." |

### Network Errors

| Code | Description | User Message Example |
|---|---|---|
| `NET_TIMEOUT` | Request timed out | "The server did not respond in time. Please try again." |
| `NET_DNS_FAILURE` | DNS resolution failed | "Unable to reach the server. Check your internet connection." |
| `NET_CONNECTION_REFUSED` | Connection refused | "The server is unavailable. Please try again later." |
| `NET_SSL_ERROR` | TLS/SSL handshake or certificate error | "Secure connection failed." |
| `NET_OFFLINE` | No network connectivity detected | "You appear to be offline." |

### Provider Errors

| Code | Description | User Message Example |
|---|---|---|
| `PROV_UNAVAILABLE` | External provider is unreachable | "Unable to verify your license at this time." |
| `PROV_ERROR` | External provider returned an error | "License verification service error." |
| `PROV_MISCONFIGURED` | Provider configuration is missing/invalid | (Internal only — not shown to users) |

### Storage Errors

| Code | Description | User Message Example |
|---|---|---|
| `STOR_READ_FAILURE` | Failed to read from storage | "Unable to read application data." |
| `STOR_WRITE_FAILURE` | Failed to write to storage | "Unable to save data. Check disk space." |
| `STOR_CORRUPT` | Stored data is corrupted or unreadable | "Application data is corrupted. Please reinstall." |
| `STOR_KEYCHAIN_UNAVAILABLE` | OS keychain/credential store unavailable | "Secure storage is unavailable." |
| `STOR_DISK_FULL` | Insufficient disk space | "Insufficient disk space." |

### Security Errors

| Code | Description | User Message Example |
|---|---|---|
| `SEC_HASH_MISMATCH` | File hash does not match expected value | "Update verification failed (integrity check)." |
| `SEC_SIGNATURE_INVALID` | Digital signature verification failed | "Update verification failed (signature check)." |
| `SEC_TAMPER_DETECTED` | Stored data appears tampered | "Security check failed. Please contact support." |
| `SEC_CLOCK_ROLLBACK` | System clock appears to have been rolled back | "System time error detected." |

### System Errors

| Code | Description | User Message Example |
|---|---|---|
| `SYS_INTERNAL` | Unexpected internal error | "An unexpected error occurred." |
| `SYS_UNAVAILABLE` | Service is temporarily unavailable | "Service temporarily unavailable. Please try again later." |
| `SYS_CONFIG_ERROR` | Configuration is missing or invalid | "Application configuration error." |

### Rate Limit Errors

| Code | Description | User Message Example |
|---|---|---|
| `RATE_IP_EXCEEDED` | Per-IP rate limit exceeded | "Too many requests. Please wait before trying again." |
| `RATE_KEY_EXCEEDED` | Per-license-key rate limit exceeded | "Too many requests for this license." |
| `RATE_GLOBAL_EXCEEDED` | Global rate limit exceeded | "Service is busy. Please try again later." |

---

## Error Handling Rules

### For All Modules

1. **Log every error** at the appropriate level (`WARN` for expected denials, `ERROR` for failures).
2. **Include the error code** in the log entry's `error_code` field.
3. **Never expose internal details** in user-facing messages — use the "User Message Example" patterns.
4. **Map server reason codes** to error codes: e.g., `INVALID_KEY` → `AUTHZ_LICENSE_INVALID`.
5. **Include correlation ID** in error logs to enable tracing.

### Retry Behavior by Category

| Category | Retryable? | Notes |
|---|---|---|
| `VALIDATION` | No | Fix input and resubmit |
| `AUTHENTICATION` | No | Re-authenticate |
| `AUTHORIZATION` | No | Policy decision; do not retry |
| `NETWORK` | Yes | Retry with exponential backoff |
| `PROVIDER` | Yes | Retry with exponential backoff |
| `STORAGE` | Conditional | Retry reads; do not retry writes without user action |
| `SECURITY` | No | Security failures must never be retried |
| `SYSTEM` | Conditional | Retry on `SYS_UNAVAILABLE` only |
| `RATE_LIMIT` | Yes | Retry after `Retry-After` duration |

---

## Mapping Licensing Reason Codes to Error Codes

| Reason Code | Error Code |
|---|---|
| `INVALID_KEY` | `AUTHZ_LICENSE_INVALID` |
| `SUBSCRIPTION_INACTIVE` | `AUTHZ_SUBSCRIPTION_INACTIVE` |
| `KEY_BOUND_TO_OTHER_DEVICE` | `AUTHZ_DEVICE_MISMATCH` |
| `REVOKED` | `AUTHZ_LICENSE_REVOKED` |
| `RATE_LIMITED` | `RATE_IP_EXCEEDED` or `RATE_KEY_EXCEEDED` |
| `SERVER_ERROR` | `SYS_INTERNAL` |

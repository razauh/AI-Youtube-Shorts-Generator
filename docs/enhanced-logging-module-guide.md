# Enhanced Logging Module (Language-Agnostic Guide)

## Purpose
The Enhanced Logging module provides a unified, structured, privacy-safe logging system for all desktop application modules. It defines how logs are created, formatted, stored, rotated, redacted, exported, and tested — enabling diagnostics, auditing, and support workflows without compromising user privacy.

All other modules (Updater, Licensing Worker, Admin Console) must use this module as their sole logging interface.

---

## Goals and Scope

1. **Diagnostics**: Enable developers and support staff to reconstruct application behavior from logs alone.
2. **Auditing**: Provide tamper-evident records of security-relevant operations (activation, revocation, admin actions).
3. **Privacy**: Never log sensitive user data in plaintext — all PII and secrets must be redacted or hashed before emission.
4. **Performance**: Logging must not degrade application responsiveness — all I/O must be asynchronous or buffered.
5. **Portability**: The logging contract is independent of any specific language, framework, or runtime.

### In Scope
- Local structured log output (file-based)
- Log levels, format, and schemas
- Correlation and operation tracking
- Rotation, retention, and export
- Sensitive data redaction
- Crash and lifecycle logging
- Module-specific event guidance
- Configuration and performance
- Testing requirements

### Out of Scope
- Remote log aggregation infrastructure (optional extension only)
- Application Performance Monitoring (APM) integration
- Real-time log streaming dashboards

---

## Log Levels

All implementations must support exactly these six levels, in ascending severity order:

| Level | Numeric | Purpose | Example |
|---|---|---|---|
| `TRACE` | 0 | Fine-grained diagnostic detail; disabled in production by default | Function entry/exit, variable state |
| `DEBUG` | 1 | Developer-oriented diagnostic information | Parsed config values, intermediate computation |
| `INFO` | 2 | Normal operational events | App started, update check completed, license validated |
| `WARN` | 3 | Unexpected but recoverable situations | Retry triggered, offline grace period warning, deprecated API usage |
| `ERROR` | 4 | Failures that prevent a specific operation from completing | Download failed, signature mismatch, database write error |
| `FATAL` | 5 | Unrecoverable failures that require application shutdown | Corrupt config, storage initialization failure |

### Level Rules
- The **minimum active level** must be configurable at runtime without restart.
- Default production level: `INFO`.
- Default development level: `DEBUG`.
- `TRACE` must never be enabled in production unless explicitly requested for diagnostics.
- `FATAL` must trigger graceful shutdown procedures after logging.

---

## Structured Logging Format

All log entries must be emitted as structured data (JSON or equivalent structured format). Free-form string logging is prohibited in production code.

### Required Fields

Every log entry must contain these fields:

| Field | Type | Description |
|---|---|---|
| `timestamp` | ISO 8601 UTC string | When the event occurred (e.g., `2026-05-09T17:10:38.123Z`) |
| `level` | String | One of: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` |
| `module` | String | Source module identifier (e.g., `updater`, `licensing`, `admin`, `logging`) |
| `event` | String | Machine-readable event name (e.g., `updater.check.started`) |
| `message` | String | Human-readable description |
| `correlation_id` | String (nullable) | Request/operation correlation identifier |

### Optional Fields

| Field | Type | Description |
|---|---|---|
| `operation_id` | String | Unique ID for a multi-step operation |
| `duration_ms` | Number | Elapsed time for timed operations |
| `error_code` | String | Machine-readable error code from error taxonomy |
| `error_message` | String | Human-readable error description (redacted) |
| `metadata` | Object | Additional key-value context (all values must be redaction-safe) |
| `stack_trace` | String | Stack trace for errors (never in production audit logs; allowed in diagnostic logs) |
| `app_version` | String | Current application version |
| `platform` | String | OS/platform identifier |
| `session_id` | String | Current application session identifier |

### Example Log Entry

```json
{
  "timestamp": "2026-05-09T17:10:38.123Z",
  "level": "INFO",
  "module": "licensing",
  "event": "licensing.validate.success",
  "message": "License validation succeeded",
  "correlation_id": "req-abc-123",
  "operation_id": "op-validate-456",
  "duration_ms": 142,
  "metadata": {
    "license_id_hash": "sha256:a1b2c3...",
    "device_hash": "sha256:d4e5f6...",
    "token_refreshed": true
  },
  "app_version": "2.1.0",
  "session_id": "sess-789"
}
```

---

## Correlation IDs / Operation IDs

### Correlation ID
- A unique identifier assigned at the **start of each user-initiated action or background task**.
- Propagated through all log entries produced during that action.
- Format: any unique string (UUID v4 recommended).
- Purpose: enables reconstructing the full sequence of events for a single user action.

### Operation ID
- A unique identifier for a **specific multi-step operation** (e.g., a complete update-check-download-install cycle).
- Multiple correlation IDs may share the same operation ID if they are part of the same logical workflow.
- Purpose: enables grouping related actions across time.

### Rules
- Every log entry must include a `correlation_id` if one is active.
- The correlation ID must be generated at the entry point and passed through all function calls.
- Background tasks must generate their own correlation IDs.
- Never reuse correlation IDs across unrelated operations.

---

## Local Log Storage

### File Location
- Logs must be stored in the **platform-standard application data directory**.
- Subdirectory: `logs/`
- The log directory must be created automatically on first use.
- The implementation must handle directory creation failure gracefully (log to stderr as fallback).

### File Naming
- Active log file: `app.log`
- Rotated files: `app.YYYY-MM-DD.N.log` where `N` is a sequence number for same-day rotations.
- Crash log file: `crash.log` (separate file for crash-specific data).

### Write Behavior
- Logs must be written asynchronously or via a buffered writer to avoid blocking the main application thread.
- The buffer must be flushed on: application shutdown, `FATAL` log entry, explicit flush call, or buffer reaching capacity.
- Buffer capacity recommendation: 64 KB or 500 entries, whichever is reached first.
- Write failures must not crash the application — degrade to stderr.

---

## Log Rotation

### Policy
- Rotate when the active log file exceeds **10 MB**.
- Rotate on **application startup** if the active log file is from a previous calendar day.
- Maximum rotated files: **10** (configurable).
- When the maximum is reached, delete the oldest rotated file before creating a new one.

### Rotation Process
1. Close the current log file handle.
2. Rename `app.log` to `app.YYYY-MM-DD.N.log`.
3. Open a new `app.log` for writing.
4. If renaming fails, append to the existing file and log a warning to stderr.

---

## Log Retention Policy

- Default retention: **30 days**.
- On application startup, scan the `logs/` directory and delete files older than the retention period.
- Retention period must be configurable.
- Crash logs (`crash.log`) follow the same retention policy but are retained independently.

---

## Sensitive Data Redaction

### Mandatory Redaction Rules

The following data must **never** appear in plaintext in any log entry:

| Data Type | Redaction Method | Example Output |
|---|---|---|
| License keys | Hash with prefix | `key:sha256:a1b2c3...` |
| Device fingerprints | Hash with prefix | `device:sha256:d4e5f6...` |
| API tokens / JWTs | First 8 chars only | `token:eyJhbGci...` |
| Admin signing secrets | Never log, even hashed | `[REDACTED]` |
| User IP addresses | Hash or truncate | `ip:sha256:...` or `ip:192.168.x.x` |
| File system paths containing usernames | Replace username segment | `/home/[USER]/...` |
| Email addresses | Mask local part | `j***@example.com` |
| Passwords / passphrases | Never log | `[REDACTED]` |

### Redaction Implementation
- Provide a `redact(field_name, value)` utility function.
- The redaction function must be used at the **point of log emission**, not at the point of log storage.
- Redaction must be applied before the value enters the logging pipeline — never rely on post-processing.
- Unit tests must verify that no known sensitive field patterns pass through unredacted.

### Redaction Anti-Patterns
- ❌ Logging full request/response bodies without filtering
- ❌ Logging exception messages that may contain user input
- ❌ Logging URLs that contain query parameters with tokens
- ❌ Logging `toString()` of objects that may contain sensitive fields

---

## Privacy-Safe Logging

Beyond redaction, the following privacy rules apply:

1. **No behavioral profiling**: Do not log patterns that could reconstruct user behavior beyond what is needed for diagnostics (e.g., do not log feature usage frequency with user-identifying correlation).
2. **Consent boundary**: If the application operates in a jurisdiction requiring logging consent, provide a configuration flag to disable non-essential diagnostic logging while retaining audit logging.
3. **Data minimization**: Log the minimum information needed. If a hash suffices, do not log even a truncated plaintext value.
4. **No cross-session correlation**: Session IDs must be regenerated on each application launch. Do not use persistent identifiers as session IDs.

---

## Crash Logs

### Capture Requirements
- Register a global unhandled-exception handler at application startup.
- On unhandled exception or panic:
  1. Write a crash log entry to `crash.log` with: timestamp, exception type, message, stack trace, application version, platform, and last 20 log entries from the in-memory buffer.
  2. Attempt to flush all buffered logs.
  3. Exit the application with a non-zero exit code.

### Crash Log Format
```json
{
  "timestamp": "2026-05-09T17:10:38.123Z",
  "level": "FATAL",
  "module": "core",
  "event": "app.crash",
  "message": "Unhandled exception",
  "error_type": "NullPointerException",
  "error_message": "Cannot read property 'x' of null",
  "stack_trace": "at Module.func (file.js:42:10)\n  at ...",
  "app_version": "2.1.0",
  "platform": "windows-x64",
  "recent_log_context": [
    "... last 20 log entries ..."
  ]
}
```

### Recovery on Next Launch
- On startup, check for the existence of `crash.log`.
- If found, log an `INFO` event: `app.crash.recovery_detected`.
- Optionally prompt the user to submit the crash log for support.

---

## Startup and Shutdown Logs

### Startup Sequence
Emit the following events in order during application startup:

1. `app.startup.begin` — first log entry; includes app version, platform, log level
2. `app.config.loaded` — configuration loaded successfully (or `app.config.error`)
3. `app.storage.initialized` — secure storage accessible (or `app.storage.error`)
4. `app.crash.recovery_detected` — only if previous crash log found
5. `app.startup.complete` — application ready for user interaction; includes `duration_ms`

### Shutdown Sequence
1. `app.shutdown.begin` — shutdown initiated (include reason: user, update, error)
2. `app.shutdown.flush` — log buffer flush started
3. `app.shutdown.complete` — final log entry; includes `uptime_ms`

---

## Module-Specific Logging Guidance

### Updater Logs
| Event | Level | When |
|---|---|---|
| `updater.check.started` | INFO | Update check initiated |
| `updater.check.up_to_date` | INFO | No update available |
| `updater.check.available` | INFO | Update found; include target version |
| `updater.check.error` | ERROR | Update check failed |
| `updater.download.started` | INFO | Download initiated |
| `updater.download.progress` | DEBUG | Progress update (throttle to max 1/sec) |
| `updater.download.complete` | INFO | Download finished |
| `updater.download.error` | ERROR | Download failed |
| `updater.verify.started` | INFO | Integrity/signature check started |
| `updater.verify.success` | INFO | Verification passed |
| `updater.verify.failure` | ERROR | Verification failed (critical security event) |
| `updater.install.started` | INFO | Install initiated |
| `updater.install.complete` | INFO | Install finished |
| `updater.install.error` | ERROR | Install failed |
| `updater.rollback.started` | WARN | Rollback initiated |
| `updater.rollback.complete` | INFO | Rollback finished |
| `updater.rollback.error` | FATAL | Rollback failed |

### Licensing Logs
| Event | Level | When |
|---|---|---|
| `licensing.activate.started` | INFO | Activation attempt |
| `licensing.activate.success` | INFO | Activation succeeded |
| `licensing.activate.denied` | WARN | Activation denied; include reason_code |
| `licensing.activate.error` | ERROR | Activation failed (server/network) |
| `licensing.validate.started` | INFO | Validation attempt |
| `licensing.validate.success` | INFO | Validation succeeded |
| `licensing.validate.denied` | WARN | Validation denied; include reason_code |
| `licensing.validate.error` | ERROR | Validation failed |
| `licensing.deactivate.success` | INFO | Deactivation completed |
| `licensing.offline.grace_warning` | WARN | Offline grace period warning threshold reached |
| `licensing.offline.grace_expired` | ERROR | Offline grace period expired; app locked |
| `licensing.token.refreshed` | DEBUG | Token rotated on validation |

### Admin Console Logs
| Event | Level | When |
|---|---|---|
| `admin.session.started` | INFO | Admin credentials entered |
| `admin.session.cleared` | INFO | Credentials cleared/locked |
| `admin.request.sent` | DEBUG | API request dispatched |
| `admin.request.success` | DEBUG | API response received |
| `admin.request.error` | ERROR | API request failed |
| `admin.action.confirmed` | INFO | Destructive action confirmed by operator |
| `admin.action.completed` | INFO | Admin action completed |
| `admin.action.failed` | ERROR | Admin action failed |
| `admin.clipboard.copy` | DEBUG | Sensitive value copied |
| `admin.stale_response.discarded` | DEBUG | Stale response discarded |

### Background Worker Logs
| Event | Level | When |
|---|---|---|
| `worker.scheduled.started` | DEBUG | Scheduled task began |
| `worker.scheduled.completed` | DEBUG | Scheduled task finished |
| `worker.scheduled.error` | ERROR | Scheduled task failed |
| `worker.scheduled.skipped` | DEBUG | Task skipped (e.g., cooldown) |

### Network Failure Logs
| Event | Level | When |
|---|---|---|
| `network.request.timeout` | WARN | Request timed out |
| `network.request.dns_failure` | ERROR | DNS resolution failed |
| `network.request.connection_refused` | ERROR | Connection refused |
| `network.request.ssl_error` | ERROR | TLS/SSL error |
| `network.offline.detected` | WARN | Network connectivity lost |
| `network.online.restored` | INFO | Network connectivity restored |
| `network.retry.attempt` | DEBUG | Retry attempt; include attempt number |

---

## Audit vs Diagnostic Logs

### Audit Logs
- Record **who did what, when, and the outcome** for security-relevant operations.
- Must always be enabled — cannot be turned off by configuration.
- Must include: timestamp, actor (hashed), action, target (hashed), outcome, correlation_id.
- Retention: governed by organizational policy (minimum 90 days recommended).
- Examples: license activation, revocation, admin actions, device reset.

### Diagnostic Logs
- Record **technical details** for debugging and support.
- Can be adjusted by log level configuration.
- May include stack traces, performance metrics, internal state.
- Retention: governed by the standard log retention policy.
- Examples: download progress, config parsing, UI state changes.

### Separation
- Audit events must be flagged with `"log_type": "audit"` in metadata.
- Diagnostic events use `"log_type": "diagnostic"` (default).
- Implementations may optionally write audit and diagnostic logs to separate files.

---

## Exporting Logs for Support

### Export Requirements
- Provide a user-accessible "Export Logs" function.
- The export must:
  1. Collect all log files within the retention window.
  2. Apply an **additional redaction pass** to catch any missed sensitive data.
  3. Package logs into a single archive (ZIP or equivalent).
  4. Include a manifest file listing: export date, app version, platform, log file count, date range.
  5. Save the archive to a user-chosen location.
- The export must not include crash logs unless the user explicitly opts in.
- Maximum export size: configurable, default 50 MB.

### Export Log Event
- `logging.export.started` (INFO)
- `logging.export.complete` (INFO) — include file count and archive size
- `logging.export.error` (ERROR)

---

## Optional Remote Log Upload

> [!NOTE]
> Remote log upload is an optional extension. If implemented, the following rules apply.

- Upload must require **explicit user consent** per upload session.
- Only diagnostic logs may be uploaded; audit logs remain local unless organizational policy requires otherwise.
- An additional redaction pass must be applied before upload.
- Transport: HTTPS only, with certificate pinning recommended.
- Upload failures must not affect application behavior.
- Log event: `logging.upload.started`, `logging.upload.complete`, `logging.upload.error`

---

## Offline Logging Behavior

- All logging must function fully offline — local file storage is the primary target.
- If optional remote upload is configured, queue uploads and retry when connectivity is restored.
- The logging system must never block or fail due to network unavailability.
- Offline periods must not cause log data loss (within disk space limits).

---

## Log Integrity Considerations

For audit logs where tamper evidence is important:

1. **Chained hashing** (optional): Each audit log entry includes a hash of the previous entry, creating a chain that detects deletion or insertion.
2. **File-level checksums**: On rotation, compute and store a checksum of the completed log file in a separate `.sha256` manifest file.
3. **Read-only after rotation**: Rotated log files should be set to read-only permissions where the OS supports it.

> [!IMPORTANT]
> These are recommendations for high-security deployments. At minimum, implementations must set rotated log files to read-only.

---

## Log Configuration

### Configuration Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `log_level` | String | `INFO` | Minimum active log level |
| `log_dir` | Path | Platform app data + `/logs/` | Log file directory |
| `max_file_size_mb` | Number | 10 | Maximum size before rotation |
| `max_rotated_files` | Number | 10 | Maximum number of rotated files |
| `retention_days` | Number | 30 | Days to retain rotated files |
| `buffer_size_kb` | Number | 64 | Write buffer size |
| `enable_console_output` | Boolean | `false` (prod) / `true` (dev) | Also write to stdout/stderr |
| `enable_audit_separation` | Boolean | `false` | Write audit logs to separate file |
| `redaction_mode` | String | `strict` | `strict` (hash+mask), `relaxed` (truncate), `off` (dev only) |

### Configuration Sources (Priority Order)
1. Runtime override (e.g., debug menu, environment variable)
2. Configuration file
3. Compiled defaults

### Hot Reloading
- `log_level` and `enable_console_output` must be changeable at runtime without restart.
- Other parameters may require restart.

---

## Performance Considerations

1. **Async I/O**: All file writes must be non-blocking. Use a dedicated logging thread or async writer.
2. **Throttling**: High-frequency events (e.g., download progress) must be throttled to a maximum emission rate (recommendation: 1 per second for progress events).
3. **Buffer management**: Flush on shutdown, fatal errors, and buffer capacity — not on every log call.
4. **String allocation**: Avoid string formatting for log entries below the active log level. Check the level before constructing the log entry.
5. **Serialization**: Use a fast JSON serializer. Avoid reflection-based serialization in hot paths.
6. **Benchmarks**: Logging overhead must not exceed 1% of total application CPU time under normal operation. Measure during development.

---

## Developer Usage Rules

### Do
- ✅ Use the structured logging API for all log output
- ✅ Include `correlation_id` in every log entry where one is active
- ✅ Use the `redact()` utility for any potentially sensitive value
- ✅ Use the event name catalog — do not invent ad-hoc event names
- ✅ Log the outcome (success/failure) of every operation, not just failures
- ✅ Include `duration_ms` for any timed operation
- ✅ Log at the appropriate level — `INFO` for normal operations, not `DEBUG`

### Don't (Anti-Patterns)
- ❌ **Never** use `print`, `console.log`, or unstructured string output in production code
- ❌ **Never** log sensitive data — even "temporarily" for debugging
- ❌ **Never** log inside tight loops without throttling
- ❌ **Never** catch and swallow exceptions without logging them
- ❌ **Never** use log levels inconsistently (e.g., `ERROR` for expected conditions)
- ❌ **Never** include stack traces in `INFO` or `WARN` level entries
- ❌ **Never** log full request/response bodies without redaction
- ❌ **Never** use mutable state in log metadata objects (risk of logging stale data)

---

## Testing Requirements

### Unit Tests
- [ ] Log entries contain all required fields
- [ ] Log level filtering works correctly (entries below active level are not emitted)
- [ ] Redaction correctly masks all sensitive field types
- [ ] Correlation ID is propagated through nested function calls
- [ ] Timestamp format is valid ISO 8601 UTC
- [ ] Event names match the catalog exactly
- [ ] Buffer flush produces complete, parseable log entries

### Integration Tests
- [ ] Log rotation triggers at the configured file size
- [ ] Rotated files follow the naming convention
- [ ] Retention cleanup deletes files older than the configured period
- [ ] Export produces a valid archive with manifest
- [ ] Crash log is written on unhandled exception
- [ ] Startup and shutdown log sequences are complete and ordered

### Performance Tests
- [ ] Logging 10,000 entries per second does not block the main thread for more than 1ms
- [ ] Log level check for suppressed entries costs less than 1 microsecond
- [ ] Buffer flush completes within 100ms under normal load

### Security Tests
- [ ] No sensitive data appears in any log file after a full application workflow
- [ ] Rotated log files have read-only permissions
- [ ] Crash logs do not contain plaintext secrets
- [ ] Export redaction pass catches data missed by inline redaction

---

## Portability Checklist

When implementing this module in any language or framework:

1. [ ] Preserve the 6-level log hierarchy (TRACE through FATAL)
2. [ ] Preserve the structured JSON format with all required fields
3. [ ] Preserve correlation ID propagation semantics
4. [ ] Preserve the redaction rules and `redact()` utility pattern
5. [ ] Preserve file-based local storage with rotation and retention
6. [ ] Preserve the audit vs diagnostic log distinction
7. [ ] Preserve the export functionality with manifest
8. [ ] Preserve crash log capture and recovery detection
9. [ ] Preserve all event names from the event catalog
10. [ ] Preserve the configuration parameters and hot-reload behavior

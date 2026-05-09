# Enhanced Logging — Event Catalog

This document is the canonical catalog of all log event names emitted across all modules. Implementations must use these exact event names. Do not invent ad-hoc event names.

Cross-reference: `docs/enhanced-logging-module-guide.md` for format, levels, and redaction rules.

---

## Event Naming Convention

- Format: `<module>.<component>.<action>`
- All lowercase with dots as separators
- Example: `updater.download.started`
- Maximum depth: 3 segments (module.component.action)
- Actions should use past-tense or present-tense verbs: `started`, `completed`, `failed`, `detected`, `skipped`

---

## Core / Application Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `app.startup.begin` | INFO | diagnostic | Application startup initiated | `app_version`, `platform`, `log_level` |
| `app.startup.complete` | INFO | diagnostic | Application ready | `duration_ms` |
| `app.config.loaded` | INFO | diagnostic | Configuration loaded | — |
| `app.config.error` | ERROR | diagnostic | Configuration failed to load | `error_code`, `error_message` |
| `app.storage.initialized` | INFO | diagnostic | Secure storage accessible | — |
| `app.storage.error` | ERROR | diagnostic | Secure storage unavailable | `error_code`, `error_message` |
| `app.crash` | FATAL | audit | Unhandled exception | `error_type`, `error_message`, `stack_trace` |
| `app.crash.recovery_detected` | INFO | diagnostic | Previous crash log found on startup | — |
| `app.shutdown.begin` | INFO | diagnostic | Shutdown initiated | `reason` (`user` / `update` / `error`) |
| `app.shutdown.flush` | DEBUG | diagnostic | Log buffer flush started | — |
| `app.shutdown.complete` | INFO | diagnostic | Final log entry | `uptime_ms` |

---

## Updater Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `updater.check.started` | INFO | diagnostic | Update check initiated | `current_version` |
| `updater.check.up_to_date` | INFO | diagnostic | No update available | `current_version` |
| `updater.check.available` | INFO | diagnostic | Update found | `current_version`, `target_version` |
| `updater.check.cooldown_skipped` | DEBUG | diagnostic | Check skipped due to cooldown | `cooldown_remaining_ms` |
| `updater.check.error` | ERROR | diagnostic | Update check failed | `error_code`, `error_message` |
| `updater.download.started` | INFO | diagnostic | Download initiated | `target_version`, `expected_size_bytes` |
| `updater.download.progress` | DEBUG | diagnostic | Download progress (max 1/sec) | `downloaded_bytes`, `total_bytes`, `percent` |
| `updater.download.complete` | INFO | diagnostic | Download finished | `target_version`, `duration_ms`, `size_bytes` |
| `updater.download.error` | ERROR | diagnostic | Download failed | `error_code`, `error_message`, `retry_count` |
| `updater.verify.started` | INFO | audit | Integrity check started | `target_version` |
| `updater.verify.success` | INFO | audit | Verification passed | `target_version` |
| `updater.verify.failure` | ERROR | audit | Verification failed — **security event** | `target_version`, `failure_reason` |
| `updater.install.started` | INFO | audit | Install initiated | `target_version` |
| `updater.install.complete` | INFO | audit | Install finished | `target_version`, `duration_ms` |
| `updater.install.error` | ERROR | audit | Install failed | `error_code`, `error_message` |
| `updater.install.restart_required` | INFO | diagnostic | Restart needed to complete | — |
| `updater.rollback.started` | WARN | audit | Rollback initiated | `target_version`, `reason` |
| `updater.rollback.complete` | INFO | audit | Rollback finished | `rolled_back_to_version` |
| `updater.rollback.error` | FATAL | audit | Rollback failed | `error_code`, `error_message` |

---

## Licensing Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `licensing.activate.started` | INFO | audit | Activation attempt | `license_key_hash`, `device_hash` |
| `licensing.activate.success` | INFO | audit | Activation succeeded | `license_id_hash`, `device_hash` |
| `licensing.activate.denied` | WARN | audit | Activation denied | `reason_code` |
| `licensing.activate.error` | ERROR | diagnostic | Activation failed (network/server) | `error_code`, `error_message` |
| `licensing.validate.started` | INFO | diagnostic | Validation attempt | — |
| `licensing.validate.success` | INFO | diagnostic | Validation succeeded | `token_refreshed` (boolean) |
| `licensing.validate.denied` | WARN | audit | Validation denied | `reason_code` |
| `licensing.validate.error` | ERROR | diagnostic | Validation failed | `error_code`, `error_message` |
| `licensing.deactivate.started` | INFO | audit | Deactivation attempt | — |
| `licensing.deactivate.success` | INFO | audit | Deactivation completed | — |
| `licensing.deactivate.error` | ERROR | diagnostic | Deactivation failed | `error_code`, `error_message` |
| `licensing.offline.grace_active` | DEBUG | diagnostic | Operating in offline grace period | `days_remaining` |
| `licensing.offline.grace_warning` | WARN | diagnostic | Grace period warning threshold | `days_remaining` |
| `licensing.offline.grace_expired` | ERROR | audit | Grace period expired; app locked | — |
| `licensing.token.refreshed` | DEBUG | diagnostic | Token rotated | — |
| `licensing.token.expired` | WARN | diagnostic | Token expired, revalidation needed | — |
| `licensing.rate_limited` | WARN | diagnostic | Rate limit encountered | `retry_after_ms` |

---

## Admin Console Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `admin.session.started` | INFO | audit | Admin credentials entered | `key_id` |
| `admin.session.cleared` | INFO | audit | Credentials cleared | — |
| `admin.request.sent` | DEBUG | diagnostic | API request dispatched | `method`, `path` |
| `admin.request.success` | DEBUG | diagnostic | API response received | `method`, `path`, `status_code`, `duration_ms` |
| `admin.request.error` | ERROR | diagnostic | API request failed | `method`, `path`, `error_code`, `error_message` |
| `admin.action.confirmed` | INFO | audit | Destructive action confirmed | `action_type`, `target_hash` |
| `admin.action.completed` | INFO | audit | Admin action completed | `action_type`, `target_hash` |
| `admin.action.failed` | ERROR | audit | Admin action failed | `action_type`, `error_code` |
| `admin.license.created` | INFO | audit | License created via admin | `license_id_hash` |
| `admin.license.revoked` | INFO | audit | License revoked via admin | `license_id_hash` |
| `admin.device.reset` | INFO | audit | Device binding reset | `license_id_hash` |
| `admin.clipboard.copy` | DEBUG | diagnostic | Sensitive value copied | — |
| `admin.stale_response.discarded` | DEBUG | diagnostic | Stale response discarded | `request_type` |

---

## Background Worker Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `worker.scheduled.started` | DEBUG | diagnostic | Scheduled task began | `task_name` |
| `worker.scheduled.completed` | DEBUG | diagnostic | Scheduled task finished | `task_name`, `duration_ms` |
| `worker.scheduled.error` | ERROR | diagnostic | Scheduled task failed | `task_name`, `error_code`, `error_message` |
| `worker.scheduled.skipped` | DEBUG | diagnostic | Task skipped (cooldown/policy) | `task_name`, `reason` |

---

## Network Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `network.request.timeout` | WARN | diagnostic | Request timed out | `url_host`, `duration_ms` |
| `network.request.dns_failure` | ERROR | diagnostic | DNS resolution failed | `url_host` |
| `network.request.connection_refused` | ERROR | diagnostic | Connection refused | `url_host`, `port` |
| `network.request.ssl_error` | ERROR | diagnostic | TLS/SSL error | `url_host`, `error_message` |
| `network.offline.detected` | WARN | diagnostic | Network connectivity lost | — |
| `network.online.restored` | INFO | diagnostic | Network connectivity restored | `offline_duration_ms` |
| `network.retry.attempt` | DEBUG | diagnostic | Retry attempt | `attempt_number`, `max_attempts`, `backoff_ms` |

---

## Logging System Events

| Event Name | Level | Log Type | Description | Required Metadata |
|---|---|---|---|---|
| `logging.initialized` | INFO | diagnostic | Logging system ready | `log_level`, `log_dir` |
| `logging.level_changed` | INFO | diagnostic | Log level changed at runtime | `old_level`, `new_level` |
| `logging.rotation.completed` | DEBUG | diagnostic | Log file rotated | `old_file`, `new_file`, `old_file_size_bytes` |
| `logging.retention.cleanup` | DEBUG | diagnostic | Old log files deleted | `deleted_count`, `freed_bytes` |
| `logging.export.started` | INFO | diagnostic | Log export started | — |
| `logging.export.complete` | INFO | diagnostic | Log export finished | `file_count`, `archive_size_bytes` |
| `logging.export.error` | ERROR | diagnostic | Log export failed | `error_code`, `error_message` |
| `logging.buffer.flush` | DEBUG | diagnostic | Buffer flushed | `entry_count` |
| `logging.write.error` | ERROR | diagnostic | Failed to write to log file | `error_message` |

---

## Custom Event Guidelines

If you need to add events not in this catalog:

1. Follow the naming convention: `<module>.<component>.<action>`
2. Assign the correct log level and log type (audit vs diagnostic)
3. Document the event in this catalog before implementing
4. Include all required metadata fields
5. Apply redaction rules to all metadata values
6. Obtain review approval for new audit-type events

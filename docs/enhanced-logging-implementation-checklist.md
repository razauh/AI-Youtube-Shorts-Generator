# Enhanced Logging — Implementation Checklist

Guide reference: `docs/enhanced-logging-module-guide.md`
Event catalog reference: `docs/enhanced-logging-event-catalog.md`

Rule: every item below must be implemented in compliance with the guide. Mark items with evidence links.

---

## Phase 1 — Foundation

- [ ] Create logging module directory/package in project structure.
- [ ] Define log level enum/constants (TRACE, DEBUG, INFO, WARN, ERROR, FATAL).
- [ ] Implement structured log entry builder with all required fields (`timestamp`, `level`, `module`, `event`, `message`, `correlation_id`).
- [ ] Implement JSON serializer for log entries.
- [ ] Implement correlation ID generator (UUID v4 or equivalent).
- [ ] Implement correlation ID context propagation (thread-local, async context, or explicit parameter passing).
- [ ] Implement log level filtering (suppress entries below active level).
- [ ] Implement level check before entry construction (performance optimization).

## Phase 2 — Redaction and Privacy

- [ ] Implement `redact(field_name, value)` utility function.
- [ ] Implement redaction rules for: license keys, device fingerprints, API tokens, admin secrets, IP addresses, file paths, email addresses, passwords.
- [ ] Verify redaction is applied at point of emission (not post-processing).
- [ ] Implement privacy-safe session ID generation (regenerated per launch).
- [ ] Add unit tests proving no sensitive data passes through unredacted.

## Phase 3 — Storage and I/O

- [ ] Implement platform-appropriate log directory detection and creation.
- [ ] Implement buffered asynchronous file writer.
- [ ] Configure buffer flush triggers: shutdown, FATAL, capacity, explicit flush.
- [ ] Implement write failure fallback to stderr.
- [ ] Implement active log file naming (`app.log`).
- [ ] Implement rotated file naming (`app.YYYY-MM-DD.N.log`).
- [ ] Implement crash log file (`crash.log`) separate from main log.

## Phase 4 — Rotation and Retention

- [ ] Implement size-based rotation (trigger at configured `max_file_size_mb`).
- [ ] Implement date-based rotation on startup (if active file is from previous day).
- [ ] Implement maximum rotated file enforcement (delete oldest when limit reached).
- [ ] Implement retention policy cleanup on startup (delete files older than `retention_days`).
- [ ] Set rotated files to read-only permissions.

## Phase 5 — Crash Logging

- [ ] Register global unhandled-exception handler.
- [ ] Implement crash log writer with: timestamp, error type, message, stack trace, version, platform, last 20 buffered entries.
- [ ] Implement crash recovery detection on startup.
- [ ] Emit `app.crash.recovery_detected` on startup when previous crash log found.

## Phase 6 — Lifecycle Logging

- [ ] Emit all startup events in order: `app.startup.begin`, `app.config.loaded`, `app.storage.initialized`, `app.startup.complete`.
- [ ] Emit all shutdown events in order: `app.shutdown.begin`, `app.shutdown.flush`, `app.shutdown.complete`.
- [ ] Include `duration_ms` in `app.startup.complete` and `uptime_ms` in `app.shutdown.complete`.

## Phase 7 — Module Integration

- [ ] Integrate logging into Updater module (all events from event catalog).
- [ ] Integrate logging into Licensing module (all events from event catalog).
- [ ] Integrate logging into Admin Console module (all events from event catalog).
- [ ] Integrate logging into background worker/scheduler (all events from event catalog).
- [ ] Integrate logging into network client (all events from event catalog).
- [ ] Verify all event names match the catalog exactly (no ad-hoc names).

## Phase 8 — Audit vs Diagnostic Separation

- [ ] Add `log_type` field to log entries (`audit` / `diagnostic`).
- [ ] Flag all security-relevant events as `audit` per the event catalog.
- [ ] Verify audit logging cannot be disabled by configuration.
- [ ] Optionally implement separate audit log file output.

## Phase 9 — Export

- [ ] Implement user-accessible "Export Logs" function.
- [ ] Implement additional redaction pass during export.
- [ ] Implement archive creation (ZIP or equivalent) with manifest file.
- [ ] Implement crash log opt-in for export.
- [ ] Emit `logging.export.started`, `logging.export.complete`, `logging.export.error` events.

## Phase 10 — Configuration

- [ ] Implement all configuration parameters from guide (log_level, log_dir, max_file_size_mb, etc.).
- [ ] Implement configuration source priority: runtime override > config file > defaults.
- [ ] Implement hot reloading for `log_level` and `enable_console_output`.
- [ ] Set production defaults: level=INFO, console=false, redaction=strict.
- [ ] Set development defaults: level=DEBUG, console=true, redaction=strict.

## Phase 11 — Performance Verification

- [ ] Verify all file writes are non-blocking (async I/O or dedicated thread).
- [ ] Implement throttling for high-frequency events (max 1/sec for progress events).
- [ ] Verify level check short-circuits before string allocation.
- [ ] Benchmark: 10,000 entries/sec does not block main thread for >1ms.
- [ ] Benchmark: logging overhead <1% CPU under normal operation.

## Phase 12 — Testing

- [ ] Unit tests: all required fields present in entries.
- [ ] Unit tests: level filtering suppresses correctly.
- [ ] Unit tests: redaction masks all sensitive field types.
- [ ] Unit tests: correlation ID propagation works.
- [ ] Unit tests: event names match catalog.
- [ ] Integration tests: rotation triggers correctly.
- [ ] Integration tests: retention cleanup works.
- [ ] Integration tests: export produces valid archive.
- [ ] Integration tests: crash log is written on exception.
- [ ] Integration tests: startup/shutdown sequences are complete.
- [ ] Security tests: no sensitive data in log files after full workflow.
- [ ] Security tests: rotated files are read-only.
- [ ] Performance tests: throughput and latency meet benchmarks.

---

## Evidence Links (required for every checked item)

- PR / commit link(s):
- CI workflow run URL:
- Unit test command + output artifact:
- Integration test command + output artifact:
- Performance benchmark results:
- Security scan results:

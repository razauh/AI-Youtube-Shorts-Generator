# Shared Test Strategy (Language-Agnostic Guide)

This document defines the testing strategy and requirements across all modules. Each module guide references this document for its test expectations.

---

## Test Pyramid

```
        ┌───────────┐
        │  Security  │   ← Focused, high-value
        ├───────────┤
        │    E2E     │   ← Few, critical user paths
        ├───────────┤
        │Integration │   ← Module boundaries
        ├───────────┤
        │   Unit     │   ← Majority of tests
        └───────────┘
```

| Layer | Proportion | Scope | Speed |
|---|---|---|---|
| **Unit** | ~60% | Single function/class, no I/O | < 10ms each |
| **Integration** | ~25% | Module boundaries, database, HTTP mocks | < 2s each |
| **E2E** | ~10% | Full user workflows | < 30s each |
| **Security** | ~5% | Targeted penetration/verification | Varies |

---

## Unit Test Requirements

### Coverage Targets
- Minimum line coverage: **80%** per module.
- Critical paths (security, state transitions, error handling): **100%** branch coverage.

### What to Unit Test
- All pure functions (hashing, version comparison, serialization, formatting)
- State machine transitions (valid and invalid transitions)
- Error code mapping (every error code produces the correct user message)
- Redaction functions (every sensitive field type is correctly masked)
- Validation logic (valid input passes, invalid input produces correct error code)
- Configuration parsing and defaults

### What NOT to Unit Test
- Framework/library internals
- Simple getters/setters without logic
- UI layout (use integration or visual tests instead)

---

## Integration Test Requirements

### Scope
- API endpoint behavior: request → processing → response
- Database operations: write → read → verify
- Storage operations: save → load → verify
- Network client: mock server → request → verify retry/timeout behavior
- Module interactions: updater → logger, licensing → storage

### Test Fixtures
- Use in-memory or temporary databases for database tests.
- Use HTTP mock servers for network tests (never call real external services in tests).
- Use temporary directories for file storage tests.
- Clean up all test artifacts after each test.

### What to Integration Test
- Each API endpoint's success and failure paths
- Database constraint enforcement (unique device binding, etc.)
- Concurrent access scenarios (race conditions)
- Storage round-trips (save → corrupt → recovery)
- Offline/online transitions

---

## E2E Test Requirements

### Critical User Paths to Cover

| Module | Path | Description |
|---|---|---|
| **Updater** | Check → download → verify → install | Complete update lifecycle |
| **Updater** | Check → no update | Up-to-date scenario |
| **Licensing** | Activate → validate → use app | Happy path |
| **Licensing** | Activate → go offline → use within grace → come online → validate | Offline grace path |
| **Licensing** | Activate on device A → try device B → denied | Device binding enforcement |
| **Admin** | Login → list licenses → revoke → verify | Admin lifecycle |
| **Admin** | Login → create license → reveal key | License creation |

### E2E Test Rules
- Tests must be repeatable and independent (no shared state between tests).
- Tests must clean up created resources.
- Tests must handle timing variability (use polling with timeout, not fixed delays).

---

## Security Test Requirements

### Per-Module Security Tests

| Module | Test | Description |
|---|---|---|
| **Updater** | Tampered artifact rejection | Modified binary fails hash check |
| **Updater** | Invalid signature rejection | Wrong key fails signature check |
| **Updater** | Manifest integrity | Modified manifest is detected |
| **Licensing** | Token tampering | Modified JWT is rejected |
| **Licensing** | Expired token rejection | Expired tokens fail validation |
| **Licensing** | Nonce replay rejection | Reused admin nonces are rejected |
| **Licensing** | No plaintext secrets in DB | Scan all tables for raw keys/fingerprints |
| **Licensing** | No plaintext secrets in logs | Scan log output for sensitive patterns |
| **Admin** | Credential memory clearing | After lock, credentials are not in memory |
| **Admin** | No secrets in logs | Scan log output for admin secrets |
| **Logging** | Redaction completeness | All sensitive field types are masked |
| **Logging** | Crash log safety | Crash logs contain no plaintext secrets |

### Sensitive Data Scanning
- Maintain a list of regex patterns for known sensitive data formats (JWT patterns, key patterns, UUID patterns used for secrets).
- Run pattern scan against all log files and database dumps as part of CI.

---

## Test Naming Convention

```
test_<module>_<component>_<scenario>_<expected_outcome>
```

Examples:
- `test_licensing_activate_valid_key_returns_token`
- `test_licensing_activate_second_device_returns_device_conflict`
- `test_updater_verify_tampered_hash_rejects`
- `test_admin_signing_stale_timestamp_rejects`
- `test_logging_redact_license_key_returns_hash`

---

## CI/CD Integration

- All unit and integration tests must run on every pull request.
- E2E tests must run on merge to main/release branches.
- Security tests must run on every pull request.
- Test results must be archived as CI artifacts.
- Failed security tests must block merge.

---

## Test Data and Fixtures

### Deterministic Test Data
- Use fixed test license keys, device fingerprints, and tokens for reproducibility.
- Never use production data in tests.
- Document all test fixtures in a shared test data file per module.

### Mock Servers
- Use lightweight HTTP mock servers (not full server deployments).
- Mock servers must support: configurable response delays, error responses, and request recording.
- Mock server behavior must be documented inline with the test.

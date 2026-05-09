# Licensing Worker Module (Language-Agnostic Guide)

## Purpose
The Licensing Worker is the central license authority for desktop clients. It activates licenses, validates entitlements/device binding, supports deactivation, and exposes admin operations.
Canonical contract reference: `docs/licensing-api-contract-matrix.md` for endpoint/auth/offline/reason-code rules.

## High-Level Architecture
1. HTTP entrypoint/router
- Route dispatch for public license endpoints and protected admin endpoints.
- Method enforcement and standardized `404/405` behavior.
- CORS handling with optional allowlist.
- Global error capture and safe failure responses.
- Edge-route handling details:
  - `/v1/admin/licenses/{licenseKey}` is valid only for a single non-empty path segment.
  - `/v1/admin/licenses/` (trailing slash without key) resolves to not found.
  - `/v1/admin/licenses/revoke` is a dedicated route and not treated as `{licenseKey}`.
  - Invalid URL-encoding in `{licenseKey}` is rejected as a validation error.

2. Public license routes
Canonical source: `docs/licensing-api-contract-matrix.md` (Endpoint Matrix section).

- `POST /v1/license/activate`
- `POST /v1/license/validate`
- `POST /v1/license/deactivate`

3. Status and admin routes
Canonical source: `docs/licensing-api-contract-matrix.md` (Endpoint Matrix section).

- `GET /v1/license/status` (public, token-authenticated self status only)
- `POST /v1/admin/license/reset-device`
- `POST /v1/admin/license/actions`
- `GET /v1/admin/license/actions/history`
- `GET /v1/admin/licenses`
- `GET /v1/admin/activations`
- `GET /v1/admin/licenses/{licenseKey}`
- `POST /v1/admin/licenses`
- `PUT|PATCH /v1/admin/licenses`
- `POST /v1/admin/licenses/revoke`
- `GET /v1/admin/metrics`
- `GET /v1/admin/license/status` (admin lookup by explicit license identifier; no alias with public route)
- `GET /v1/admin/licenses/` (trailing slash only) -> not found behavior by design

## Core Service Components
1. Activation service
- Validates input payload.
- Applies per-IP and per-license-key rate limits.
- Hashes incoming license key + device fingerprint.
- Verifies entitlement with billing provider.
- Creates or updates license record.
- Attempts single-device binding.
- Issues signed token on success.
- Emits activation events and metrics.

2. Validation service
- Validates bearer token signature and payload.
- Hashes request device fingerprint and enforces device match.
- Confirms license record exists and is not revoked/suspended.
- Confirms active (non-revoked) activation record.
- Touches `last_seen` timestamps.
- Rotates/refreshes token on success.

3. Deactivation service
- Validates bearer token.
- Revokes active activation binding.
- Emits audit activation event.

4. Admin license-management service
- Reset device binding.
- Revoke/suspend/unsuspend license.
- Reissue license key hash.
- Delete/force-unbind-delete flows.
- Add admin notes.
- Record all admin actions to audit log.

## Security Components
1. Public token subsystem
- Token issue/verify routines.
- Payload includes license ID and bound device hash.
- Supports key configuration + active key ID for rotation.
- Mandatory token standard:
  - JWT with asymmetric signing (`EdDSA`/`Ed25519` preferred, `ES256` acceptable fallback).
  - Validate `alg` and `kid` strictly.
  - Reject expired tokens and future-issued tokens beyond allowed skew.

2. Admin request authentication
- Verifies timestamp, nonce, key ID, and signature headers.
- Reconstructs canonical payload from method/path/query/body hash.
- Rejects stale timestamps, invalid signatures, and replayed nonces.

3. Hashing subsystem
- Keyed hashing for:
  - License keys at rest.
  - Device fingerprints at rest.
  - Rate-limit keys.
- Prevents plaintext persistence of sensitive identifiers.

4. Provider reference sealing
- Encrypt/decrypt helper for provider references.
- Supports key rotation patterns for encrypted provider metadata.

5. Rate limiting subsystem
- Namespace-based counters (activate/validate scopes).
- Distinct keys by IP, license key, and device fingerprint.
- Sliding/fixed window enforcement with retry-after semantics.

## Data & Persistence Components
1. Primary tables
- `licenses`
- `activations`
- `activation_events`
- `admin_actions`
- `request_limits`
- `admin_nonces`

2. License schema fields
- Identity: `id`, `license_key_hash`
- Billing: `provider`, `provider_ref`, `plan_id`
- Status/lifecycle: `status`, `tier`, `expires_at`
- Timestamps: `created_at`, `updated_at`

3. Activation schema fields
- `license_id` (single active binding semantics)
- `device_hash`, `install_id`
- `first_activated_at`, `last_seen_at`, `revoked_at`

4. Repository layer responsibilities
- Lookup by hash/ID.
- Upsert license + entitlement metadata.
- Atomic bind/rebind rules for device activation.
- Activation revoke/touch operations.
- Event and admin action inserts.
- Admin list/filter/pagination queries.
- Metrics aggregation queries.

## Billing Provider Abstraction
1. Provider interface
- `verifyEntitlement(licenseKey)` returning validity/status/plan data.

2. Implementations
- External provider adapter (e.g., Gumroad).
- Internal/test provider adapter.
- Provider factory selected by runtime config.

## Validation & Response Utilities
1. Input parsing
- Strict JSON parsing.
- Typed/shape checks for activate/validate payloads.
- Positive integer/date parsing for admin payloads.

2. Unified response envelope
- Success: `{ ok: true, data: ... }`
- Failure: `{ ok: false, error: { code, message, reason_code } }`

3. Reason code taxonomy
Canonical source: `docs/licensing-api-contract-matrix.md` (Reason Code Taxonomy section).

- `INVALID_KEY`
- `SUBSCRIPTION_INACTIVE`
- `KEY_BOUND_TO_OTHER_DEVICE`
- `REVOKED`
- `RATE_LIMITED`
- `SERVER_ERROR`

## Observability Components
1. Structured logger
- Consistent event names per route/outcome.
- Context metadata: path, method, reason code, counts, etc.

2. Metrics emitter
- Counter increments for:
  - rate-limit hits
  - denied validations/activations
  - revoked/device-conflict incidents
  - exceptions

## Configuration Components
1. Runtime env config
- Billing mode/provider.
- Token TTL and key settings.
- Hash secrets/peppers.
- Admin HMAC secret + key ID.
- Rate-limit thresholds/window.
- CORS allowed origins.

2. Deployment config
- Environment-specific bindings.
- DB identifiers.
- Secret management for staging/production.

## Test Components
- Unit tests: hashing/token/sealing/admin-auth.
- Integration tests: route/service behavior.
- Race tests: activation conflict and rate-limit concurrency behavior.
- Test utilities: fake database helpers.

## Critical Behavioral Invariants
1. One license maps to one active device binding at a time.
2. Validation must deny token-device mismatch.
3. Admin operations must be signed and replay-protected.
4. All sensitive identifiers are stored hashed or sealed.
5. All policy-relevant transitions are audit logged.

## Portability Checklist
1. Preserve endpoint contracts + response envelope.
2. Preserve admin signature and nonce validation semantics.
3. Preserve one-device binding conflict rules.
4. Preserve reason-code taxonomy for client compatibility.
5. Preserve audit/event logging and rate-limit behavior.

---

## License Lifecycle State Machine

```
                    ┌─────────────┐
        create ───►│   active     │◄─── unsuspend
                    └──┬──┬──┬────┘
                       │  │  │
          suspend ─────┘  │  └───── revoke ──────┐
                          │                      │
                   cancel/expire                 ▼
                          │              ┌───────────┐
                          ▼              │  revoked   │ (terminal)
                   ┌──────────────┐      └───────────┘
                   │  cancelled/  │
                   │  expired     │
                   └──────┬───────┘
                          │
                   refund │
                          ▼
                   ┌──────────┐
                   │ refunded │ (terminal)
                   └──────────┘
```

### State Definitions

| State | Description | Allowed Transitions |
|---|---|---|
| `active` | License is valid and entitlement is current | → `suspended`, `cancelled`, `expired`, `revoked`, `refunded` |
| `past_due` | Payment is overdue but grace period active | → `active` (on payment), `cancelled`, `revoked` |
| `suspended` | Temporarily disabled by admin action | → `active` (unsuspend), `revoked` |
| `cancelled` | Subscription cancelled by user or provider | → `revoked` |
| `expired` | Time-limited license has expired | → `active` (on renewal), `revoked` |
| `revoked` | Permanently disabled (terminal state) | None |
| `refunded` | Payment refunded (terminal state) | None |

### Transition Rules
- Only `active` and `past_due` states allow successful validation.
- `suspended` state denies validation with reason code `REVOKED`.
- Terminal states (`revoked`, `refunded`) can never transition to any other state.
- All state transitions must be recorded in `activation_events` as audit entries.
- Provider adapter state mapping must be documented per provider.

---

## Clock Manipulation Defenses

Desktop clients enforcing offline grace periods are vulnerable to system clock manipulation. The following defenses must be implemented:

### Server-Side Defenses
1. **Canonical time**: All timestamps in tokens and responses use server UTC time, not client time.
2. **Token `iat` and `exp`**: Issue tokens with server-generated `iat` (issued-at) and `exp` (expiry). Reject tokens with `iat` in the future beyond a small skew (recommended: 60 seconds).
3. **`last_seen_at` progression**: On each validation, verify that the client's `last_seen_at` has not moved backward. If it has moved backward by more than the allowed skew, flag the activation for review and deny with reason code `REVOKED`.

### Client-Side Defenses
1. **Monotonic clock for intervals**: Use monotonic clock (not wall clock) for measuring elapsed time since last validation. This prevents clock rollback from extending grace periods.
2. **Server timestamp anchoring**: Store the server-provided timestamp from the last successful validation. Compare elapsed time using `max(monotonic_elapsed, wall_clock_elapsed)` — use whichever is larger to prevent both clock rollback and sleep/hibernate manipulation.
3. **Startup time check**: On app startup, if the current wall clock time is earlier than the stored `last_validated_at`, log a warning (`licensing.clock.rollback_detected`) and force an online validation before allowing offline grace.

### Anti-Tampering
- Store `last_validated_at` and `grace_until` in secure storage (OS keychain or encrypted file), not in user-accessible plaintext configuration.
- If secure storage is tampered with or unreadable, treat as first-launch and require fresh activation.

---

## Token Refresh Flow

### When Tokens Are Refreshed
- Tokens are refreshed on every successful `/validate` call.
- The server issues a new token with a new `iat` and `exp` while maintaining the same `license_id` and `device_hash` claims.
- The old token is not explicitly revoked — it simply expires based on its original `exp`.

### Refresh Mechanics
1. Client calls `POST /v1/license/validate` with current bearer token and device fingerprint.
2. Server validates the token signature, `kid`, `alg`, `exp`, and device hash match.
3. If valid: server issues a new JWT with updated `iat`/`exp`, returns it in the response.
4. Client replaces the stored token with the new token in secure storage.
5. If the client fails to store the new token, it continues using the old token until its `exp`.

### Token TTL
- Default access token TTL: **24 hours**.
- Revalidation interval: **every app start** + **background check every 24 hours**.
- Token TTL must be configurable via server environment.

### Key Rotation During Refresh
- If the server has rotated to a new signing key, the new token is signed with the new key and includes the new `kid`.
- The client does not need to be aware of key rotation — it validates tokens using the public key(s) available to it.
- The server should accept tokens signed with both the current and previous key during a rotation grace period.

---

## Concurrency and Isolation

### Single-Device Binding Enforcement
The constraint "one active activation per license" must be enforced atomically:

1. **Database-level enforcement**: Use a unique constraint or equivalent on `(license_id)` for rows where `revoked_at IS NULL` in the `activations` table.
2. **Transaction isolation**: All activation operations (check-existing → revoke-if-needed → insert-new) must execute within a single transaction at `SERIALIZABLE` or equivalent isolation level.
3. **Conflict resolution**: If two concurrent activation requests arrive for the same license:
   - Exactly one must succeed; the other must receive `KEY_BOUND_TO_OTHER_DEVICE`.
   - Use optimistic concurrency control (retry on conflict) or pessimistic locking (row lock on license).

### Race Condition Scenarios
| Scenario | Expected Outcome |
|---|---|
| Two devices activate simultaneously | Exactly one succeeds; other gets `KEY_BOUND_TO_OTHER_DEVICE` |
| Validate and admin-reset arrive simultaneously | Reset completes; next validate re-activates or gets denial |
| Validate and revoke arrive simultaneously | If revoke commits first, validate gets `REVOKED`; if validate commits first, revoke still succeeds |

### Rate-Limit Atomicity
- Rate-limit counter increments must be atomic (use atomic increment operations or transactions).
- Sliding window calculations must handle concurrent requests without double-counting.

---

## Device Fingerprint Composition

### Fingerprint Goals
- **Stable**: Does not change across reboots or minor OS updates.
- **Unique**: Different physical machines produce different fingerprints with high probability.
- **Non-PII**: Does not contain user-identifiable information.

### Recommended Inputs
Combine **3 or more** of the following stable hardware/OS identifiers:

| Input | Example Source | Stability |
|---|---|---|
| Machine ID / Hardware UUID | `/etc/machine-id` (Linux), `IOPlatformUUID` (macOS), `MachineGuid` registry key (Windows) | High |
| CPU model + core count | System API / `/proc/cpuinfo` | High |
| Total RAM (rounded to GB) | System API | Medium-High |
| Disk serial number (primary) | System API / `lsblk` | High |
| OS installation ID | OS-specific APIs | High |

### Composition Process
1. Collect inputs as a sorted, deterministic key-value list.
2. Concatenate with a stable delimiter.
3. Hash the concatenation with SHA-256.
4. The hash is the device fingerprint — never transmit or store the raw inputs.
5. Combine with a per-install random `install_id` (generated once and persisted) to distinguish reinstalls on the same hardware.

### Fingerprint Transmission
- Transmit only the hash to the server.
- The server stores only a keyed hash (HMAC) of the fingerprint hash, never the raw hash.
- This provides two layers of hashing: `HMAC(server_key, SHA256(device_inputs))`.

---

## Privacy and Data Retention

### Data Classification

| Data | Classification | Storage Rule |
|---|---|---|
| License key | Secret | Store only keyed hash; never plaintext |
| Device fingerprint | PII-adjacent | Store only keyed hash |
| IP address | PII | Store only hash in audit events; do not retain raw |
| User agent | Low sensitivity | Store hash in audit events |
| Email address | PII | Do not store unless explicitly required; hash if stored |
| Admin actor identifier | Internal | Store in admin audit log |

### Retention Policy
- `activations`: Retain active records indefinitely. Retain revoked records for **90 days**, then anonymize (replace hashes with tombstone marker).
- `activation_events`: Retain for **180 days**, then delete.
- `admin_actions`: Retain for **1 year**, then archive.
- `admin_nonces`: Retain for **24 hours** (replay protection window), then delete.
- `request_limits`: Retain for the rate-limit window duration only.

### Data Deletion / Right to Erasure
- Provide an admin operation to delete all records associated with a license key hash.
- Deletion must cascade: license → activations → activation_events → admin_actions.
- Deletion must be audit-logged before execution (log the deletion intent, then delete).
- After deletion, the license key hash must not be recoverable from any table.

### Anonymization
- Where retention is required beyond the deletion request (e.g., aggregate metrics), replace identifying hashes with a deterministic anonymization token that cannot be reversed.

---

## Expanded Test Strategy

### Unit Tests
- **Hashing**: Verify deterministic output for same input; verify different output for different inputs; verify keyed hash differs from plain hash.
- **Token signing/verification**: Round-trip sign → verify; expired token rejection; future-issued token rejection; wrong `kid` rejection; wrong `alg` rejection; tampered payload rejection.
- **Provider sealing**: Round-trip encrypt → decrypt; key rotation (encrypt with key A, rotate, decrypt with key A still works).
- **Admin auth**: Valid signature acceptance; stale timestamp rejection; replayed nonce rejection; wrong key ID rejection; tampered body rejection.
- **Rate limiting**: Counter increment accuracy; window expiry; distinct namespace isolation.
- **Reason codes**: Every denial path produces the correct reason code.

### Integration Tests
- **Activation flow**: First activation succeeds → same device re-activation succeeds → different device denied → admin reset → different device succeeds.
- **Validation flow**: Valid token + matching device succeeds → mismatched device denied → expired token denied → revoked license denied.
- **Deactivation flow**: Active deactivation succeeds → subsequent validation denied → re-activation succeeds.
- **Provider integration**: Mock provider returns active → allowed; returns cancelled → denied; returns error → server error with reason code.
- **Admin operations**: Each admin action records audit log entry; action history query returns correct records.

### Concurrency Tests
- Two simultaneous activations for same license: exactly one succeeds.
- Rapid sequential validations: no race in `last_seen_at` update.
- Rate limit under concurrent load: counters are accurate.
- Admin reset during active validation: no inconsistent state.

### Security Tests
- Token tampered after signing → rejected.
- Token replayed after expiry → rejected.
- Admin request with clock skew beyond threshold → rejected.
- Admin nonce reuse → rejected.
- Plaintext key search across all DB tables → zero results.
- Plaintext device fingerprint search across all DB tables → zero results.

### Offline Grace Period Tests (Client-Side)
- Validate succeeds → go offline → app launches for 20 days → no warning.
- Validate succeeds → go offline → app launches at day 21 → warning shown.
- Validate succeeds → go offline → app launches at day 31 → app locked.
- Validate succeeds → go offline → clock rolled back → forced online validation.
- Validate succeeds → online validation returns revoked → immediate lock.

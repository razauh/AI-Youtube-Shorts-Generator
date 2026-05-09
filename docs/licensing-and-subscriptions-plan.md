# Licensing + Subscription Architecture Plan (Cloudflare Workers, 1 Key = 1 Device)

> **Document Classification: PROJECT-SPECIFIC**
> This file is a project-specific implementation plan, not a language-agnostic guide.
> For language-agnostic specifications, see: `licensing-worker-language-agnostic-guide.md`, `licensing-api-contract-matrix.md`.

## Summary
Build a licensing control plane on Cloudflare Workers that gates the Tauri desktop app at startup and enforces `1 license key = 1 active device`.

This plan supports two business modes:
1. **Mode A (recommended): Gumroad + Worker** for payment status + device binding.
2. **Mode B: Worker-only licensing** when Gumroad is not used for subscriptions.

Both modes share the same app flow, token format, device binding logic, admin operations, and security controls.
Canonical contract reference for implementation: `docs/licensing-api-contract-matrix.md`.

---

## 1) Product/Policy Contract (lock first)
- `max_devices = 1`
- Offline fallback after successful activation is allowed with bounded controls:
  - allow launch for up to **30 days** since last successful `/validate`
  - show warning after **21 days** offline
  - require successful online validation after **30 days**
  - once online validation observes revocation/refund/cancel policy denial, lock immediately
- Reset policy: **manual support reset only**
- Revalidation interval: every app start + background check every 24h
- Token expiry: 24h access token + refresh on successful revalidation

Define license states:
- `active`
- `past_due`
- `cancelled`
- `refunded`
- `revoked`

Define activation outcomes:
- `allowed`
- `already_bound_same_device`
- `denied_bound_other_device`
- `denied_subscription_invalid`

---

## 2) Worker API (single implementation with provider adapter)
Create a Worker API with provider-aware verification:
Canonical source: `docs/licensing-api-contract-matrix.md` (Endpoint Matrix section).

- `POST /v1/license/activate`
  - Input: `license_key`, `device_fingerprint`, `app_version`, `platform`, optional `install_id`
  - Output: signed activation JWT + `next_revalidate_at`

- `POST /v1/license/validate`
  - Input: activation token + current device fingerprint
  - Output: refreshed token (if valid) or explicit denial reason

- `POST /v1/license/deactivate`
  - Device release (admin or policy-restricted)

- `GET /v1/license/status`
  - Public, token-authenticated self status view only.

- `POST /v1/admin/license/reset-device`
  - Manual support operation.

- `POST /v1/admin/license/actions`
- `GET /v1/admin/license/actions/history`
- `GET /v1/admin/licenses`
- `GET /v1/admin/activations`
- `GET /v1/admin/licenses/{licenseKey}`
- `POST /v1/admin/licenses`
- `PUT|PATCH /v1/admin/licenses`
- `POST /v1/admin/licenses/revoke`
- `GET /v1/admin/metrics`
- `GET /v1/admin/license/status`
  - Admin/support lookup by explicit license identifier.

### Provider adapter behavior

**Mode A (Gumroad + Worker):**
- Worker verifies key/subscription via Gumroad API before bind/refresh.
- Subscription/refund/cancel state drives entitlement.

**Mode B (Worker-only):**
- Worker verifies against internal `licenses` table only (no Gumroad call).
- Entitlements maintained in internal admin panel/API.

---

## 3) Data Model (D1 as source of truth; KV optional cache)
Use D1 for authoritative writes and constraints. Avoid KV-only binding logic.

### `licenses`
- `id`
- `license_key_hash` (unique)
- `plan_id`
- `status`
- `provider` (`gumroad|internal`)
- `provider_ref`
- `created_at`
- `updated_at`

### `activations`
- `id`
- `license_id`
- `device_fingerprint_hash`
- `install_id`
- `first_activated_at`
- `last_seen_at`
- `revoked_at`

Constraint:
- one active activation per license (`license_id` where `revoked_at IS NULL`)

### `activation_events` (audit)
- `event_type`
- `license_id`
- `device_hash`
- `ip_hash`
- `user_agent_hash`
- `reason`
- `timestamp`

### `admin_actions`
- `actor`
- `action`
- `target_license`
- `before_after_snapshot`
- `timestamp`

Notes:
- Store only hashes for keys/device IDs.
- KV can be used for read cache only; D1 remains final authority.

---

## 4) Desktop App Gating (Tauri)
Add licensing gate before feature routes load.

- Show Activation screen when no valid token exists.
- On key submit:
  - Build deterministic device fingerprint + random install ID.
  - Call `/activate`.
  - Store token in keychain/secure storage.
- On startup:
  - If token fresh and within offline grace, allow launch.
  - Attempt background `/validate` when online.
  - If grace expired and validation unavailable/denied, lock app.
- On denial, show explicit reason:
  - `bound to another device`
  - `subscription inactive`
  - `revoked`
- Keep existing API key config inaccessible until license is valid.

---

## 5) Security and Abuse Controls
- JWT signing with asymmetric keys is mandatory:
  - `EdDSA` (`Ed25519`) preferred; `ES256` acceptable fallback.
  - include `kid` in header and reject unknown `kid`/unexpected `alg`.
- Keep private signing key in Worker secrets.
- Support key rotation via `kid`.
- Rate-limit `/activate` and `/validate` by IP and key.
- Replay resistance with nonce/issued-at checks + short validity.
- Device fingerprint hardening:
  - combine multiple stable OS inputs + app install ID.
  - hash before transmit/store.
- Protect admin endpoints with Cloudflare Access and per-request HMAC signing headers (`x-admin-timestamp`, `x-admin-nonce`, `x-admin-signature`, `x-admin-key-id`).
- Structured redacted logs (no raw keys/device IDs).

---

## 6) Dual-Mode Strategy (Gumroad + non-Gumroad)
Implement `BillingProvider` abstraction in Worker:
- `verify_entitlement(license_key) -> entitlement state`

Providers:
- `GumroadProvider`
- `InternalProvider`

Config switch:
- `BILLING_PROVIDER=gumroad|internal`

Result:
- App behavior stays unchanged.
- Only backend entitlement source changes.

---

## Public APIs / Interfaces / Types
Reason codes and auth expectations must remain aligned with `docs/licensing-api-contract-matrix.md`.

### `ActivationRequest`
- `license_key`
- `device_fingerprint`
- `install_id`
- `app_version`
- `platform`

### `ActivationResponse`
- `status`
- `token`
- `expires_at`
- `next_revalidate_at`
- `reason_code`

### `ValidationResponse`
- `status`
- `token` (optional)
- `grace_until` (optional)
- `reason_code` (optional)

### `reason_code` enum
- `INVALID_KEY`
- `SUBSCRIPTION_INACTIVE`
- `KEY_BOUND_TO_OTHER_DEVICE`
- `REVOKED`
- `RATE_LIMITED`
- `SERVER_ERROR`

### `BillingProvider` backend interface
- `verify_key`
- `verify_subscription_status`
- `map_provider_state_to_entitlement`

---

## Test Plan and Acceptance Criteria

### Backend tests
- First activation binds key to one device.
- Activation on second device is denied.
- Re-activation on same device succeeds.
- Validate fails when subscription turns inactive (Mode A).
- Validate succeeds for active internal license (Mode B).
- Reset-device endpoint clears binding and enables next activation.
- Concurrency test: two simultaneous activations, only one succeeds.
- Token tamper/expired/replay cases rejected.
- Rate limits trigger and recover correctly.

### App tests
- Fresh install is blocked until activation succeeds.
- Offline after prior activation continues to work when validation endpoint is unavailable, up to the 30-day offline limit.
- Explicit revocation or device reset locks app on next successful online validation.
- Revoked key transitions app to locked on next validation.
- Error UX displays clear reason codes.

### Operational acceptance
- p95 activation latency under 400ms (excluding external provider latency).
- All admin actions are audited.
- No plaintext keys/device IDs in logs or DB.

---

## Rollout Plan
1. Build Worker + D1 + admin reset endpoints in staging.
2. Integrate app activation gate behind feature flag.
3. Soft launch with internal provider and small beta.
4. Enable Gumroad provider in production (or remain internal).
5. Turn on strict gating for all users and monitor failures/support load.

---

## Assumptions and Defaults
- App is desktop-first Tauri and not hosted on Gumroad.
- Cloudflare Workers + D1 is the backend stack.
- `1 key = 1 device` is strictly enforced server-side.
- Offline fallback is bounded (30-day max since last successful validation) for previously activated devices.
- Device transfer is manual support reset only.
- Gumroad integration is optional via provider adapter.

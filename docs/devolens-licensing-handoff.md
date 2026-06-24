# Devolens Licensing Migration Handoff

## Purpose

This document hands off the current state of the licensing-provider migration from the custom Cloudflare license Worker toward Devolens/Cryptolens.

The important context is that this repository already had a security-sensitive custom licensing stack:

- A Cloudflare Worker under `worker/` implementing activation, validation, reset requests, Gumroad purchase verification, admin review, privacy deletion, admin reporting, and updater routing.
- A Tauri/Rust desktop auth layer using `license-control-suite` traits and local secure storage.
- A Svelte frontend that depends on the existing Tauri auth commands and auth-state contract.

The first implementation pass did not remove the custom Worker. It added a Devolens backend adapter behind the existing Rust `WorkerClient` abstraction so the app can be switched to Devolens for key activation without breaking the current UI or auth command surface.

## What Has Been Implemented

### Backend mode selection

The app now understands a new license backend mode:

```env
LICENSE_BACKEND_MODE=devolens
```

This was added alongside the existing modes:

- `reference`
- `hosted`
- `mock`
- `devolens`

The existing modes were intentionally kept intact. The Devolens integration is opt-in and should not affect the current custom Worker flow unless `LICENSE_BACKEND_MODE=devolens` is configured.

Relevant implementation:

- `app/src-tauri/src/core/config.rs`
  - Added `LicenseBackendMode::Devolens`.
  - Added `DEFAULT_DEVOLENS_BASE_URL`.
  - Added Devolens config fields to `Config` and `LicenseWorkerConfig`.
  - Added environment loading and validation for Devolens credentials.

### Devolens configuration

The following new environment variables are supported:

```env
DEVOLENS_BASE_URL=https://api.cryptolens.io
DEVOLENS_ACCESS_TOKEN=your_devolens_access_token_here
DEVOLENS_PRODUCT_ID=your_devolens_product_id_here
```

Validation rules implemented:

- `DEVOLENS_BASE_URL` must start with `http://` or `https://` when Devolens mode is active.
- `DEVOLENS_ACCESS_TOKEN` is required when `LICENSE_BACKEND_MODE=devolens`.
- `DEVOLENS_PRODUCT_ID` is required when `LICENSE_BACKEND_MODE=devolens`.
- Existing `LICENSE_WORKER_*` retry, timeout, circuit-breaker, storage namespace, and keychain settings still load as before.

`.env.example` now includes a commented Devolens/Cryptolens configuration block.

### Devolens worker adapter

A new `DevolensWorkerClient` was added in `app/src-tauri/src/auth_worker.rs`.

It implements the existing `license_control_suite::core::WorkerClient` trait, which means it can be used by the current `AuthService` without changing the frontend command API.

Implemented method behavior:

- `activate`
  - Builds a Devolens activation request.
  - Calls:

    ```text
    POST {DEVOLENS_BASE_URL}/api/key/Activate
    ```

  - Sends form fields:

    ```text
    token
    ProductId
    Key
    MachineCode
    ```

  - Uses the app's existing derived `DeviceId` as the Devolens `MachineCode`.
  - Parses a Devolens-style response with `result`/`Result` and `licenseKey`/`LicenseKey` variants.
  - Treats result code `0` as success.
  - Treats blocked or expired license responses as not active.
  - Returns the current app's `ActivationOutcome`, preserving:
    - masked license key
    - bound device summary
    - active entitlement
    - token expiry timestamp

- `validate_session`
  - Returns `ValidationOutcome::ReauthRequired`.
  - This is intentional for the first pass. The existing `AuthService` already reacts to `ReauthRequired` by reactivating with the locally stored license key when available.
  - This avoids inventing a custom long-lived local Devolens session token before the final Devolens validation/offline strategy is designed.

- `request_device_reset`
  - Returns `AuthError::InvalidResetRequest`.
  - This is intentional because the current app's reset flow is an admin-reviewed custom Worker flow, and Devolens device management needs a separate product/operations mapping.

- `get_device_reset_status`
  - Returns `AuthError::ResetRequestNotFound`.
  - This prevents the app from pretending Devolens supports the current custom reset-status contract.

### Policy wrapper integration

`build_worker_client` now routes:

- `reference` and `hosted` to the existing `HttpWorkerClient`.
- `mock` to the existing `MockLicenseWorkerClient`.
- `devolens` to the new `DevolensWorkerClient`.

The Devolens adapter is wrapped in the existing `PolicyWorkerClient`, so timeout, retry, and circuit-breaker behavior still apply.

### Readiness flag

`LicenseWorkerReadiness::local` now marks `provider_adapter_enabled` as true when `LICENSE_BACKEND_MODE=devolens`.

This is a coarse signal only. It does not mean the full replacement is production-ready.

### Tests written

Focused tests were added or updated, but not run by the agent.

Tests added in `app/src-tauri/tests/config_tests.rs`:

- Devolens defaults are loaded when no Devolens mode is active.
- Devolens env overrides are trimmed and normalized.
- Devolens mode requires `DEVOLENS_ACCESS_TOKEN`.
- Devolens mode requires `DEVOLENS_PRODUCT_ID`.
- Devolens mode loads provider config successfully when all required values are set.

Tests added in `app/src-tauri/tests/auth_worker_tests.rs`:

- `devolens` backend mode builds and activates through a local HTTP server using the provider API shape.
- The activation request uses `POST /api/key/Activate`.
- The activation request is form-encoded.
- The activation request includes `ProductId` and `MachineCode`.
- The adapter returns active entitlement and the existing masked-license-key format.
- Existing Devolens local session markers are treated as requiring reauth instead of being trusted as standalone provider sessions.

Existing auth-worker test helpers were adjusted to include Devolens config defaults in `LicenseWorkerConfig` test literals.

### Validation script update

The existing tracked script `.scripts/run-license-fallback-validation.sh` was updated to include Devolens-related Rust tests:

```bash
cargo test --locked --manifest-path app/src-tauri/Cargo.toml --test auth_worker_tests --test config_tests
```

This was added to the existing script instead of creating a new script because `.scripts/*` is ignored by `.gitignore` unless the script is already tracked.

The agent did not run this script, per repository policy.

## Current Runtime Behavior

### Existing Worker mode

If the app is configured with the current Worker-backed modes, behavior should remain unchanged:

```env
LICENSE_BACKEND_MODE=hosted
LICENSE_WORKER_BASE_URL=https://license-worker.demandscout.workers.dev
```

or:

```env
LICENSE_BACKEND_MODE=reference
```

These still use the existing `HttpWorkerClient` and custom Worker contract.

### Devolens mode

If configured as:

```env
LICENSE_BACKEND_MODE=devolens
DEVOLENS_ACCESS_TOKEN=[redacted]
DEVOLENS_PRODUCT_ID=1234
DEVOLENS_BASE_URL=https://api.cryptolens.io
```

then activation goes directly from the Tauri app to the Devolens/Cryptolens API through the new Rust adapter.

The app still stores session state through the existing `AuthService` and local secure-storage/fallback mechanisms. The frontend receives the same high-level auth views as before.

Session validation currently causes reactivation with the stored license key instead of performing a separate Devolens session validation call. This is acceptable for a first adapter pass, but it is not the final production design.

## Important Security Notes

### Devolens access token placement must be reviewed

The first implementation reads `DEVOLENS_ACCESS_TOKEN` from env or secure store and uses it in the desktop app process. That is convenient for evaluation, but it may not be acceptable for production depending on the Devolens token scope.

Before public release, confirm whether the Devolens token used for activation is safe to embed or distribute in a desktop app. If it can create, mutate, list, or delete licenses, it must not ship in the client.

Production-safe options:

- Use a narrowly scoped Devolens token intended only for client-side activation/verification, if Devolens supports that.
- Put a thin backend proxy in front of Devolens and keep the privileged Devolens token server-side.
- Use Devolens SDK/offline verification with public-key-style verification if that satisfies the app's licensing model.

Do not ship a broad management API token in a Tauri desktop binary.

### Devolens activation currently sends the raw license key to Devolens

This is expected for provider activation, but the app must continue avoiding raw license keys in logs, snapshots, frontend persistent state, crash drafts, and support exports.

The current adapter does not log the request body or provider response body.

### Local token is a compatibility marker

The current Devolens adapter returns a local access-token-like marker:

```text
devolens:{device_id}:{timestamp_ms}
```

This exists only to satisfy the current `AuthService` contract after activation. It is not a signed Devolens token and must not be treated as a provider authorization artifact.

The adapter intentionally returns `ReauthRequired` during validation so the existing service rechecks the license via activation when possible.

This must be replaced or formalized before production.

## What Is Left To Implement

### 1. Decide the production Devolens architecture

This is the most important remaining decision.

The current code supports direct Devolens activation from the desktop app. That may be acceptable only if the Devolens token can be safely exposed to the client with minimal privileges.

Need to decide one of these production architectures:

1. Direct desktop-to-Devolens integration
   - Use only if Devolens supports a client-safe token or public verification mode.
   - Lowest infrastructure burden.
   - Highest risk if the token is privileged.

2. Thin backend proxy in front of Devolens
   - Desktop app calls your backend.
   - Backend calls Devolens with the privileged token.
   - Lets you preserve privacy deletion, Gumroad migration, admin-only actions, rate limiting, and support workflows.
   - More infrastructure, but safer.

3. Devolens SDK/offline-license model
   - Use provider-supported signed/offline validation if available for this app's commercial model.
   - Best for desktop reliability and limited offline use.
   - Requires a deeper Devolens-specific implementation and provider setup.

Recommended next step: verify Devolens token scopes and preferred desktop integration model with official Devolens docs/support before public release.

### 2. Replace the temporary validation strategy

Current behavior:

- Activation calls Devolens.
- Validation returns `ReauthRequired`.
- `AuthService` reactivates with the stored license key when possible.

This is intentionally conservative but incomplete.

Production work needed:

- Identify the Devolens-supported validation flow for an already activated machine.
- Decide whether validation should:
  - call Devolens activation again,
  - call a Devolens validation endpoint,
  - verify a signed/offline license file,
  - use provider-maintained activation records,
  - or use a backend proxy session token.
- Define exact mapping from Devolens status to local states:
  - active
  - expired
  - blocked/revoked
  - refunded/disabled
  - too many machines
  - network failure
  - provider unavailable
- Replace the local `devolens:{device_id}:{timestamp_ms}` marker if a real provider/session token exists.
- Add tests for every status mapping.

Acceptance criteria:

- A previously activated user can reopen the app and validate without unnecessary customer friction.
- Revoked/expired/blocked licenses reliably move to `reauth_required` or equivalent locked state.
- Network failure enters existing offline grace only when the last known valid state permits it.
- No raw license key is required for routine validation unless the chosen Devolens architecture explicitly requires reactivation.

### 3. Design Devolens device binding and reset behavior

The current custom Worker has an admin-reviewed reset workflow:

- User requests a reset.
- Admin reviews pending reset requests.
- Admin approves or rejects.
- Approval deactivates active bindings for the license.
- User can activate again.

The first Devolens adapter does not implement this.

Work left:

- Identify how Devolens models machine activations for a license.
- Decide whether the current app's "device reset" should become:
  - Devolens dashboard operation,
  - Devolens API operation,
  - support-only manual operation,
  - customer portal operation,
  - or a retained backend proxy workflow.
- Map "too many machines" / "machine limit reached" provider errors to current app UX.
- Implement `request_device_reset` and `get_device_reset_status`, or deliberately remove/replace the UI if Devolens makes this self-service elsewhere.
- Update admin desktop behavior if reset review no longer lives in the custom Worker.
- Add tests for:
  - reset request without stored license key
  - reset request for active license
  - provider machine-limit error
  - admin/manual reset completion
  - rejected/unavailable reset state

Acceptance criteria:

- A customer with a new machine has a clear recovery path.
- Support/admin can unbind or reset devices without direct database editing.
- The app does not show reset controls that do nothing in Devolens mode.

### 4. Decide what happens to the custom Worker

The custom Worker currently does more than activation.

It still owns or supports:

- Gumroad webhook verification.
- License creation/upsert from Gumroad sales.
- Device bindings.
- Reset requests.
- Admin overview.
- Admin license disable.
- Admin audit events.
- User-data deletion request/status.
- Customer updater route.
- Idempotency records.
- D1 persistence.

Moving to Devolens does not automatically replace all of that.

Work left:

- Decide whether to fully remove `worker/` or reduce it to a thin companion service.
- For every Worker route, choose:
  - remove,
  - keep unchanged,
  - replace with Devolens,
  - replace with a thin proxy,
  - or move to manual support operations.

Suggested classification:

- Activation/validation: move to Devolens.
- Gumroad purchase provisioning: likely move to Devolens automation, Zapier/n8n, Devolens API, or a thin webhook proxy.
- Device reset: needs Devolens-specific design.
- Privacy deletion: likely keep some backend process, because provider data deletion may involve external vendor/admin operations.
- Admin overview/audit: replace with Devolens dashboard if sufficient; otherwise keep companion admin tooling.
- Updater route: not licensing-specific; keep separate from Devolens.

Acceptance criteria:

- No dead Worker route remains referenced by frontend/Rust commands.
- No production workflow depends on D1 data that is no longer authoritative.
- Admin and support docs say where each operation is performed after migration.

### 5. Migrate existing license records

The current Worker stores license records in D1 keyed by a peppered license hash and originally connected to Gumroad purchases.

Work left:

- Export existing active licenses from the current source of truth.
- Determine whether Devolens license keys should be:
  - newly generated and sent to existing users,
  - imported from existing Gumroad/custom keys,
  - or mapped via metadata/custom fields.
- Preserve support lookup data without exposing raw customer emails unnecessarily.
- Decide how to handle:
  - refunded purchases,
  - disabled licenses,
  - deleted/anonymized records,
  - pending reset requests,
  - active device bindings,
  - and idempotency records.
- Create a migration dry-run report before changing production data.

Acceptance criteria:

- Existing paying customers can activate after migration.
- Disabled/refunded users do not regain access.
- Privacy-deleted records remain deleted/anonymized.
- There is a rollback plan if Devolens import or activation fails.

### 6. Rework Gumroad purchase automation

The current Worker verifies Gumroad sales and upserts local license records.

With Devolens, the desired flow should likely be:

```text
Gumroad sale -> Devolens license creation/update -> customer receives/uses Devolens license key
```

Work left:

- Decide whether Gumroad stays as payment provider.
- Decide whether automation uses:
  - Devolens payment integrations,
  - Zapier/n8n,
  - Gumroad webhook to a thin backend,
  - manual license creation for early beta,
  - or Devolens API scripts.
- Ensure purchase refunds/chargebacks disable or revoke Devolens licenses.
- Ensure purchaser email is handled according to the app privacy notice.
- Add tests or operational runbooks for:
  - successful purchase
  - duplicate webhook
  - refund
  - chargeback/dispute
  - failed Devolens API call
  - retry/replay

Acceptance criteria:

- A real purchase results in a usable Devolens license.
- A refund/dispute disables access.
- Duplicate payment events do not create duplicate active licenses.
- No Gumroad secret or Devolens token is stored in the desktop app.

### 7. Update admin desktop behavior

The admin desktop currently assumes the custom Worker API.

Work left:

- Decide whether the admin desktop remains part of the product.
- If yes, decide whether it:
  - calls Devolens APIs,
  - links admins to Devolens dashboard,
  - calls a thin backend proxy,
  - or only handles app-specific operations like privacy deletion and update publishing.
- Remove or hide admin sections that no longer work in Devolens mode:
  - reset requests
  - delete requests
  - licenses
  - device bindings
  - audit events
  - idempotency records
- Add a visible backend-mode indicator so an admin can tell whether they are managing the custom Worker or Devolens.

Acceptance criteria:

- Admin UI does not call stale Worker routes when Devolens is active.
- Admin actions are protected by an appropriate auth model.
- Admin UI does not display misleading D1 data after Devolens becomes authoritative.

### 8. Update privacy deletion and legal copy

Current legal/privacy copy describes backend licensing data deletion handled by the Worker.

If Devolens becomes the licensing source of truth, the copy must be updated.

Work left:

- Identify what personal data Devolens stores:
  - license keys
  - customer emails
  - activation/machine identifiers
  - IPs
  - logs/analytics
  - payment metadata
- Update privacy notice and in-app policy text to name Devolens/Cryptolens as a processor/provider where appropriate.
- Decide whether in-app data deletion still submits to the custom Worker, a new backend proxy, or manual support.
- Update support process for deletion requests involving:
  - Devolens data,
  - Gumroad data,
  - local app data,
  - update-host logs,
  - and support records.

Acceptance criteria:

- The app does not claim the Worker deletes data that now lives in Devolens.
- User deletion requests have a real operational path.
- Legal copy reflects actual providers and data flows.

### 9. Improve error mapping and user messages

The first adapter maps provider failures conservatively:

- failed result -> `invalid_license_key`
- machine-related message -> `device_already_bound`
- blocked/expired -> `reauth_required`
- unauthorized provider response -> `unauthorized`
- network/unknown provider failure -> `worker_unreachable`

This is not detailed enough for production.

Work left:

- Confirm exact Devolens error response shapes.
- Map Devolens error codes to local `AuthError` variants.
- Avoid surfacing raw provider messages to end users if they may include sensitive or confusing internals.
- Add friendly UI messages for:
  - invalid key
  - expired key
  - blocked/revoked key
  - too many machines
  - provider unavailable
  - Devolens account/config issue

Acceptance criteria:

- Users see actionable messages.
- Support can diagnose provider/config errors without exposing secrets.
- Existing redaction tests still pass.

### 10. Decide offline behavior

The current app has an offline grace concept in local auth state.

The Devolens adapter currently relies on the existing offline grace behavior when the provider is unreachable, but it does not implement provider-backed offline license files or signed offline verification.

Work left:

- Decide whether offline support is required for launch.
- If yes, evaluate Devolens offline verification or license-file support.
- Decide offline grace duration and whether it differs between trial, perpetual, and subscription licenses.
- Ensure revoked or expired licenses cannot stay valid indefinitely offline.
- Add tests for:
  - valid offline grace
  - expired offline grace
  - revoked after reconnect
  - clock skew
  - missing local token

Acceptance criteria:

- Offline behavior is documented, predictable, and aligned with the product's commercial policy.
- Offline state cannot bypass reasonable revocation/expiration controls.

### 11. Validate Devolens response parsing against real API samples

The first adapter supports common casing variants:

- `result`
- `Result`
- `licenseKey`
- `LicenseKey`
- `blocked`
- `Blocked`
- `expired`
- `Expired`

But it has not been verified against real Devolens API responses in this repo session.

Work left:

- Capture real success, invalid key, blocked key, expired key, and machine-limit responses from a Devolens test product.
- Add fixture-based tests with those exact shapes.
- Update parser fields if Devolens uses different names for expiration, blocking, machine limits, or activated machines.
- Confirm whether `result == 0` is the only success indicator.

Acceptance criteria:

- Parser tests use real provider response samples with secrets removed.
- The adapter does not rely on guessed field names for critical licensing decisions.

### 12. Add production configuration and secret-management docs

Work left:

- Document how to create a Devolens product.
- Document how to create the correct token/scope.
- Document where `DEVOLENS_ACCESS_TOKEN` should live for:
  - local development,
  - CI,
  - production builds,
  - release signing,
  - and runtime.
- Decide whether the token is ever allowed in `.env`.
- Decide whether production builds should fail if `LICENSE_BACKEND_MODE=devolens` and token scope is unknown.
- Update release checklist with Devolens-specific checks.

Acceptance criteria:

- A release operator can configure Devolens without reading source code.
- No production secret is baked into artifacts accidentally.
- The setup process distinguishes evaluation credentials from production credentials.

### 13. Update tests and fixtures across the repo

The first pass added focused tests only.

More tests are needed before production:

- Rust auth service tests for Devolens mode.
- Frontend UI tests showing Devolens-mode auth failures correctly.
- Privacy command tests if deletion flow changes.
- Admin client/UI tests if admin behavior changes.
- Contract/fixture tests for any retained compatibility backend.
- Regression tests that license-gated generator UI still stays hidden when unauthenticated.
- Tests that raw Devolens tokens and raw license keys are not logged or serialized.

Acceptance criteria:

- Existing custom Worker tests still pass when Worker mode is used.
- Devolens-mode tests cover both happy path and failure paths.
- No tests are weakened or deleted to make the migration pass.

### 14. Run validation manually

The agent did not run tests or validation scripts because repository instructions prohibit agent-run toolchain validation.

Manual validation commands to run from the repository root:

```bash
.scripts/run-license-fallback-validation.sh
```

For a narrower manual Rust check of the Devolens work:

```bash
cargo test --locked --manifest-path app/src-tauri/Cargo.toml --test auth_worker_tests --test config_tests
```

For a broader Rust compile check:

```bash
cargo check --locked --manifest-path app/src-tauri/Cargo.toml
```

Do not run dependency install/update commands as part of validation unless separately approved.

## Known Limitations Of The Current Implementation

- Devolens activation path is implemented, but full Devolens validation is not.
- Devolens reset/device-unbind behavior is not implemented.
- Devolens privacy deletion behavior is not implemented.
- Gumroad-to-Devolens license provisioning is not implemented.
- Admin desktop still targets the custom Worker API.
- The custom Worker still exists and remains necessary for several flows unless those flows are redesigned.
- The Devolens token safety model is unresolved.
- Real Devolens response fixtures have not been captured.
- Provider-specific error mapping is incomplete.
- Offline verification is not provider-backed.

## Suggested Next Implementation Order

1. Confirm Devolens token scope and production-safe architecture.
2. Capture real Devolens activation/validation response samples.
3. Replace temporary validation behavior with the chosen Devolens validation/offline strategy.
4. Decide and implement device reset/unbind behavior.
5. Decide Gumroad purchase provisioning into Devolens.
6. Decide which Worker routes remain, move, or are removed.
7. Update admin UI and privacy/legal copy.
8. Add fixture-backed tests and run the manual validation commands.

## Production Readiness Checklist

Before public launch on Devolens, all of the following should be true:

- No broad Devolens management token is shipped in the desktop app.
- Activation and validation are backed by confirmed Devolens API behavior.
- Revoked, expired, refunded, blocked, and machine-limit states are correctly enforced.
- Device reset or support recovery is operational.
- Gumroad purchase and refund automation is operational.
- Privacy deletion and legal copy match the real data flow.
- Admin tooling is either updated or intentionally retired.
- Existing Worker dependencies are either removed or documented as retained companion services.
- Manual validation passes.
- Release checklist includes Devolens-specific configuration and secret checks.

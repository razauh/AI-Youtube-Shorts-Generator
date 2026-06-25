# Devolens Source-of-Truth Simplification Task Cards

## Scope

This document defines task cards for simplifying licensing so Devolens is the source of truth for license creation, activation, validation, deactivation, refunds, and device ownership. The Worker should become a narrow integration boundary for Gumroad, admin/support, privacy, and updater functions, with D1 reduced to idempotency, audit, mapping, and short-lived cache data where needed.

This file is documentation-only. It intentionally does not change production code, tests, scripts, manifests, or existing dirty documentation.

Evidence was taken from current source and Graphify output. `graphify-out/GRAPH_REPORT.md` reports 927 files, 3831 nodes, auth/admin/privacy communities in the corpus graph, and graph freshness built from commit `91114417`, which matches current `HEAD` (`911144171b91d99dfee4ab0611e34a119c074507`).

## Non-Goals

- Do not remove D1 entirely in the first migration. D1 still has valid uses for idempotency, audit, privacy request tracking, webhook replay detection, updater state if retained, and Devolens-to-provider mapping.
- Do not weaken license gates, token redaction, Gumroad verification, updater signing, Tauri capabilities, or admin authentication.
- Do not migrate dependency managers or edit dependency manifests as part of this work.
- Do not delete legacy reset/admin routes until compatible app releases and tests exist.
- Do not run validation commands from an agent session. Repository policy requires manual validation by the user.

## Current-State Findings

- Worker routing currently exposes health, readiness, updater, privacy deletion, admin reset, admin overview/listing, admin privacy, admin license disable, and Gumroad webhook routes from one module (`worker/src/index.js:39`, `worker/src/index.js:43`, `worker/src/index.js:47`, `worker/src/index.js:56`, `worker/src/index.js:62`, `worker/src/index.js:80`, `worker/src/index.js:95`, `worker/src/index.js:98`).
- `/readyz` checks D1 tables and secret presence and can perform a deep updater manifest fetch (`worker/src/index.js:110`). This risks making readiness too broad and potentially exposing deployment shape through detailed check output.
- Updater traffic is proxied through the Worker route (`worker/src/index.js:204`), while Tauri is configured to use the Worker updater endpoint (`app/src-tauri/tauri.conf.json:37`, `app/src-tauri/tauri.conf.json:40`).
- Privacy deletion requests are Worker/D1 backed (`worker/src/index.js:330`), while desktop privacy code can call Devolens `BlockKey` directly in Devolens mode (`app/src-tauri/src/commands/privacy.rs:94`, `app/src-tauri/src/commands/privacy.rs:123`, `app/src-tauri/src/commands/privacy.rs:194`, `app/src-tauri/src/commands/privacy.rs:232`).
- Admin reset decisions are D1 backed and can deactivate D1 device bindings (`worker/src/index.js:501`, `worker/src/store.js:321`), but Devolens mode also maps reset to Devolens `Deactivate` in the desktop client (`app/src-tauri/src/auth_worker.rs:432`).
- Admin privacy approval is Worker/D1 backed (`worker/src/index.js:696`) and deletes/anonymizes D1 data through store helpers (`worker/src/store.js:345`, `worker/src/store.js:366`).
- Admin license disable uses a hash-prefix lookup and D1 status mutation (`worker/src/index.js:885`, `worker/src/store.js:110`, `worker/src/store.js:123`). This should not remain a source-of-truth operation when Devolens owns license state.
- Gumroad webhook verification creates or blocks keys in Devolens but then writes D1 license rows as if D1 is also authoritative (`worker/src/index.js:1123`, `worker/src/index.js:1174`, `worker/src/index.js:1235`, `worker/src/index.js:1273`, `worker/src/store.js:67`, `worker/src/store.js:99`).
- D1 schema currently has `licenses`, `device_bindings`, `reset_requests`, `idempotency_records`, and `audit_events` tables (`worker/migrations/0001_init.sql:1`, `worker/migrations/0001_init.sql:10`, `worker/migrations/0001_init.sql:19`, `worker/migrations/0001_init.sql:28`, `worker/migrations/0001_init.sql:38`). Privacy deletion adds `user_data_deletion_requests` and `privacy_deleted_at_ms` (`worker/migrations/0003_user_data_deletion_requests.sql:1`).
- Admin overview and listing helpers read across license, binding, reset, privacy, and audit tables (`worker/src/store.js:380`, `worker/src/store.js:400`, `worker/src/store.js:435`), duplicating Devolens support-console concerns.
- Tauri config defaults to Devolens mode and carries Devolens URL, token, product id, and offline grace settings (`app/src-tauri/src/core/config.rs:50`, `app/src-tauri/src/core/config.rs:93`, `app/src-tauri/src/core/config.rs:103`).
- Desktop auth builds a Devolens worker client in Devolens mode (`app/src-tauri/src/auth_worker.rs:205`), stores a Devolens access token in the client (`app/src-tauri/src/auth_worker.rs:226`), activates through `api/key/Activate` (`app/src-tauri/src/auth_worker.rs:285`), validates Devolens tokens (`app/src-tauri/src/auth_worker.rs:365`), and maps reset to Devolens deactivation (`app/src-tauri/src/auth_worker.rs:432`).
- Local auth storage persists license/session/device identity through secret store and fallback files, clears session secrets locally, and creates a stable device identity (`app/src-tauri/src/auth.rs:162`, `app/src-tauri/src/auth.rs:219`, `app/src-tauri/src/auth.rs:235`, `app/src-tauri/src/auth.rs:288`, `app/src-tauri/src/auth.rs:320`).
- Tauri auth commands still expose activation, validation, reset request/status, local clear, and auth state (`app/src-tauri/src/commands/auth.rs:7`, `app/src-tauri/src/commands/auth.rs:22`, `app/src-tauri/src/commands/auth.rs:38`).
- Main Svelte UI still presents "Device Reset" flows in unauthenticated and Settings areas (`app/src/routes/+page.svelte:380`, `app/src/routes/+page.svelte:918`, `app/src/routes/+page.svelte:961`, `app/src/routes/+page.svelte:1382`).
- Admin UI contains reset, delete, license, device binding, audit, and idempotency sections, refresh logic, disable-license action, and overview metrics (`app/src/admin/AdminApp.svelte:22`, `app/src/admin/AdminApp.svelte:156`, `app/src/admin/AdminApp.svelte:266`, `app/src/admin/AdminApp.svelte:337`).

## Target Architecture

- Devolens owns license state: key creation, blocking/refunds, activation, validation, deactivation, device binding, and machine-limit semantics.
- Worker owns only integration boundaries that must remain server-side: Gumroad authenticity/idempotency, privileged Devolens management calls, support/admin access control, privacy request audit workflow, updater endpoint if explicitly retained, and safe health/readiness signals.
- D1 stores only non-authoritative data: idempotency records, audit events, provider-sale-to-Devolens mapping, privacy request state, migration compatibility state, and short-lived cache with clear TTLs.
- Desktop uses one stable device identity for activation and deactivation. "Deactivate this device" replaces the support-driven "Device Reset" concept for current-device release.
- Admin/support tooling avoids broad license/device tables unless Devolens cannot provide the needed operation. Any retained support lookup must be minimal, audited, and strongly authenticated.

## Migration Principles

- Preserve current customer access before removing legacy flows.
- Characterize existing Worker, Tauri, admin, privacy, and webhook behavior before refactoring.
- Move authority first, delete legacy code last.
- Prefer explicit Devolens bridge functions over scattered ad hoc `fetch` calls.
- Keep secrets scoped by purpose. Client, webhook, deactivation, and admin/support tokens should not share broad management scopes.
- Keep D1 rows compatible during the transition, but label them as mapping/cache rather than license truth.
- Make terminal states explicit before clearing local secrets.
- Do not expose raw license keys, emails, tokens, machine codes, device identifiers, or full local paths in logs, errors, audit metadata, or support output.

## Risk Register

- Token scope risk: the same Devolens token may be used for client activation and privileged management. Mitigation: split token scopes and tests for scope rejection.
- Double authority risk: Devolens and D1 can disagree on entitlement or device state. Mitigation: make Devolens decisions authoritative and D1 cache invalidatable.
- Device deactivation risk: using a different machine code for deactivation could leave a device bound. Mitigation: reuse the exact activation device identity.
- Webhook failure risk: Gumroad verification, Devolens calls, and D1 writes can partially succeed. Mitigation: idempotency, retryable state, audit events, and explicit failure contracts.
- Privacy risk: deletion flows may delete D1 data while leaving Devolens data intact, or block Devolens keys while losing local audit. Mitigation: define Devolens-owned data steps separately from D1 anonymization.
- Admin exposure risk: broad D1 listing exposes purchaser emails, hashes, device data, and audit metadata. Mitigation: least-privilege endpoints, redaction, and minimal support lookup.
- Updater risk: dynamic Worker updater adds availability and security complexity. Mitigation: choose static signed JSON or harden Worker proxy with narrow validation.
- Compatibility risk: older app versions may still call reset routes. Mitigation: compatibility window and route-level deprecation plan.

## Task Card Index

- `AUD-01`: Inventory Worker routes and ownership.
- `AUD-02`: Classify D1 tables as keep, deprecate, migrate, or delete later.
- `AUD-03`: Locate every D1-as-license-authority read/write.
- `AUD-04`: Inventory Devolens calls and token usage.
- `AUD-05`: Document Tauri license/session/device storage.
- `AUD-06`: Document Gumroad webhook assumptions and failure paths.
- `AUD-07`: Document admin console behavior and duplicated Devolens functionality.
- `DEV-01`: Extract/harden a Worker-side Devolens bridge for CreateKey, BlockKey, Deactivate, and future support calls.
- `DEV-02`: Make activation/validation/deactivation semantics explicitly Devolens-authoritative.
- `DEV-03`: Reduce D1 license rows to mapping/cache only.
- `DEV-04`: Replace or remove disable-by-license-hash-prefix.
- `DEV-05`: Separate Devolens token scopes for client, webhook, deactivation, and admin/support.
- `APP-01`: Add current-device deactivation command/API contract.
- `APP-02`: Reuse the exact activation machine code/device identity for deactivation.
- `APP-03`: Clear local license/session/cache only after successful or explicitly terminal deactivation.
- `APP-04`: Replace Settings "Device Reset" UI with "Deactivate this device" confirmation UX.
- `APP-05`: Update unauthenticated machine-limit flow messaging.
- `RESET-01`: Deprecate/remove admin reset list/approve/reject routes and UI.
- `ADMIN-01`: Remove broad custom admin license/device listing or convert to minimal support lookup.
- `ADMIN-02`: Require stronger access control for retained support endpoints.
- `WEBHOOK-01`: Refactor Gumroad webhook into thin verify/idempotency/Devolens/mapping pipeline.
- `WEBHOOK-02`: Define refund, duplicate, invalid payload, and Devolens failure behavior.
- `PRIV-01`: Preserve privacy request/status flow while decoupling from license authority.
- `PRIV-02`: Simplify admin privacy review and clarify Devolens-owned data operations.
- `UPD-01`: Decide static updater JSON vs dynamic Worker updater and implement the chosen path.
- `HEALTH-01`: Harden `/health` and `/readyz` response safety.
- `DOC-01`: Update operator/support/release docs after behavior changes.
- `CLEAN-01`: Remove legacy code only after compatibility window and tests.

## Task Cards

Each card below is written as a TDD card. For audit and documentation cards, the "test" can be a failing documentation check, inventory assertion, contract test placeholder, or characterization test that proves the current gap before implementation starts.

Use this sequence for every card:

1. Red: add or update the smallest focused test, fixture, inventory assertion, or documentation check that fails for the current behavior.
2. Green: make the smallest production or documentation change needed to satisfy that test.
3. Refactor: simplify only the code or document structure touched by the card.
4. Verify: do not run validation from an agent session; update the relevant manual validation script only when code/tests change and ask the user to run it.

### AUD-01: Inventory Worker routes and ownership

- Status: Proposed
- Priority: P0
- Type: TDD Audit
- Area: Worker
- Problem: Worker routes mix health, updater, privacy, admin, license disable, reset, and webhook ownership in one route table.
- Current evidence: Routes are declared in `worker/src/index.js:39`, `worker/src/index.js:43`, `worker/src/index.js:47`, `worker/src/index.js:56`, `worker/src/index.js:62`, `worker/src/index.js:80`, `worker/src/index.js:95`, and `worker/src/index.js:98`.
- Target behavior: Every Worker route has a documented owner, authority source, retained/deprecated decision, auth requirement, and compatibility deadline.
- Red test first: Add a route inventory/security test or docs check that fails when any Worker route lacks owner, authority source, auth requirement, retained/deprecated decision, caller surface, response contract, or compatibility deadline.
- Green implementation: Create the route inventory table and fill each required field for every route.
- Refactor constraints: Keep this documentation-only until route decisions are implemented; do not change route behavior in this card.
- Security/privacy requirements: Mark routes that expose purchaser email, license hash, device binding data, audit metadata, tokens, or updater details.
- Done criteria: The failing inventory check can pass, and a maintainer can tell which routes remain after Devolens migration and why.
- Rollback notes: Route inventory is documentation-only and can be revised without runtime impact.
- Dependencies: None.

### AUD-02: Classify D1 tables as keep, deprecate, migrate, or delete later

- Status: Proposed
- Priority: P0
- Type: TDD Audit
- Area: Worker/D1
- Problem: D1 schema includes authoritative-looking license and device tables even though Devolens should own those concepts.
- Current evidence: D1 creates `licenses`, `device_bindings`, `reset_requests`, `idempotency_records`, and `audit_events` (`worker/migrations/0001_init.sql:1`, `worker/migrations/0001_init.sql:10`, `worker/migrations/0001_init.sql:19`, `worker/migrations/0001_init.sql:28`, `worker/migrations/0001_init.sql:38`) plus privacy deletion requests (`worker/migrations/0003_user_data_deletion_requests.sql:1`).
- Target behavior: Each table is classified as keep, deprecate, migrate, or delete later with a data retention and compatibility rule.
- Red test first: Add a schema inventory or migration contract check that fails when any D1 table or retained column lacks classification, retention rule, compatibility rule, or caller mapping.
- Green implementation: Build the table/column inventory, map read/write callers, decide retention, and define migration/anonymization behavior.
- Refactor constraints: Keep old tables available during compatibility analysis and do not modify migrations in this audit card.
- Security/privacy requirements: Do not retain purchaser emails, hashes, or device fingerprints longer than needed.
- Done criteria: Migration tests for retained columns and deleted authority paths are specified, and D1 is documented as non-authoritative except for explicitly retained workflow state.
- Rollback notes: Keep old tables read-only during compatibility period.
- Dependencies: `AUD-03`, `PRIV-01`.

### AUD-03: Locate every D1-as-license-authority read/write

- Status: Proposed
- Priority: P0
- Type: TDD Audit
- Area: Worker/D1
- Problem: D1 helpers create, read, disable, list, and anonymize license rows.
- Current evidence: `writeVerifiedGumroadSale` writes active license rows (`worker/src/store.js:67`), `getLicenseByHash` reads D1 entitlement (`worker/src/store.js:99`), hash-prefix lookup and entitlement updates exist (`worker/src/store.js:110`, `worker/src/store.js:123`), and admin listing reads D1 license/device state (`worker/src/store.js:400`, `worker/src/store.js:435`).
- Target behavior: No runtime decision treats D1 as final license authority.
- Red test first: Add contract tests that fail if activation, validation, deactivation, refund, or disable decisions can be made solely from D1 state.
- Green implementation: Tag each D1 read/write as authority, cache, mapping, audit, or privacy, then convert authority reads to Devolens-backed calls in later implementation cards.
- Refactor constraints: Do not remove D1 compatibility reads until Devolens-backed behavior has passing tests.
- Security/privacy requirements: Remove broad hash-prefix authority operations or gate them as audited support lookups.
- Done criteria: D1 license rows are used only for mapping/cache/audit after migration, with tests preventing D1-only authority decisions.
- Rollback notes: Keep old reads behind compatibility flags until Devolens paths pass manual validation.
- Dependencies: `DEV-02`, `DEV-03`.

### AUD-04: Inventory Devolens calls and token usage

- Status: Proposed
- Priority: P0
- Type: TDD Audit
- Area: Devolens integration
- Problem: Devolens calls are scattered across desktop auth, desktop privacy, and Worker webhook code.
- Current evidence: Desktop activation uses `api/key/Activate` (`app/src-tauri/src/auth_worker.rs:285`), validation parses Devolens session tokens (`app/src-tauri/src/auth_worker.rs:365`), reset calls `api/key/Deactivate` (`app/src-tauri/src/auth_worker.rs:432`), privacy calls `BlockKey` (`app/src-tauri/src/commands/privacy.rs:123`), and webhook calls `BlockKey`/`CreateKey` (`worker/src/index.js:1174`, `worker/src/index.js:1235`).
- Target behavior: Devolens endpoints, required token scopes, callers, error mapping, and retry rules are documented and centralized.
- Red test first: Add mocked endpoint-shape and token-scope tests that fail for undocumented Devolens endpoints or callers without an explicit scope.
- Green implementation: List endpoints and parameters, map tokens to scopes, document safe error handling, and identify calls that must move server-side.
- Refactor constraints: Preserve current endpoint behavior while moving calls behind wrappers in later cards.
- Security/privacy requirements: Never expose management-capable tokens to desktop clients.
- Done criteria: No Devolens call exists without a documented scope, caller, error mapping, and retry rule.
- Rollback notes: Keep existing endpoint behavior while moving calls behind wrappers.
- Dependencies: `DEV-01`, `DEV-05`.

### AUD-05: Document Tauri license/session/device storage

- Status: Proposed
- Priority: P0
- Type: TDD Audit
- Area: Tauri auth
- Problem: Deactivation must use the same device identity used during activation and must not clear local state too early.
- Current evidence: Secret store writes and fallback behavior are in `app/src-tauri/src/auth.rs:162`; local session clear removes fallback secrets (`app/src-tauri/src/auth.rs:219`); device identity is stored on disk (`app/src-tauri/src/auth.rs:235`) and created through `get_or_create_keypair` (`app/src-tauri/src/auth.rs:288`); auth state is built from config and app data dir (`app/src-tauri/src/auth.rs:320`).
- Target behavior: Storage responsibilities and clearing order are documented before deactivation changes.
- Red test first: Add failing storage-state tests for failed deactivation preserving retryable state and terminal deactivation clearing only approved state.
- Green implementation: Map license key, access token, device keypair, fingerprint, and auth state storage, then define which fields survive failed deactivation.
- Refactor constraints: Do not change storage behavior until the state machine is documented and covered.
- Security/privacy requirements: Do not store plaintext license keys outside current approved secret store behavior.
- Done criteria: The deactivation implementation has a clear state machine backed by preservation and clearing tests.
- Rollback notes: Restore prior `clear_local_session` behavior if deactivation rollout is paused.
- Dependencies: `APP-01`, `APP-03`.

### AUD-06: Document Gumroad webhook assumptions and failure paths

- Status: Proposed
- Priority: P0
- Type: TDD Audit
- Area: Worker/Gumroad
- Problem: Gumroad webhook flow mixes verification, Devolens mutation, D1 mutation, idempotency, and partial-failure handling.
- Current evidence: Webhook begins at `worker/src/index.js:1123`, requires form payload fields, uses D1 idempotency, verifies Gumroad, calls Devolens BlockKey/CreateKey, then writes D1 rows (`worker/src/index.js:1144`, `worker/src/index.js:1162`, `worker/src/index.js:1174`, `worker/src/index.js:1235`, `worker/src/index.js:1273`).
- Target behavior: Webhook states and failure outcomes are explicit and replay-safe.
- Red test first: Add contract tests that fail for invalid content type, duplicate payload mismatch, Gumroad failure, Devolens failure, D1 failure, and ambiguous retry behavior.
- Green implementation: Document valid payloads, duplicate sale handling, refund/dispute handling, Devolens retry behavior, and D1 write failures.
- Refactor constraints: Keep current webhook path while the new pipeline is characterized.
- Security/privacy requirements: Verify provider authenticity before any Devolens or D1 mutation.
- Done criteria: Operators know when to retry, when to block, and when to escalate, and tests cover each failure path.
- Rollback notes: Keep current webhook path while new pipeline is characterized.
- Dependencies: `WEBHOOK-01`, `WEBHOOK-02`.

### AUD-07: Document admin console behavior and duplicated Devolens functionality

- Status: Proposed
- Priority: P1
- Type: TDD Audit
- Area: Admin UI
- Problem: Admin UI duplicates license/device/reset operations that Devolens should own.
- Current evidence: Admin UI sections include reset, delete, licenses, device bindings, audit, and idempotency (`app/src/admin/AdminApp.svelte:22`); refresh loads per-section data (`app/src/admin/AdminApp.svelte:156`); disable action calls custom disable flow (`app/src/admin/AdminApp.svelte:266`); overview shows D1 license, binding, reset, deletion, and audit metrics (`app/src/admin/AdminApp.svelte:337`).
- Target behavior: Admin console only retains workflows that are safer or necessary outside Devolens.
- Red test first: Add UI/command inventory tests that fail when an admin section lacks a retain/remove/replace decision or required auth level.
- Green implementation: Inventory each admin screen and command, then decide remove, replace with Devolens console link/process, or retain with stronger auth.
- Refactor constraints: Do not remove UI sections until retained workflow decisions and replacement tests exist.
- Security/privacy requirements: Minimize broad listing of emails, license hashes, and device identifiers.
- Done criteria: Admin UI does not imply D1 is license source of truth, and removed/retained sections are covered by tests.
- Rollback notes: Hide sections behind feature flags during rollout if needed.
- Dependencies: `ADMIN-01`, `RESET-01`.

### DEV-01: Extract/harden a Worker-side Devolens bridge for CreateKey, BlockKey, Deactivate, and future support calls

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Worker/Devolens
- Problem: Worker Devolens calls are embedded directly in webhook branches.
- Current evidence: BlockKey and CreateKey calls are inline in `worker/src/index.js:1174` and `worker/src/index.js:1235`.
- Target behavior: A small Worker bridge owns Devolens URL construction, form encoding, token selection, timeout, error mapping, and redaction.
- Red test first: Add bridge unit tests for request shape, token selection, timeout behavior, safe errors, non-2xx responses, malformed JSON, and retryable classification before adding the bridge.
- Green implementation: Add a bridge module, move CreateKey and BlockKey, add Deactivate support, keep response errors safe, and update webhook/admin callers.
- Refactor constraints: Keep the bridge small and operation-specific; do not add broad generic Devolens abstractions.
- Security/privacy requirements: Never log tokens or raw keys; choose token by operation.
- Done criteria: No Worker route constructs Devolens URLs or forms directly, and bridge tests cover the operation contracts.
- Rollback notes: Revert bridge caller changes only; leave tests as characterization where possible.
- Dependencies: `AUD-04`, `DEV-05`.

### DEV-02: Make activation/validation/deactivation semantics explicitly Devolens-authoritative

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Auth semantics
- Problem: Desktop already uses Devolens for activation/validation/deactivation, but Worker/D1 and UI names still reflect reset-era authority.
- Current evidence: `build_worker_client` selects Devolens mode (`app/src-tauri/src/auth_worker.rs:205`), activation checks Devolens success and active license (`app/src-tauri/src/auth_worker.rs:329`), validation rejects invalid Devolens token shape (`app/src-tauri/src/auth_worker.rs:365`), and reset maps to Devolens deactivation (`app/src-tauri/src/auth_worker.rs:432`).
- Target behavior: Product behavior and docs say Devolens owns these decisions.
- Red test first: Add tests for invalid license, active license, revoked/blocked key, bound elsewhere, deactivation success, deactivation failure, and D1 disagreement with Devolens.
- Green implementation: Rename reset concepts where behavior changes, update contract names carefully, keep compatibility adapters, and remove D1 authority checks.
- Refactor constraints: Preserve unauthenticated gating and old command compatibility until replacement UI/contracts are tested.
- Security/privacy requirements: Do not allow unauthenticated access to app UI when Devolens validation fails.
- Done criteria: D1 disagreement cannot grant access or block a valid Devolens decision, and tests prove Devolens authority for each state.
- Rollback notes: Compatibility adapters can continue old command names while semantics are updated.
- Dependencies: `APP-01`, `DEV-03`.

### DEV-03: Reduce D1 license rows to mapping/cache only

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Worker/D1
- Problem: D1 `licenses` rows store entitlement status and purchaser email as if authoritative.
- Current evidence: D1 schema stores entitlement status (`worker/migrations/0001_init.sql:1`), Gumroad writes active rows (`worker/src/store.js:67`), and admin disable mutates entitlement status (`worker/src/store.js:123`).
- Target behavior: D1 rows are provider mapping/cache with TTL or explicit invalidation, not source-of-truth.
- Red test first: Add tests proving stale, missing, or contradictory D1 cache data cannot override Devolens license decisions.
- Green implementation: Rename or document cache semantics, stop using D1 entitlement for decisions, update writes to store mapping metadata, and add stale-cache handling.
- Refactor constraints: Keep old columns during the migration window and avoid destructive schema changes in the first pass.
- Security/privacy requirements: Minimize purchaser email storage; mask or hash where support does not need plaintext.
- Done criteria: License access state comes from Devolens, with D1 used only for mapping/cache/audit behavior covered by tests.
- Rollback notes: Keep old columns until migration window ends.
- Dependencies: `AUD-02`, `AUD-03`, `WEBHOOK-01`.

### DEV-04: Replace or remove disable-by-license-hash-prefix

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Admin/Worker
- Problem: Hash-prefix disable can affect the wrong license and mutates D1 state rather than Devolens authority.
- Current evidence: Disable route exists at `worker/src/index.js:95`; handler starts at `worker/src/index.js:885`; lookup/update helpers are `worker/src/store.js:110` and `worker/src/store.js:123`; Admin UI triggers disable at `app/src/admin/AdminApp.svelte:266`.
- Target behavior: Disable either calls Devolens BlockKey through an exact verified identifier or is removed in favor of Devolens console/support process.
- Red test first: Add tests for ambiguous prefix rejection, exact verified lookup, Devolens failure, audit write, and absence of D1 authority mutation.
- Green implementation: Decide remove vs exact support lookup, require exact key/sale lookup, call bridge BlockKey, write audit event, and remove hash-prefix mutation.
- Refactor constraints: Do not retain partial-hash mutation as a fallback path.
- Security/privacy requirements: Avoid prefix ambiguity and raw key logging.
- Done criteria: Admin cannot disable solely by partial hash, and disable behavior is either exact/audited or removed.
- Rollback notes: Retain route as disabled/deprecated with safe error during rollout.
- Dependencies: `DEV-01`, `ADMIN-02`.

### DEV-05: Separate Devolens token scopes for client, webhook, deactivation, and admin/support

- Status: Proposed
- Priority: P0
- Type: TDD Security
- Area: Secrets/config
- Problem: Token capabilities appear shared across client and management operations.
- Current evidence: Config loads one `devolens_access_token` and one product id (`app/src-tauri/src/core/config.rs:103`); Devolens client stores that token (`app/src-tauri/src/auth_worker.rs:226`); privacy and Worker management operations also use Devolens tokens (`app/src-tauri/src/commands/privacy.rs:123`, `worker/src/index.js:1174`, `worker/src/index.js:1235`).
- Target behavior: Distinct tokens exist for client Activate/Deactivate, webhook CreateKey/BlockKey, privacy/admin support, and optional validation.
- Red test first: Add config tests that reject missing operation tokens, privileged client tokens, wrong-scope tokens, and unredacted diagnostics.
- Green implementation: Define env vars, validate required scopes, update bridge/client config, update docs, and add redacted diagnostics.
- Refactor constraints: Allow the old env var only as a temporary compatibility alias with explicit warnings and reduced privileges.
- Security/privacy requirements: Desktop client must not receive CreateKey/BlockKey/GetKeys/GetProducts capable token.
- Done criteria: Compromise of a client token cannot create or block arbitrary keys, and scope tests enforce that boundary.
- Rollback notes: Support old env var only as a temporary compatibility alias with warnings.
- Dependencies: `AUD-04`, `DEV-01`.

### APP-01: Add current-device deactivation command/API contract

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Tauri auth
- Problem: Current command names expose support-style reset rather than user-controlled current-device deactivation.
- Current evidence: Tauri commands expose `request_device_reset` and status polling (`app/src-tauri/src/commands/auth.rs:22`) and local clear (`app/src-tauri/src/commands/auth.rs:38`).
- Target behavior: App exposes an explicit `deactivate_current_device` command with typed result and terminal/error states.
- Red test first: Add command inventory and contract tests for success, network failure, invalid license, already-deactivated terminal state, unauthenticated call, and capability registration.
- Green implementation: Add the command contract, route to Devolens deactivation, keep old reset commands as compatibility wrappers if needed, and register capabilities.
- Refactor constraints: Do not accept arbitrary machine code from UI; derive identity from active local auth state.
- Security/privacy requirements: Require active local auth state and never accept arbitrary machine code from UI.
- Done criteria: UI can deactivate current device without manual support reset, and terminal/error states are typed and tested.
- Rollback notes: Keep old reset commands until UI migration is complete.
- Dependencies: `APP-02`, `APP-03`.

### APP-02: Reuse the exact activation machine code/device identity for deactivation

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Tauri auth
- Problem: Deactivation must use the machine code Devolens associated with activation.
- Current evidence: Activation derives `DeviceId` from public key (`app/src-tauri/src/auth_worker.rs:330`), Deactivate uses `DeviceId::from_public_key` and sends `MachineCode` (`app/src-tauri/src/auth_worker.rs:439`), and device identity is persisted by `RuntimeDeviceIdentityProvider` (`app/src-tauri/src/auth.rs:235`, `app/src-tauri/src/auth.rs:288`).
- Target behavior: Deactivation always uses the persisted activation identity.
- Red test first: Add tests proving activation and deactivation machine codes match across restarts and that missing identity fails safely without regenerating before deactivation.
- Green implementation: Audit activation identity creation, prevent regeneration before deactivation, add a guard if keypair is missing, and document the recovery path.
- Refactor constraints: Preserve the old identity file format and keep private key material out of logs/errors.
- Security/privacy requirements: Do not expose private key material or full machine fingerprint.
- Done criteria: A successfully activated device can be deactivated using stored identity, with restart behavior covered by tests.
- Rollback notes: Preserve old identity file format.
- Dependencies: `AUD-05`, `APP-01`.

### APP-03: Clear local license/session/cache only after successful or explicitly terminal deactivation

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Tauri auth
- Problem: Clearing local state before confirmed deactivation can strand a device binding.
- Current evidence: `clear_session_secrets` removes local fallback file (`app/src-tauri/src/auth.rs:219`), while reset/deactivation currently returns approved status on Devolens success (`app/src-tauri/src/auth_worker.rs:472`).
- Target behavior: Local secrets are cleared only after Devolens success or a documented terminal state such as already deactivated.
- Red test first: Add tests for success clearing, transient failure preserving retry state, already-deactivated terminal clearing, and safe user-facing errors.
- Green implementation: Define terminal states, update command flow, keep retry state on transient failure, and surface safe UI messages.
- Refactor constraints: Keep `clear_local_session` only as an intentional local-only escape hatch if retained.
- Security/privacy requirements: Do not retain plaintext license longer than current storage policy permits.
- Done criteria: Failed deactivation leaves enough state to retry safely, while successful or terminal deactivation clears approved local state.
- Rollback notes: Existing `clear_local_session` remains manual local-only escape hatch if intentionally retained.
- Dependencies: `APP-01`, `APP-02`.

### APP-04: Replace Settings "Device Reset" UI with "Deactivate this device" confirmation UX

- Status: Proposed
- Priority: P1
- Type: TDD Implementation
- Area: Svelte UI
- Problem: Settings UI asks for a license key to request device reset, which conflicts with current-device deactivation.
- Current evidence: Settings reset handler starts at `app/src/routes/+page.svelte:380`; Settings reset panel starts at `app/src/routes/+page.svelte:1382`.
- Target behavior: Authenticated Settings UI offers "Deactivate this device" with clear confirmation and no re-entry of license key unless required by backend contract.
- Red test first: Add UI tests for authenticated visibility, unauthenticated hidden state, confirm, cancel, success, failure, and absence of license-key re-entry.
- Green implementation: Replace reset tab content, add confirmation UX, call the new Tauri command, update status messages, and keep reset compatibility only where needed.
- Refactor constraints: Keep privileged actions hidden from unauthenticated users and avoid exposing raw license material in UI state.
- Security/privacy requirements: Do not display raw license key; do not make privileged actions available unauthenticated.
- Done criteria: A user can release the current device from Settings, with all confirmation states covered by UI tests.
- Rollback notes: Keep old reset component behind compatibility branch during release window.
- Dependencies: `APP-01`, `APP-03`.

### APP-05: Update unauthenticated machine-limit flow messaging

- Status: Proposed
- Priority: P1
- Type: TDD UX
- Area: Svelte UI
- Problem: Machine-limit state tells users to request device reset instead of explaining Devolens machine binding/deactivation.
- Current evidence: Unauthenticated reset status and machine-bound flow appear at `app/src/routes/+page.svelte:918` and `app/src/routes/+page.svelte:961`.
- Target behavior: Messaging explains that the license is active on another device and gives the supported path: deactivate on the old device or contact support.
- Red test first: Add UI flow tests for machine-limit state, support path visibility, no reset polling when deprecated, and no extra license-existence disclosure.
- Green implementation: Update copy and actions, remove unsupported reset polling if the route is deprecated, and keep safe support instructions.
- Refactor constraints: Keep messaging factual and avoid changing activation behavior in this card.
- Security/privacy requirements: Do not reveal whether a specific license exists beyond existing contract behavior.
- Done criteria: Users no longer see admin-reset language for normal Devolens machine limits, and tests cover the machine-limit flow.
- Rollback notes: Restore old messaging only if legacy reset remains required.
- Dependencies: `RESET-01`, `APP-04`.

### RESET-01: Deprecate/remove admin reset list/approve/reject routes and UI

- Status: Proposed
- Priority: P1
- Type: TDD Implementation
- Area: Worker/Admin
- Problem: Admin reset approval duplicates Devolens deactivation and requires D1 reset request state.
- Current evidence: Worker reset routes are `worker/src/index.js:62` and `worker/src/index.js:80`; decision handler starts at `worker/src/index.js:501`; Admin UI reset section is part of `app/src/admin/AdminApp.svelte:22` and refresh logic at `app/src/admin/AdminApp.svelte:156`.
- Target behavior: Legacy admin reset routes are deprecated, then removed after compatible app releases.
- Red test first: Add tests for deprecated route response during the compatibility window, hidden/reset-free UI state, and final route removal after the window.
- Green implementation: Add deprecation response or hidden UI, monitor usage, then remove route handlers and D1 reset mutations after the compatibility window.
- Refactor constraints: Do not delete reset routes until compatible app releases and tests exist.
- Security/privacy requirements: Do not expose reset queues if no longer operational.
- Done criteria: No support workflow depends on D1 reset approval, and deprecation/removal states are tested.
- Rollback notes: Re-enable route handlers if old clients still require them during compatibility period.
- Dependencies: `APP-01`, `APP-05`.

### ADMIN-01: Remove broad custom admin license/device listing or convert to minimal support lookup

- Status: Proposed
- Priority: P1
- Type: TDD Implementation
- Area: Admin
- Problem: Admin license/device lists duplicate Devolens support functionality and expose sensitive data.
- Current evidence: Worker exposes admin license and device-binding routes (`worker/src/index.js:68`, `worker/src/index.js:71`); store helpers list license/device data (`worker/src/store.js:400`, `worker/src/store.js:435`); Admin UI has license/device sections (`app/src/admin/AdminApp.svelte:22`).
- Target behavior: Either remove broad lists or replace them with exact, audited support lookup backed by Devolens/mapping.
- Red test first: Add admin command and UI tests that fail when broad license/device listing is available without exact lookup, scoped auth, masking, and audit requirements.
- Green implementation: Decide retained support use cases, remove broad list UI, add exact lookup if needed, and update admin contracts.
- Refactor constraints: Do not add a new browseable customer dataset as a replacement.
- Security/privacy requirements: Minimize data returned; mask emails and hashes; audit all lookups.
- Done criteria: Admin cannot browse broad customer license/device datasets through custom UI, and retained lookup behavior is tested.
- Rollback notes: Temporarily retain read-only lists behind stronger auth if operationally necessary.
- Dependencies: `ADMIN-02`, `DEV-03`.

### ADMIN-02: Require stronger access control for retained support endpoints

- Status: Proposed
- Priority: P0
- Type: TDD Security
- Area: Admin/Worker
- Problem: Retained support endpoints need stronger controls than a broad shared admin token.
- Current evidence: `/readyz` checks `ADMIN_API_TOKEN` presence (`worker/src/index.js:110`), and all admin routes depend on `requireAdminAuth` before D1 operations such as reset decisions and disable (`worker/src/index.js:501`, `worker/src/index.js:885`).
- Target behavior: Retained support endpoints use least privilege, scoped credentials, audit logging, rate limits, and redacted responses.
- Red test first: Add auth tests for missing token, wrong scope, right scope, rate limit, actor/reason requirement, and audit redaction.
- Green implementation: Define endpoint scopes, add per-action auth policy, rate limit sensitive actions, and audit support actor and reason.
- Refactor constraints: Keep old token accepted only during migration with reduced privileges and explicit tests.
- Security/privacy requirements: No broad admin token for all read/write support operations.
- Done criteria: Support access is scoped by operation, and tests prove least-privilege behavior.
- Rollback notes: Keep old token accepted only during migration with reduced privileges.
- Dependencies: `AUD-07`, `DEV-05`.

### WEBHOOK-01: Refactor Gumroad webhook into thin verify/idempotency/Devolens/mapping pipeline

- Status: Proposed
- Priority: P0
- Type: TDD Implementation
- Area: Worker/Gumroad
- Problem: Current webhook interleaves provider verification, Devolens calls, D1 writes, and response creation.
- Current evidence: Webhook starts at `worker/src/index.js:1123`, does D1 idempotency around sale id, calls Devolens, and writes D1 rows (`worker/src/index.js:1144`, `worker/src/index.js:1235`, `worker/src/index.js:1273`).
- Target behavior: Webhook is a small pipeline: parse, verify Gumroad, check idempotency, call Devolens bridge, persist mapping/audit, return stable response.
- Red test first: Add Worker contract tests for parse, verification-before-mutation, idempotency replay, Devolens bridge failure, mapping/audit persistence, and stable response codes.
- Green implementation: Extract pure steps, centralize response contracts, make partial failures explicit, and update fixtures.
- Refactor constraints: Preserve Gumroad retry semantics and existing idempotency records while changing internals.
- Security/privacy requirements: Provider verification must precede Devolens/D1 mutation.
- Done criteria: Webhook code path is readable, replay-safe, and Devolens-authoritative, with every pipeline branch covered.
- Rollback notes: Keep old webhook tests to prove behavior parity during refactor.
- Dependencies: `DEV-01`, `WEBHOOK-02`.

### WEBHOOK-02: Define refund, duplicate, invalid payload, and Devolens failure behavior

- Status: Proposed
- Priority: P0
- Type: TDD Contract
- Area: Worker/Gumroad
- Problem: Failure behavior is partly implicit, including ignored BlockKey network errors on refunds.
- Current evidence: Refund/dispute path ignores Devolens BlockKey network errors (`worker/src/index.js:1183`), while CreateKey failures return retryable or non-retryable errors (`worker/src/index.js:1244`).
- Target behavior: Each webhook outcome has documented HTTP status, retryability, idempotency behavior, D1/audit effect, and operator action.
- Red test first: Add tests for duplicate matching payload, duplicate mismatched payload, refund BlockKey failure, CreateKey failure, missing license key, invalid provider payload, and retryability.
- Green implementation: Write the outcome matrix, implement explicit retry/audit states, and update fixtures.
- Refactor constraints: Preserve existing response codes until provider retry behavior is confirmed.
- Security/privacy requirements: Invalid provider payloads must not mutate Devolens or D1.
- Done criteria: Gumroad can safely retry without duplicate side effects, and every outcome has a tested contract.
- Rollback notes: Preserve existing response codes until provider retry behavior is confirmed.
- Dependencies: `AUD-06`, `WEBHOOK-01`.

### PRIV-01: Preserve privacy request/status flow while decoupling from license authority

- Status: Proposed
- Priority: P1
- Type: TDD Implementation
- Area: Privacy
- Problem: Privacy deletion must continue, but D1 license state should not be treated as license authority.
- Current evidence: Worker privacy routes are `worker/src/index.js:56`; deletion request handler starts at `worker/src/index.js:330`; store preview/anonymization touches license, bindings, and reset rows (`worker/src/store.js:321`, `worker/src/store.js:345`, `worker/src/store.js:352`, `worker/src/store.js:366`); desktop privacy can call Devolens in Devolens mode (`app/src-tauri/src/commands/privacy.rs:194`).
- Target behavior: Privacy flow tracks requests/status in D1 but delegates Devolens-owned data actions to Devolens support/API operations.
- Red test first: Add tests for request creation, status lookup, Devolens action success/failure, D1 anonymization, no D1 authority mutation, and masked responses.
- Green implementation: Separate D1 anonymization from Devolens block/delete action, document what Devolens can delete/anonymize, and keep lookup token status flow.
- Refactor constraints: Keep the privacy flow operational while license authority is moved out of D1.
- Security/privacy requirements: Continue masking license/email data and sanitize completed requests.
- Done criteria: Privacy compliance flow survives D1 license simplification, with Devolens and D1 responsibilities tested separately.
- Rollback notes: Keep current D1-only anonymization as fallback if Devolens action is unavailable, clearly marked incomplete.
- Dependencies: `DEV-01`, `AUD-02`.

### PRIV-02: Simplify admin privacy review and clarify Devolens-owned data operations

- Status: Proposed
- Priority: P2
- Type: TDD Implementation
- Area: Admin/Privacy
- Problem: Admin privacy approval may imply D1 deletion completes all licensing data deletion even when Devolens owns data.
- Current evidence: Admin privacy approval route starts at `worker/src/index.js:696`; admin delete request section is part of `app/src/admin/AdminApp.svelte:22`; privacy request table stores status and summary (`worker/migrations/0003_user_data_deletion_requests.sql:1`).
- Target behavior: Admin privacy UI distinguishes local D1 anonymization from Devolens-owned actions and support handoffs.
- Red test first: Add Admin UI and Worker tests for pending, approved, rejected, Devolens-action-success, Devolens-action-failure, local-D1-complete, and redacted review states.
- Green implementation: Update admin privacy status fields, add Devolens action result, revise UI copy, and audit decisions.
- Refactor constraints: Do not imply D1 cleanup equals complete Devolens deletion.
- Security/privacy requirements: Avoid showing raw license keys or full emails in review screens.
- Done criteria: Operators cannot mistake D1 cleanup for complete Devolens deletion, and privacy decision states are tested.
- Rollback notes: Keep existing summary fields while adding Devolens result metadata.
- Dependencies: `PRIV-01`.

### UPD-01: Decide static updater JSON vs dynamic Worker updater and implement the chosen path

- Status: Proposed
- Priority: P1
- Type: TDD Architecture
- Area: Updater
- Problem: Tauri updater currently points to a dynamic Worker route, which may be unnecessary complexity.
- Current evidence: Worker updater route is matched at `worker/src/index.js:47` and handled at `worker/src/index.js:204`; Tauri updater endpoint points to the Worker (`app/src-tauri/tauri.conf.json:37`, `app/src-tauri/tauri.conf.json:40`).
- Target behavior: Updater architecture is explicit: static signed JSON if possible, dynamic Worker only if needed for release policy.
- Red test first: Add updater endpoint validation and manifest generation tests for the chosen path before changing endpoint behavior.
- Green implementation: Compare requirements, choose static or dynamic, then update endpoint/docs for static or harden route validation/readiness dependencies for dynamic.
- Refactor constraints: Preserve signed updater verification and keep the previous Worker endpoint available until the replacement is manually validated.
- Security/privacy requirements: Preserve signed updater verification and HTTPS-only update URLs.
- Done criteria: Updater path is simple, documented, manually validated, and covered by endpoint/manifest tests.
- Rollback notes: Repoint Tauri endpoint to prior Worker route if static hosting fails.
- Dependencies: `HEALTH-01`, `DOC-01`.

### HEALTH-01: Harden `/health` and `/readyz` response safety

- Status: Proposed
- Priority: P1
- Type: TDD Security
- Area: Worker
- Problem: Readiness exposes detailed table/secret/config checks and may fetch updater manifest in deep mode.
- Current evidence: `/health` returns simple status (`worker/src/index.js:39`); `/readyz` starts at `worker/src/index.js:110` and includes D1, secrets, config, and deep updater checks.
- Target behavior: Public health is minimal; readiness details are authenticated or redacted; deep checks do not leak secrets or topology.
- Red test first: Add tests for unauthenticated public health response, unauthenticated readiness redaction/denial, authenticated detailed readiness, deep-check authorization, and no secret/topology leakage.
- Green implementation: Split public health from private readiness, redact check details, protect deep mode, and document operational use.
- Refactor constraints: Keep health behavior minimal and avoid coupling public readiness to updater manifest fetches.
- Security/privacy requirements: Do not reveal which secrets are configured to unauthenticated callers.
- Done criteria: Health endpoints are safe to expose, and readiness detail is authenticated or redacted in tests.
- Rollback notes: Keep old readyz behind admin auth during transition.
- Dependencies: `ADMIN-02`.

### DOC-01: Update operator/support/release docs after behavior changes

- Status: Proposed
- Priority: P2
- Type: TDD Documentation
- Area: Docs
- Problem: Docs must match Devolens source-of-truth behavior after implementation.
- Current evidence: Existing docs are dirty in this working tree, so this task intentionally creates a new source-of-truth task-card file rather than editing them.
- Target behavior: Operator, support, privacy, Gumroad, updater, and release docs describe the final architecture and manual validation commands.
- Red test first: Add documentation checks or review checklist items that fail while operator/support/release docs still describe reset-era or D1-authority behavior.
- Green implementation: Update docs only after behavior changes land, remove reset-era instructions, and document Devolens support workflows and token scopes.
- Refactor constraints: Do not edit unrelated dirty documentation as part of behavior cards.
- Security/privacy requirements: Do not include secrets, real keys, raw customer emails, or private endpoints.
- Done criteria: Support and release operators have one consistent procedure set, with manual validation references present.
- Rollback notes: Keep legacy docs labeled for old versions only if needed.
- Dependencies: All implementation cards.

### CLEAN-01: Remove legacy code only after compatibility window and tests

- Status: Proposed
- Priority: P2
- Type: TDD Cleanup
- Area: Worker/Tauri/UI/Admin
- Problem: Reset-era and D1-authority code should not be removed until new Devolens flows are validated.
- Current evidence: Legacy reset commands, Worker routes, D1 tables, admin UI sections, and tests remain across `app/src-tauri/src/commands/auth.rs:22`, `worker/src/index.js:62`, `worker/migrations/0001_init.sql:19`, and `app/src/admin/AdminApp.svelte:22`.
- Target behavior: Legacy routes, commands, UI, tests, and tables are removed or frozen only after compatible releases and rollback plan.
- Red test first: Add tests that fail if legacy reset or D1-authority paths remain reachable after the compatibility window, while preserving audit/privacy records.
- Green implementation: Define compatibility window, track old client usage, remove code in small PRs, and keep migration scripts non-destructive until data retention expires.
- Refactor constraints: Cleanup is last; do not remove compatibility wrappers until replacement paths and rollback plan are tested.
- Security/privacy requirements: Do not delete audit/privacy records prematurely.
- Done criteria: No dead reset/D1-authority paths remain after the window, and the full manual validation suite is referenced before and after cleanup.
- Rollback notes: Cleanup is last; rollback by restoring compatibility wrappers if old clients surface.
- Dependencies: `RESET-01`, `APP-04`, `WEBHOOK-01`, `PRIV-01`.

## Validation Strategy

No validation script is needed for this documentation-only task.

Repository policy forbids agents from running test, build, install, formatter, or validation commands. The following commands are manual-only and should be run by the user after implementation work related to these cards:

```bash
.scripts/run-devolens-only-validation.sh
.scripts/run-admin-devolens-validation.sh
.scripts/run-privacy-devolens-validation.sh
.scripts/run-updater-endpoint-validation.sh
pnpm run worker:test
pnpm --dir app run test
cargo test --locked --manifest-path app/src-tauri/Cargo.toml --test auth_worker_tests --test config_tests --test admin_devolens_tests
.venv/bin/python -m pytest tests/parity/license_worker_contract_v1_parity_test.py tests/migration_validation.py
```

For future code changes, add or update focused tests first where behavior is risky, then update the relevant `.scripts/` validation runner without executing it from an agent session.

## Rollback Strategy

- Keep compatibility wrappers for old reset commands and routes until a release window confirms they are unused.
- Move Devolens bridge extraction behind existing contracts first; do not remove old code until tests prove parity.
- Preserve D1 schema during migration. Mark rows as mapping/cache before deleting columns or tables.
- For webhook changes, keep idempotency records and stable response contracts so Gumroad retries remain safe.
- For UI changes, keep old reset states readable even if actions are hidden, so older persisted auth state does not break rendering.
- For updater changes, retain the previous Worker endpoint until the replacement static or dynamic endpoint is manually validated.

## Open Questions / Repository-Specific Unknowns

- Which Devolens token scopes are available in production, and can CreateKey, BlockKey, Deactivate, Activate, Validate, and support lookup be split into separate credentials?
- Does Devolens provide a first-class current-device deactivation response that can distinguish success from already-deactivated terminal state?
- Does Gumroad send enough signed/verified refund or dispute data to make BlockKey retry behavior deterministic without manual support intervention?
- Are any released desktop versions still calling Worker reset request/status routes?
- Should the updater stay on the Worker for release-channel policy, or can signed static JSON satisfy all release requirements?
- What support lookup information is genuinely required outside the Devolens console?
- What is the retention requirement for purchaser emails, license hashes, reset requests, device bindings, and privacy deletion requests after Devolens becomes authoritative?

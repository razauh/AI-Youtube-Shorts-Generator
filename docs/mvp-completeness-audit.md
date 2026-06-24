# MVP Completeness Audit

## 1. Purpose

This audit evaluates whether the production MVP is complete, usable, integrated, and release-ready across the customer desktop app, admin desktop app, Tauri command boundary, Rust/MuAPI pipeline, licensing/auth integration, Cloudflare Worker, updater, legal/privacy content, secure storage, diagnostics, legacy Python separation, and packaging/runtime bundle surface.

## 2. Audit Scope

Production/runtime files inspected:

- Customer app: `app/src/main.js`, `app/src/App.svelte`, `app/src/routes/+page.svelte`, `app/src/design-tokens.css`, `app/src/lib/components/FormStatus.svelte`, `app/src/lib/components/ThemedSelect.svelte`, `app/src/lib/stores/authState.ts`, `app/src/lib/stores/runState.ts`
- Customer clients/contracts: `app/src/lib/api/*.ts`, `app/src/lib/authContracts.ts`, `app/src/lib/contracts.ts`
- Admin app: `app/src/admin/*`, `app/src-tauri/src/bin/admin_desktop.rs`, `app/src-tauri/tauri.admin.conf.json`, `app/src-tauri/capabilities/admin.json`
- Tauri shell/commands: `app/src-tauri/src/main.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/src/commands/*.rs`, `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json`
- Rust generation: `app/src-tauri/src/core/*.rs`, `app/src-tauri/src/core/api_mode/*.rs`, `app/src-tauri/src/core/network/client.rs`
- Auth/licensing: `app/src-tauri/src/auth.rs`, `app/src-tauri/src/auth_worker.rs`, selected production vendor modules under `vendor/license-control-suite/src/modules/user_reg/` and `vendor/license-control-suite/src/modules/auth_core/`
- Worker: `worker/src/index.js`, `worker/src/contracts.js`, `worker/src/store.js`, `worker/wrangler.toml`, `worker/migrations/*.sql`
- Storage/support/legal/legacy/packaging production docs and source named in the task.

## 3. Method

I used existing `graphify-out` artifacts as the structural starting point:

- `graphify-out/GRAPH_REPORT.md` reported 1,930 nodes and 4,742 edges.
- `graphify-out/MERGED_GRAPH_REPORT.md` reported merged app/worker/vendor/docs/packaging scopes and highlighted command, Worker, auth, generation, runtime, and admin communities.
- `graphify-out/.graphify_detect.json` showed the corpus includes tests, so Graphify was used only as a navigation aid.

I then verified every material claim against production source with direct file inspection and source search. Where Graphify output was stale or ambiguous, current source files were treated as controlling.

## 4. README Usage Limitation

Project `README.md` was not used as source of truth for MVP completeness. Packaging README files were inspected only as secondary documentation evidence for packaging/runtime documentation mismatch checks.

## 5. Test Exclusion Statement

Tests, fixtures, snapshots, golden files, parity/regression directories, UI tests, contract tests, updater config tests, and test scripts were excluded from the audit evidence and verdict. No finding below relies on test implementation or test coverage.

## 6. Executive Summary

**MVP INCOMPLETE**

What appears to work end-to-end:

- Customer shell boots through `app/src/main.js` -> `App.svelte` -> `routes/+page.svelte`.
- Licensed customer UI is gated behind auth state and includes activation, reset, MuAPI setup, generation, progress, output links, library, policies, deletion request, diagnostics, crash draft, and customer updater controls.
- Customer generation command path is connected: Svelte route -> `tauriClient.ts` -> `generate_shorts_stream` -> Rust auth gate -> Rust API pipeline -> MuAPI stages.
- Admin app is a separate entry point/binary/config and connects to admin Tauri commands and Worker admin routes.
- Worker routes exist for activation, validation, reset request/status, Gumroad webhook, user data deletion, admin listing, admin reset decisions, admin deletion decisions, license disabling, and customer update checks.

What is missing or incomplete:

- Worker activation does not enforce the one-active-device/device-bound contract expected by the desktop auth UI and reset flow.
- Pipeline can report generation success when every clip render failed and no usable clip URL exists.
- Runtime secure-store fallback writes plaintext API/admin/deletion secrets into app-data fallback files when OS keychain storage fails.
- `app/src/DELETION_NOTICE.md` from the required policy inventory is missing.
- Admin updater support is absent while customer updater support is present.
- Several stale/deprecated or over-broad command/storage surfaces remain exposed.

Release blockers:

- `MVP-CRIT-001`: device binding/reset contract is not enforced by the Worker.
- `MVP-CRIT-002`: successful generation does not guarantee usable generated clips.

Deferrable:

- Packaging README polish, stale OpenAI configuration leftovers, unguarded migration rerun behavior, and cleanup of unused frontend storage abstractions can be deferred if the release explicitly accepts the risk.

Needs manual decision:

- Whether admin desktop requires the same updater mechanism as the customer app for MVP.
- Whether API/admin/deletion secrets may ever fall back to plaintext local files.
- Whether OpenAI config/profile support is intentionally reserved for future providers or should be removed from MVP.
- Whether advanced runtime filesystem/machine-secret commands are intended customer command surface.

## 7. MVP Status Matrix

| Area | Status | Evidence | Main gaps | Priority |
| --- | --- | --- | --- | --- |
| Customer desktop app | Partial | `app/src/main.js`, `app/src/App.svelte`, `app/src/routes/+page.svelte`, `app/src/lib/stores/*.ts` | UI is connected, but success can contain no usable clips; crash redaction is narrow. | Critical |
| Customer API clients | Partial | `app/src/lib/api/tauriClient.ts`, `authClient.ts`, `runtimeClient.ts`, `updaterClient.ts` | Runtime client exposes unused/broad filesystem and machine-secret commands; OpenAI provider contract remains without UI workflow. | Medium |
| Admin desktop app | Partial | `app/src/admin/AdminApp.svelte`, `adminClient.ts`, `admin_desktop.rs`, `tauri.admin.conf.json` | Core admin workflow exists; admin updater is absent; detail modal can show full sanitized objects but depends on Worker masking. | High |
| Tauri desktop shell and command boundary | Partial | `app/src-tauri/src/main.rs`, `src/bin/admin_desktop.rs`, `commands/*.rs`, capabilities JSON | Customer registers deprecated/unused generation and broad runtime commands; admin/customer command surfaces are separate. | Medium |
| Rust generation / MuAPI pipeline | Partial | `core/pipeline.rs`, `api_mode/*.rs`, `core/contracts.rs` | End-to-end path exists; all failed clips still produce `ok: true`; no hard validation that at least one rendered clip URL exists. | Critical |
| Licensing/auth desktop integration | Partial | `auth.rs`, `auth_worker.rs`, `commands/auth.rs`, `authState.ts` | Desktop expects device-bound errors and reset lifecycle; Worker does not enforce device-bound activation. | Critical |
| License Worker service | Partial | `worker/src/index.js`, `store.js`, `contracts.js`, migrations | Routes/schema exist; device binding policy missing; token/idempotency primitives are weak for security-sensitive MVP. | Critical |
| Updater | Partial | `updaterClient.ts`, `tauri.conf.json`, `tauri.admin.conf.json`, `worker/src/index.js` | Customer updater wired; admin updater not wired/configured; update endpoint depends on external manifest config. | High |
| Policies/legal/privacy/refund | Partial | `policiesContent.ts`, `routes/+page.svelte`; missing `app/src/DELETION_NOTICE.md` | In-app text exists, but required deletion notice file is missing and secure-storage claims need manual reconciliation. | High |
| Secure/local storage | Partial | `commands/runtime.rs`, `auth.rs`, `app/src/storage/*.ts` | Runtime secure fallback writes plaintext; machine secret command is exposed; frontend storage abstractions appear unused. | High |
| Crash/support diagnostics | Partial | `support/crashDraft.ts`, `routes/+page.svelte` | Draft generation/UI exists; redaction misses emails, API keys, tokens, local paths/usernames, and machine IDs. | High |
| Legacy Python pipeline | Complete for separation / Needs review for retention | `python_legacy/main.py`, `python_legacy/shorts_generator/pipeline.py`; source search | Not wired into production customer app; retained as legacy/reference. Retention/removal needs product decision. | Low |
| Packaging/runtime bundles | Partial | `packaging/*/README.md`, `tauri.conf.json`, source search | Docs say no local runtime bundles for API-only release; packaging docs are sparse and reference excluded smoke scripts as validation. | Medium |

## 8. Critical Blockers

### MVP-CRIT-001

- **MVP area:** Licensing/auth desktop integration; License Worker service
- **Classification:** worker / backend
- **File path:** `worker/src/index.js`; `worker/src/store.js`; `app/src/lib/stores/authState.ts`; `app/src-tauri/src/auth_worker.rs`
- **Function/component/type/command/config/string involved:** `handleActivate`, `upsertDeviceBinding`, `device_already_bound`, `device_bound_elsewhere`
- **What is missing or incomplete:** Activation does not check for an existing active binding for the same license on another device before upserting the new `device_id`.
- **Why it matters for MVP completeness:** The desktop app has a device-bound-elsewhere lifecycle and reset request workflow, and the auth adapter maps `AuthError::DeviceAlreadyBound` to `device_already_bound`. The Worker currently allows multiple active devices for one license, so the main license-enforcement behavior is incomplete.
- **Evidence from code or Graphify:** Graphify identified auth and reset communities around activation/reset. Source verification shows `handleActivate` validates license status and writes `upsertDeviceBinding` but performs no active-binding-by-license conflict check (`worker/src/index.js:346`, `worker/src/index.js:403`). `upsertDeviceBinding` keys only by `device_id` and updates that row (`worker/src/store.js:134`). The frontend explicitly handles `device_bound_elsewhere` on activation errors (`app/src/lib/stores/authState.ts:387`). The Rust auth worker includes a `DeviceAlreadyBound` contract code (`app/src-tauri/src/auth_worker.rs:50`).
- **Recommended action:** Add a Worker query for active device bindings by `license_key_hash`; allow same `device_id`, reject different active device with `device_already_bound`, and keep reset approval deactivating prior bindings.
- **Priority:** Critical
- **Risk level:** High
- **Confidence:** High

### MVP-CRIT-002

- **MVP area:** Rust generation / MuAPI pipeline; Customer desktop app
- **Classification:** backend / frontend
- **File path:** `app/src-tauri/src/core/api_mode/clipper.rs`; `app/src-tauri/src/core/pipeline.rs`; `app/src/routes/+page.svelte`
- **Function/component/type/command/config/string involved:** `crop_highlights`, `generate_shorts_with_progress`, `runState.onSuccess`, project `status = 'exported'`
- **What is missing or incomplete:** Clip rendering failures are stored per-short with `clip_url: null`, but the pipeline still returns `PipelineSuccess` even if all clips failed. The UI then marks the run/project as successful/exported.
- **Why it matters for MVP completeness:** The expected MVP flow ends with generated clips/output. A successful run with zero usable clip URLs is not release-ready behavior unless explicitly treated as partial failure.
- **Evidence from code or Graphify:** Graphify highlighted `crop_highlights` and `PipelineSuccess` as core generation nodes. Source verification shows `crop_highlights` catches each `crop_clip` error and pushes a short with `clip_url = null` and `error` (`app/src-tauri/src/core/api_mode/clipper.rs:77`). `generate_shorts_with_progress` deserializes those values and returns `Ok(PipelineSuccess)` without requiring any usable clip URL (`app/src-tauri/src/core/pipeline.rs:273`). The route sets `status = envelope.ok ? 'exported' : 'draft'` and calls `runState.onSuccess` for any `ok` envelope (`app/src/routes/+page.svelte:831`, `app/src/routes/+page.svelte:846`).
- **Recommended action:** Treat zero successful clip URLs as a failure or explicit partial-success state; surface retry guidance and do not mark the project exported unless at least one generated clip is usable.
- **Priority:** Critical
- **Risk level:** High
- **Confidence:** High

## 9. High-Priority Incomplete Items

### MVP-HIGH-001

- **MVP area:** Secure/local storage
- **Classification:** backend / frontend / admin
- **File path:** `app/src-tauri/src/commands/runtime.rs`; `app/src/routes/+page.svelte`; `app/src-tauri/src/commands/admin.rs`
- **Function/component/type/command/config/string involved:** `secure_store_save`, `save_secure_fallback`, `api_key_profile_add`, `admin_config_save`, `USER_DATA_DELETION_LOOKUP_TOKEN`
- **What is missing or incomplete:** Runtime secure storage falls back to writing plaintext values to `secure-fallback/{key}.secret` when keychain operations fail.
- **Why it matters for MVP completeness:** MuAPI keys, admin API tokens, and deletion lookup tokens are sensitive. Plaintext fallback contradicts a secure-by-default MVP posture and needs an explicit product/security decision.
- **Evidence from code or Graphify:** `secure_store_save` falls back to `save_secure_fallback` on keychain error (`app/src-tauri/src/commands/runtime.rs:756`). `save_secure_fallback` writes the raw value directly (`app/src-tauri/src/commands/runtime.rs:333`). API key profiles store secrets through this path (`app/src-tauri/src/commands/runtime.rs:811`), admin config stores the admin token through it (`app/src-tauri/src/commands/admin.rs:571`), and deletion lookup tokens use `secureStoreSave` in the customer route (`app/src/routes/+page.svelte:437`).
- **Recommended action:** Encrypt runtime secure fallback or fail closed for secrets when OS credential storage is unavailable. Decide whether Linux/keychain-unavailable fallback is allowed for MVP.
- **Priority:** High
- **Risk level:** High
- **Confidence:** High

### MVP-HIGH-002

- **MVP area:** Secure/local storage; Tauri command boundary
- **Classification:** backend / frontend
- **File path:** `app/src-tauri/src/main.rs`; `app/src-tauri/src/commands/runtime.rs`; `app/src/lib/api/runtimeClient.ts`
- **Function/component/type/command/config/string involved:** `runtime_machine_secret`, `runtimeMachineSecret`
- **What is missing or incomplete:** A command returning the runtime machine secret is registered and exported to frontend code, but no production UI flow uses it.
- **Why it matters for MVP completeness:** Exposing a machine secret over the general frontend command boundary is unnecessary attack surface for a production MVP.
- **Evidence from code or Graphify:** The customer Tauri handler registers `runtime_machine_secret` (`app/src-tauri/src/main.rs:32`), the command returns `get_or_create_machine_secret()` (`app/src-tauri/src/commands/runtime.rs:663`), and the frontend exports `runtimeMachineSecret()` (`app/src/lib/api/runtimeClient.ts:75`). Source search found no production caller beyond the export/registration.
- **Recommended action:** Remove or restrict the command unless a production feature requires it. If needed, document threat model and scope.
- **Priority:** High
- **Risk level:** High
- **Confidence:** High

### MVP-HIGH-003

- **MVP area:** Policies, legal, privacy, refund text
- **Classification:** docs / frontend
- **File path:** `app/src/DELETION_NOTICE.md`; `app/src/lib/legal/policiesContent.ts`; `app/src/routes/+page.svelte`
- **Function/component/type/command/config/string involved:** required `DELETION_NOTICE.md`; `POLICY_SECTIONS.deletion`
- **What is missing or incomplete:** The required inventory file `app/src/DELETION_NOTICE.md` does not exist. The in-app deletion notice exists in `policiesContent.ts`.
- **Why it matters for MVP completeness:** The required production policy surface is partly missing as a file artifact. Release packaging/support may expect the standalone deletion notice.
- **Evidence from code or Graphify:** File inspection returned `No such file or directory` for `app/src/DELETION_NOTICE.md`. In-app deletion content is present under `POLICY_SECTIONS.deletion` (`app/src/lib/legal/policiesContent.ts:189`) and rendered in the Policies tab (`app/src/routes/+page.svelte:1503`).
- **Recommended action:** Create the standalone deletion notice or remove it from the MVP inventory after a manual product decision.
- **Priority:** High
- **Risk level:** Medium
- **Confidence:** High

### MVP-HIGH-004

- **MVP area:** Updater; Admin desktop app
- **Classification:** admin / config
- **File path:** `app/src-tauri/src/bin/admin_desktop.rs`; `app/src-tauri/tauri.admin.conf.json`; `app/src-tauri/capabilities/admin.json`; `app/src/admin/AdminApp.svelte`
- **Function/component/type/command/config/string involved:** Tauri updater plugin/config, admin maintenance UI
- **What is missing or incomplete:** The customer app wires the updater plugin and update UI, but the admin app does not register the updater plugin, does not configure updater endpoints, and has no admin update UI.
- **Why it matters for MVP completeness:** If admin desktop is part of the release surface, its update/security patch path is incomplete compared with the customer app.
- **Evidence from code or Graphify:** Customer main registers `tauri_plugin_updater` (`app/src-tauri/src/main.rs:8`) and customer config includes updater `pubkey`/`endpoints` (`app/src-tauri/tauri.conf.json:51`). Admin binary registers only admin commands (`app/src-tauri/src/bin/admin_desktop.rs:4`), admin config has no `plugins.updater` block (`app/src-tauri/tauri.admin.conf.json:60`), and admin capability has only `core:default` (`app/src-tauri/capabilities/admin.json:13`).
- **Recommended action:** Decide whether admin updates are required for MVP. If yes, add updater plugin/config/UI for admin; if no, document the manual admin update path.
- **Priority:** High
- **Risk level:** Medium
- **Confidence:** High

### MVP-HIGH-005

- **MVP area:** Crash/support diagnostics
- **Classification:** frontend
- **File path:** `app/src/support/crashDraft.ts`; `app/src/routes/+page.svelte`
- **Function/component/type/command/config/string involved:** `redactSensitiveText`, `submitPendingCrashDraft`
- **What is missing or incomplete:** Crash draft redaction only covers one license-like hex pattern, `license_key`, and generic `secret`; it does not redact emails, API keys, access tokens, admin tokens, local paths/usernames, device IDs, or machine identifiers.
- **Why it matters for MVP completeness:** The app lets users submit crash drafts to a configured endpoint, and support diagnostics must not leak sensitive data.
- **Evidence from code or Graphify:** Crash drafts are created from window errors and unhandled rejections (`app/src/routes/+page.svelte:623`) and can be posted to `VITE_CRASH_REPORT_ENDPOINT` (`app/src/routes/+page.svelte:645`). Redaction is limited to three regex replacements (`app/src/support/crashDraft.ts:18`).
- **Recommended action:** Expand redaction for emails, common token/API key names, bearer tokens, license formats, device IDs, local user paths, and known secret keys before enabling crash submission in production.
- **Priority:** High
- **Risk level:** High
- **Confidence:** High

### MVP-HIGH-006

- **MVP area:** License Worker service
- **Classification:** worker / security
- **File path:** `worker/src/index.js`; `worker/src/store.js`
- **Function/component/type/command/config/string involved:** `issueAccessToken`, `verifyAccessToken`, `stableHash`
- **What is missing or incomplete:** Worker token signing uses `sha256(secret:payload)` and direct string comparison; idempotency payload hashing uses a non-cryptographic 32-bit hash with a comment saying it is scaffold behavior.
- **Why it matters for MVP completeness:** Licensing is security-sensitive. Token signing and idempotency hashing should be robust and clearly production-grade.
- **Evidence from code or Graphify:** Access token issue/verify uses `sha256Hex(`${secret}:${payloadB64}`)` and `signature !== expected` (`worker/src/index.js:1860`). `stableHash` is a simple rolling hash and the comment says “Swap with stronger canonical hashing in production implementation” (`worker/src/store.js:29`).
- **Recommended action:** Replace token signature with WebCrypto HMAC-SHA-256 and constant-time verification where possible; replace idempotency hash with canonical JSON plus SHA-256.
- **Priority:** High
- **Risk level:** High
- **Confidence:** High

## 10. Medium/Low-Priority Gaps

### MVP-MED-001

- **MVP area:** Customer API clients; Secure/local storage
- **Classification:** frontend / backend
- **File path:** `app/src/lib/api/runtimeClient.ts`; `app/src-tauri/src/commands/runtime.rs`; `app/src/storage/*.ts`
- **Function/component/type/command/config/string involved:** runtime filesystem commands, frontend storage abstractions
- **What is missing or incomplete:** Many runtime filesystem APIs and frontend storage abstractions are exported but not used by the production customer UI.
- **Why it matters for MVP completeness:** Unused command surface increases review and maintenance burden for a security-sensitive desktop app.
- **Evidence from code or Graphify:** Runtime client exports read/write/append/remove/list/rename/chmod/size APIs (`app/src/lib/api/runtimeClient.ts:79`). Customer Tauri registers those commands (`app/src-tauri/src/main.rs:33`). Source search found no production callers outside exports/registration.
- **Recommended action:** Remove unused commands/exports or document the production feature that requires them.
- **Priority:** Medium
- **Risk level:** Medium
- **Confidence:** Medium

### MVP-MED-002

- **MVP area:** Tauri command boundary
- **Classification:** backend
- **File path:** `app/src-tauri/src/main.rs`; `app/src-tauri/src/commands/generate.rs`; `app/src/lib/api/tauriClient.ts`
- **Function/component/type/command/config/string involved:** `generate_shorts`, `generate_shorts_with_events`, `generate_shorts_stream`
- **What is missing or incomplete:** Customer frontend uses only `generate_shorts_stream`, but `generate_shorts` and `generate_shorts_with_events` remain registered.
- **Why it matters for MVP completeness:** Deprecated/alternate generation commands should be removed or justified before release to reduce behavioral variants.
- **Evidence from code or Graphify:** Frontend invokes only `generate_shorts_stream` (`app/src/lib/api/tauriClient.ts:61`). Main registers three generation commands (`app/src-tauri/src/main.rs:25`), and the command file exposes all three (`app/src-tauri/src/commands/generate.rs:299`).
- **Recommended action:** Keep only the production command or document why the alternate commands remain supported.
- **Priority:** Medium
- **Risk level:** Medium
- **Confidence:** High

### MVP-MED-003

- **MVP area:** Customer API clients; Removed local-processing leftovers
- **Classification:** frontend / backend
- **File path:** `app/src-tauri/src/core/config.rs`; `app/src-tauri/src/commands/runtime.rs`; `app/src/lib/api/runtimeClient.ts`; `app/src/routes/+page.svelte`
- **Function/component/type/command/config/string involved:** `OPENAI_API_KEY`, `OPENAI_MODEL`, provider `'openai'`
- **What is missing or incomplete:** OpenAI provider configuration remains in runtime/config contracts even though the current UI supports only MuAPI profiles and the Rust generation path uses MuAPI `gpt-5-mini` through MuAPI.
- **Why it matters for MVP completeness:** This is a stale or future-provider contract surface that can confuse setup and policy claims.
- **Evidence from code or Graphify:** Config reads `OPENAI_API_KEY` and `OPENAI_MODEL` (`app/src-tauri/src/core/config.rs:77`), runtime provider normalization accepts `openai` (`app/src-tauri/src/commands/runtime.rs:189`), and frontend type allows `'openai'` (`app/src/lib/api/runtimeClient.ts:27`). The customer UI only loads/saves MuAPI profiles (`app/src/routes/+page.svelte:265` and `app/src/routes/+page.svelte:536`).
- **Recommended action:** Mark OpenAI provider support as reserved/future or remove it from the MVP runtime surface.
- **Priority:** Medium
- **Risk level:** Low
- **Confidence:** High

### MVP-MED-004

- **MVP area:** License Worker service
- **Classification:** worker
- **File path:** `worker/src/index.js`; `worker/wrangler.toml`
- **Function/component/type/command/config/string involved:** `/readyz`, `GUMROAD_ACCESS_TOKEN`, `UPDATE_MANIFEST_URL`
- **What is missing or incomplete:** Readiness reports `gumroad_ok` but does not include Gumroad readiness in the final `secrets.ok` gate.
- **Why it matters for MVP completeness:** Gumroad webhook purchase ingestion is part of the licensing MVP. `/readyz` can report ready even when Gumroad purchase verification cannot work.
- **Evidence from code or Graphify:** `/readyz` computes `gumroad_ok` (`worker/src/index.js:155`) but sets `checks.secrets.ok = checks.secrets.core_ok && checks.secrets.admin_ok` (`worker/src/index.js:156`). `wrangler.toml` configures the Worker but secret presence is runtime-only (`worker/wrangler.toml:5`).
- **Recommended action:** Include Gumroad readiness in `/readyz` or add a separate readiness status that clearly states purchase ingestion is unavailable.
- **Priority:** Medium
- **Risk level:** Medium
- **Confidence:** High

### MVP-MED-005

- **MVP area:** Packaging/runtime bundles
- **Classification:** packaging / docs
- **File path:** `packaging/linux/README.md`, `packaging/macos/README.md`, `packaging/windows/README.md`
- **Function/component/type/command/config/string involved:** local runtime/model bundles; smoke validation text
- **What is missing or incomplete:** Packaging docs correctly say local runtime/model bundles are not packaged for API-only release, but they are sparse and rely on excluded smoke test scripts for validation instructions.
- **Why it matters for MVP completeness:** Production release packaging needs clear non-test operational packaging notes, including updater artifacts, signing/notarization expectations, and runtime asset absence.
- **Evidence from code or Graphify:** Packaging READMEs state “Local runtime/model bundles are not packaged for the API-only release path” (`packaging/linux/README.md:7`, `packaging/macos/README.md:21`, `packaging/windows/README.md:36`) and point smoke validation to test scripts (`packaging/linux/README.md:12`).
- **Recommended action:** Add production packaging checklist separate from test scripts. Keep the API-only/no-local-runtime statement.
- **Priority:** Medium
- **Risk level:** Low
- **Confidence:** High

### MVP-LOW-001

- **MVP area:** License Worker service
- **Classification:** worker / config
- **File path:** `worker/migrations/0002_add_masked_license_key_to_reset_requests.sql`
- **Function/component/type/command/config/string involved:** `ALTER TABLE reset_requests ADD COLUMN masked_license_key TEXT`
- **What is missing or incomplete:** Migration 0002 is not idempotent if manually replayed against a DB where the column already exists.
- **Why it matters for MVP completeness:** Cloudflare migration tracking should prevent duplicate application, but manual recovery/replay is less forgiving.
- **Evidence from code or Graphify:** The migration is a plain `ALTER TABLE ... ADD COLUMN` (`worker/migrations/0002_add_masked_license_key_to_reset_requests.sql:1`).
- **Recommended action:** Document migration replay limitations or use guarded migration strategy where D1 supports it.
- **Priority:** Low
- **Risk level:** Low
- **Confidence:** Medium

## 11. Component-by-Component Audit

Customer app bootstraps correctly through `main.js` and `App.svelte`. The main route is the actual app, not a placeholder, and it connects to auth, runtime, generation, updater, deletion, legal, and diagnostics flows. The main production incompleteness is not route existence; it is the behavior of downstream generation/auth/storage contracts.

Customer API clients mostly map to registered Tauri commands. Contract shapes for generation and auth are coherent at the TypeScript/Rust boundary. The primary concern is excess exported/runtime command surface and stale/future provider fields.

Admin app has a real MVP operations console with setup, overview, reset queue, deletion queue, licenses, device bindings, audit events, idempotency records, decisions, and license disabling. It is separated from the customer app by entry point, binary, Tauri config, and capabilities. Admin update strategy needs manual decision.

Tauri command boundary is functional but broad. Customer and admin commands are separated by binary, but customer registers more commands than the current UI uses.

Rust generation is API-only in current source. Graphify output still references removed local-mode nodes, but current production source search did not find active `local_mode`, `python_runtime`, `tool_resolver`, or runtime pack modules under `app/src-tauri/src/core`.

Worker implementation is broad and integrated, but device-binding enforcement and security primitives are not release-ready.

## 12. Frontend Customer App Findings

- See `MVP-CRIT-002` for successful UI state with no usable clips.
- See `MVP-HIGH-005` for insufficient crash draft redaction.
- Positive evidence: `FormStatus` and `ThemedSelect` are used in production route forms (`app/src/routes/+page.svelte:3`, `app/src/routes/+page.svelte:1109`, `app/src/routes/+page.svelte:1126`). Design tokens are imported by `main.js` (`app/src/main.js:1`) and used throughout route CSS through `var(--...)`.

## 13. Customer API Client Findings

- See `MVP-MED-001` and `MVP-MED-003`.
- Positive evidence: auth client command names match Rust commands (`app/src/lib/api/authClient.ts:30`; `app/src-tauri/src/commands/auth.rs:7`). Generation stream command matches Rust registration (`app/src/lib/api/tauriClient.ts:61`; `app/src-tauri/src/main.rs:27`). Updater client is connected to customer diagnostics UI (`app/src/routes/+page.svelte:670`).

## 14. Admin App Findings

- See `MVP-HIGH-004`.
- Positive evidence: admin frontend calls admin client functions for overview/list/decision flows (`app/src/admin/AdminApp.svelte:156`), client command names match Rust command names (`app/src/admin/lib/adminClient.ts:35`), and admin binary registers only admin commands (`app/src-tauri/src/bin/admin_desktop.rs:5`).

## 15. Tauri Command Boundary Findings

- See `MVP-HIGH-002`, `MVP-MED-001`, and `MVP-MED-002`.
- Positive evidence: customer main registers auth, privacy, file picker, generation, health, runtime, secure-store, and API profile commands (`app/src-tauri/src/main.rs:15`). Admin binary is separate (`app/src-tauri/src/bin/admin_desktop.rs:3`).

## 16. Rust Generation / MuAPI Pipeline Findings

- See `MVP-CRIT-002`.
- Positive evidence: pipeline enforces `mode == "api"` (`app/src-tauri/src/core/pipeline.rs:111`), calls download/transcribe/highlights/clip stages in order (`app/src-tauri/src/core/pipeline.rs:129`, `app/src-tauri/src/core/pipeline.rs:151`, `app/src-tauri/src/core/pipeline.rs:198`, `app/src-tauri/src/core/pipeline.rs:250`), and MuAPI client submits/polls hosted jobs (`app/src-tauri/src/core/api_mode/muapi.rs:251`).

## 17. Licensing/Auth Findings

- See `MVP-CRIT-001`.
- Positive evidence: desktop auth state covers unauthenticated, licensed, offline grace, reauth, device-bound, reset pending/approved/rejected/expired, and error states (`app/src/lib/stores/authState.ts:11`). Rust auth state is built with vendor `AuthService`, keychain-backed secrets, encrypted license-session fallback for auth service secrets, and device identity provider (`app/src-tauri/src/auth.rs:320`).

## 18. License Worker Findings

- See `MVP-CRIT-001`, `MVP-HIGH-006`, and `MVP-MED-004`.
- Positive evidence: Worker routes exist for all required high-level operations (`worker/src/index.js:59` through `worker/src/index.js:114`), admin endpoints require bearer token (`worker/src/index.js:1733`), and D1 migrations define licenses, device bindings, reset requests, idempotency records, audit events, and deletion requests (`worker/migrations/0001_init.sql:1`, `worker/migrations/0003_user_data_deletion_requests.sql:1`).

## 19. Updater Findings

- See `MVP-HIGH-004`.
- Positive evidence: customer config includes updater `pubkey` and endpoint (`app/src-tauri/tauri.conf.json:51`), customer binary registers updater plugin (`app/src-tauri/src/main.rs:8`), customer UI exposes check/install controls (`app/src/routes/+page.svelte:1440`), and Worker serves `/updates/{target}/{arch}/{current_version}` (`worker/src/index.js:50`).

## 20. Legal/Privacy/Policy Findings

- See `MVP-HIGH-003`.
- Positive evidence: in-app policies cover terms, privacy, deletion, compliance, notices, and refund (`app/src/lib/legal/policiesContent.ts:10`) and are accessible from Settings -> Policies (`app/src/routes/+page.svelte:1498`).

## 21. Secure Storage Findings

- See `MVP-HIGH-001` and `MVP-HIGH-002`.
- Positive evidence: auth/session-specific fallback in `auth.rs` is encrypted and does not include raw license keys in the fallback structure (`app/src-tauri/src/auth.rs:47`, `app/src-tauri/src/auth.rs:140`). Runtime secure-store fallback is the separate plaintext-risk surface.

## 22. Crash/Support Diagnostics Findings

- See `MVP-HIGH-005`.
- Positive evidence: crash drafts are user-initiated for submission and are locally dismissible (`app/src/routes/+page.svelte:645`, `app/src/routes/+page.svelte:639`).

## 23. Legacy Python Pipeline Findings

Status: complete for separation, needs manual retention decision.

Evidence:

- Legacy CLI accepts only `--mode api` (`python_legacy/main.py:18`).
- Legacy pipeline is API/MuAPI-based and not called by customer Tauri source (`python_legacy/shorts_generator/pipeline.py:76`).
- Current production source search found no active app packaging/runtime call into `python_legacy`.

Recommended action:

- Keep `python_legacy` as explicit reference/parity code or move it out of production release artifacts. Do not present it as the active desktop runtime.

## 24. Packaging/Runtime Bundle Findings

- See `MVP-MED-005`.
- Positive evidence: Tauri customer bundle has empty `resources` (`app/src-tauri/tauri.conf.json:49`), and packaging docs say no local runtime/model bundles are packaged for API-only release.

## 25. Removed Local-Processing Leftovers

Current production source does not show active local-processing runtime code under `app/src-tauri/src/core`; Graphify’s merged report contains stale local-mode nodes from older graph output or excluded/non-current files. Source search found no active `local_mode`, `python_runtime`, `tool_resolver`, `process_supervisor`, runtime pack, `yt-dlp`, or `ffmpeg` production Rust modules in the current tree.

Remaining stale/future-provider leftovers:

- OpenAI provider config and runtime profile support remain despite MuAPI-only UI and pipeline. See `MVP-MED-003`.
- CSS class names containing `model-*` remain in `app/src/routes/+page.svelte`, but source search did not find active local-model behavior tied to those names.

## 26. End-to-End MVP Flow Verification

### Customer flow

Status: **Partial**

Flow check:

- Open app: complete (`main.js` -> `App.svelte` -> route).
- Authenticate/validate license: integrated, but backend device-bound semantics are incomplete.
- Configure required inputs: partial; MuAPI profile setup exists.
- Start API-based generation: complete at command level.
- Track progress/status: partial; progress events exist, but cancellation only takes effect at stage boundaries.
- Receive generated clips/output: partial; all clip failures can still be reported as success.
- Handle failure/retry: partial; error panel exists, but retry UX is generic.
- Access support/privacy/legal/update actions: partial; actions exist, but crash redaction and deletion notice artifact are incomplete.

### Admin flow

Status: **Partial**

Flow check:

- Open admin app: complete.
- Access admin functions: complete after base URL/token setup.
- View licenses/devices/audit events: complete at route/client level.
- Approve/reject reset requests: complete at route/client level.
- Approve deletion requests: complete at route/client level.
- Call worker/admin endpoints safely: partial; bearer auth exists, but admin updater/manual update path needs decision and Worker security primitives need hardening.

### Worker flow

Status: **Partial**

Flow check:

- Receive activation/validation/reset/webhook/deletion/admin request: complete route table.
- Validate request: partial; request shape checks exist, device binding policy missing.
- Read/write D1 state: complete schema/helpers for current routes.
- Return stable contract response: partial; envelope shape is stable, but token/hash primitives need hardening.
- Support desktop/admin clients: partial; route paths match, but auth policy mismatch blocks MVP release.

## 27. Recommended Completion Plan

### Critical before MVP release

- `worker/src/index.js`, `worker/src/store.js`: enforce one active device per license in `handleActivate`; return `device_already_bound` for different active device. Risk: high. Suggested validation: manual activation on device A then device B, reset approval, reactivation.
- `app/src-tauri/src/core/pipeline.rs`, `app/src-tauri/src/core/api_mode/clipper.rs`, `app/src/routes/+page.svelte`: convert zero successful clip URLs into failure or partial-success UX. Risk: high. Suggested validation: MuAPI autocrop failure simulation or controlled provider failure.

### High priority before public release

- `app/src-tauri/src/commands/runtime.rs`: remove plaintext secure fallback or encrypt it; fail closed for API/admin/deletion secrets if keychain unavailable. Risk: high. Suggested validation: keychain unavailable scenario and saved profile/admin token inspection.
- `app/src-tauri/src/main.rs`, `app/src-tauri/src/commands/runtime.rs`, `app/src/lib/api/runtimeClient.ts`: remove or restrict `runtime_machine_secret`. Risk: high. Suggested validation: command inventory review.
- `app/src/DELETION_NOTICE.md`: create standalone deletion notice or remove from release inventory by manual decision. Risk: medium. Suggested validation: packaging/docs inventory check.
- `app/src-tauri/src/bin/admin_desktop.rs`, `app/src-tauri/tauri.admin.conf.json`, `app/src/admin/AdminApp.svelte`: decide and implement/document admin updater path. Risk: medium. Suggested validation: admin update/manual update QA.
- `app/src/support/crashDraft.ts`: expand redaction before enabling crash submission endpoint. Risk: high. Suggested validation: crash draft redaction samples with tokens/emails/paths.
- `worker/src/index.js`, `worker/src/store.js`: replace token/idempotency crypto primitives. Risk: high. Suggested validation: activation/validation compatibility check and replay/idempotency checks.

### Medium priority after MVP

- `app/src/lib/api/runtimeClient.ts`, `app/src-tauri/src/commands/runtime.rs`: remove unused runtime filesystem exports/commands or document production need.
- `app/src-tauri/src/main.rs`, `app/src-tauri/src/commands/generate.rs`: remove or intentionally support unused generation commands.
- `app/src-tauri/src/core/config.rs`, `app/src-tauri/src/commands/runtime.rs`, `app/src/lib/api/runtimeClient.ts`: remove or reserve OpenAI provider config.
- `worker/src/index.js`: include Gumroad readiness in `/readyz` or expose purchase-ingestion readiness separately.

### Low priority / cleanup

- `worker/migrations/0002_add_masked_license_key_to_reset_requests.sql`: document non-idempotent replay behavior.
- `packaging/*/README.md`: expand non-test packaging/release notes.

### Needs manual product decision

- Is admin desktop part of the same auto-update release channel as customer desktop?
- Is plaintext local fallback ever acceptable for API keys/admin tokens/deletion lookup tokens?
- Is OpenAI provider support future scope or stale MVP scope?
- Should advanced runtime filesystem/machine-secret commands remain in customer build?
- Should `python_legacy` ship with production source artifacts or be moved to reference-only archival location?

## 28. Validation Plan

Per repository instructions, this audit did not run tests, builds, lints, package installs, or validation scripts.

Manual production validation categories for the user to run after fixes:

- Frontend type checking for customer/admin app.
- Customer frontend production build.
- Admin frontend production build.
- Rust/Tauri customer build.
- Rust/Tauri admin build.
- Worker syntax/build/deployment dry-run validation.
- Search validation for stale removed-feature references.
- Manual QA for customer activation, device-bound rejection, reset approval, MuAPI profile setup, successful generation, clip-render failure, output JSON write, updater check, deletion request/status, crash draft redaction, and policies.
- Manual QA for admin setup, overview, licenses, devices, audit events, reset decisions, deletion approval/retry, and license disabling.

Exact commands were not executed by the agent. Suggested manual commands, adjusted to the repository's existing script/pnpm policy:

```bash
pnpm --dir app run check
pnpm --dir app run build
cargo check --locked
```

Run only the repository-approved Worker/admin/customer validation scripts that are relevant to the changed area. Do not run dependency installation as part of validation.

## 29. Final Verdict

**MVP INCOMPLETE**

The production application is substantially integrated and much of the MVP surface is present, but release readiness is blocked by two critical production gaps: Worker activation does not enforce the device-binding/reset contract, and generation can succeed without producing any usable clip. High-priority security and release-readiness gaps also remain around runtime secure-storage fallback, exposed machine-secret command surface, crash redaction, admin updater strategy, missing standalone deletion notice, and Worker crypto/idempotency primitives.

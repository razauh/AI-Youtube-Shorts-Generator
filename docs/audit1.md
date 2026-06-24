# MVP Completeness Audit

## 1. Executive Summary

Verdict: **MVP mostly complete with non-blocking gaps and manual decisions**.

The current product source supports an API-based MuAPI MVP: licensed desktop access, MuAPI generation, output JSON, hosted clip links, updater wiring for the customer app, user deletion/reset flows, admin desktop review flows, Worker-backed license/device/reset/deletion/Gumroad contracts, and focused test fixtures/scripts. The main release blockers are not broad missing implementation; they are unresolved product/security decisions and documentation gaps around admin updater support, deletion notice packaging, MuAPI retry semantics, and release-readiness proof.

No tests, builds, validation scripts, dependency installs, deployments, or Graphify regeneration were run for this documentation-only audit.

## 2. Scope

This audit covers:

- Frontend app and admin UI under `app/src/`.
- Tauri Rust commands, capabilities, auth, runtime, and API-mode pipeline under `app/src-tauri/`.
- Cloudflare Worker licensing service under `worker/`.
- License-control-suite integration under `vendor/license-control-suite/`.
- Python legacy API pipeline under `python_legacy/`.
- Packaging docs and validation scripts under `packaging/`, `.scripts/`, `tests/`, and `docs/`.
- Existing Graphify artifacts under `graphify-out/`.

This audit does not cover uncommitted behavior after running test/build commands, production cloud state, real Gumroad/Cloudflare secrets, live MuAPI behavior, or signed release artifact inspection.

## 3. Methodology

Evidence was gathered through read-only source inspection:

- `git status --short` to preserve current dirty state.
- `rg --files`, `find`, `sed`, and `rg` for source and Graphify inspection.
- Existing Graphify reports from 2026-06-04 were used as structural triage only.
- Direct source files were treated as authoritative when Graphify inferred edges or stale relationships.

## 4. Evidence Inventory

Primary Graphify evidence:

- `graphify-out/GRAPH_REPORT.md`
- `graphify-out/MERGED_GRAPH_REPORT.md`
- `graphify-out/graph.json`
- `graphify-out/merged-graph.json`
- `graphify-out/scopes/app/GRAPH_REPORT.md`
- `graphify-out/scopes/worker/GRAPH_REPORT.md`
- `graphify-out/scopes/tests/GRAPH_REPORT.md`
- `graphify-out/scopes/packaging/GRAPH_REPORT.md`
- `graphify-out/scopes/vendor/GRAPH_REPORT.md`

Primary source evidence:

- `app/src/routes/+page.svelte`
- `app/src/lib/api/tauriClient.ts`
- `app/src/lib/api/authClient.ts`
- `app/src/lib/api/runtimeClient.ts`
- `app/src/lib/api/updaterClient.ts`
- `app/src/admin/lib/adminClient.ts`
- `app/src-tauri/src/main.rs`
- `app/src-tauri/src/bin/admin_desktop.rs`
- `app/src-tauri/src/commands/*.rs`
- `app/src-tauri/tauri.conf.json`
- `app/src-tauri/tauri.admin.conf.json`
- `app/src-tauri/capabilities/default.json`
- `app/src-tauri/capabilities/admin.json`
- `app/src-tauri/src/core/api_mode/*.rs`
- `worker/src/index.js`
- `worker/src/contracts.js`
- `worker/src/store.js`
- `worker/migrations/*.sql`
- `worker/test/contract.test.js`
- `packaging/*/README.md`
- `.scripts/run-*.sh`

## 5. Existing Dirty Worktree

Existing dirty files observed before edits:

- Modified: `.gitignore`
- Modified: `app/src/lib/legal/policiesContent.ts`
- Modified: `app/src/tests/ui_flow.test.ts`
- Untracked: `.scripts/run-policy-muapi-liability-validation.sh`

These files were not modified by this audit. The only intended new file is this report.

## 6. Graphify Structural Summary

Graphify identifies the codebase as a large, multi-surface system. The merged report shows 2008 nodes, 3908 edges, and 50 communities. Major clusters map to:

- Customer app generation, auth, settings, updater, privacy, and secure storage.
- Worker license, reset, deletion, Gumroad, update, and admin routes.
- License-control-suite auth/admin/shared-contract modules.
- MuAPI pipeline and parity fixtures.
- Packaging and validation scripts.

Graphify also flags inferred edges around generic functions like `invoke()`, `fetch()`, `ok()`, and `err()`. Those edges were not accepted blindly; the audit verified relevant claims against source files.

## 7. MVP Architecture Assumption

This audit assumes the MVP architecture is **API-based MuAPI generation**, not local-model or bundled local-processing generation.

Evidence:

- `README.md` describes API-based generation through MuAPI.
- `app/src/routes/+page.svelte` initializes `mode = 'api'`.
- `app/src/lib/api/tauriClient.ts` types `GenerateRequest.mode` as `'api'`.
- Packaging READMEs state local runtime/model bundles are not packaged for the API-only release path.
- `docs/local-processing-removal-audit.md` separately concluded the active app path is API mode only.

## 8. MVP Status Matrix

| Area | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Customer license gate | Complete | `app/src/routes/+page.svelte`, `app/src/lib/stores/authState.ts`, `app/src-tauri/src/commands/generate.rs` | Main app UI and generation require licensed or offline-grace state. |
| License activation/session | Complete | `app/src/lib/api/authClient.ts`, `app/src-tauri/src/commands/auth.rs`, `vendor/license-control-suite/` | Uses existing auth service and secure persistence patterns. |
| Device reset | Complete | `app/src/routes/+page.svelte`, `worker/src/index.js`, `worker/test/contract.test.js` | Customer request/status and admin approve/reject exist. |
| User data deletion | Mostly complete | `app/src/routes/+page.svelte`, `worker/src/index.js`, `worker/migrations/0003_user_data_deletion_requests.sql` | Source flow exists; standalone deletion notice file is missing. |
| MuAPI generation | Mostly complete | `app/src-tauri/src/core/api_mode/muapi.rs`, `app/src-tauri/src/core/pipeline.rs` | Functional path exists; explicit 429/5xx retry policy remains a TODO/manual decision. |
| Output JSON | Complete | `app/src/lib/api/tauriClient.ts`, `app/src-tauri/src/commands/generate.rs` | Output path picker and atomic JSON write are wired. |
| Hosted clip library/actions | Complete | `app/src/routes/+page.svelte` | Open/copy actions are for remote clip URLs. |
| API key profiles | Complete | `app/src/lib/api/runtimeClient.ts`, `app/src-tauri/src/commands/runtime.rs` | MuAPI/OpenAI profile wrappers and secure-store commands exist. |
| Crash draft | Complete with manual endpoint | `app/src/routes/+page.svelte`, `app/src/support/crashDraft.ts` | User-initiated only; endpoint is env-configured. |
| Customer updater | Complete pending release validation | `app/src-tauri/tauri.conf.json`, `app/src/lib/api/updaterClient.ts`, `worker/src/index.js` | Tauri updater plugin and Worker update endpoint exist. |
| Admin updater | Needs manual decision | `app/src-tauri/tauri.admin.conf.json`, `app/src-tauri/src/bin/admin_desktop.rs` | Admin config has no updater plugin/endpoints. |
| Admin desktop | Complete | `app/src/admin/lib/adminClient.ts`, `app/src-tauri/src/bin/admin_desktop.rs`, `app/src-tauri/src/commands/admin.rs` | Admin command surface and Worker calls exist. |
| Worker licensing | Complete with production validation needed | `worker/src/index.js`, `worker/src/store.js`, `worker/test/contract.test.js` | Routes, D1 helpers, and contract tests exist. |
| Gumroad ingestion | Complete with live-secret validation needed | `worker/src/index.js`, `worker/README.md` | Server-to-server sale verification is documented. |
| Packaging docs | Mostly complete | `packaging/linux/README.md`, `packaging/macos/README.md`, `packaging/windows/README.md` | API-only path is documented. |
| Validation scripts | Complete | `.scripts/run-*.sh` | Manual execution required by repo policy. |

## 9. Customer End-to-End Flow

Expected MVP path:

1. Customer activates Gumroad license.
2. App reaches licensed or offline-grace state.
3. Customer configures MuAPI profile.
4. Customer submits YouTube URL and generation settings.
5. Tauri command starts API-mode generation and emits progress.
6. MuAPI stages download/transcribe/highlight/clip.
7. App records project result, hosted clip URLs, and optional output JSON.

Evidence:

- `app/src/routes/+page.svelte` imports auth, runtime, generation, updater, crash, and legal clients.
- `isLicensedAppSession` controls the authenticated shell.
- `submitGenerate` calls `runGenerateAndStream`.
- `app/src/lib/api/tauriClient.ts` invokes `generate_shorts_stream`.
- `app/src-tauri/src/main.rs` registers `generate_shorts_stream`.
- `app/src-tauri/src/commands/generate.rs` checks auth state before generation and writes optional output JSON.

## 10. Frontend UI Verification

The customer frontend is MVP-capable:

- License activation and reset states are represented.
- Generator, library, settings, diagnostics, policies, updater, deletion, and crash-draft UI are present.
- Settings load `appConfigSummary`, `runtimeContext`, and MuAPI profiles.
- Setup blockers focus on MuAPI configuration.
- Deletion status lookup token is stored through secure store rather than ordinary local storage.

Risk note: local project history remains in local storage, which is acceptable for current MVP assumptions but should remain documented as user data.

## 11. Frontend-To-Tauri Contract Verification

Frontend command wrappers match registered customer commands:

- `activate_license`
- `validate_session`
- `request_device_reset`
- `get_device_reset_status`
- `request_user_data_deletion`
- `get_user_data_deletion_status`
- `clear_local_session`
- `get_auth_state`
- `pick_output_json_path`
- `generate_shorts_stream`
- `cancel_generate_run`
- `app_config_summary`
- `runtime_context`
- secure store and API key profile commands

Evidence:

- Wrappers: `app/src/lib/api/authClient.ts`, `app/src/lib/api/tauriClient.ts`, `app/src/lib/api/runtimeClient.ts`
- Registration: `app/src-tauri/src/main.rs`

No frontend wrapper was found for a local video picker or local-model generation mode.

## 12. Tauri Command And Capability Verification

Customer Tauri app:

- Registers auth, privacy, file picker, generation, health, runtime, secure-store, and API key profile commands.
- Registers Tauri updater plugin.
- Uses `app/src-tauri/capabilities/default.json`, which grants `core:default` and `updater:default`.

Admin Tauri app:

- Registers admin config, overview, list, reset decision, deletion decision, and disable-license commands.
- Uses `app/src-tauri/capabilities/admin.json`, which grants `core:default`.
- Does not register updater plugin.

This separation is security-appropriate, but admin update policy remains unresolved.

## 13. Rust Pipeline Verification

The Rust generation command path is MVP-capable:

- `GenerateShortsCommand` accepts URL, clip count, aspect ratio, format, language, output JSON, and mode.
- `generate_shorts_stream` emits `generate-progress` events.
- `generation_auth_error` rejects generation unless auth state is licensed or offline grace.
- `run_generate_with_sink` calls `generate_shorts_with_progress_live`.
- Output JSON uses `write_result_json_atomic`.
- Cancellation and timeout paths produce safe error envelopes.

Relevant files:

- `app/src-tauri/src/commands/generate.rs`
- `app/src-tauri/src/core/pipeline.rs`
- `app/src-tauri/src/core/contracts.rs`
- `app/src-tauri/src/runtime/fs_output.rs`

## 14. MuAPI Integration Verification

MuAPI client source supports:

- API key header construction.
- Submit endpoint POST returning request ID.
- Poll result endpoint.
- Status-change progress emission.
- Timeout handling.
- Transport retry on timeout/connect errors.

Risk: `app/src-tauri/src/core/api_mode/muapi.rs` has a TODO: `decide explicit retry policy for 429/5xx parity-extension`. Current source returns API errors immediately for client/server status failures instead of a documented 429/5xx retry/backoff policy.

## 15. Licensing And Auth Verification

Licensing integration is MVP-capable:

- Customer auth commands are routed through Tauri wrappers.
- License-control-suite provides auth core, Tauri persistence, admin domain, shared contracts, and downstream harnesses.
- `app/src-tauri/src/commands/generate.rs` enforces license state before generation.
- Worker activation validates license state and persists device binding.
- Worker validation checks signed token, active license, and active device binding.

Sensitive handling observed:

- Worker hashes license keys with `HASH_PEPPER` and SHA-256.
- Frontend tests include assertions that plaintext license material is not persisted in local storage.
- UI error mapping uses safe user-facing messages.

## 16. Worker Contract Verification

Worker routes cover MVP licensing/service operations:

- `GET /health`
- `GET /readyz`
- `GET /updates/:target/:arch/:current_version`
- `POST /v1/license/activate`
- `POST /v1/license/validate`
- `POST /v1/license/reset/request`
- `POST /v1/license/reset/status`
- `POST /v1/privacy/delete/request`
- `POST /v1/privacy/delete/status`
- Admin overview/list/reset/deletion/disable routes
- `POST /v1/license/webhooks/gumroad`

Worker D1 helpers cover licenses, device bindings, resets, deletion requests, idempotency records, audit events, and admin listing.

Risk: `worker/src/store.js` uses `stableHash()` for idempotency payload matching and comments that it is deterministic scaffold behavior. License secrecy is not directly affected because license key hashes use SHA-256 with `HASH_PEPPER`, but idempotency hashing should be upgraded or explicitly accepted before production.

## 17. Admin Desktop Verification

Admin frontend and Tauri contracts are MVP-capable:

- `app/src/admin/lib/adminClient.ts` invokes admin config, overview, license list, device binding list, audit list, idempotency list, reset list, deletion list, reset approve/reject, deletion approve/reject, and disable-license commands.
- `app/src-tauri/src/bin/admin_desktop.rs` registers corresponding commands.
- Admin app has a separate Tauri config and window label.
- Admin capability is separate from the customer default capability.

Manual decision: whether admin desktop requires the same updater lifecycle as the customer app.

## 18. User Data Deletion Verification

Implemented flow:

- Customer submits license key, optional purchaser email, and `DELETE` confirmation.
- Worker creates a deletion request and lookup token.
- Customer can refresh status with request ID and lookup token.
- Admin can list, approve, or reject deletion requests.
- Approval can disable/anonymize backend licensing data and record summary metadata.

Evidence:

- `app/src/routes/+page.svelte`
- `app/src/lib/api/authClient.ts`
- `app/src-tauri/src/commands/privacy.rs`
- `worker/src/index.js`
- `worker/src/store.js`
- `worker/migrations/0003_user_data_deletion_requests.sql`
- `.scripts/run-user-data-deletion-validation.sh`

Gap: `app/src/DELETION_NOTICE.md` is missing from the current filesystem. If release expectations require a standalone deletion notice artifact, MVP is incomplete until it is restored or the requirement is removed.

## 19. Updater Verification

Customer updater:

- `app/src-tauri/src/main.rs` registers `tauri_plugin_updater`.
- `app/src-tauri/capabilities/default.json` includes `updater:default`.
- `app/src-tauri/tauri.conf.json` sets `createUpdaterArtifacts`, updater pubkey, and Worker update endpoint.
- `app/src/lib/api/updaterClient.ts` uses the official Tauri updater plugin.
- `worker/src/index.js` implements `/updates/:target/:arch/:current_version`.
- `.scripts/generate-customer-updater-manifest.mjs` intentionally filters customer artifacts only.

Admin updater:

- `app/src-tauri/src/bin/admin_desktop.rs` does not register updater plugin.
- `app/src-tauri/tauri.admin.conf.json` has no updater config.
- `app/src-tauri/capabilities/admin.json` has no updater permission.

Classification: needs manual product decision, not proven defect.

## 20. Packaging And Release Verification

Packaging docs currently align with the API-only MVP:

- `packaging/linux/README.md` states local runtime/model bundles are not packaged for the API-only release path.
- macOS and Windows packaging docs should be kept consistent with that statement.
- `app/package.json` has separate customer/admin Tauri dev and build scripts.
- Root `package.json` uses pnpm scripts and includes customer/admin bundle scripts.
- `.scripts/validate-release-ci-config.sh` checks customer/admin release artifact expectations and customer updater config.

Release validation still requires manual execution. No release workflow, signing, or artifact command was run.

## 21. Python Legacy Verification

Python legacy remains documented as a CLI/API-mode pipeline:

- `README.md` lists `python_legacy/` as Python CLI and API-mode pipeline.
- `python_legacy/shorts_generator/muapi.py` includes MuAPI submit/poll retry behavior.
- Parity and characterization tests exist under `tests/`.

This is compatible with the API-based MVP assumption. It should not be treated as evidence of local desktop processing unless local runtime/model source paths are restored.

## 22. Test And Fixture Coverage

Observed coverage areas:

- Frontend boot and UI flow tests under `app/src/tests/` and `app/tests/`.
- Auth state and storage tests.
- Admin frontend tests.
- Updater client tests.
- Rust command/config/MuAPI/auth/updater/pipeline tests under `app/src-tauri/tests/`.
- Worker contract tests under `worker/test/contract.test.js`.
- License Worker fixture parity under `tests/parity/`.
- Golden pipeline fixtures under `tests/fixtures/`.
- License-control-suite regression, baseline, IPC, integration, and contract tests.

Coverage appears broad enough for MVP, but this audit did not run it.

## 23. Validation Plan

Manual commands the user should run from the repository root:

```bash
pnpm --dir app run test
pnpm --dir app run build
pnpm --dir worker run test
cargo test --manifest-path app/src-tauri/Cargo.toml --locked
bash .scripts/run-updater-endpoint-validation.sh
bash .scripts/run-admin-desktop-validation.sh
bash .scripts/run-customer-onboarding-validation.sh
bash .scripts/run-user-data-deletion-validation.sh
bash .scripts/run-remove-local-processing-validation.sh
bash .scripts/run-policy-muapi-liability-validation.sh
bash .scripts/validate-release-ci-config.sh
```

Do not run deploy/sign/release commands until secrets, release target, and signing behavior are explicitly approved.

## 24. Findings

### MVP-AUD-001

- MVP area: User data deletion/legal notice
- Classification: Missing artifact
- File path: `app/src/DELETION_NOTICE.md`
- Symbol/string/config: standalone deletion notice file
- Missing/incomplete behavior: The file is absent from the current filesystem, while deletion flow exists in UI, Tauri, Worker, migrations, and validation script.
- MVP impact: Users/operators may lack a standalone inspectable deletion notice if release packaging or policy expects one.
- Source or Graphify evidence: `find app/src ...` found only `app/src/lib/legal`; deletion implementation exists in `app/src/routes/+page.svelte`, `worker/src/index.js`, and `.scripts/run-user-data-deletion-validation.sh`.
- Recommended action: Restore `app/src/DELETION_NOTICE.md` or document that in-app policy text replaces it.
- Priority: High
- Risk: Compliance/support ambiguity
- Confidence: High

### MVP-AUD-002

- MVP area: Admin updater/release lifecycle
- Classification: Needs manual decision
- File path: `app/src-tauri/tauri.admin.conf.json`, `app/src-tauri/src/bin/admin_desktop.rs`, `app/src-tauri/capabilities/admin.json`
- Symbol/string/config: updater plugin, `plugins.updater`, `updater:default`
- Missing/incomplete behavior: Customer app has updater plugin/config/capability; admin app does not.
- MVP impact: Admin desktop may require manual reinstall/update unless intentionally excluded.
- Source or Graphify evidence: Customer config contains updater endpoint and pubkey; admin config has no updater section; admin main registers no updater plugin.
- Recommended action: Decide whether admin app ships with updater support. If yes, add a scoped admin updater design and tests. If no, document manual admin update policy.
- Priority: Medium
- Risk: Operational drift or unsupported admin installs
- Confidence: High

### MVP-AUD-003

- MVP area: MuAPI resilience
- Classification: Risk/manual decision
- File path: `app/src-tauri/src/core/api_mode/muapi.rs`
- Symbol/string/config: `TODO: decide explicit retry policy for 429/5xx parity-extension`
- Missing/incomplete behavior: Timeout/connect retries exist; explicit 429/5xx retry/backoff behavior is not documented or implemented.
- MVP impact: Rate limits or transient server failures may fail immediately instead of following a deliberate retry policy.
- Source or Graphify evidence: `MuApiClient::submit` and `fetch_result` return API errors for client/server statuses; Graphify app community identifies MuAPI client as core pipeline node.
- Recommended action: Decide accepted MVP behavior. If retries are required, define 429/5xx retry count, backoff, max wait, progress messaging, and tests.
- Priority: Medium
- Risk: User-visible generation failures under provider throttling/outage
- Confidence: High

### MVP-AUD-004

- MVP area: Worker idempotency
- Classification: Production-hardening risk
- File path: `worker/src/store.js`
- Symbol/string/config: `stableHash()`
- Missing/incomplete behavior: Idempotency payload matching uses a deterministic scaffold hash.
- MVP impact: Idempotency collision resistance is weaker than expected for production-grade request replay protection.
- Source or Graphify evidence: `stableHash()` comment says scaffold behavior; Worker route handlers use it for activation, reset, deletion, admin decisions, disable, and webhook idempotency payload hashes.
- Recommended action: Replace with SHA-256 over canonical JSON or explicitly accept current risk for MVP with compensating tests.
- Priority: Medium
- Risk: Replay/idempotency integrity weakness
- Confidence: Medium

### MVP-AUD-005

- MVP area: Release readiness proof
- Classification: Unverified
- File path: `.scripts/run-*.sh`, `app/src-tauri/tests/*`, `worker/test/contract.test.js`
- Symbol/string/config: manual validation commands
- Missing/incomplete behavior: Test/build/validation status is unknown in this audit because repo policy prohibits agents from running validation commands.
- MVP impact: Source appears mostly complete, but release cannot be declared ready until manual validation passes.
- Source or Graphify evidence: Broad test and validation scripts exist; none were executed for this report.
- Recommended action: Run the manual validation plan in section 23 and attach results before release decision.
- Priority: High
- Risk: Hidden regression
- Confidence: High

### MVP-AUD-006

- MVP area: Worker admin route documentation
- Classification: Documentation gap
- File path: `worker/README.md`
- Symbol/string/config: admin routes list
- Missing/incomplete behavior: README lists privacy admin routes but omits several implemented admin routes such as overview, licenses, device bindings, audit events, idempotency records, reset lists/decisions, and license disable.
- MVP impact: Operator documentation understates the admin API surface.
- Source or Graphify evidence: `worker/src/index.js` implements the routes; `worker/README.md` route list is shorter.
- Recommended action: Update Worker README route inventory after confirming public/admin documentation expectations.
- Priority: Low
- Risk: Operator confusion
- Confidence: High

### MVP-AUD-007

- MVP area: Third-party notices
- Classification: Needs manual completion
- File path: `app/src/lib/legal/policiesContent.ts`
- Symbol/string/config: `Third-Party Notices`
- Missing/incomplete behavior: In-app notice states exact license metadata is not shown and operator must generate release notices from final inventories.
- MVP impact: Release may be incomplete if shipped binaries lack final third-party notice artifacts.
- Source or Graphify evidence: Policy text explicitly says exact license metadata is not shown in-app.
- Recommended action: Generate and include final third-party license notices for distributed dependencies and binary contents.
- Priority: Medium
- Risk: Licensing/compliance gap
- Confidence: High

### MVP-AUD-008

- MVP area: Live service readiness
- Classification: Unverified external dependency
- File path: `worker/README.md`, `worker/src/index.js`, `app/src-tauri/tauri.conf.json`
- Symbol/string/config: Gumroad verification, Cloudflare D1/secrets, updater manifest URL, MuAPI credentials
- Missing/incomplete behavior: Source supports these flows, but live secrets, D1 migrations, Worker deployment, update manifest, and MuAPI account availability were not verified.
- MVP impact: Local source can be complete while production service setup is incomplete.
- Source or Graphify evidence: Worker readiness checks require D1 and secrets; updater config points to Worker update route; no cloud-state command was run.
- Recommended action: Perform a separate production readiness runbook with approved secrets and non-agent deployment validation.
- Priority: High
- Risk: Production launch failure
- Confidence: High

## 25. Completion Plan

1. Resolve MVP-AUD-001 by restoring or formally replacing the deletion notice artifact.
2. Resolve MVP-AUD-002 with a product decision on admin updater support.
3. Resolve MVP-AUD-003 by documenting or implementing MuAPI 429/5xx retry behavior.
4. Resolve MVP-AUD-004 by replacing Worker idempotency hash or approving current MVP risk.
5. Run the validation plan and record pass/fail evidence.
6. Update Worker admin route documentation.
7. Generate final third-party notices for release artifacts.
8. Complete production readiness checks for Cloudflare, Gumroad, updater manifest, and MuAPI credentials.

## 26. Manual Decisions Required

- Should the admin desktop receive automatic updater support, or is manual admin update acceptable for MVP?
- Is `app/src/DELETION_NOTICE.md` required as a standalone file, or does in-app policy text satisfy the release requirement?
- Should MuAPI 429/5xx errors retry in MVP, and with what backoff/limits?
- Is Worker `stableHash()` acceptable for MVP idempotency, or must it be replaced before launch?
- Are exact third-party notices required inside the app, bundled beside installers, or both?
- What production readiness evidence is required before declaring launch-ready?

## 27. Commands Not Run

The following categories were intentionally not run:

- No `pnpm install`, `npm`, `npx`, `cargo update`, `pip install`, or dependency-changing command.
- No tests, builds, linters, format checks, validation scripts, or Tauri bundling.
- No dev server, Tauri GUI, browser GUI, deployment, Wrangler deploy, release publication, signing, or updater publication.
- No destructive cleanup commands.
- No Graphify regeneration.

## 28. Final Verdict

**MVP mostly complete with non-blocking gaps and manual decisions.**

The source tree has the core MVP implementation for an API-based, license-gated MuAPI desktop product. The strongest evidenced blockers are release/process readiness, not missing core generation or licensing code. The MVP should not be called release-ready until the deletion notice decision, admin updater decision, MuAPI retry decision, Worker idempotency hardening decision, final notices, and manual validation results are closed.

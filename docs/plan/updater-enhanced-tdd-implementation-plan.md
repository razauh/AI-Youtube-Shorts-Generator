# Updater Implementation Plan (Enhanced TDD)

## 1) Scope and Inputs

This plan implements the updater module for the Tauri app using:

- `docs/updater-module-language-agnostic-guide.md`
- `docs/shared-state-machines.md`
- `docs/enhanced-logging-event-catalog.md`
- `docs/enhanced-logging-module-guide.md`
- `docs/shared-error-taxonomy.md`
- `docs/shared-network-client-abstraction.md`
- `docs/shared-secure-storage-abstraction.md`
- `docs/shared-test-strategy.md`
- `docs/enhanced-logging-implementation-checklist.md`

Graphify analysis was run on `2026-05-09` (`graphify update .`), producing `412 nodes / 755 edges / 19 communities` in `graphify-out/GRAPH_REPORT.md`. It shows updater is currently doc-defined but not implemented in code paths, so this plan adds new backend + frontend updater slices with strict TDD.

## 2) Current Project Fit (from graph + code scan)

Existing integration seams:

- Tauri command registration: `app/src-tauri/src/main.rs`
- Commands modules: `app/src-tauri/src/commands/mod.rs`, existing patterns in `commands/generate.rs` and `commands/health.rs`
- Frontend Tauri bridge: `app/src/lib/api/tauriClient.ts`
- App config surface: `app/src-tauri/tauri.conf.json`
- Existing Rust test layout: `app/src-tauri/tests/*.rs`

Conclusion: updater should follow the same architecture pattern as generate/health commands:

1. `core` for domain/state logic
2. `commands` for Tauri API surface
3. `frontend` typed client wrappers
4. integration tests in `src-tauri/tests`

## 3) Target Updater Architecture

### 3.1 Backend modules

Add:

- `app/src-tauri/src/core/updater/mod.rs`
- `app/src-tauri/src/core/updater/contracts.rs`
- `app/src-tauri/src/core/updater/state.rs`
- `app/src-tauri/src/core/updater/service.rs`
- `app/src-tauri/src/core/updater/errors.rs`
- `app/src-tauri/src/core/updater/events.rs`
- `app/src-tauri/src/commands/updater.rs`

Wire:

- `app/src-tauri/src/core/mod.rs` to export updater module
- `app/src-tauri/src/commands/mod.rs` to export updater command module
- `app/src-tauri/src/main.rs` to register updater commands

### 3.2 Frontend modules

Add:

- `app/src/lib/api/updaterClient.ts`
- `app/src/lib/stores/updaterState.ts`

Update:

- `app/src/lib/api/tauriClient.ts` only if shared helpers are needed
- `app/src/routes/+page.svelte` to expose check/install UX

### 3.3 State model (must match docs)

State enum:

- `IDLE`
- `CHECKING`
- `UP_TO_DATE`
- `UPDATE_AVAILABLE`
- `DOWNLOADING`
- `VERIFYING`
- `INSTALLING`
- `RESTART_REQUIRED`
- `ERROR`
- `ROLLING_BACK`

Rules:

- `install` valid only from `UPDATE_AVAILABLE`
- only one cached verified pending update candidate
- check cooldown enforced after successful check

## 4) TDD Strategy (Red -> Green -> Refactor)

Every phase must start with failing tests.

### Phase A: Contracts + State Machine

Red:

- add Rust unit tests for state transitions and illegal transitions.
- add serde contract tests for updater check/install response shapes.

Green:

- implement minimal `contracts.rs` + `state.rs` to satisfy tests.

Refactor:

- remove duplication in transition guards and use table-driven tests.

### Phase B: Command Surface + In-Memory Pending Update

Red:

- tests for `check_for_updates` and `install_available_update` command behavior:
  - no update
  - update available (candidate cached)
  - install without cached candidate -> domain error

Green:

- add `commands/updater.rs`, in-memory shared state (`Mutex`/`RwLock`) and command handlers.

Refactor:

- isolate command parsing/mapping from service logic.

### Phase C: Runtime Adapter + Progress Events

Red:

- tests for progress event names and throttling:
  - emits `updater-progress` max 1/sec
  - emits `updater-finished` on terminal states
- tests for event payload schema.

Green:

- implement `events.rs` and event emission hooks in service.

Refactor:

- centralize throttle policy in one helper.

### Phase D: Security Verification + Failure Paths

Red:

- tests for hash mismatch and signature invalid behavior:
  - artifact deleted
  - error codes mapped to taxonomy (`SEC_HASH_MISMATCH`, `SEC_SIGNATURE_INVALID`)
  - `updater.verify.failure` log event emitted

Green:

- implement verification checks and error mapping.

Refactor:

- make verifier injectable for deterministic tests.

### Phase E: Cooldown + Policy + Channel Config

Red:

- tests for cooldown logic:
  - check skipped inside cooldown window
  - cooldown reset on check failure
- tests for update modes:
  - `manual`, `notify`, `auto-download`
- tests for channel routing (`stable` / `beta` / `nightly`).

Green:

- implement policy resolver and storage-backed settings.

Refactor:

- extract policy evaluation pure functions.

### Phase F: Frontend Client + Store + UI Wiring

Red:

- frontend unit tests (`vitest`) for client and store:
  - state updates per backend responses/events
  - disable install CTA when not `UPDATE_AVAILABLE`
  - user-facing copy for offline/check/install failures

Green:

- implement `updaterClient.ts`, `updaterState.ts`, and minimal UI controls.

Refactor:

- deduplicate event handling and error display formatting.

### Phase G: End-to-End + Regression

Red:

- integration tests in `app/src-tauri/tests/` with mocked updater adapter:
  - full lifecycle happy path
  - no update path
  - tampered artifact path
  - rollback path

Green:

- implement missing wiring + restart-required behavior.

Refactor:

- tighten fixture reuse and naming for maintainability.

## 5) Test Inventory (Enhanced)

## 5.1 Rust unit tests

Add files:

- `app/src-tauri/src/core/updater/state_tests.rs`
- `app/src-tauri/src/core/updater/service_tests.rs`
- `app/src-tauri/src/core/updater/errors_tests.rs`

Critical cases:

- legal/illegal transitions
- pending-candidate lifecycle semantics
- cooldown computation boundaries
- semantic version comparison edge cases (same version, non-semver fallback behavior)
- mandatory update behavior

## 5.2 Rust integration tests

Add files:

- `app/src-tauri/tests/updater_command_tests.rs`
- `app/src-tauri/tests/updater_lifecycle_tests.rs`
- `app/src-tauri/tests/updater_security_tests.rs`
- `app/src-tauri/tests/updater_logging_tests.rs`

Critical cases:

- command-to-service mapping and JSON response contract
- event names and payloads
- structured log events emitted exactly as cataloged
- rollback trigger and completion/error outcomes

## 5.3 Frontend tests

Add files:

- `app/src/lib/api/updaterClient.test.ts`
- `app/src/lib/stores/updaterState.test.ts`
- extend `app/src/tests/ui_flow.test.ts` for updater interactions

Critical cases:

- event-driven state updates
- stale concurrent request handling
- retry UX for network/offline failures

## 5.4 Contract fixtures

Add fixtures under:

- `tests/fixtures/updater/manifest_valid.json`
- `tests/fixtures/updater/manifest_hash_mismatch.json`
- `tests/fixtures/updater/manifest_signature_invalid.json`
- `tests/fixtures/updater/check_no_update.json`
- `tests/fixtures/updater/check_update_available.json`

## 6) Logging and Error Taxonomy Compliance

Must emit all updater events defined in:

- `docs/enhanced-logging-event-catalog.md` (check/download/verify/install/rollback set)

Must map failures to shared taxonomy in:

- `docs/shared-error-taxonomy.md`

Must include structured fields:

- `error_code`, `error_message`, stage-specific metadata

Definition of done for logging:

- each updater lifecycle step has success + failure logs
- throttled progress logs are capped as specified
- tests assert both event names and required metadata keys

## 7) Incremental Delivery Plan (PR slices)

1. PR-1: Contracts + state machine + tests (no runtime update calls yet)
2. PR-2: Updater commands + in-memory pending state + tests
3. PR-3: Runtime adapter + progress/finished events + tests
4. PR-4: Security verification + rollback + tests
5. PR-5: Frontend client/store/UI + tests
6. PR-6: Cross-layer integration tests + docs sync

Each PR must pass:

- `cargo test`
- frontend tests (`npm test`/`vitest`)
- parity/regression suites that may be impacted by command registration or event APIs

## 8) Risks and Mitigations

- Tauri updater plugin/runtime differences by OS:
  - mitigate with adapter trait + OS-specific test fixtures.
- Signature/hash verification complexity:
  - mitigate with injectable verifier and deterministic fixtures.
- Event spam/perf noise:
  - mitigate with strict 1/sec throttle tests.
- State race conditions (parallel check/install):
  - mitigate with lock-protected state + transition-guard tests.

## 9) Definition of Done

Updater is done when all are true:

1. check/install commands implemented and registered in Tauri.
2. frontend can check, show availability, install, and surface restart-required state.
3. lifecycle follows documented state machine and cooldown rules.
4. security verification and rollback behaviors are implemented and tested.
5. structured logs and errors comply with shared catalogs/taxonomy.
6. full Rust + frontend updater test suites pass in CI.


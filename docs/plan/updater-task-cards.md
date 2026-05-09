# Updater Task Cards (Independent Execution)

## Card U1: Updater Contracts and State Model

- Goal: Define updater DTOs and state machine primitives without runtime integration.
- Scope:
  - Add `core/updater/contracts.rs`
  - Add `core/updater/state.rs`
  - Export updater module from `core/mod.rs`
- Files:
  - `app/src-tauri/src/core/mod.rs`
  - `app/src-tauri/src/core/updater/contracts.rs`
  - `app/src-tauri/src/core/updater/state.rs`
  - `app/src-tauri/src/core/updater/mod.rs`
- Tests:
  - `app/src-tauri/src/core/updater/state_tests.rs`
  - validate legal/illegal transitions
  - validate serde contract stability
- Dependencies: none
- Done when:
  - all tests pass
  - no command wiring yet

## Card U2: Updater Error Taxonomy Mapping

- Goal: Implement updater-specific error domain mapped to shared taxonomy codes.
- Scope:
  - Add `core/updater/errors.rs`
  - map runtime/validation failures to stable error codes
- Files:
  - `app/src-tauri/src/core/updater/errors.rs`
  - `app/src-tauri/src/core/updater/mod.rs`
- Tests:
  - `app/src-tauri/src/core/updater/errors_tests.rs`
  - include hash mismatch, signature invalid, offline, timeout, no-pending-update
- Dependencies: U1
- Done when:
  - error mapping table exists and is fully unit tested

## Card U3: In-Memory Updater State Store

- Goal: Add lock-safe in-memory store for lifecycle state + pending candidate + cooldown timestamps.
- Scope:
  - Add `core/updater/state_store` inside `service.rs` or `state.rs` companion
  - no Tauri commands yet
- Files:
  - `app/src-tauri/src/core/updater/service.rs`
  - `app/src-tauri/src/core/updater/mod.rs`
- Tests:
  - `app/src-tauri/src/core/updater/service_tests.rs`
  - concurrent read/write safety
  - single pending-candidate semantics
  - cooldown boundary behavior
- Dependencies: U1, U2
- Done when:
  - service APIs are deterministic and thread-safe in tests

## Card U4: Updater Command Surface (Check + Install)

- Goal: Expose `check_for_updates` and `install_available_update` as Tauri commands.
- Scope:
  - Add `commands/updater.rs`
  - register commands in `commands/mod.rs` and `main.rs`
  - wire to updater service abstraction (mockable)
- Files:
  - `app/src-tauri/src/commands/updater.rs`
  - `app/src-tauri/src/commands/mod.rs`
  - `app/src-tauri/src/main.rs`
- Tests:
  - `app/src-tauri/tests/updater_command_tests.rs`
  - command response contract
  - install rejected when no pending update
- Dependencies: U3
- Done when:
  - commands callable and tested without real updater runtime

## Card U5: Updater Runtime Adapter

- Goal: Implement adapter boundary for platform updater runtime/plugin calls.
- Scope:
  - add adapter trait and concrete implementation
  - keep business logic in service, runtime behind trait
- Files:
  - `app/src-tauri/src/core/updater/service.rs`
  - `app/src-tauri/src/core/updater/mod.rs`
- Tests:
  - adapter mocked in service tests
  - no network/plugin dependency in unit tests
- Dependencies: U3
- Done when:
  - runtime can be replaced by test doubles with full coverage

## Card U6: Updater Events Emission and Throttle

- Goal: Emit updater lifecycle events with exact names and throttle policy.
- Scope:
  - Add `core/updater/events.rs`
  - emit `updater-progress` and `updater-finished`
  - enforce progress max 1 event/sec
- Files:
  - `app/src-tauri/src/core/updater/events.rs`
  - `app/src-tauri/src/core/updater/service.rs`
- Tests:
  - `app/src-tauri/tests/updater_lifecycle_tests.rs`
  - throttle behavior and terminal event emission
- Dependencies: U4, U5
- Done when:
  - event payloads and emission cadence are validated

## Card U7: Manifest Parsing and Semver Policy

- Goal: Parse update manifest and implement version/channel policy decisions.
- Scope:
  - parse required manifest fields
  - apply semver comparison policy including non-semver fallback
  - enforce `minimum_client_version` rules
- Files:
  - `app/src-tauri/src/core/updater/contracts.rs`
  - `app/src-tauri/src/core/updater/service.rs`
- Tests:
  - `app/src-tauri/src/core/updater/service_tests.rs`
  - fixture-based parser tests under `tests/fixtures/updater/`
- Dependencies: U1, U3
- Done when:
  - decision outputs are deterministic for all fixtures

## Card U8: Download Verification (Hash + Signature)

- Goal: Enforce artifact integrity/authenticity checks before install.
- Scope:
  - implement hash validation
  - implement signature validation
  - delete invalid artifacts
- Files:
  - `app/src-tauri/src/core/updater/service.rs`
  - `app/src-tauri/src/core/updater/errors.rs`
- Tests:
  - `app/src-tauri/tests/updater_security_tests.rs`
  - mismatch and invalid-signature paths
- Dependencies: U5, U7
- Done when:
  - failure paths produce correct taxonomy errors and cleanup behavior

## Card U9: Install, Restart-Required, and Rollback Flow

- Goal: Complete install path with rollback and restart-required signaling.
- Scope:
  - install only from `UPDATE_AVAILABLE`
  - emit restart-required signal when needed
  - rollback on install/post-launch failure
- Files:
  - `app/src-tauri/src/core/updater/service.rs`
  - `app/src-tauri/src/core/updater/state.rs`
- Tests:
  - `app/src-tauri/tests/updater_lifecycle_tests.rs`
  - rollback success and rollback failure behavior
- Dependencies: U8
- Done when:
  - end-to-end lifecycle tests pass with mocked runtime

## Card U10: Structured Logging Integration

- Goal: Emit full updater logging catalog events and metadata.
- Scope:
  - add structured logging calls in each lifecycle stage
  - include required fields (`error_code`, `error_message`, versions, durations)
- Files:
  - `app/src-tauri/src/core/updater/service.rs`
  - shared logging integration points used by other modules
- Tests:
  - `app/src-tauri/tests/updater_logging_tests.rs`
  - assert event names and required metadata keys
- Dependencies: U6, U8, U9
- Done when:
  - logs conform to `enhanced-logging-event-catalog.md`

## Card U11: Frontend Updater API Client

- Goal: Add typed frontend invocation layer for updater commands/events.
- Scope:
  - add `src/lib/api/updaterClient.ts`
  - command invocation + event subscription wrappers
- Files:
  - `app/src/lib/api/updaterClient.ts`
  - optional shared helpers in `app/src/lib/api/tauriClient.ts`
- Tests:
  - `app/src/lib/api/updaterClient.test.ts`
- Dependencies: U4, U6
- Done when:
  - frontend can call commands and receive typed updater events

## Card U12: Frontend Updater Store

- Goal: Add reactive updater store implementing lifecycle transitions and UI-ready flags.
- Scope:
  - add `src/lib/stores/updaterState.ts`
  - handle progress, errors, restart-required status
- Files:
  - `app/src/lib/stores/updaterState.ts`
- Tests:
  - `app/src/lib/stores/updaterState.test.ts`
- Dependencies: U11
- Done when:
  - store transitions mirror backend state model

## Card U13: UI Integration (Check/Install UX)

- Goal: Surface updater controls and status in app UI.
- Scope:
  - integrate check/install actions
  - show state, progress, and failure messages
  - disable install action outside valid states
- Files:
  - `app/src/routes/+page.svelte`
- Tests:
  - extend `app/src/tests/ui_flow.test.ts`
- Dependencies: U12
- Done when:
  - UI behavior verified for happy and failure flows

## Card U14: Test Fixtures and Regression Coverage

- Goal: Add updater fixtures and stabilize cross-layer regression tests.
- Scope:
  - add manifest/check fixtures
  - ensure integration tests use shared deterministic fixtures
- Files:
  - `tests/fixtures/updater/*`
  - updater test files in `app/src-tauri/tests/` and `app/src/lib/**`
- Tests:
  - all updater tests green in one run
- Dependencies: U7, U8, U9, U13
- Done when:
  - test fixture set is complete and reused consistently

## Card U15: Final Wiring and Documentation Sync

- Goal: finalize production wiring and document implemented behavior.
- Scope:
  - confirm Tauri config, command registration, and frontend wiring
  - update relevant docs/checklists with implementation status
- Files:
  - `app/src-tauri/src/main.rs`
  - `app/src-tauri/src/commands/mod.rs`
  - `docs/plan/updater-enhanced-tdd-implementation-plan.md` (status notes)
  - related docs checklists if required
- Tests:
  - full rust + frontend test run
- Dependencies: U10, U14
- Done when:
  - complete updater slice is shippable and documented

## Suggested Parallel Tracks

- Track A (backend core): U1 -> U2 -> U3 -> U5 -> U7 -> U8 -> U9 -> U10
- Track B (command surface): U4 (after U3), then assist U6
- Track C (frontend): U11 -> U12 -> U13 (starts after U4/U6)
- Track D (quality): U14 ongoing after U7; U15 at end

## Independence Rules for Implementation

- Each card must compile and test in isolation with mocks/fakes.
- Cards must avoid hidden coupling by using trait-based seams.
- No card should require partial work from a later card.
- If a dependency is discovered mid-card, split the card before coding.

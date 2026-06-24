# Devolens Licensing Migration Task Cards (TDD-Based)

This document outlines the remaining tasks required to migrate the licensing backend from the custom Cloudflare Worker to Devolens/Cryptolens. Each task card follows a strict Test-Driven Development (TDD) workflow (Red-Green-Refactor) to ensure safety and correctness.

---

## Pre-Implementation Requirements

Before starting execution, ensure the following are complete:
1. **Devolens Account Setup**: Access to a Devolens (Cryptolens) developer account.
2. **Product Configuration**: A product configured in Devolens with an assigned Product ID.
3. **Environment Setup**:
   - Ensure the following variables are documented but NOT committed to version control:
     - `DEVOLENS_BASE_URL` (usually `https://api.cryptolens.io`)
     - `DEVOLENS_ACCESS_TOKEN` (privileged API key)
     - `DEVOLENS_PRODUCT_ID`
   - Mode is set in environment files as `LICENSE_BACKEND_MODE=devolens`.
4. **Access Token Permission Configuration**:
   - **Tauri Desktop Client Token (Client-Facing)**:
     - Select: `Activate` (required), `Deactivate` (recommended for self-service resets).
     - Do NOT Select: `Create Key`, `Block Key`, `Get Keys`, `Get Products` (keeps license creation secure).
   - **Backend / Webhook Token (Gumroad & Admin Proxy)**:
     - Select: `Create Key`, `Block Key`, `Unblock Key`, `Get Key`, `Deactivate`.
     - Optional: `Add Customer`, `Get Customers`.

---

## Migration Task Cards

### Task Card 1: Token Scope & Production Architecture Decision
- **Description**: Determine the production architecture for Devolens integration (Direct Tauri-to-Devolens vs. Thin Backend Proxy) based on the scope and safety of the Devolens access token.
- **Preconditions**:
  - Review Devolens documentation regarding token scopes and permissions.
  - Review `app/src-tauri/src/auth_worker.rs` and `app/src-tauri/src/core/config.rs`.
- **TDD Workflow**:
  1. **Red Phase**: Write an architectural constraint test in a new file `vendor/license-control-suite/tests/baseline/devolens_token_safety.rs` that asserts the desktop configuration loader throws a compile-time check or runtime error if a token with broad license write/delete permissions is loaded directly. Run tests manually to verify compile or runtime failure.
  2. **Green Phase**: Document the token capability decision. If the token is too privileged, implement a thin proxy server-side. Update the configuration validation logic so the test passes.
  3. **Refactor Phase**: Clean up any duplicate verification logic in config parsing.
- **Verification Criteria**:
  - `devolens_token_safety.rs` test compiles and passes.
  - A formal Architecture Decision Record (ADR) file is committed to the repository.

---

### Task Card 2: Devolens API Response Fixtures Collection
- **Description**: Collect real, sanitized JSON response payloads from the Devolens API for various test cases to construct robust mock fixtures.
- **Preconditions**:
  - Access to a Devolens test product.
- **TDD Workflow**:
  1. **Red Phase**: Write a parser test `test_devolens_parser_handles_all_fixtures` in `app/src-tauri/tests/auth_worker_tests.rs` that attempts to load and deserialize expected fixture files from `vendor/license-control-suite/fixtures/devolens/`. Verify the test fails because the files do not exist.
  2. **Green Phase**: Query the Devolens API (or use official docs) to collect JSON responses for: success activation, invalid key, expired key, blocked key, machine limit reached, and network error. Sanitize all secrets and write them to the target folder. Verify the parser test now passes.
  3. **Refactor Phase**: Deduplicate JSON field parsing helpers and structure response structures for clean deserialization.
- **Verification Criteria**:
  - `test_devolens_parser_handles_all_fixtures` passes.
  - Fixture JSON files exist and contain no real license keys or tokens.

---

### Task Card 3: Switch Validation Strategy from ReauthRequired to Real Provider Validation
- **Description**: Replace the temporary `ValidationOutcome::ReauthRequired` stub in the Devolens adapter with a production-grade session validation call.
- **Preconditions**:
  - Task Card 1 & 2 complete.
- **TDD Workflow**:
  1. **Red Phase**: In `app/src-tauri/tests/auth_worker_tests.rs`, write an integration test using `wiremock` expecting `/api/key/Validate` to be queried with the session token. Assert that the client returns `ValidationOutcome::Active` or `ValidationOutcome::Revoked`. Verify that running this test fails with `ValidationOutcome::ReauthRequired`.
  2. **Green Phase**: Implement the real validation request in `validate_session` in `DevolensWorkerClient` (in `app/src-tauri/src/auth_worker.rs`). Map the deserialized response fields to `ValidationOutcome`. Verify the test passes.
  3. **Refactor Phase**: Extract the HTTP request construction into a shared private helper in `DevolensWorkerClient`.
- **Verification Criteria**:
  - `wiremock` validation tests pass successfully.
  - `validate_session` contains no hardcoded `ValidationOutcome::ReauthRequired` fallback.

---

### Task Card 4: Devolens-native Session / Access Token Logic
- **Description**: Replace the temporary local `devolens:{device_id}:{timestamp_ms}` session token with a robust provider-returned token or a signed session assertion.
- **Preconditions**:
  - Task Card 3 complete.
- **TDD Workflow**:
  1. **Red Phase**: Write a unit test `test_devolens_access_token_integrity` in `auth_worker_tests.rs` that calls validation using a forged or modified access token and asserts it returns `AuthError::ReauthRequired`. Verify that the test currently passes the forged token because the client accepts the temporary hardcoded string pattern without validation.
  2. **Green Phase**: Implement token signing/decoding (using cryptographically secure local mechanisms or real provider tokens) during activation. Ensure `validate_session` verifies the signature. The test should now fail on forged tokens and pass on genuine ones.
  3. **Refactor Phase**: Ensure cryptography helpers are shared cleanly and utilize existing project-standard crypto modules.
- **Verification Criteria**:
  - `test_devolens_access_token_integrity` successfully blocks invalid tokens and passes valid ones.

---

### Task Card 5: Device Reset/Unbind Design and Mapping
- **Description**: Map the existing custom Worker admin-reviewed reset flow to Devolens' machine activation model.
- **Preconditions**:
  - Review `worker/src/index.js` reset implementation (lines 800-871).
- **TDD Workflow**:
  1. **Red Phase**: Write a test in `app/src-tauri/tests/auth_command_inventory_tests.rs` (or equivalent mock command test) asserting that when a machine limit error is received from the provider, the Tauri command mapping returns a structured, user-facing error code representing the option to request a device reset. Running this test fails with generic worker unreachable or parse error.
  2. **Green Phase**: Implement the design mapping by updating the Tauri command parser to catch machine limit responses and translate them to the appropriate frontend state. Verify the test passes.
  3. **Refactor Phase**: Document the UI transitions and error message mappings in a markdown design file in `docs/`.
- **Verification Criteria**:
  - Reset mapping test passes and maps the machine limit code correctly.

---

### Task Card 6: Implement `request_device_reset` for Devolens
- **Description**: Implement the `request_device_reset` and `get_device_reset_status` methods in the Devolens adapter.
- **Preconditions**:
  - Task Card 5 complete.
- **TDD Workflow**:
  1. **Red Phase**: Write a wiremock integration test `test_request_device_reset_calls_devolens` in `auth_worker_tests.rs` asserting that calling `request_device_reset` routes a request to the Devolens API. Run the test and verify it fails with `AuthError::InvalidResetRequest`.
  2. **Green Phase**: Implement `request_device_reset` and `get_device_reset_status` in `DevolensWorkerClient` to hit the Devolens API or proxy backend and return a valid status. Verify the test passes.
  3. **Refactor Phase**: Deduplicate reset request and status structures.
- **Verification Criteria**:
  - Reset integration tests pass and request correct provider endpoints.

---

### Task Card 7: Rework Gumroad Webhook Integration
- **Description**: Rework the legacy Worker's Gumroad webhook to parse the Gumroad-generated license key on purchase and provision that exact key string into Devolens using `/api/key/CreateKey`.
- **Preconditions**:
  - Access to Gumroad webhook configurations.
  - Access to Devolens/Cryptolens Web API token with `Create Key` and `Block Key` permissions.
- **TDD Workflow**:
  1. **Red Phase**: In `worker/test/contract.test.js` (or webhook unit tests), write a test `test_gumroad_webhook_provisions_gumroad_license_to_devolens` that posts a fake Gumroad sale payload containing `license_key: "GUMROAD-VAL-KEY"`. Assert that it calls the Devolens `/api/key/CreateKey` API with the query parameters/body field `Key=GUMROAD-VAL-KEY`. Verify the test fails because the endpoint does not call Devolens (or does not forward the custom key).
  2. **Green Phase**: Update the webhook handler logic to extract Gumroad's `license_key` from the webhook body and pass it as the `Key` parameter when invoking Devolens `CreateKey`. Also ensure that refund/dispute webhooks deactivate or block that exact key string. Verify the test passes.
  3. **Refactor Phase**: Clean up and deduplicate request signature checks and parameter extraction.
- **Verification Criteria**:
  - Gumroad webhook integration tests pass.
  - Webhook verified to correctly pass Gumroad-generated keys as Devolens custom key strings.

---

### Task Card 8: Design and Implement License Import/Migration Plan
- **Description**: Extract existing active customer license records from the D1 database and migrate them to Devolens.
- **Preconditions**:
  - D1 database access and production exports.
- **TDD Workflow**:
  1. **Red Phase**: Create a migration validation script `tests/migration_validation.py` (or Rust equivalent) that parses a sample D1 database file and asserts that the migration script output perfectly maps to Devolens payload fields. Run the test and verify it fails due to missing migration scripts.
  2. **Green Phase**: Write the migration script to extract, map, and import keys to Devolens. Run the validation test and verify it passes.
  3. **Refactor Phase**: Ensure the migration script logs any skipped or invalid records cleanly and redacts emails.
- **Verification Criteria**:
  - Migration script successfully maps 100% of sample active license records.
  - Validation test exits with zero status.

---

### Task Card 9: Refactor/Retain Worker Components as a Thin Companion Service
- **Description**: Remove deprecated licensing routes from the Cloudflare Worker under `worker/` and simplify it to a thin companion service.
- **Preconditions**:
  - Task Cards 1, 6, and 7 complete.
- **TDD Workflow**:
  1. **Red Phase**: Write a test suite in `worker/test/contract.test.js` asserting that the endpoints `/v1/license/activate` and `/v1/license/validate` return `404 Not Found` or `410 Gone` in Devolens mode. Verify the tests fail because the routes are still active and functional.
  2. **Green Phase**: Modify the routing logic in `worker/src/index.js` to remove these routes. Run the tests to verify they pass.
  3. **Refactor Phase**: Remove unused database helpers, unused migration scripts, and dead endpoints from the worker codebase.
- **Verification Criteria**:
  - Refactored Worker tests pass.
  - Unused routes are deleted and return 404/410.

---

### Task Card 10: Rework Tauri Admin Commands and UI for Devolens
- **Description**: Update the desktop app's Admin Dashboard command and UI code to integrate with Devolens or direct admins to the Devolens Dashboard.
- **Preconditions**:
  - Review `app/src-tauri/src/commands/admin.rs`.
- **TDD Workflow**:
  1. **Red Phase**: Write unit tests in `vendor/license-control-suite/tests/ipc/tauri_command_composition.rs` (or equivalent admin test) that assert admin operations (like listing licenses or device bindings) return an error or return zero records when Devolens mode is active. Verify the tests fail because they still try to query the custom Worker.
  2. **Green Phase**: Update `app/src-tauri/src/commands/admin.rs` to handle Devolens mode. Disable or redirect admin actions. Run tests to verify they pass.
  3. **Refactor Phase**: Clean up any unused Tauri invoke declarations that were strictly D1 admin-dependent.
- **Verification Criteria**:
  - Admin integration tests pass.
  - UI hides D1-specific options in Devolens mode.

---

### Task Card 11: Privacy Deletion (GDPR) Workflow for Devolens
- **Description**: Update the user privacy deletion command to wipe customer data from Devolens.
- **Preconditions**:
  - Review `app/src-tauri/src/commands/privacy.rs`.
- **TDD Workflow**:
  1. **Red Phase**: In `app/src-tauri/src/commands/privacy.rs` unit tests, write a test asserting that a deletion request calls Devolens' customer data deletion/anonymization API. Verify the test fails because it still calls the custom Worker privacy endpoint.
  2. **Green Phase**: Update `request_user_data_deletion` and `get_user_data_deletion_status` in `privacy.rs` to call Devolens API. Verify the test passes.
  3. **Refactor Phase**: Ensure the request uses secure idempotency keys and properly handles transient network errors.
- **Verification Criteria**:
  - Privacy tests verify deletion calls are successfully routed to Devolens.

---

### Task Card 12: In-App and External Privacy Policy/Legal Copy Updates
- **Description**: Update terms, privacy policy, and in-app legal copy to specify Devolens/Cryptolens as a data processor.
- **Preconditions**:
  - Knowledge of Devolens personal data collection policies (IPs, emails, machine codes).
- **TDD Workflow**:
  1. **Red Phase**: Write a regex assertion test `test_legal_copy_mentions_devolens` in `tests/baseline/ci_workflow_sequence.rs` (or a documentation checker script) that scans legal and policy files for the string "Devolens" or "Cryptolens". Verify the test fails.
  2. **Green Phase**: Update the in-app help texts, terms of service, and privacy policies to detail Devolens licensing. Run the test to verify it passes.
  3. **Refactor Phase**: Ensure legal copy is concise, clear, and consistent across all files.
- **Verification Criteria**:
  - Documentation checker test passes.
  - No legacy database claims remain in the copy.

---

### Task Card 13: Enhanced Error Mapping and User-friendly UI Messages
- **Description**: Replace coarse error mappings with detailed Devolens error mapping and secure user-facing messages.
- **Preconditions**:
  - Task Card 2 (fixtures) complete.
- **TDD Workflow**:
  1. **Red Phase**: Write a test `test_devolens_error_mapping` in `auth_worker_tests.rs` that inputs various Devolens error codes (e.g., product ID mismatch, expired key, blocked key, limit reached) and asserts they map to specific local `AuthError` enums. Verify the test fails because all map to a generic error.
  2. **Green Phase**: Modify the deserialization and mapping logic in `DevolensWorkerClient` to parse and map all expected codes. Verify the test passes.
  3. **Refactor Phase**: Ensure that raw error messages from Devolens are redacted so they do not print to standard logs.
- **Verification Criteria**:
  - `test_devolens_error_mapping` passes with all tested codes.

---

### Task Card 14: Offline Mode and Clock Skew Handling
- **Description**: Define and implement offline license validation rules and clock skew tolerance for Devolens mode.
- **Preconditions**:
  - Task Card 3 complete.
- **TDD Workflow**:
  1. **Red Phase**: Write tests in `auth_worker_tests.rs` simulating offline behavior:
     - `test_devolens_offline_grace_valid`: Asserts app is valid offline within the grace period.
     - `test_devolens_offline_grace_expired`: Asserts app blocks usage after grace period.
     - `test_devolens_clock_skew`: Asserts local clock skew does not result in false activation failures.
     Verify these tests fail because offline logic isn't wired to Devolens mode.
  2. **Green Phase**: Implement offline verification logic and local cache checks in `DevolensWorkerClient`. Verify the tests pass.
  3. **Refactor Phase**: Optimize cache storage mechanism and clean up datetime calculations.
- **Verification Criteria**:
  - Offline tests successfully verify grace constraints and handle clock skew.

---

### Task Card 15: Full Integration Testing, Security Auditing, and Manual Validation
- **Description**: Perform end-to-end verification, verify zero secret leakage, and run the manual validation commands.
- **Preconditions**:
  - Task Cards 1-14 complete.
- **TDD Workflow**:
  1. **Red Phase**: Modify the validation script `.scripts/run-license-fallback-validation.sh` (or create a custom security checker script) to verify that no test API keys, Devolens tokens, or raw license keys exist in version-controlled files. Running this script will fail if any secrets are present.
  2. **Green Phase**: Clean up any residual development secrets, configure proper environment loading, and verify that the validation script passes.
  3. **Refactor Phase**: Ensure the validation script creates clean, timestamped logs under `.logs/` and cleans up temporary test artifacts.
- **Verification Criteria**:
  - `.scripts/run-license-fallback-validation.sh` exits zero.
  - Codebase security audits detect no committed credentials.

---

## Execution Roadmap

1. **Phase 1: Foundational Setup & Design** (Cards 1, 2, 5)
2. **Phase 2: Client Implementation** (Cards 3, 4, 6, 13)
3. **Phase 3: Automation & Integration** (Cards 7, 8, 9)
4. **Phase 4: Admin, Privacy, & Legal** (Cards 10, 11, 12)
5. **Phase 5: Offline, Hardening & Validation** (Cards 14, 15)

---

## Production Readiness Checklist

- [ ] No broad Devolens management tokens are included or compiled into the client Tauri app.
- [ ] Devolens activation and validation responses are correctly parsed and mapped to local states.
- [ ] Revoked, expired, refunded, and machine-limit states are enforced.
- [ ] Existing users' license keys are successfully imported into Devolens.
- [ ] Gumroad purchase, refund, and dispute webhooks are active and mapped.
- [ ] User privacy deletion requests delete/anonymize relevant records in Devolens.
- [ ] Admin panel is updated or disabled appropriately in Devolens mode.
- [ ] Privacy policy and in-app legal copy reflect the updated processor data flow.
- [ ] All tests pass and the validation script executes successfully.

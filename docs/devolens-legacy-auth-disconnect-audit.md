# Devolens Legacy Auth Disconnect Audit

Date: 2026-06-25

## Scope

This audit checks whether legacy auth/licensing surfaces are disconnected after the Devolens work. "Disconnected" means not referenced by tracked code, tests, scripts, manifests, configs, docs workflows, runtime mode selection, or validation tooling.

Evidence sources used:

- `graphify-out/` query for auth/licensing relationships.
- Direct `rg` searches across app, Worker, vendor, tests, scripts, manifests, configs, and docs.
- Direct reads of Rust, Svelte-adjacent config, Worker, test, script, and manifest files.
- `git ls-files` and ignored-status checks for candidate cleanup artifacts.

## Conclusion

Do not delete tracked legacy Worker/auth/licensing files in this pass. The tracked Worker, Rust `HttpWorkerClient` path, `LicenseBackendMode` variants, fixtures, migrations, and tests are still connected.

Only ignored local artifacts were proven disconnected and safe to remove:

- `.scripts/debug-worker-activate.sh`
- `.scripts/run-reset-schema-fix.sh`
- `worker/.wrangler/`

## Connected Legacy Surfaces To Keep

### Rust Backend Selection

`app/src-tauri/src/core/config.rs` still defines and parses four backend modes: `reference`, `hosted`, `devolens`, and `mock` (`LicenseBackendMode`, lines 51-78). `Config::from_env` reads `LICENSE_BACKEND_MODE`, defaulting to `reference` when unset (lines 99-100).

`app/src-tauri/src/auth_worker.rs` still routes:

- `Reference` and `Hosted` to the vendor `HttpWorkerClient`.
- `Devolens` to `DevolensWorkerClient`.
- `Mock` to `MockLicenseWorkerClient`.

This routing is in `build_worker_client` (lines 206-230), so the vendor Worker client path is still active code.

### Environment Configuration

`.env.example` still sets `LICENSE_BACKEND_MODE=hosted` and points `LICENSE_WORKER_BASE_URL` at the hosted Worker (lines 2-3). It also documents Devolens as an alternative mode (lines 12-16). This keeps hosted Worker configuration user-facing.

### Vendor Client

`app/src-tauri/Cargo.toml` depends on `license-control-suite` with the `desktop-tauri` feature enabled (line 19). `app/src-tauri/src/auth_worker.rs` imports and uses `HttpWorkerClient`, so `vendor/license-control-suite/src/modules/user_reg/auth_licensing_tauri/http_client.rs` is still compiled and connected through app code.

### Worker Service

The Worker remains referenced by scripts and docs:

- Root `package.json` exposes `worker:test` as `pnpm --dir worker run test` (line 10).
- `worker/README.md` documents license, privacy, admin, Gumroad webhook, and updater routes (lines 10-24).
- `worker/wrangler.toml` still defines the Worker entrypoint and D1 binding (lines 1-13).

`worker/src/index.js` still exposes Worker routes. In Worker env `LICENSE_BACKEND_MODE=devolens`, only `/v1/license/activate` and `/v1/license/validate` return `410 Gone` (lines 59-62). Other routes remain registered, including reset, privacy deletion, admin, Gumroad webhook, and updater paths (lines 71-120).

### Worker Tests, Fixtures, And Contract Parity

`worker/test/contract.test.js` still tests the Worker contract and explicitly asserts activate/validate return `410 Gone` in Devolens mode (lines 1536-1562).

`tests/fixtures/license_worker_contract_v1/` is still used by `tests/parity/license_worker_contract_v1_parity_test.py`, where `FIXTURE_DIR` points at that fixture directory (lines 5-8). The parity test validates frozen error codes and response shapes, so these fixtures are still connected.

### Privacy Flow

`app/src-tauri/src/commands/privacy.rs` uses Devolens directly only when `LICENSE_BACKEND_MODE=devolens`:

- Deletion requests call `devolens_block_key` in Devolens mode (lines 206-210), otherwise they post to `/v1/privacy/delete/request` (lines 224-229).
- Deletion status calls `devolens_get_key` only for Devolens-style request IDs in Devolens mode (lines 244-255), otherwise it falls back to `/v1/privacy/delete/status`.

This means Worker privacy routes are still active for non-Devolens modes.

### Validation Scripts

`.scripts/security-audit.sh` is ignored but connected: `.scripts/run-license-fallback-validation.sh` calls it at line 22. It should not be deleted in this pass.

## Disconnected Ignored Artifacts Removed

The following artifacts were ignored, not tracked, and had no active references from tracked repo content:

- `.scripts/debug-worker-activate.sh`
- `.scripts/run-reset-schema-fix.sh`
- `worker/.wrangler/`

These were local Worker/debug/cache artifacts only. Removing them does not change tracked source behavior.

## Needs Human Decision

The following cleanup requires an explicit product/security decision before code deletion:

- Whether to remove `reference` and `hosted` backend modes entirely.
- Whether the Worker remains as a Gumroad, updater, privacy, admin, and migration companion service only.
- Whether D1 migrations and Worker contract fixtures should be archived after a formal Worker-route removal refactor.

Until those decisions are made, tracked Worker/auth/licensing surfaces should remain.

## Manual Verification Commands

Repository policy forbids agents from running cargo, pnpm, node test, pytest, build, lint, install, and validation-script commands. The following commands were not run by the agent and should be run manually if verification is needed:

```bash
.scripts/run-license-fallback-validation.sh
.scripts/run-admin-devolens-validation.sh
.scripts/run-privacy-devolens-validation.sh
pnpm run worker:test
pnpm --dir app run test
cargo test --locked --manifest-path app/src-tauri/Cargo.toml --test auth_worker_tests --test config_tests --test admin_devolens_tests
.venv/bin/python -m pytest tests/parity/license_worker_contract_v1_parity_test.py tests/migration_validation.py
```

No new validation script was added because this pass changed documentation and removed ignored local artifacts only.

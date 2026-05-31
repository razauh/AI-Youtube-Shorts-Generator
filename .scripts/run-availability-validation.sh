#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="${LOG_DIR}/availability-validation-${TS}.log"

mkdir -p "${LOG_DIR}"
exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] availability validation started: ${TS}"

cd "${ROOT_DIR}"

echo "[check] worker contract tests (includes /readyz + runtime-pack manifest route)"
node --test worker/test/contract.test.js

echo "[check] bundled runtime scan"
bash .scripts/scan-bundled-runtime.sh

echo "[check] d1 migration chain sanity (masked_license_key added only once)"
if rg -n "masked_license_key" worker/migrations/0001_init.sql >/dev/null; then
  echo "[fail] masked_license_key must not appear in worker/migrations/0001_init.sql"
  exit 1
fi
if ! rg -n "ALTER TABLE reset_requests ADD COLUMN masked_license_key" worker/migrations/0002_add_masked_license_key_to_reset_requests.sql >/dev/null; then
  echo "[fail] expected masked_license_key migration missing in 0002"
  exit 1
fi

echo "[check] runtime-pack download is streaming (no full-body bytes load)"
if rg -n "runtime pack download body read failed" app/src-tauri/src/commands/runtime.rs >/dev/null && rg -n "\\.bytes\\(\\)\\s*\\.await" app/src-tauri/src/commands/runtime.rs >/dev/null; then
  echo "[fail] runtime-pack download appears to use full-body bytes() loading"
  exit 1
fi

echo "[check] generation recovery wiring (run_id + cancel command)"
if ! rg -n "pub run_id: Option<String>" app/src-tauri/src/commands/generate.rs >/dev/null; then
  echo "[fail] expected generate run_id support missing"
  exit 1
fi
if ! rg -n "pub struct ProgressEvent" app/src-tauri/src/core/contracts.rs >/dev/null || ! rg -n "pub run_id: Option<String>" app/src-tauri/src/core/contracts.rs >/dev/null; then
  echo "[fail] expected ProgressEvent.run_id missing"
  exit 1
fi
if ! rg -n "export interface ProgressEvent" app/src/lib/contracts.ts >/dev/null || ! rg -n "run_id\\?: string" app/src/lib/contracts.ts >/dev/null; then
  echo "[fail] expected frontend ProgressEvent.run_id missing"
  exit 1
fi
if ! rg -n "cancel_generate_run" app/src-tauri/src/main.rs >/dev/null; then
  echo "[fail] expected cancel_generate_run command not registered"
  exit 1
fi

echo "[check] rust tests"
cargo test --locked --manifest-path app/src-tauri/Cargo.toml

echo "[check] app tests"
pnpm --dir app run test

echo "[info] availability validation complete: PASS"
echo "[info] log written to ${LOG_FILE}"

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

contains() {
  if command -v rg >/dev/null 2>&1; then
    rg -n "$1" "$2" >/dev/null
  else
    grep -En "$1" "$2" >/dev/null
  fi
}

echo "[check] worker contract tests (includes /readyz + runtime-pack manifest route)"
node --test worker/test/contract.test.js

echo "[check] bundled runtime scan"
bash .scripts/scan-bundled-runtime.sh

echo "[check] d1 migration chain sanity (masked_license_key added only once)"
if contains "masked_license_key" worker/migrations/0001_init.sql; then
  echo "[fail] masked_license_key must not appear in worker/migrations/0001_init.sql"
  exit 1
fi
if ! contains "ALTER TABLE reset_requests ADD COLUMN masked_license_key" worker/migrations/0002_add_masked_license_key_to_reset_requests.sql; then
  echo "[fail] expected masked_license_key migration missing in 0002"
  exit 1
fi

echo "[check] runtime-pack download is streaming (no full-body bytes load)"
if contains "runtime pack download body read failed" app/src-tauri/src/commands/runtime.rs && contains "\\.bytes\\(\\)\\s*\\.await" app/src-tauri/src/commands/runtime.rs; then
  echo "[fail] runtime-pack download appears to use full-body bytes() loading"
  exit 1
fi

echo "[check] generation recovery wiring (run_id + cancel command)"
if ! contains "pub run_id: Option<String>" app/src-tauri/src/commands/generate.rs; then
  echo "[fail] expected generate run_id support missing"
  exit 1
fi
if ! contains "pub struct ProgressEvent" app/src-tauri/src/core/contracts.rs || ! contains "pub run_id: Option<String>" app/src-tauri/src/core/contracts.rs; then
  echo "[fail] expected ProgressEvent.run_id missing"
  exit 1
fi
if ! contains "export interface ProgressEvent" app/src/lib/contracts.ts || ! contains "run_id\\?: string" app/src/lib/contracts.ts; then
  echo "[fail] expected frontend ProgressEvent.run_id missing"
  exit 1
fi
if ! contains "cancel_generate_run" app/src-tauri/src/main.rs; then
  echo "[fail] expected cancel_generate_run command not registered"
  exit 1
fi

echo "[check] rust tests"
cargo test --locked --manifest-path app/src-tauri/Cargo.toml

echo "[check] app tests"
pnpm --dir app run test

echo "[info] availability validation complete: PASS"
echo "[info] log written to ${LOG_FILE}"

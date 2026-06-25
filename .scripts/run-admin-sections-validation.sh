#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p .logs
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE=".logs/admin-sections-validation-${TS}.log"

run_and_log() {
  echo "[$(date +%Y-%m-%dT%H:%M:%S%z)] $*" | tee -a "$LOG_FILE"
  "$@" 2>&1 | tee -a "$LOG_FILE"
}

run_and_log node --test worker/test/contract.test.js
run_and_log pnpm --dir app run test -- src/tests/admin/admin_client.test.ts src/tests/admin/admin_messages.test.ts src/tests/admin/admin_ui.test.ts src/tests/admin/admin_inventory.test.ts
run_and_log cargo test --locked --manifest-path app/src-tauri/Cargo.toml admin_

echo "Validation completed. Log: $LOG_FILE" | tee -a "$LOG_FILE"

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

if rg -n "admin_(list|approve|reject)_reset_request|listResetRequests|approveResetRequest|rejectResetRequest" \
  app/src-tauri/src/bin/admin_desktop.rs \
  app/src/admin/AdminApp.svelte \
  app/src/admin/lib/adminClient.ts 2>&1 | tee -a "$LOG_FILE"; then
  echo "Deprecated admin reset queue command/UI surface is still exposed." | tee -a "$LOG_FILE"
  exit 1
fi

if rg -n 'admin_list_licenses|admin_list_device_bindings|listLicenses|listDeviceBindings|"\/v1/admin/licenses"|"\/v1/admin/device-bindings"' \
  app/src-tauri/src/bin/admin_desktop.rs \
  app/src/admin/AdminApp.svelte \
  app/src/admin/lib/adminClient.ts \
  app/src/tests/admin/admin_client.test.ts \
  app/src/tests/admin/admin_ui.test.ts \
  app/src/tests/admin/admin_inventory.test.ts \
  worker/src/index.js \
  worker/src/routes_inventory.json \
  worker/src/d1_authority_inventory.json \
  worker/src/d1_schema_inventory.json 2>&1 | tee -a "$LOG_FILE"; then
  echo "Broad admin license/device listing surface is still exposed." | tee -a "$LOG_FILE"
  exit 1
fi

run_and_log node --test worker/test/contract.test.js worker/test/routes_inventory.test.js worker/test/d1_authority_inventory.test.js
run_and_log pnpm --dir app run test -- src/tests/admin/admin_client.test.ts src/tests/admin/admin_messages.test.ts src/tests/admin/admin_ui.test.ts src/tests/admin/admin_inventory.test.ts
run_and_log cargo test --locked --manifest-path app/src-tauri/Cargo.toml admin_

echo "Validation completed. Log: $LOG_FILE" | tee -a "$LOG_FILE"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/user-data-deletion-validation-${STAMP}.log"

mkdir -p "${LOG_DIR}"

run_step() {
  local name="$1"
  shift
  {
    echo "== ${name} =="
    echo "command: $*"
    "$@"
    echo
  } 2>&1 | tee -a "${LOG_FILE}"
}

priv_01_contract_source_check() {
  local pattern="blockDevolensKeyForPrivacy|local_privacy_data_deleted_or_anonymized|devolens_action"
  if command -v rg >/dev/null 2>&1; then
    rg -n "${pattern}" worker/src worker/test
    return
  fi
  grep -R -n -E "${pattern}" worker/src worker/test
}

priv_02_admin_review_source_check() {
  local pattern="privacy_review|local_d1_status|devolens_action_status|operator_next_step|Devolens-owned license deletion"
  if command -v rg >/dev/null 2>&1; then
    rg -n "${pattern}" worker/src worker/test app/src/admin app/src/tests/admin app/src-tauri/src/commands/admin.rs
    return
  fi
  grep -R -n -E "${pattern}" worker/src worker/test app/src/admin app/src/tests/admin app/src-tauri/src/commands/admin.rs
}

cd "${ROOT_DIR}"

run_step "priv-01-contract-source-check" priv_01_contract_source_check
run_step "priv-02-admin-review-source-check" priv_02_admin_review_source_check
run_step "worker-contract-tests" pnpm run worker:test
run_step "frontend-and-rust-tests" pnpm run test
run_step "app-build" pnpm run build

echo "validation logs: ${LOG_FILE}"

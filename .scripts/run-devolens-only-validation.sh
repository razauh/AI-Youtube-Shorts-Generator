#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/.logs"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

mkdir -p "$LOG_DIR"
cd "$ROOT_DIR"

run_check() {
  local name="$1"
  shift
  local log_file="$LOG_DIR/${STAMP}-${name}.log"
  echo
  echo "Running $name"
  echo "Command: $*"
  echo "Log: $log_file"
  "$@" 2>&1 | tee "$log_file"
}

run_check rust-auth-config-admin \
  cargo test --locked --manifest-path app/src-tauri/Cargo.toml \
    --test auth_worker_tests \
    --test config_tests \
    --test admin_devolens_tests

run_check worker-contract pnpm run worker:test

run_check app-frontend pnpm --dir app run test

run_check python-companion-parity \
  .venv/bin/python -m pytest \
    tests/parity/license_worker_contract_v1_parity_test.py \
    tests/migration_validation.py

echo "Validation complete. Logs written to $LOG_DIR with timestamp $STAMP."

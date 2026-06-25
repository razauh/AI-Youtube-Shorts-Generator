#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/tauri-storage-validation-${STAMP}.log"

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

cd "${ROOT_DIR}"

run_step "cargo-test-license-control-suite" cargo test --manifest-path vendor/license-control-suite/Cargo.toml --test user_reg_core_service_int
run_step "cargo-test-license-control-suite-ipc" cargo test --manifest-path vendor/license-control-suite/Cargo.toml --test user_reg_command_contracts
run_step "cargo-test-app-tauri-commands" cargo test --manifest-path app/src-tauri/Cargo.toml --test tauri_command_tests

echo "validation logs: ${LOG_FILE}"

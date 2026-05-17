#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="${ROOT}/app/src-tauri/Cargo.toml"
LOG_DIR="${ROOT}/logs"
LOG_FILE="${LOG_FILE:-${LOG_DIR}/local-license-readiness.log}"

mkdir -p "${LOG_DIR}"
: > "${LOG_FILE}"

run_step() {
  local name="$1"
  shift
  printf '[local-readiness] %s...\n' "${name}"
  {
    printf '\n===== %s =====\n' "${name}"
    printf '$'
    printf ' %q' "$@"
    printf '\n'
  } >> "${LOG_FILE}"
  if "$@" >> "${LOG_FILE}" 2>&1; then
    printf '[local-readiness] %s passed\n' "${name}"
  else
    printf '[local-readiness] %s failed. See %s\n' "${name}" "${LOG_FILE}" >&2
    exit 1
  fi
}

run_step "cargo check" cargo check --manifest-path "${MANIFEST}"
run_step "config tests" cargo test --manifest-path "${MANIFEST}" --test config_tests
run_step "auth worker tests" cargo test --manifest-path "${MANIFEST}" --test auth_worker_tests
run_step "auth command inventory tests" cargo test --manifest-path "${MANIFEST}" --test auth_command_inventory_tests
run_step "tauri command tests" cargo test --manifest-path "${MANIFEST}" --test tauri_command_tests

printf '[local-readiness] all checks passed. Log: %s\n' "${LOG_FILE}"

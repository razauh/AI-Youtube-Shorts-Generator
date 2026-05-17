#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/logs"
LOG_FILE="${LOG_FILE:-${LOG_DIR}/secure-cargo-check.log}"
TAURI_MANIFEST="${ROOT_DIR}/app/src-tauri/Cargo.toml"
ROOT_MANIFEST="${ROOT_DIR}/Cargo.toml"

mkdir -p "${LOG_DIR}"
: > "${LOG_FILE}"

run_step() {
  local name="$1"
  shift
  printf '[secure-cargo] %s...\n' "${name}"
  {
    printf '\n===== %s =====\n' "${name}"
    printf '$'
    printf ' %q' "$@"
    printf '\n'
  } >> "${LOG_FILE}"
  if "$@" >> "${LOG_FILE}" 2>&1; then
    printf '[secure-cargo] %s passed\n' "${name}"
  else
    printf '[secure-cargo] %s failed. See %s\n' "${name}" "${LOG_FILE}" >&2
    exit 1
  fi
}

run_step "tauri check (locked)" cargo check --manifest-path "${TAURI_MANIFEST}" --locked
run_step "tauri tests (locked)" cargo test --manifest-path "${TAURI_MANIFEST}" --locked
run_step "root check (locked)" cargo check --manifest-path "${ROOT_MANIFEST}" --locked

run_step "lockfile drift (root)" git -C "${ROOT_DIR}" diff --exit-code -- Cargo.lock
run_step "lockfile drift (tauri)" git -C "${ROOT_DIR}" diff --exit-code -- app/src-tauri/Cargo.lock

printf '[secure-cargo] all checks passed. Log: %s\n' "${LOG_FILE}"

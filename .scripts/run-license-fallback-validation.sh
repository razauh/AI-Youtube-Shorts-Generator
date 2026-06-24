#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date +%Y%m%d-%H%M%S)"

mkdir -p "${LOG_DIR}"

run_step() {
  local name="$1"
  shift
  local log_file="${LOG_DIR}/license-fallback-${name}-${TS}.log"

  echo "[info] running ${name}; log: ${log_file}"
  (
    cd "${ROOT_DIR}"
    "$@"
  ) 2>&1 | tee "${log_file}"
}

run_step "rust-fallback-tests" cargo test --locked --manifest-path app/src-tauri/Cargo.toml fallback
run_step "rust-resilient-store-tests" cargo test --locked --manifest-path app/src-tauri/Cargo.toml resilient_secret_store
run_step "rust-devolens-auth-worker-tests" cargo test --locked --manifest-path app/src-tauri/Cargo.toml --test auth_worker_tests --test config_tests
run_step "rust-devolens-token-safety-baseline-test" cargo test --locked --manifest-path vendor/license-control-suite/Cargo.toml --test devolens_token_safety
run_step "frontend-policy-tests" pnpm --dir app run test -- src/tests/ui_flow.test.ts

echo "[pass] license fallback validation completed"

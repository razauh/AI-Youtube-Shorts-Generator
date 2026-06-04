#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="${LOG_DIR}/customer-csp-validation-${TS}.log"

mkdir -p "${LOG_DIR}"

exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] customer CSP validation started: ${TS}"

cd "${ROOT_DIR}"

cargo test --locked --manifest-path app/src-tauri/Cargo.toml customer_tauri_csp

echo "[info] customer CSP validation complete: PASS"
echo "[info] log written to ${LOG_FILE}"

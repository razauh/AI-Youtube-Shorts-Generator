#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="${LOG_DIR}/updater-endpoint-validation-${TS}.log"

mkdir -p "${LOG_DIR}"

exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] updater endpoint validation started: ${TS}"

cd "${ROOT_DIR}"

pnpm --dir worker run test
cargo test --locked --manifest-path app/src-tauri/Cargo.toml --test updater_plugin_config_tests
node --test .scripts/generate-customer-updater-manifest.test.mjs
bash .scripts/validate-release-ci-config.sh

echo "[info] updater endpoint validation complete: PASS"
echo "[info] log written to ${LOG_FILE}"

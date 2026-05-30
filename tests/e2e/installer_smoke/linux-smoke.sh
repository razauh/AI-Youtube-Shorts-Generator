#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/installer-smoke-linux-${TS}.log"
ARTIFACT_DIR="${1:-${ROOT_DIR}/app/src-tauri/target/release/bundle}"

mkdir -p "${LOG_DIR}"
exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] linux installer smoke started: ${TS}"
echo "[info] artifact dir: ${ARTIFACT_DIR}"

if [[ ! -d "${ARTIFACT_DIR}" ]]; then
  echo "[fail] artifact directory does not exist"
  exit 1
fi

shopt -s globstar nullglob
artifacts=(
  "${ARTIFACT_DIR}"/**/*.AppImage
  "${ARTIFACT_DIR}"/**/*.deb
  "${ARTIFACT_DIR}"/**/*.rpm
)

if [[ ${#artifacts[@]} -eq 0 ]]; then
  echo "[fail] no Linux installer artifacts found"
  exit 1
fi

for artifact in "${artifacts[@]}"; do
  [[ -s "${artifact}" ]] || { echo "[fail] empty artifact: ${artifact}"; exit 1; }
  echo "[pass] found artifact: ${artifact}"
done

if [[ ! -f "${ROOT_DIR}/app/src-tauri/tauri.conf.json" ]]; then
  echo "[fail] missing customer Tauri config"
  exit 1
fi

if grep -Eq 'YOUR_CLOUDFLARE_SUBDOMAIN|REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY|AI Shorts App|Signal Forge' "${ROOT_DIR}/app/src-tauri/tauri.conf.json" "${ROOT_DIR}/app/index.html"; then
  echo "[fail] release config still contains placeholder updater or stale product text"
  exit 1
fi

echo "[pass] Linux installer smoke checks completed"
echo "[info] log written to ${LOG_FILE}"

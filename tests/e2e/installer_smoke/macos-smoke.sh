#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${LOG_DIR}/installer-smoke-macos-${TS}.log"
ARTIFACT_DIR="${1:-${ROOT_DIR}/app/src-tauri/target/release/bundle}"

mkdir -p "${LOG_DIR}"
exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] macOS installer smoke started: ${TS}"
echo "[info] artifact dir: ${ARTIFACT_DIR}"

if [[ ! -d "${ARTIFACT_DIR}" ]]; then
  echo "[fail] artifact directory does not exist"
  exit 1
fi

shopt -s globstar nullglob
artifacts=(
  "${ARTIFACT_DIR}"/**/*.dmg
  "${ARTIFACT_DIR}"/**/*.app.tar.gz
)

if [[ ${#artifacts[@]} -eq 0 ]]; then
  echo "[fail] no macOS release artifacts found"
  exit 1
fi

for artifact in "${artifacts[@]}"; do
  [[ -s "${artifact}" ]] || { echo "[fail] empty artifact: ${artifact}"; exit 1; }
  echo "[pass] found artifact: ${artifact}"
done

if command -v spctl >/dev/null 2>&1; then
  for dmg in "${ARTIFACT_DIR}"/**/*.dmg; do
    [[ -e "${dmg}" ]] || continue
    spctl --assess --type open --context context:primary-signature --verbose "${dmg}"
    echo "[pass] spctl accepted: ${dmg}"
  done
else
  echo "[warn] spctl not available; notarization assessment must be run on macOS"
fi

echo "[pass] macOS installer smoke checks completed"
echo "[info] log written to ${LOG_FILE}"

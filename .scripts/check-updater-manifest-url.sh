#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.logs"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="${LOG_DIR}/check-updater-manifest-url-${TS}.log"

MANIFEST_URL="${1:-https://github.com/razauh/AI-Youtube-Shorts-Generator/releases/latest/download/customer-latest.json}"

mkdir -p "${LOG_DIR}"
exec > >(tee -a "${LOG_FILE}") 2>&1

echo "[info] updater manifest url check started: ${TS}"
echo "[info] manifest url: ${MANIFEST_URL}"
echo

echo "[cmd] curl -i \"${MANIFEST_URL}\""
curl -i "${MANIFEST_URL}"
echo

echo "[cmd] curl -iL \"${MANIFEST_URL}\""
curl -iL "${MANIFEST_URL}"
echo

echo "[cmd] curl -sSL \"${MANIFEST_URL}\" | head -n 40"
curl -sSL "${MANIFEST_URL}" | head -n 40
echo

echo "[info] updater manifest url check complete"
echo "[info] log written to ${LOG_FILE}"


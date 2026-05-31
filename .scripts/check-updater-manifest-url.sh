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

FAILED=0

run_check() {
  local label="$1"
  shift
  echo "[cmd] $*"
  set +e
  "$@"
  local status=$?
  set -e
  if [[ ${status} -ne 0 ]]; then
    echo "[fail] ${label} exited with status ${status}"
    FAILED=1
  fi
  echo
}

run_check "headers_without_redirects" curl --connect-timeout 15 -i "${MANIFEST_URL}"

run_check "headers_with_redirects" curl --connect-timeout 15 -iL "${MANIFEST_URL}"

echo "[cmd] curl -sSIL -o /dev/null -w final_url=%{url_effective} final_code=%{http_code} \"${MANIFEST_URL}\""
set +e
curl --connect-timeout 15 -sSIL -o /dev/null -w "final_url=%{url_effective}\nfinal_code=%{http_code}\n" "${MANIFEST_URL}"
STATUS=$?
set -e
if [[ ${STATUS} -ne 0 ]]; then
  echo "[fail] final_status exited with status ${STATUS}"
  FAILED=1
fi
echo

echo "[cmd] curl -sSL \"${MANIFEST_URL}\" | sed -n '1,40p'"
set +e
curl --connect-timeout 15 -sSL "${MANIFEST_URL}" | sed -n '1,40p'
PIPE_STATUS=("${PIPESTATUS[@]}")
set -e
if [[ ${PIPE_STATUS[0]} -ne 0 ]]; then
  echo "[fail] body_preview exited with status ${PIPE_STATUS[0]}"
  FAILED=1
fi
echo

echo "[info] updater manifest url check complete"
echo "[info] log written to ${LOG_FILE}"

exit "${FAILED}"

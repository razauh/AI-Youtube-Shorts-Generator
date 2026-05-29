#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/logs"
REPORT="${LOG_DIR}/pnpm-installed-audit-report.log"
BASELINE_COMMIT="${1:-}"

mkdir -p "${LOG_DIR}"
: > "${REPORT}"

log() {
  printf '%s\n' "$*" | tee -a "${REPORT}"
}

run_capture() {
  local title="$1"
  shift
  log ""
  log "===== ${title} ====="
  log "$ $*"
  if "$@" >> "${REPORT}" 2>&1; then
    log "[ok] ${title}"
  else
    log "[warn] ${title} failed"
  fi
}

search_cmd() {
  if command -v rg >/dev/null 2>&1; then
    printf 'rg'
  else
    printf 'grep'
  fi
}

log "[pnpm-audit] root=${ROOT_DIR}"
log "[pnpm-audit] report=${REPORT}"

run_capture "Git status for pnpm manifests" \
  git -C "${ROOT_DIR}" status --short -- package.json app/package.json worker/package.json pnpm-workspace.yaml pnpm-lock.yaml

run_capture "Git diff for pnpm manifests" \
  git -C "${ROOT_DIR}" diff -- package.json app/package.json worker/package.json pnpm-workspace.yaml pnpm-lock.yaml

run_capture "Top-level installed packages" \
  pnpm list --recursive --depth 0

run_capture "Full dependency tree" \
  pnpm list --recursive --depth Infinity

if [[ -n "${BASELINE_COMMIT}" ]]; then
  run_capture "Lockfile additions since baseline commit" \
    bash -lc "git -C '${ROOT_DIR}' diff --unified=0 '${BASELINE_COMMIT}' -- pnpm-lock.yaml | rg '^\\+\\s{2,}[^[:space:]].*:'"
fi

if [[ "$(search_cmd)" == "rg" ]]; then
  run_capture "Packages declaring install hooks in node_modules" \
    bash -lc "rg -n '\"(preinstall|install|postinstall)\"' '${ROOT_DIR}/node_modules' '${ROOT_DIR}/app/node_modules' '${ROOT_DIR}/worker/node_modules' -g 'package.json'"
else
  run_capture "Packages declaring install hooks in node_modules" \
    bash -lc "grep -RInE '\"(preinstall|install|postinstall)\"' '${ROOT_DIR}/node_modules' '${ROOT_DIR}/app/node_modules' '${ROOT_DIR}/worker/node_modules' --include='package.json'"
fi

if [[ "$(search_cmd)" == "rg" ]]; then
  run_capture "High-risk code patterns in node_modules" \
    bash -lc "rg -n 'eval\\(|Function\\(|child_process|curl|wget|powershell|base64|atob|fromCharCode' '${ROOT_DIR}/node_modules' '${ROOT_DIR}/app/node_modules' '${ROOT_DIR}/worker/node_modules'"
else
  run_capture "High-risk code patterns in node_modules" \
    bash -lc "grep -RInE 'eval\\(|Function\\(|child_process|curl|wget|powershell|base64|atob|fromCharCode' '${ROOT_DIR}/node_modules' '${ROOT_DIR}/app/node_modules' '${ROOT_DIR}/worker/node_modules'"
fi

run_capture "Production vulnerability audit" \
  pnpm audit --prod

log ""
log "[pnpm-audit] done"
log "[pnpm-audit] report: ${REPORT}"
if [[ -z "${BASELINE_COMMIT}" ]]; then
  log "[pnpm-audit] tip: pass a trusted commit to diff lockfile additions:"
  log "  bash scripts/audit-installed-pnpm.sh <trusted_commit>"
fi

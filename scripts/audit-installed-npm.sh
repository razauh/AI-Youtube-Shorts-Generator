#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="${ROOT_DIR}/app"
LOG_DIR="${ROOT_DIR}/logs"
REPORT="${LOG_DIR}/npm-installed-audit-report.log"
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

log "[npm-audit] root=${ROOT_DIR}"
log "[npm-audit] app=${APP_DIR}"
log "[npm-audit] report=${REPORT}"

run_capture "Git status for npm manifests" \
  git -C "${ROOT_DIR}" status --short -- app/package.json app/package-lock.json app/.npmrc

run_capture "Git diff for npm manifests" \
  git -C "${ROOT_DIR}" diff -- app/package.json app/package-lock.json app/.npmrc

run_capture "Top-level installed packages" \
  npm --prefix "${APP_DIR}" ls --depth=0

run_capture "Full dependency tree" \
  npm --prefix "${APP_DIR}" ls --all

if [[ -n "${BASELINE_COMMIT}" ]]; then
  run_capture "Lockfile additions since baseline commit" \
    bash -lc "git -C '${ROOT_DIR}' diff --unified=0 '${BASELINE_COMMIT}' -- app/package-lock.json | rg '^\+.*\"name\":|^\+.*\"version\":'"
fi

if [[ "$(search_cmd)" == "rg" ]]; then
  run_capture "Packages declaring install hooks in node_modules" \
    bash -lc "rg -n '\"(preinstall|install|postinstall)\"' '${APP_DIR}/node_modules'/*/package.json"
else
  run_capture "Packages declaring install hooks in node_modules" \
    bash -lc "grep -RInE '\"(preinstall|install|postinstall)\"' '${APP_DIR}/node_modules' --include='package.json'"
fi

if [[ "$(search_cmd)" == "rg" ]]; then
  run_capture "High-risk code patterns in node_modules" \
    bash -lc "rg -n 'eval\\(|Function\\(|child_process|curl|wget|powershell|base64|atob|fromCharCode' '${APP_DIR}/node_modules'"
else
  run_capture "High-risk code patterns in node_modules" \
    bash -lc "grep -RInE 'eval\\(|Function\\(|child_process|curl|wget|powershell|base64|atob|fromCharCode' '${APP_DIR}/node_modules'"
fi

run_capture "Production vulnerability audit" \
  npm --prefix "${APP_DIR}" audit --production

log ""
log "[npm-audit] done"
log "[npm-audit] report: ${REPORT}"
if [[ -z "${BASELINE_COMMIT}" ]]; then
  log "[npm-audit] tip: pass a trusted commit to diff lockfile additions:"
  log "  bash scripts/audit-installed-npm.sh <trusted_commit>"
fi

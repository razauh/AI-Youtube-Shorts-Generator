#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/logs"
REPORT="${LOG_DIR}/rust-supplychain-triage.log"
CRITICAL_DIFF="${LOG_DIR}/critical-deps-diff.log"
BASELINE_COMMIT="${1:-}"
TAURI_MANIFEST="${ROOT_DIR}/app/src-tauri/Cargo.toml"

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

log "[rust-triage] root=${ROOT_DIR}"
log "[rust-triage] report=${REPORT}"
log "[rust-triage] critical-diff=${CRITICAL_DIFF}"

run_capture "Lockfile status snapshot" \
  git -C "${ROOT_DIR}" status --short -- Cargo.lock app/src-tauri/Cargo.lock

run_capture "Lockfile source policy (non-crates.io)" \
  bash -lc "grep -n '^source = ' '${ROOT_DIR}/Cargo.lock' | grep -v 'registry+https://github.com/rust-lang/crates.io-index' > '${LOG_DIR}/rust-non-cratesio-sources.log' || true; wc -l '${LOG_DIR}/rust-non-cratesio-sources.log'; cat '${LOG_DIR}/rust-non-cratesio-sources.log'"

if [[ -n "${BASELINE_COMMIT}" ]]; then
  run_capture "Baseline diff against trusted commit" \
    git -C "${ROOT_DIR}" diff "${BASELINE_COMMIT}" -- Cargo.lock app/src-tauri/Cargo.lock
fi

if [[ ! -f "${CRITICAL_DIFF}" ]]; then
  run_capture "Generate critical deps diff log" \
    bash -lc "git -C '${ROOT_DIR}' diff -- Cargo.lock app/src-tauri/Cargo.lock > '${CRITICAL_DIFF}' 2>&1"
fi

run_capture "Extract newly added crate names" \
  bash -lc "grep -E '^\\+name = ' '${CRITICAL_DIFF}' | sed 's/^+name = \"//;s/\"$//' | sort -u > '${LOG_DIR}/new-rust-crates.txt'; wc -l '${LOG_DIR}/new-rust-crates.txt'; cat '${LOG_DIR}/new-rust-crates.txt'"

run_capture "Extract newly added crate versions" \
  bash -lc "awk 'BEGIN{open=0} /^\\+name = \\\"/{name=\$0;sub(/^\\+name = \\\"/,\"\",name);sub(/\\\"$/,\"\",name);open=1} /^\\+version = \\\"/ && open==1 {ver=\$0;sub(/^\\+version = \\\"/,\"\",ver);sub(/\\\"$/,\"\",ver);print name \" \" ver;open=0}' '${CRITICAL_DIFF}' > '${LOG_DIR}/new-rust-crates-with-versions.txt'; wc -l '${LOG_DIR}/new-rust-crates-with-versions.txt'; cat '${LOG_DIR}/new-rust-crates-with-versions.txt'"

run_capture "Extract checksums for added crates" \
  bash -lc "grep -n '^\\+checksum = \"' '${CRITICAL_DIFF}' > '${LOG_DIR}/new-rust-crate-checksums.txt' || true; wc -l '${LOG_DIR}/new-rust-crate-checksums.txt'; sed -n '1,120p' '${LOG_DIR}/new-rust-crate-checksums.txt'"

run_capture "Locked build check" \
  cargo check --manifest-path "${TAURI_MANIFEST}" --locked

log ""
log "[rust-triage] done"
log "[rust-triage] report: ${REPORT}"
log "[rust-triage] artifacts:"
log "  - ${LOG_DIR}/rust-non-cratesio-sources.log"
log "  - ${LOG_DIR}/new-rust-crates.txt"
log "  - ${LOG_DIR}/new-rust-crates-with-versions.txt"
log "  - ${LOG_DIR}/new-rust-crate-checksums.txt"
if [[ -z "${BASELINE_COMMIT}" ]]; then
  log "[rust-triage] tip: pass trusted baseline commit for comparison:"
  log "  bash scripts/rust-supplychain-triage.sh <trusted_commit>"
fi

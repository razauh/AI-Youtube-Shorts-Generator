#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-all}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_ROOT="${REPO_ROOT}/logs"
STAMP="$(date +"%Y%m%d_%H%M%S")"
RUN_DIR="${LOG_ROOT}/ci_${MODE}_${STAMP}"
SUMMARY="${RUN_DIR}/summary.log"

mkdir -p "${RUN_DIR}"

run_cmd() {
  local name="$1"
  local cwd="$2"
  local cmd="$3"
  local logfile="${RUN_DIR}/${name}.log"
  local code=0

  {
    echo "=== ${name} ==="
    echo "cwd: ${cwd}"
    echo "cmd: ${cmd}"
    echo "started: $(date -Is)"
  } > "${logfile}"

  (
    cd "${cwd}" && bash -lc "${cmd}"
  ) >> "${logfile}" 2>&1 || code=$?

  {
    echo "finished: $(date -Is)"
    echo "exit_code: ${code}"
  } >> "${logfile}"

  if [[ ${code} -eq 0 ]]; then
    echo "[PASS] ${name}" >> "${SUMMARY}"
  else
    echo "[FAIL] ${name} (exit ${code})" >> "${SUMMARY}"
    return "${code}"
  fi
}

write_header() {
  {
    echo "mode=${MODE}"
    echo "ci_run_dir=${RUN_DIR}"
    echo "started=$(date -Is)"
  } > "${SUMMARY}"
}

run_verify() {
  run_cmd "01_check_all" "${REPO_ROOT}" "bash scripts/check_all.sh"
  run_cmd "02_check_dependencies" "${REPO_ROOT}" "bash scripts/check_dependencies.sh"
  run_cmd "03_check_tauri_capabilities" "${REPO_ROOT}" "bash scripts/check_tauri_capabilities.sh"
}

run_downstream_consumers() {
  run_cmd "01_verify_downstream_consumers" "${REPO_ROOT}" "bash scripts/verify_downstream_consumers.sh"
}

run_publish_dry_run() {
  run_cmd "01_cargo_package" "${REPO_ROOT}" "cargo package --allow-dirty"
  run_cmd "02_cargo_publish_dry_run" "${REPO_ROOT}" "cargo publish --dry-run --allow-dirty"
}

run_tauri_debug_build() {
  if command -v cargo-tauri >/dev/null 2>&1; then
    run_cmd "01_cargo_tauri_build_debug" "${REPO_ROOT}" "cargo tauri build --debug"
  else
    run_cmd "01_install_tauri_cli" "${REPO_ROOT}" "cargo install tauri-cli --version '^2' --locked"
    run_cmd "02_cargo_tauri_build_debug" "${REPO_ROOT}" "cargo tauri build --debug"
  fi
}

write_header

case "${MODE}" in
  verify)
    run_verify
    ;;
  downstream-consumers)
    run_downstream_consumers
    ;;
  publish-dry-run)
    run_publish_dry_run
    ;;
  tauri-debug-build)
    run_tauri_debug_build
    ;;
  all)
    run_verify
    run_downstream_consumers
    run_publish_dry_run
    run_tauri_debug_build
    ;;
  *)
    {
      echo "[FAIL] unknown mode: ${MODE}"
      echo "expected one of: verify, downstream-consumers, publish-dry-run, tauri-debug-build, all"
    } >> "${SUMMARY}"
    exit 2
    ;;
esac

{
  echo "finished=$(date -Is)"
  echo "summary_log=${SUMMARY}"
} >> "${SUMMARY}"

printf 'ci_run_dir=%s\nsummary_log=%s\n' "${RUN_DIR}" "${SUMMARY}"

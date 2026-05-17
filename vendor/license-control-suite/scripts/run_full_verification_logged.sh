#!/usr/bin/env bash
set -u

ROOT="/home/pc/Downloads/inf/plan"
UNIFIED="${ROOT}/license-control-suite"
LOG_ROOT="${UNIFIED}/logs"
STAMP="$(date +"%Y%m%d_%H%M%S")"
RUN_DIR="${LOG_ROOT}/verification_${STAMP}"
SUMMARY="${RUN_DIR}/summary.log"

mkdir -p "${RUN_DIR}"

run_cmd() {
  local name="$1"
  local cwd="$2"
  local cmd="$3"
  local logfile="${RUN_DIR}/${name}.log"

  {
    echo "=== ${name} ==="
    echo "cwd: ${cwd}"
    echo "cmd: ${cmd}"
    echo "started: $(date -Is)"
  } > "${logfile}"

  (
    cd "${cwd}" && bash -lc "${cmd}"
  ) >> "${logfile}" 2>&1
  local code=$?

  {
    echo "finished: $(date -Is)"
    echo "exit_code: ${code}"
  } >> "${logfile}"

  if [[ ${code} -eq 0 ]]; then
    echo "[PASS] ${name}" | tee -a "${SUMMARY}" >/dev/null
  else
    echo "[FAIL] ${name} (exit ${code})" | tee -a "${SUMMARY}" >/dev/null
  fi
}

echo "verification_run_dir=${RUN_DIR}" > "${SUMMARY}"
echo "started=$(date -Is)" >> "${SUMMARY}"

# 0) Final docs existence
run_cmd "00_final_regression_report_exists" "${UNIFIED}" "test -f docs/migration/final_regression_report.md"
run_cmd "01_final_acceptance_checklist_exists" "${UNIFIED}" "test -f docs/migration/final_acceptance_checklist.md"
run_cmd "02_handoff_summary_exists" "${UNIFIED}" "test -f docs/migration/handoff_summary.md"

# 1) Baseline/discovery checks
run_cmd "03_check_tauri_inventory" "${ROOT}" "bash ${UNIFIED}/docs/baseline/check_tauri_inventory.sh"
run_cmd "04_user_reg_tauri_pattern_scan" "${ROOT}" "if command -v rg >/dev/null 2>&1; then rg -n \"#\\[tauri::command\\]|generate_handler|invoke_handler|session_state.json|reset_status.json|license_key|access_token|device_keypair\" ${ROOT}/user-reg; else grep -RInE \"#\\[tauri::command\\]|generate_handler|invoke_handler|session_state.json|reset_status.json|license_key|access_token|device_keypair\" ${ROOT}/user-reg; fi"
run_cmd "05_find_tauri_or_package_json" "${ROOT}" "find ${ROOT} -name 'tauri.conf.*' -o -name package.json -o -path '*/src-tauri/*'"
run_cmd "06_find_frontend_asset_paths" "${ROOT}" "find ${ROOT} -maxdepth 4 \\( -name package.json -o -name 'vite.config.*' -o -path '*/src-tauri/*' -o -path '*/frontend/*' -o -path '*/public/*' -o -path '*/assets/*' \\)"

# 2) Legacy metadata capture
run_cmd "07_metadata_shared_contracts" "${ROOT}/shared-contracts" "cargo metadata --no-deps --format-version 1"
run_cmd "08_metadata_admin_dashboard" "${ROOT}/admin-dashboard" "cargo metadata --no-deps --format-version 1"
run_cmd "09_metadata_auth_core" "${ROOT}/auth-core" "cargo metadata --no-deps --format-version 1"
run_cmd "10_metadata_user_reg" "${ROOT}/user-reg" "cargo metadata --no-deps --format-version 1"

# 3) Unified repo checks
run_cmd "11_unified_cargo_check" "${UNIFIED}" "cargo check"
run_cmd "12_check_dependencies_script" "${UNIFIED}" "bash scripts/check_dependencies.sh"
run_cmd "13_unified_metadata" "${UNIFIED}" "cargo metadata --format-version 1"
run_cmd "14_unified_tree_duplicates" "${UNIFIED}" "cargo tree -d"

# 4) Targeted test slices
run_cmd "15_test_shared_contracts" "${UNIFIED}" "cargo test shared_contracts"
run_cmd "16_test_admin_dashboard" "${UNIFIED}" "cargo test admin_dashboard"
run_cmd "17_test_auth_core" "${UNIFIED}" "cargo test auth_core"
run_cmd "18_test_user_reg_core" "${UNIFIED}" "cargo test user_reg_core"
run_cmd "19_test_user_reg_tauri" "${UNIFIED}" "cargo test user_reg_tauri"
run_cmd "20_test_licensing_worker" "${UNIFIED}" "cargo test licensing_worker"
run_cmd "21_test_ipc" "${UNIFIED}" "cargo test ipc"
run_cmd "22_test_serialization_overlap" "${UNIFIED}" "cargo test serialization_overlap"
run_cmd "23_test_runtime_storage" "${UNIFIED}" "cargo test runtime_storage"
run_cmd "24_test_concurrency" "${UNIFIED}" "cargo test concurrency"
run_cmd "25_test_command_inventory" "${UNIFIED}" "cargo test command_inventory"
run_cmd "26_test_frontend_source_verification" "${UNIFIED}" "cargo test frontend_source_verification"

# 5) Full regression
run_cmd "27_check_all_script" "${UNIFIED}" "bash scripts/check_all.sh"
run_cmd "28_test_all" "${UNIFIED}" "cargo test"

# 6) Packaging / tauri smoke
run_cmd "29_build_release" "${UNIFIED}" "cargo build --release"
run_cmd "30_tauri_smoke_script" "${UNIFIED}" "bash scripts/tauri_smoke.sh"
run_cmd "31_tauri_capabilities_script" "${UNIFIED}" "bash scripts/check_tauri_capabilities.sh"

# 7) Conditional tauri build
if command -v cargo >/dev/null 2>&1 && cargo tauri --help >/dev/null 2>&1; then
  run_cmd "32_tauri_debug_build" "${UNIFIED}" "cargo tauri build --debug"
else
  {
    echo "[SKIP] 32_tauri_debug_build (cargo-tauri not available)"
  } | tee -a "${SUMMARY}" >/dev/null
fi

echo "finished=$(date -Is)" >> "${SUMMARY}"
echo "summary_log=${SUMMARY}" >> "${SUMMARY}"

echo "Done. Logs written to: ${RUN_DIR}"
echo "Summary: ${SUMMARY}"

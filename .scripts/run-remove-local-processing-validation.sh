#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p .logs
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE=".logs/remove-local-processing-validation-${TS}.log"

exec > >(tee -a "$LOG_FILE") 2>&1

run_step() {
  echo "[STEP] $*"
  "$@"
}

python_bin() {
  if [[ -x "$ROOT_DIR/.venv/bin/python" ]]; then
    echo "$ROOT_DIR/.venv/bin/python"
    return 0
  fi

  if command -v python >/dev/null 2>&1; then
    command -v python
    return 0
  fi

  if command -v python3 >/dev/null 2>&1; then
    command -v python3
    return 0
  fi

  echo "python or python3 was not found on PATH" >&2
  return 1
}

search_removed_refs() {
  local pattern='local processing|Local Processing|local mode|Local mode|local/offline|runtime-pack|runtime pack|local model|faster-whisper|faster_whisper|yt-dlp|LOCAL_WHISPER|LOCAL_OUTPUT|requirements-local|bridge_entry|prefetch_local_model|run_local|python_runtime|tool_resolver|process_supervisor|local_mode|local_model|local_runtime_pack|pick_local_video_file|open_in_file_manager|validate_runtime|pickLocalVideoFile|openInFileManager|validateRuntime|localModel|localRuntime|mode": "local"'
  local paths=(.github .scripts app docs packaging python_legacy scripts tests worker README.md)

  if command -v rg >/dev/null 2>&1; then
    rg -n \
      --glob '!app/dist/**' \
      --glob '!docs/local-processing-components.md' \
      --glob '!docs/local-processing-removal-audit.md' \
      --glob '!docs/local-processing-removal-final-report.md' \
      --glob '!docs/remove-local-processing-plan.md' \
      --glob '!target/**' \
      --glob '!node_modules/**' \
      --glob '!graphify-out/**' \
      --glob '!app/.cargo-target/**' \
      --glob '!app/.logs/**' \
      --glob '!.scripts/run-remove-local-processing-validation.sh' \
      "$pattern" \
      "${paths[@]}"
    return $?
  fi

  grep -RInE \
    --exclude-dir=dist \
    --exclude-dir=target \
    --exclude-dir=node_modules \
    --exclude-dir=graphify-out \
    --exclude-dir=.cargo-target \
    --exclude-dir=.logs \
    --exclude=local-processing-components.md \
    --exclude=local-processing-removal-audit.md \
    --exclude=local-processing-removal-final-report.md \
    --exclude=remove-local-processing-plan.md \
    --exclude=run-remove-local-processing-validation.sh \
    "$pattern" \
    "${paths[@]}"
}

search_api_generation_entrypoints() {
  local pattern='runGenerateAndStream|mode: .api.|generate_shorts_stream|MuAPI|muapi'
  local paths=(app/src python_legacy app/src-tauri/src)

  if command -v rg >/dev/null 2>&1; then
    rg -n "$pattern" "${paths[@]}" >/dev/null
    return $?
  fi

  grep -RInE "$pattern" "${paths[@]}" >/dev/null
}

echo "[INFO] Validation started at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "[INFO] Repo root: $ROOT_DIR"

echo "[STEP] Search validation for removed local-processing commands and strings"
if search_removed_refs; then
  echo "[FAIL] Removed local-processing references are still present."
  exit 1
fi

echo "[STEP] API generation entry points still present"
search_api_generation_entrypoints

echo "[STEP] Frontend tests"
run_step pnpm --dir app run test

echo "[STEP] Frontend build"
run_step pnpm --dir app run build

echo "[STEP] Worker contract tests"
run_step pnpm run worker:test

echo "[STEP] Backend Rust check"
run_step cargo check --manifest-path app/src-tauri/Cargo.toml --locked

echo "[STEP] Backend Rust tests"
run_step cargo test --manifest-path app/src-tauri/Cargo.toml --locked

echo "[STEP] Python tests"
PYTHON_BIN="$(python_bin)"
run_step "$PYTHON_BIN" -m pytest tests

echo "[STEP] Release CI config validation"
run_step bash .scripts/validate-release-ci-config.sh

echo "[INFO] Validation completed successfully at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "[INFO] Log file: $LOG_FILE"

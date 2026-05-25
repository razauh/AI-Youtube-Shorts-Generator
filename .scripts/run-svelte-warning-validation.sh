#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p .logs
TS="$(date +%Y%m%d-%H%M%S)"
LOG_FILE=".logs/svelte-warning-validation-${TS}.log"

run_and_log() {
  echo "[$(date +%Y-%m-%dT%H:%M:%S%z)] $*" | tee -a "$LOG_FILE"
  "$@" 2>&1 | tee -a "$LOG_FILE"
}

run_and_log npm --prefix app run build

echo "Svelte warning validation completed. Log: $LOG_FILE" | tee -a "$LOG_FILE"

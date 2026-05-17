#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

echo "[baseline] source inventory checks"
bash docs/baseline/check_source_inventory.sh

echo "[baseline] baseline run log structure checks"
bash docs/baseline/original-runs/validate_baseline_logs.sh

echo "[baseline] dependency matrix checks"
bash docs/baseline/cargo_metadata/validate_dependency_matrix.sh

echo "[baseline] tauri/frontend inventory checks"
bash docs/baseline/check_tauri_inventory.sh

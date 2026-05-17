#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

echo "[check_all] cargo check"
cargo check

echo "[check_all] cargo test"
cargo test

echo "[check_all] tauri smoke"
bash scripts/tauri_smoke.sh

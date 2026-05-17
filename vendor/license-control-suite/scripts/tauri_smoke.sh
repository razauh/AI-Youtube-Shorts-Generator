#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

shells=(
  "examples/client-desktop-shell/src-tauri"
  "examples/admin-desktop-shell/src-tauri"
)

found=0
for shell in "${shells[@]}"; do
  if [[ -d "${shell}" ]] && find "${shell}" -maxdepth 2 -type f -name "tauri.conf.*" | grep -q .; then
    echo "[tauri_smoke] shell detected: ${shell}"
    if [[ ! -d "${shell}/capabilities" ]]; then
      echo "[tauri_smoke] BLOCKED: ${shell}/capabilities is missing."
      exit 2
    fi
    echo "[tauri_smoke] capability directory detected: ${shell}/capabilities"
    found=1
  fi
done

if [[ ${found} -eq 0 ]]; then
  echo "[tauri_smoke] BLOCKED: no verified example Tauri desktop shell exists yet under examples/*/src-tauri."
  exit 2
fi

echo "[tauri_smoke] NOTE: execute example-shell Tauri builds manually outside this script."

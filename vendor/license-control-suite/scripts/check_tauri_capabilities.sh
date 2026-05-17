#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

config="examples/client-desktop-shell/src-tauri/tauri.conf.json"
caps_dir="examples/client-desktop-shell/src-tauri/capabilities"

if [[ ! -f "${config}" ]]; then
  echo "[tauri-capabilities] BLOCKED: missing ${config} (requires source verification)." >&2
  exit 2
fi

if [[ ! -d "${caps_dir}" ]]; then
  echo "[tauri-capabilities] BLOCKED: missing ${caps_dir} (requires source verification)." >&2
  exit 2
fi

if ! find "${caps_dir}" -maxdepth 1 -type f -name '*.json' | grep -q .; then
  echo "[tauri-capabilities] BLOCKED: no capability JSON files found in ${caps_dir}." >&2
  exit 2
fi

required_cmds=(
  "activate_license"
  "validate_session"
  "request_device_reset"
  "get_device_reset_status"
  "clear_local_session"
  "get_auth_state"
)

cap_json="$(cat "${caps_dir}"/*.json)"
for cmd in "${required_cmds[@]}"; do
  if command -v rg >/dev/null 2>&1; then
    matches_command() {
      printf "%s" "${cap_json}" | rg -q "$1"
    }
  else
    matches_command() {
      printf "%s" "${cap_json}" | grep -q "$1"
    }
  fi

  if ! matches_command "${cmd}"; then
    echo "[tauri-capabilities] Missing command permission mapping for ${cmd}" >&2
    exit 1
  fi
done

echo "[tauri-capabilities] capability smoke check passed."

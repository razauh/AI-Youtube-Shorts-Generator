#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUNDLED_DIR="${ROOT_DIR}/app/src-tauri/bundled-runtime"

if [[ ! -d "${BUNDLED_DIR}" ]]; then
  echo "[skip] bundled runtime directory not found: ${BUNDLED_DIR}"
  exit 0
fi

echo "[info] scanning bundled runtime: ${BUNDLED_DIR}"

echo "[check] rejecting absolute symlinks"
ABS_SYMLINKS="$(find "${BUNDLED_DIR}" -type l -lname '/*' -print || true)"
if [[ -n "${ABS_SYMLINKS}" ]]; then
  echo "[fail] found absolute symlinks under bundled-runtime:"
  echo "${ABS_SYMLINKS}"
  exit 1
fi

echo "[check] rejecting host python wrapper"
PY_WRAPPER="${BUNDLED_DIR}/python3"
if [[ -f "${PY_WRAPPER}" ]] && rg -n "/usr/bin/python3" "${PY_WRAPPER}" >/dev/null; then
  echo "[fail] bundled runtime python3 wrapper references /usr/bin/python3: ${PY_WRAPPER}"
  exit 1
fi

echo "[pass] bundled runtime scan ok"


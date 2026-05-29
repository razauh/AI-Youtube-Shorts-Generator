#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "[secure-install] Running locked pnpm workspace install"
pnpm install --frozen-lockfile

echo "[secure-install] Running production-focused audit"
pnpm audit --prod

echo "[secure-install] Listing installed dependency tree"
pnpm list --recursive --depth Infinity

cat <<'MSG'
[secure-install] Done.
If you need to run trusted build scripts manually, review and run only explicit commands:
  pnpm --dir app run build
  pnpm --dir app run tauri:dev
MSG

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="/home/pc/Downloads/inf/finalized/AI-Youtube-Shorts-Generator"
APP_DIR="$ROOT_DIR/app"

echo "[secure-install] Running locked install with scripts disabled"
npm --prefix "$APP_DIR" ci --ignore-scripts

echo "[secure-install] Running production-focused audit"
npm --prefix "$APP_DIR" audit --production

echo "[secure-install] Listing installed dependency tree"
npm --prefix "$APP_DIR" ls --all

cat <<'MSG'
[secure-install] Done.
If you need to run trusted build scripts manually, review and run only explicit commands:
  npm --prefix app run build
  npm --prefix app run tauri dev
MSG

#!/usr/bin/env bash
set -euo pipefail

# Builds the runtime payload consumed by Tauri bundle resources.
# Expected input layout:
#   bundled-runtime-input/<target>/
#     python3|python.exe
#     ffmpeg|ffmpeg.exe
#     yt-dlp|yt-dlp.exe
#     python_legacy/
#     site-packages/

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-}"

if [[ -z "$TARGET" ]]; then
  echo "Usage: $0 <target>"
  exit 1
fi

SRC_DIR="$ROOT_DIR/bundled-runtime-input/$TARGET"
DEST_DIR="$ROOT_DIR/app/src-tauri/bundled-runtime"

if [[ ! -d "$SRC_DIR" ]]; then
  echo "Missing runtime input directory: $SRC_DIR"
  exit 1
fi

if [[ -e "$DEST_DIR" ]]; then
  echo "Destination already exists: $DEST_DIR"
  echo "Move or remove it manually before rebuilding bundled runtime."
  exit 1
fi

mkdir -p "$DEST_DIR"
cp -R "$SRC_DIR"/. "$DEST_DIR"/

if [[ ! -f "$DEST_DIR/python_legacy/bridge_entry.py" ]]; then
  echo "bundled runtime is incomplete: missing python_legacy/bridge_entry.py"
  exit 1
fi

echo "Bundled runtime prepared at $DEST_DIR"

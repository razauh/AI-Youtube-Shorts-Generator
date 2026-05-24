#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="$ROOT_DIR/app/src-tauri/bundled-runtime"
PYTHON_BIN="${PYTHON_BIN:-$(command -v python3 || true)}"
FORCE_REPLACE="${FORCE_REPLACE:-0}"

if [[ -z "$PYTHON_BIN" ]]; then
  echo "python3 was not found on PATH. Set PYTHON_BIN=/abs/path/to/python3." >&2
  exit 1
fi

if [[ ! -x "$PYTHON_BIN" ]]; then
  echo "Python interpreter is not executable: $PYTHON_BIN" >&2
  exit 1
fi

if [[ -e "$DEST_DIR" ]]; then
  if [[ "$FORCE_REPLACE" == "1" ]]; then
    rm -rf "$DEST_DIR"
  else
    if [[ -f "$DEST_DIR/README.md" ]] && [[ "$(find "$DEST_DIR" -mindepth 1 -maxdepth 1 | wc -l)" -eq 1 ]]; then
      rm -rf "$DEST_DIR"
    else
      echo "Destination already exists and is not scaffold-only: $DEST_DIR" >&2
      echo "Set FORCE_REPLACE=1 to replace it." >&2
      exit 1
    fi
  fi
fi

mkdir -p "$DEST_DIR"
cp -R "$ROOT_DIR/python_legacy" "$DEST_DIR/python_legacy"

cat > "$DEST_DIR/python3" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
exec "${PYTHON_BIN_OVERRIDE:-/usr/bin/env python3}" "$@"
SH
chmod +x "$DEST_DIR/python3"
sed -i "s#\${PYTHON_BIN_OVERRIDE:-/usr/bin/env python3}#$PYTHON_BIN#g" "$DEST_DIR/python3"

if command -v ffmpeg >/dev/null 2>&1; then
  cp "$(command -v ffmpeg)" "$DEST_DIR/ffmpeg"
  chmod +x "$DEST_DIR/ffmpeg"
fi

if command -v yt-dlp >/dev/null 2>&1; then
  cp "$(command -v yt-dlp)" "$DEST_DIR/yt-dlp"
  chmod +x "$DEST_DIR/yt-dlp"
fi

SITE_PACKAGES="$($PYTHON_BIN - <<'PY'
import sysconfig
print(sysconfig.get_paths().get('purelib', '').strip())
PY
)"

if [[ -n "$SITE_PACKAGES" && -d "$SITE_PACKAGES" ]]; then
  ln -s "$SITE_PACKAGES" "$DEST_DIR/site-packages"
fi

cat > "$DEST_DIR/RUNTIME_INFO.txt" <<INFO
python_bin=$PYTHON_BIN
site_packages=$SITE_PACKAGES
ffmpeg=$(command -v ffmpeg || echo missing)
yt_dlp=$(command -v yt-dlp || echo missing)
INFO

"$PYTHON_BIN" - <<'PY'
mods = ["faster_whisper", "ctranslate2", "huggingface_hub", "tokenizers", "av", "numpy"]
for m in mods:
    try:
        __import__(m)
        print(f"{m}: OK")
    except Exception as e:
        print(f"{m}: FAIL {type(e).__name__}: {e}")
PY

echo "Bundled runtime bootstrapped at: $DEST_DIR"

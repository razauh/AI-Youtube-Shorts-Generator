#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-}"

if [[ -z "${TARGET}" ]]; then
  echo "Usage: $0 <linux-x86_64|macos-x86_64|macos-aarch64>" >&2
  exit 1
fi

case "${TARGET}" in
  linux-x86_64)
    PBS_PATTERN='cpython-3\.12.*x86_64-unknown-linux-gnu-install_only_stripped\.tar\.gz$'
    YTDLP_ASSET="yt-dlp"
    PYTHON_EXE="python/bin/python3"
    ;;
  macos-x86_64)
    PBS_PATTERN='cpython-3\.12.*x86_64-apple-darwin-install_only_stripped\.tar\.gz$'
    YTDLP_ASSET="yt-dlp_macos"
    PYTHON_EXE="python/bin/python3"
    ;;
  macos-aarch64)
    PBS_PATTERN='cpython-3\.12.*aarch64-apple-darwin-install_only_stripped\.tar\.gz$'
    YTDLP_ASSET="yt-dlp_macos"
    PYTHON_EXE="python/bin/python3"
    ;;
  *)
    echo "unsupported runtime target: ${TARGET}" >&2
    exit 1
    ;;
esac

INPUT_DIR="${ROOT_DIR}/bundled-runtime-input/${TARGET}"
WORK_DIR="${ROOT_DIR}/.tmp-runtime-build/${TARGET}"

rm -rf "${INPUT_DIR}" "${WORK_DIR}"
mkdir -p "${INPUT_DIR}" "${WORK_DIR}"

PBS_URL="$(
  PBS_PATTERN="${PBS_PATTERN}" node --input-type=module <<'NODE'
const pattern = new RegExp(process.env.PBS_PATTERN);
const res = await fetch('https://api.github.com/repos/astral-sh/python-build-standalone/releases/latest', {
  headers: { 'user-agent': 'ai-youtube-shorts-generator-runtime-builder' },
});
if (!res.ok) throw new Error(`python-build-standalone release lookup failed: ${res.status}`);
const release = await res.json();
const asset = release.assets.find((item) => pattern.test(item.browser_download_url));
if (!asset) throw new Error(`no python-build-standalone asset matched ${pattern}`);
console.log(asset.browser_download_url);
NODE
)"

echo "[info] python-build-standalone: ${PBS_URL}"
curl -fL "${PBS_URL}" -o "${WORK_DIR}/python-standalone.tar.gz"
tar -xzf "${WORK_DIR}/python-standalone.tar.gz" -C "${WORK_DIR}"
cp -R "${WORK_DIR}/python" "${INPUT_DIR}/python"

cat > "${INPUT_DIR}/python3" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${DIR}/python/bin/python3" "$@"
SH
chmod +x "${INPUT_DIR}/python3"

mkdir -p "${INPUT_DIR}/site-packages"
"${INPUT_DIR}/${PYTHON_EXE}" -m ensurepip
"${INPUT_DIR}/${PYTHON_EXE}" -m pip install --upgrade pip
"${INPUT_DIR}/${PYTHON_EXE}" -m pip install --target "${INPUT_DIR}/site-packages" -r "${ROOT_DIR}/requirements.txt" -r "${ROOT_DIR}/requirements-local.txt"
"${INPUT_DIR}/${PYTHON_EXE}" -m pip install --target "${INPUT_DIR}/site-packages" imageio-ffmpeg

cp -R "${ROOT_DIR}/python_legacy" "${INPUT_DIR}/python_legacy"

YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/${YTDLP_ASSET}"
echo "[info] yt-dlp: ${YTDLP_URL}"
curl -fL "${YTDLP_URL}" -o "${INPUT_DIR}/yt-dlp"
chmod +x "${INPUT_DIR}/yt-dlp"

FFMPEG_SOURCE="$(
  PYTHONPATH="${INPUT_DIR}/site-packages" "${INPUT_DIR}/python3" - <<'PY'
import imageio_ffmpeg
print(imageio_ffmpeg.get_ffmpeg_exe())
PY
)"
cp "${FFMPEG_SOURCE}" "${INPUT_DIR}/ffmpeg"
chmod +x "${INPUT_DIR}/ffmpeg"

cat > "${INPUT_DIR}/RUNTIME_INFO.txt" <<INFO
target=${TARGET}
python_bin=${PYTHON_EXE}
site_packages=site-packages
ffmpeg=ffmpeg
ffmpeg_source=${FFMPEG_SOURCE}
yt_dlp=yt-dlp
INFO

PYTHONPATH="${INPUT_DIR}/site-packages" "${INPUT_DIR}/python3" - <<'PY'
mods = ["requests", "dotenv", "yt_dlp", "faster_whisper", "openai", "cv2", "imageio_ffmpeg"]
for mod in mods:
    __import__(mod)
    print(f"{mod}: OK")
PY

"${INPUT_DIR}/yt-dlp" --version
"${INPUT_DIR}/ffmpeg" -version | sed -n '1p'

echo "[info] bundled runtime input prepared: ${INPUT_DIR}"

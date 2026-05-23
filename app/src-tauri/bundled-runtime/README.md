Bundled runtime payload for packaged builds.

This directory is populated by `scripts/prepare-bundled-runtime.sh` before creating installer artifacts.
Expected contents include:
- Python runtime binary
- `python_legacy/bridge_entry.py` and related bridge files
- Python package payload for local mode (including `faster_whisper`)
- `ffmpeg` and `yt-dlp` binaries for the target platform

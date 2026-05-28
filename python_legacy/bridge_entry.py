#!/usr/bin/env python3
import json
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import time
import traceback

CONTRACT_VERSION = "1"
DEPENDENCY_INSTALL_TIMEOUT_SECONDS = 900
MODULE_REQUIREMENTS = {
    "requests": ("requirements.txt", "requests"),
    "dotenv": ("requirements.txt", "python-dotenv"),
    "yt_dlp": ("requirements-local.txt", "yt-dlp"),
    "faster_whisper": ("requirements-local.txt", "faster-whisper"),
    "openai": ("requirements-local.txt", "openai"),
    "cv2": ("requirements-local.txt", "opencv-python"),
}


def _read_request():
    raw = sys.stdin.read()
    if not raw:
        raise RuntimeError("empty bridge stdin")
    return json.loads(raw)


def _write(payload):
    sys.stdout.write(json.dumps(payload, ensure_ascii=False))
    sys.stdout.flush()


def _ok(result):
    _write({"version": CONTRACT_VERSION, "ok": True, "result": result, "error": None})


def _err(message, code="PYTHON_ERROR", details=None):
    _write(
        {
            "version": CONTRACT_VERSION,
            "ok": False,
            "result": {},
            "error": {"code": code, "message": message, "details": details or {}},
        }
    )


def _progress(phase, progress, message):
    sys.stderr.write(
        json.dumps(
            {
                "event": "local_model_setup_progress",
                "phase": phase,
                "progress": progress,
                "message": message,
            },
            ensure_ascii=False,
        )
        + "\n"
    )
    sys.stderr.flush()


def _runtime_details():
    return {
        "python_executable": sys.executable,
        "python_version": sys.version,
        "cwd": os.getcwd(),
        "sys_path": list(sys.path),
    }


def _cause_details(exc):
    cause = exc.__cause__ or exc.__context__
    if cause is None:
        return {}
    return {
        "cause_type": cause.__class__.__name__,
        "cause_message": str(cause),
    }


def _repo_root():
    return Path(__file__).resolve().parent.parent


def _requirement_spec(import_name):
    item = MODULE_REQUIREMENTS.get(import_name)
    if item is None:
        raise RuntimeError(f"missing module is not allowlisted for auto-install: {import_name}")

    filename, distribution = item
    req_path = _repo_root() / filename
    if not req_path.exists():
        raise RuntimeError(f"dependency requirements file is missing: {filename}")

    distribution_key = distribution.lower().replace("_", "-")
    for raw_line in req_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        normalized = line.lower().replace("_", "-")
        if normalized == distribution_key or normalized.startswith(
            (
                f"{distribution_key}<",
                f"{distribution_key}>",
                f"{distribution_key}=",
                f"{distribution_key}~",
                f"{distribution_key}!",
                f"{distribution_key}[",
                f"{distribution_key};",
            )
        ):
            return line

    raise RuntimeError(f"dependency requirement is not declared: {distribution}")


def _install_requirement(import_name):
    spec = _requirement_spec(import_name)
    _progress(
        "installing_dependency",
        0.45,
        f"Installing Python dependency: {spec}",
    )
    cmd = [
        sys.executable,
        "-m",
        "pip",
        "install",
        "--disable-pip-version-check",
        spec,
    ]
    proc = subprocess.run(
        cmd,
        cwd=str(_repo_root()),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        timeout=DEPENDENCY_INSTALL_TIMEOUT_SECONDS,
        shell=False,
    )
    if proc.returncode != 0:
        stderr = (proc.stderr or proc.stdout or "").strip()[-2000:]
        raise RuntimeError(f"failed to install Python dependency {spec}: {stderr}")
    _progress(
        "installing_dependency",
        0.55,
        f"Installed Python dependency: {spec}",
    )


def _ensure_python_modules(import_names):
    installed = []
    for import_name in import_names:
        if import_name not in MODULE_REQUIREMENTS:
            raise RuntimeError(f"module is not allowlisted for auto-install: {import_name}")
        if importlib.util.find_spec(import_name) is not None:
            continue
        _install_requirement(import_name)
        installed.append(import_name)
    return installed


def _dependency_repair_details(installed):
    if not installed:
        return {}
    return {"installed_modules": installed}


def main() -> int:
    req = _read_request()
    if req.get("version") != CONTRACT_VERSION:
        _err(f"unsupported contract version: {req.get('version')}", code="CONTRACT_VERSION")
        return 0

    action = req.get("action")
    payload = req.get("payload") or {}

    # Test hooks for Rust bridge supervision tests.
    if action == "emit_malformed":
        sys.stdout.write("not-json")
        sys.stdout.flush()
        return 0
    if action == "sleep":
        time.sleep(float(payload.get("seconds", 5)))
        _ok({"mode": "local", "source_video_url": "sleep", "transcript": {"duration": 0.0, "segments": []}, "highlights": [], "shorts": []})
        return 0
    if action == "exit_nonzero":
        sys.stderr.write("exit_nonzero requested\n")
        sys.stderr.flush()
        return 17
    if action == "stderr_then_fail":
        sys.stderr.write("bridge stderr test\n")
        sys.stderr.flush()
        _err("forced python failure", details={"stage": "test"})
        return 0

    if action != "run_local":
        if action == "prefetch_local_model":
            installed_modules = []
            try:
                installed_modules = _ensure_python_modules(["faster_whisper"])
                _progress(
                    "downloading_model",
                    0.68,
                    f"Downloading Whisper model: {payload.get('model', '') or 'default'}",
                )
                from shorts_generator.local.transcriber import prefetch_local_model
                _ok(
                    prefetch_local_model(
                        model_name=payload.get("model", ""),
                        device=payload.get("device", "auto"),
                        cache_dir=payload.get("cache_dir"),
                    )
                )
            except Exception as exc:
                _err(
                    str(exc),
                    details={
                        "stage": "prefetch_local_model",
                        "exception_type": exc.__class__.__name__,
                        "traceback": traceback.format_exc(),
                        "runtime": _runtime_details(),
                        **_dependency_repair_details(installed_modules),
                        **_cause_details(exc),
                    },
                )
            return 0
        _err(f"unknown action: {action}", code="BAD_ACTION")
        return 0

    # Fixture mode for deterministic parity-style checks in Rust tests.
    if payload.get("youtube_url") == "https://youtube.com/watch?v=abc":
        _ok(
            {
                "mode": "local",
                "source_video_url": "/tmp/source_abc.mp4",
                "transcript": {
                    "duration": 12.0,
                    "segments": [
                        {"start": 0.0, "end": 2.5, "text": "hello"},
                        {"start": 2.5, "end": 6.0, "text": "world"},
                    ],
                },
                "highlights": [
                    {
                        "title": "hello",
                        "start_time": 0.0,
                        "end_time": 6.0,
                        "score": 90,
                        "hook_sentence": "hello",
                        "virality_reason": "test",
                    }
                ],
                "shorts": [
                    {
                        "title": "hello",
                        "start_time": 0.0,
                        "end_time": 6.0,
                        "score": 90,
                        "hook_sentence": "hello",
                        "virality_reason": "test",
                        "clip_url": "/tmp/short_01.mp4",
                    }
                ],
            }
        )
        return 0

    try:
        installed_modules = _ensure_python_modules(
            ["requests", "dotenv", "yt_dlp", "faster_whisper", "openai", "cv2"]
        )
        from shorts_generator import generate_shorts
        result = generate_shorts(
            youtube_url=payload.get("youtube_url"),
            num_clips=int(payload.get("num_clips", 3)),
            aspect_ratio=payload.get("aspect_ratio", "9:16"),
            download_format=payload.get("download_format", "720"),
            language=payload.get("language"),
            mode="local",
        )
        _ok(result)
        return 0
    except Exception as exc:  # parity: structured per-run failure
        _err(
            str(exc),
            details={
                "stage": "run_local",
                "exception_type": exc.__class__.__name__,
                "traceback": traceback.format_exc(),
                "runtime": _runtime_details(),
                **_dependency_repair_details(locals().get("installed_modules", [])),
                **_cause_details(exc),
            },
        )
        return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        _err(
            str(exc),
            code="BRIDGE_CRASH",
            details={
                "stage": "bridge_crash",
                "exception_type": exc.__class__.__name__,
                "traceback": traceback.format_exc(),
                "runtime": _runtime_details(),
                **_cause_details(exc),
            },
        )
        raise SystemExit(0)

#!/usr/bin/env python3
import json
import sys
import time

from shorts_generator import generate_shorts
from shorts_generator.local.transcriber import prefetch_local_model

CONTRACT_VERSION = "1"


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
            try:
                _ok(
                    prefetch_local_model(
                        model_name=payload.get("model", ""),
                        device=payload.get("device", "auto"),
                        cache_dir=payload.get("cache_dir"),
                    )
                )
            except Exception as exc:
                _err(str(exc), details={"stage": "prefetch_local_model"})
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
        _err(str(exc), details={"stage": "run_local"})
        return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        _err(str(exc), code="BRIDGE_CRASH")
        raise SystemExit(0)

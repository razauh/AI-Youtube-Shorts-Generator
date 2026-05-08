import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from python_legacy.shorts_generator import pipeline

ROOT = Path(__file__).resolve().parents[1]
GOLDEN_DIR = ROOT / "fixtures" / "golden" / "v1"


def _base_transcript() -> Dict[str, Any]:
    return {
        "duration": 140.0,
        "segments": [
            {"start": 0.0, "end": 4.0, "text": "Most creators miss this."},
            {"start": 4.0, "end": 9.2, "text": "One change doubled retention."},
        ],
    }


def _base_highlights() -> List[Dict[str, Any]]:
    return [
        {
            "title": "Retention Secret",
            "start_time": 0.0,
            "end_time": 54.4,
            "score": 92,
            "hook_sentence": "Most creators miss this.",
            "virality_reason": "Strong hook plus concrete outcome creates immediate curiosity.",
        },
        {
            "title": "One Change",
            "start_time": 60.0,
            "end_time": 110.0,
            "score": 81,
            "hook_sentence": "One change doubled retention.",
            "virality_reason": "Clear payoff and actionable tip.",
        },
    ]


def _run_api_success() -> Dict[str, Any]:
    highlights = _base_highlights()
    clips = [{**highlights[0], "clip_url": "https://cdn.example.com/short_01.mp4"}]

    with (
        mock.patch("python_legacy.shorts_generator.pipeline.download_youtube", return_value="https://cdn.example.com/video.mp4"),
        mock.patch("python_legacy.shorts_generator.pipeline.transcribe", return_value=_base_transcript()),
        mock.patch("python_legacy.shorts_generator.pipeline.get_highlights", return_value={"highlights": highlights}),
        mock.patch("python_legacy.shorts_generator.pipeline.crop_highlights", return_value=clips),
    ):
        return pipeline.generate_shorts("https://youtube.com/watch?v=abc", num_clips=1, mode="api")


def _run_no_segments() -> Dict[str, Any]:
    with (
        mock.patch("python_legacy.shorts_generator.pipeline.download_youtube", return_value="https://cdn.example.com/video.mp4"),
        mock.patch("python_legacy.shorts_generator.pipeline.transcribe", return_value={"duration": 0.0, "segments": []}),
    ):
        try:
            pipeline.generate_shorts("https://youtube.com/watch?v=abc", num_clips=1, mode="api")
        except Exception as e:
            return {
                "mode": "api",
                "source_video_url": "https://cdn.example.com/video.mp4",
                "error": str(e),
                "details": {"stage": "transcribe", "retryable": False},
            }
    raise RuntimeError("expected no-segments failure")


def _run_clip_failure() -> Dict[str, Any]:
    highlights = _base_highlights()[:1]
    clip_results = [{**highlights[0], "clip_url": None, "error": "autocrop upstream timeout"}]
    with (
        mock.patch("python_legacy.shorts_generator.pipeline.download_youtube", return_value="https://cdn.example.com/video.mp4"),
        mock.patch("python_legacy.shorts_generator.pipeline.transcribe", return_value=_base_transcript()),
        mock.patch("python_legacy.shorts_generator.pipeline.get_highlights", return_value={"highlights": highlights}),
        mock.patch("python_legacy.shorts_generator.pipeline.crop_highlights", return_value=clip_results),
    ):
        return pipeline.generate_shorts("https://youtube.com/watch?v=abc", num_clips=1, mode="api")


def _run_local_live() -> Dict[str, Any]:
    return pipeline.generate_shorts(
        youtube_url=os.environ["BASELINE_LOCAL_URL"],
        num_clips=int(os.environ.get("BASELINE_LOCAL_NUM_CLIPS", "1")),
        aspect_ratio=os.environ.get("BASELINE_LOCAL_ASPECT_RATIO", "9:16"),
        download_format=os.environ.get("BASELINE_LOCAL_FORMAT", "360"),
        language=os.environ.get("BASELINE_LOCAL_LANGUAGE") or None,
        mode="local",
    )


def _write_json(path: Path, payload: Dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate Python baseline golden fixtures")
    ap.add_argument("--out-dir", default=str(GOLDEN_DIR))
    ap.add_argument("--include-local-live", action="store_true")
    args = ap.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    api_success = _run_api_success()
    no_segments = _run_no_segments()
    clip_failure = _run_clip_failure()

    _write_json(out_dir / "api_success.json", api_success)
    _write_json(out_dir / "no_segments.json", no_segments)
    _write_json(out_dir / "clip_failure.json", clip_failure)

    scenarios = [
        {"id": "api_success", "file": "api_success.json", "kind": "mocked"},
        {"id": "no_segments", "file": "no_segments.json", "kind": "mocked"},
        {"id": "clip_failure", "file": "clip_failure.json", "kind": "mocked"},
    ]

    if args.include_local_live and os.environ.get("BASELINE_LOCAL_URL"):
        local_live = _run_local_live()
        _write_json(out_dir / "local_success_live.json", local_live)
        scenarios.append({"id": "local_success_live", "file": "local_success_live.json", "kind": "live"})

    manifest = {
        "version": "v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "generator": "tests/characterization/run_python_baseline.py",
        "deterministic_fields": [
            "mode",
            "source_video_url",
            "transcript.duration",
            "transcript.segments[*].start",
            "transcript.segments[*].end",
            "highlights[*].start_time",
            "highlights[*].end_time",
            "highlights[*].score",
            "shorts[*].clip_url",
            "shorts[*].error",
        ],
        "nondeterministic_fields": [
            "highlights[*].title",
            "highlights[*].hook_sentence",
            "highlights[*].virality_reason",
            "shorts[*].title",
            "shorts[*].hook_sentence",
            "shorts[*].virality_reason",
        ],
        "scenarios": scenarios,
    }
    _write_json(out_dir / "manifest.json", manifest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

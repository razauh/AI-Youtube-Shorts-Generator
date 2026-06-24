import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from compare_outputs import compare_outputs

GOLDEN_DIR = Path(__file__).resolve().parents[1] / "fixtures" / "golden" / "v1"
REQUIRED = {
    "api_success.json",
    "no_segments.json",
    "clip_failure.json",
    "manifest.json",
    "REFRESH_POLICY.md",
}


def _load(name: str):
    with (GOLDEN_DIR / name).open("r", encoding="utf-8") as f:
        return json.load(f)


def test_required_golden_files_exist():
    missing = [name for name in sorted(REQUIRED) if not (GOLDEN_DIR / name).exists()]
    assert not missing, f"missing golden files: {missing}"


def test_manifest_lists_required_scenarios():
    manifest = _load("manifest.json")
    scenario_ids = {s["id"] for s in manifest.get("scenarios", [])}
    assert {"api_success", "no_segments", "clip_failure"}.issubset(scenario_ids)


def test_strict_comparator_detects_any_drift():
    baseline = {
        "mode": "api",
        "source_video_url": "https://cdn.example.com/src.mp4",
        "transcript": {"duration": 10.0, "segments": [{"start": 0.0, "end": 1.0, "text": "x"}]},
        "highlights": [{"title": "A", "start_time": 0.0, "end_time": 5.0, "score": 80, "hook_sentence": "h", "virality_reason": "r"}],
        "shorts": [{"title": "A", "start_time": 0.0, "end_time": 5.0, "score": 80, "hook_sentence": "h", "virality_reason": "r", "clip_url": "url"}],
    }
    changed = dict(baseline)
    changed["highlights"] = [dict(baseline["highlights"][0], hook_sentence="different")]

    result = compare_outputs(baseline, changed, mode="strict")
    assert not result["match"]
    assert result["diffs"]


def test_tolerant_comparator_ignores_llm_fields():
    baseline = {
        "mode": "api",
        "source_video_url": "https://cdn.example.com/src.mp4",
        "transcript": {"duration": 10.0, "segments": [{"start": 0.0, "end": 1.0, "text": "x"}]},
        "highlights": [{"title": "A", "start_time": 0.0, "end_time": 5.0, "score": 80, "hook_sentence": "h", "virality_reason": "r"}],
        "shorts": [{"title": "A", "start_time": 0.0, "end_time": 5.0, "score": 80, "hook_sentence": "h", "virality_reason": "r", "clip_url": "url"}],
    }
    changed = {
        **baseline,
        "highlights": [{"title": "B", "start_time": 0.0, "end_time": 5.0, "score": 80, "hook_sentence": "new hook", "virality_reason": "new reason"}],
        "shorts": [{"title": "B", "start_time": 0.0, "end_time": 5.0, "score": 80, "hook_sentence": "new hook", "virality_reason": "new reason", "clip_url": "url"}],
    }

    result = compare_outputs(baseline, changed, mode="tolerant")
    assert result["match"], result["diffs"]

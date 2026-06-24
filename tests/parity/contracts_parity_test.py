import json
from pathlib import Path

FIXTURE_DIR = Path(__file__).resolve().parents[1] / "fixtures" / "contracts"

REQUIRED_ROOT_KEYS = {"mode", "source_video_url", "transcript", "highlights", "shorts"}
REQUIRED_TRANSCRIPT_KEYS = {"duration", "segments"}
REQUIRED_SEGMENT_KEYS = {"start", "end", "text"}
REQUIRED_HIGHLIGHT_KEYS = {
    "title",
    "start_time",
    "end_time",
    "score",
    "hook_sentence",
    "virality_reason",
}


def _load(name: str):
    with (FIXTURE_DIR / name).open("r", encoding="utf-8") as f:
        return json.load(f)


def _assert_success_shape(doc):
    missing = REQUIRED_ROOT_KEYS - set(doc.keys())
    assert not missing, f"missing root keys: {sorted(missing)}"

    transcript = doc["transcript"]
    missing_transcript = REQUIRED_TRANSCRIPT_KEYS - set(transcript.keys())
    assert not missing_transcript, f"missing transcript keys: {sorted(missing_transcript)}"

    assert isinstance(transcript["segments"], list), "transcript.segments must be list"
    for seg in transcript["segments"]:
        missing_segment = REQUIRED_SEGMENT_KEYS - set(seg.keys())
        assert not missing_segment, f"missing segment keys: {sorted(missing_segment)}"

    assert isinstance(doc["highlights"], list), "highlights must be list"
    for h in doc["highlights"]:
        missing_highlight = REQUIRED_HIGHLIGHT_KEYS - set(h.keys())
        assert not missing_highlight, f"missing highlight keys: {sorted(missing_highlight)}"

    assert isinstance(doc["shorts"], list), "shorts must be list"
    for s in doc["shorts"]:
        missing_short = REQUIRED_HIGHLIGHT_KEYS - set(s.keys())
        assert not missing_short, f"missing short keys: {sorted(missing_short)}"
        assert "clip_url" in s, "short.clip_url key must exist"


def test_success_fixture_required_keys_match_python_contracts():
    _assert_success_shape(_load("success.json"))


def test_partial_failure_nullable_clip_url_and_error_behavior():
    partial = _load("partial_failure.json")
    _assert_success_shape(partial)

    assert any(s.get("clip_url") is None for s in partial["shorts"]), "need at least one null clip_url"
    assert any(isinstance(s.get("error"), str) for s in partial["shorts"]), "need at least one error string"
    assert any(isinstance(s.get("clip_url"), str) for s in partial["shorts"]), "need at least one successful clip_url"


def test_hard_failure_envelope_shape():
    hard = _load("hard_failure.json")
    assert "error" in hard and isinstance(hard["error"], str) and hard["error"], "hard failure must include error"
    assert "mode" in hard
    assert "source_video_url" in hard

if __name__ == "__main__":
    test_success_fixture_required_keys_match_python_contracts()
    test_partial_failure_nullable_clip_url_and_error_behavior()
    test_hard_failure_envelope_shape()
    print("contracts parity: ok")

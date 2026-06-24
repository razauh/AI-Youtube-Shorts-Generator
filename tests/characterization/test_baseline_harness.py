from pathlib import Path

GOLDEN_DIR = Path(__file__).resolve().parents[1] / "fixtures" / "golden" / "v1"


def test_refresh_policy_documented():
    policy = GOLDEN_DIR / "REFRESH_POLICY.md"
    assert policy.exists(), "fixture refresh policy missing"
    text = policy.read_text(encoding="utf-8")
    assert "How to refresh" in text
    assert "live" in text.lower()


def test_manifest_contains_deterministic_vs_nondeterministic_fields():
    manifest = GOLDEN_DIR / "manifest.json"
    assert manifest.exists(), "golden manifest missing"
    text = manifest.read_text(encoding="utf-8")
    assert "deterministic_fields" in text
    assert "nondeterministic_fields" in text

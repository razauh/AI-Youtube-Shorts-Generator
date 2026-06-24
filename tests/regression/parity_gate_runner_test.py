import json
from pathlib import Path

from tests.regression.parity_gate_runner import run_gate


FIX = Path("tests/fixtures/contracts")


def test_gate_pass_on_identical_fixture(tmp_path: Path) -> None:
    expected = FIX / "success.json"
    actual = tmp_path / "actual.json"
    actual.write_text(expected.read_text(encoding="utf-8"), encoding="utf-8")

    out = run_gate(expected, actual, mode="strict", max_diffs=0)
    assert out["gate_pass"] is True
    assert out["diff_count"] == 0


def test_gate_fail_on_drift_when_threshold_zero(tmp_path: Path) -> None:
    expected = FIX / "success.json"
    actual = tmp_path / "actual.json"
    data = json.loads(expected.read_text(encoding="utf-8"))
    data["mode"] = "api-drift"
    actual.write_text(json.dumps(data), encoding="utf-8")

    out = run_gate(expected, actual, mode="strict", max_diffs=0)
    assert out["gate_pass"] is False
    assert out["diff_count"] > 0

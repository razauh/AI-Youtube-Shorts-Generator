import argparse
import json
from pathlib import Path
from typing import Any, Dict

from tests.parity.compare_outputs import compare_outputs


def run_gate(expected_path: Path, actual_path: Path, mode: str, max_diffs: int) -> Dict[str, Any]:
    expected = json.loads(expected_path.read_text(encoding="utf-8"))
    actual = json.loads(actual_path.read_text(encoding="utf-8"))

    result = compare_outputs(expected, actual, mode=mode)
    diff_count = len(result["diffs"])
    gate_pass = result["match"] or diff_count <= max_diffs

    return {
        "gate_pass": gate_pass,
        "mode": mode,
        "max_diffs": max_diffs,
        "diff_count": diff_count,
        "diffs": result["diffs"],
    }


def _main() -> int:
    ap = argparse.ArgumentParser(description="Regression parity gate")
    ap.add_argument("expected")
    ap.add_argument("actual")
    ap.add_argument("--mode", choices=["strict", "tolerant"], default="strict")
    ap.add_argument("--max-diffs", type=int, default=0)
    args = ap.parse_args()

    out = run_gate(Path(args.expected), Path(args.actual), args.mode, args.max_diffs)
    print(json.dumps(out, indent=2))
    return 0 if out["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(_main())

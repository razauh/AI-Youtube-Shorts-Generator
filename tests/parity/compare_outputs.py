import argparse
import json
from pathlib import Path
from typing import Any, Dict, List, Set

TOLERANT_IGNORE_FIELDS: Set[str] = {
    "title",
    "hook_sentence",
    "virality_reason",
}


def _norm(value: Any, mode: str) -> Any:
    if isinstance(value, dict):
        out = {}
        for k in sorted(value.keys()):
            if mode == "tolerant" and k in TOLERANT_IGNORE_FIELDS:
                continue
            out[k] = _norm(value[k], mode)
        return out
    if isinstance(value, list):
        return [_norm(v, mode) for v in value]
    if isinstance(value, float):
        return round(value, 3)
    return value


def _collect_diffs(left: Any, right: Any, path: str, out: List[str]) -> None:
    if type(left) is not type(right):
        out.append(f"{path}: type {type(left).__name__} != {type(right).__name__}")
        return
    if isinstance(left, dict):
        left_keys = set(left.keys())
        right_keys = set(right.keys())
        for k in sorted(left_keys - right_keys):
            out.append(f"{path}.{k}: missing on right")
        for k in sorted(right_keys - left_keys):
            out.append(f"{path}.{k}: missing on left")
        for k in sorted(left_keys & right_keys):
            _collect_diffs(left[k], right[k], f"{path}.{k}", out)
        return
    if isinstance(left, list):
        if len(left) != len(right):
            out.append(f"{path}: len {len(left)} != {len(right)}")
            return
        for i, (lv, rv) in enumerate(zip(left, right)):
            _collect_diffs(lv, rv, f"{path}[{i}]", out)
        return
    if left != right:
        out.append(f"{path}: {left!r} != {right!r}")


def compare_outputs(expected: Dict[str, Any], actual: Dict[str, Any], mode: str = "strict") -> Dict[str, Any]:
    if mode not in {"strict", "tolerant"}:
        raise ValueError("mode must be strict|tolerant")
    left = _norm(expected, mode)
    right = _norm(actual, mode)
    diffs: List[str] = []
    _collect_diffs(left, right, "$", diffs)
    return {"match": len(diffs) == 0, "mode": mode, "diffs": diffs}


def _main() -> int:
    ap = argparse.ArgumentParser(description="Compare two pipeline outputs")
    ap.add_argument("expected")
    ap.add_argument("actual")
    ap.add_argument("--mode", choices=["strict", "tolerant"], default="strict")
    args = ap.parse_args()

    expected = json.loads(Path(args.expected).read_text(encoding="utf-8"))
    actual = json.loads(Path(args.actual).read_text(encoding="utf-8"))

    result = compare_outputs(expected, actual, mode=args.mode)
    print(json.dumps(result, indent=2))
    return 0 if result["match"] else 1


if __name__ == "__main__":
    raise SystemExit(_main())

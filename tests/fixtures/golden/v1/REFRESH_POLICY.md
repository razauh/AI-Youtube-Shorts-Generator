# Golden Fixture Refresh Policy (v1)

## How to refresh

Run:

```bash
. .venv/bin/activate
python tests/characterization/run_python_baseline.py
```

## Rules

- `api_success`, `no_segments`, `clip_failure` are mocked and deterministic for structure/parity tests.
- Live fixture refreshes are intentionally out of scope for this mocked golden set.
- Use `tests/parity/compare_outputs.py --mode strict` for deterministic fixture checks.
- Use `tests/parity/compare_outputs.py --mode tolerant` when LLM wording drift expected.

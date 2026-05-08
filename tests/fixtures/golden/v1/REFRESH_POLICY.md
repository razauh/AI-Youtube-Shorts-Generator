# Golden Fixture Refresh Policy (v1)

## How to refresh

Run:

```bash
. .venv/bin/activate
python tests/characterization/run_python_baseline.py
```

Optional live local smoke (requires local deps + ffmpeg + yt-dlp + OpenAI key):

```bash
BASELINE_LOCAL_URL='https://www.youtube.com/watch?v=<id>' \
python tests/characterization/run_python_baseline.py --include-local-live
```

## Rules

- `api_success`, `no_segments`, `clip_failure` are mocked and deterministic for structure/parity tests.
- `local_success_live` is tagged live smoke only; do not gate CI on exact values.
- Use `tests/parity/compare_outputs.py --mode strict` for deterministic fixture checks.
- Use `tests/parity/compare_outputs.py --mode tolerant` when LLM wording drift expected.

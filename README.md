# AI YouTube Shorts Generator

Desktop app and Python CLI for generating short-form clips from YouTube URLs through API-based processing.

The app is built for creators, agencies, and developers who want ranked short clips, transcript metadata, and clip URLs without subscription-style minute caps. The current application workflow uses MuAPI for video download, transcription, highlight selection, autocrop, and rendered clip delivery.

## Features

- API-based generation through MuAPI.
- License-gated Tauri desktop app with Svelte UI.
- MuAPI and OpenAI key profile management.
- YouTube URL input with configurable clip count, aspect ratio, resolution, and language.
- Ranked highlights with score, title, hook sentence, and virality reason.
- Hosted clip URLs with Open Clip and Copy Link actions.
- JSON export for downstream automation.
- Settings pages for API providers, device reset, diagnostics, and policies.

## Paid Desktop Customer Setup

1. Activate the Gumroad license in the desktop app.
2. Open Settings -> Configuration -> API Providers.
3. Add and activate a MuAPI profile.
4. Paste a YouTube URL in the generator and run API generation.
5. Use Open Clip, Copy Link, or JSON export for completed clips.

Refund, support, and policy information is available under Settings -> Policies and through the Gumroad purchase/support channel.

## Developer CLI Setup

### Prerequisites

- Python 3.10+
- MuAPI API key
- Existing repository dependencies installed by the approved project setup process

### Environment

Create a `.env` file in the project root:

```bash
MUAPI_API_KEY=your_muapi_key_here
OPENAI_API_KEY=your_openai_key_here
OPENAI_MODEL=gpt-4o-mini
```

`OPENAI_API_KEY` is only needed for flows that explicitly use OpenAI-backed text ranking or configuration checks.

## Usage

### Single Video

```bash
python main.py "https://www.youtube.com/watch?v=VIDEO_ID"
```

### With Options

```bash
python main.py "https://www.youtube.com/watch?v=VIDEO_ID" \
    --mode api \
    --num-clips 5 \
    --aspect-ratio 9:16 \
    --output-json result.json
```

### Python API

```python
from shorts_generator import generate_shorts

result = generate_shorts(
    "https://www.youtube.com/watch?v=VIDEO_ID",
    mode="api",
    num_clips=5,
    aspect_ratio="9:16",
)

for short in result["shorts"]:
    print(short["score"], short["title"], short["clip_url"])
```

### Batch Processing

Create a `urls.txt` file with one URL per line, then:

```bash
xargs -a urls.txt -I{} python main.py "{}"
```

### CLI Flags

| Flag | Default | Notes |
|------|---------|-------|
| `--mode` | `api` | API processing mode |
| `--num-clips` | `3` | How many shorts to render |
| `--aspect-ratio` | `9:16` | Any ratio; `9:16` for TikTok/Reels/Shorts, `1:1` for square |
| `--format` | `720` | Source download resolution: `360` / `480` / `720` / `1080` |
| `--language` | auto | Force transcription language code, such as `en` |
| `--output-json` | none | Dump the full result to a file |

## How It Works

1. Download: MuAPI fetches the source video from YouTube.
2. Transcribe: MuAPI produces a timestamped transcript.
3. Detect content type: the provider classifies the source so highlight prompts can be tuned.
4. Long-video chunking: long videos are split into overlapping transcript windows.
5. Highlight ranking: candidates are scored for hooks, emotional peaks, conflict, quotable lines, story peaks, and practical value.
6. Dedupe: overlapping candidates are collapsed by score.
7. Top-N selection: the requested number of clips is selected.
8. Autocrop and render: MuAPI returns hosted clip URLs for the final shorts.

## Output

`--output-json result.json` produces:

```json
{
  "mode": "api",
  "source_video_url": "https://www.youtube.com/watch?v=VIDEO_ID",
  "transcript": { "duration": 1873.4, "segments": [] },
  "highlights": [],
  "shorts": [
    {
      "title": "Example Short",
      "start_time": 124.3,
      "end_time": 187.6,
      "score": 92,
      "hook_sentence": "Example hook",
      "virality_reason": "Example reason",
      "clip_url": "https://example.test/short_1.mp4"
    }
  ]
}
```

## Project Layout

```text
app/                         Tauri desktop application
app/src/                     Svelte frontend
app/src-tauri/               Rust backend and Tauri command handlers
python_legacy/               Python CLI and API-mode pipeline
tests/                       Python characterization, parity, and regression tests
docs/                        Project documentation
worker/                      Licensing Cloudflare Worker
```

## Validation

Agents must not run repository validation commands directly. After code changes, run the task-specific validation script manually from the repository root:

```bash
bash .scripts/run-remove-local-processing-validation.sh
```

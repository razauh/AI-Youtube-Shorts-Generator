"""Local transcription via faster-whisper.

Reads a local media file and returns the same shape the highlight generator
expects: {duration, segments[start, end, text]}.
"""
import os
from pathlib import Path
from typing import Dict, Optional

from ..config import LOCAL_WHISPER_DEVICE, LOCAL_WHISPER_MODEL


def _resolve_device() -> str:
    if LOCAL_WHISPER_DEVICE != "auto":
        return LOCAL_WHISPER_DEVICE
    try:
        import torch  # type: ignore
        return "cuda" if torch.cuda.is_available() else "cpu"
    except ImportError:
        return "cpu"


def _model_cache_dir() -> Optional[str]:
    raw = os.getenv("LOCAL_MODEL_CACHE_DIR", "").strip()
    if not raw:
        return None
    path = Path(raw).expanduser()
    path.mkdir(parents=True, exist_ok=True)
    return str(path)


def prefetch_local_model(model_name: str, device: str = "auto", cache_dir: Optional[str] = None) -> Dict:
    """Download/cache a faster-whisper model before a transcription run."""
    try:
        from faster_whisper import WhisperModel  # type: ignore
    except ImportError as e:
        raise RuntimeError(
            "faster-whisper is required for local model downloads. Install it with:\n"
            "    pip install -r requirements-local.txt"
        ) from e

    model_name = (model_name or "").strip()
    if not model_name:
        raise RuntimeError("model is required")

    resolved_device = device if device in {"cpu", "cuda"} else _resolve_device()
    compute_type = "float16" if resolved_device == "cuda" else "int8"
    download_root = cache_dir or _model_cache_dir()
    print(f"[transcribe/local] prefetch model={model_name} device={resolved_device}", flush=True)
    WhisperModel(model_name, device=resolved_device, compute_type=compute_type, download_root=download_root)
    return {"model": model_name, "device": resolved_device, "cached": True}


def transcribe_local(media_path: str, language: Optional[str] = None) -> Dict:
    """Run faster-whisper on a local file path."""
    try:
        from faster_whisper import WhisperModel  # type: ignore
    except ImportError as e:
        raise RuntimeError(
            "faster-whisper is required for --mode local. Install it with:\n"
            "    pip install -r requirements-local.txt"
        ) from e

    device = _resolve_device()
    compute_type = "float16" if device == "cuda" else "int8"
    print(f"[transcribe/local] faster-whisper model={LOCAL_WHISPER_MODEL} device={device}", flush=True)

    model = WhisperModel(
        LOCAL_WHISPER_MODEL,
        device=device,
        compute_type=compute_type,
        download_root=_model_cache_dir(),
    )
    segments_iter, info = model.transcribe(
        media_path,
        language=language,
        beam_size=5,
        vad_filter=True,
        condition_on_previous_text=False,
    )

    segments = []
    for s in segments_iter:
        segments.append({
            "start": float(s.start),
            "end": float(s.end),
            "text": (s.text or "").strip(),
        })

    duration = float(getattr(info, "duration", 0.0)) or (segments[-1]["end"] if segments else 0.0)
    print(f"[transcribe/local] {len(segments)} segments, {duration:.0f}s of audio", flush=True)
    return {"duration": duration, "segments": segments}

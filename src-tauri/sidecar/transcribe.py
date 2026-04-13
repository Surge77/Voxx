from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path


MODEL_NAME = "large-v3-turbo"
LANGUAGE = "en"


def transcribe(audio_path: Path) -> dict[str, object]:
    started = time.perf_counter()

    fixture = os.getenv("VOXX_FIXTURE_TRANSCRIPT")
    if fixture:
        return {
            "rawText": fixture,
            "segments": [],
            "durationMs": int((time.perf_counter() - started) * 1000),
            "error": None,
        }

    try:
        import noisereduce as nr
        import numpy as np
        import soundfile as sf
        from faster_whisper import WhisperModel
    except Exception as exc:
        return {
            "rawText": "",
            "segments": [],
            "durationMs": int((time.perf_counter() - started) * 1000),
            "error": f"Missing Python dependency: {exc}",
        }

    data, sample_rate = sf.read(str(audio_path))
    reduced = nr.reduce_noise(y=np.asarray(data), sr=sample_rate)
    clean_path = audio_path.with_name(f"{audio_path.stem}.clean.wav")
    sf.write(str(clean_path), reduced, sample_rate)

    model = WhisperModel(MODEL_NAME, device="auto", compute_type="auto")
    segments, info = model.transcribe(str(clean_path), language=LANGUAGE, vad_filter=False)
    parsed_segments = [
        {"start": segment.start, "end": segment.end, "text": segment.text}
        for segment in segments
    ]
    raw_text = " ".join(segment["text"].strip() for segment in parsed_segments).strip()

    return {
        "rawText": raw_text,
        "segments": parsed_segments,
        "language": info.language,
        "durationMs": int((time.perf_counter() - started) * 1000),
        "error": None,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Voxx transcription sidecar")
    parser.add_argument("--audio", required=True, type=Path)
    args = parser.parse_args()

    if not args.audio.exists():
        print(json.dumps({"rawText": "", "segments": [], "durationMs": 0, "error": "Audio file not found"}))
        return 2

    result = transcribe(args.audio)
    print(json.dumps(result, ensure_ascii=True))
    return 1 if result.get("error") else 0


if __name__ == "__main__":
    sys.exit(main())


from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path
from typing import Any


MODEL_NAME = os.getenv("VOXX_WHISPER_MODEL", "tiny.en")
DEVICE = os.getenv("VOXX_WHISPER_DEVICE", "cpu")
COMPUTE_TYPE = os.getenv("VOXX_WHISPER_COMPUTE_TYPE", "int8")
LANGUAGE = "en"


def prepare_audio_for_noise_reduction(data):
    import numpy as np

    audio = np.asarray(data)
    if audio.ndim == 2:
        # soundfile returns (frames, channels); noisereduce expects mono or
        # (channels, frames). Mix to mono to avoid treating frames as channels.
        audio = audio.mean(axis=1)
    elif audio.ndim > 2:
        audio = audio.reshape(-1)

    return audio.astype(np.float32, copy=False)


def transcribe(audio_path: Path, model: Any | None = None) -> dict[str, object]:
    started = time.perf_counter()
    timings: dict[str, int] = {}

    fixture = os.getenv("VOXX_FIXTURE_TRANSCRIPT")
    if fixture:
        return {
            "rawText": fixture,
            "segments": [],
            "durationMs": int((time.perf_counter() - started) * 1000),
            "timings": timings,
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
            "timings": timings,
            "error": f"Missing Python dependency: {exc}",
        }

    read_started = time.perf_counter()
    data, sample_rate = sf.read(str(audio_path))
    audio = prepare_audio_for_noise_reduction(data)
    timings["readAudioMs"] = int((time.perf_counter() - read_started) * 1000)

    noise_started = time.perf_counter()
    reduced = nr.reduce_noise(y=audio, sr=sample_rate)
    clean_path = audio_path.with_name(f"{audio_path.stem}.clean.wav")
    sf.write(str(clean_path), reduced, sample_rate)
    timings["noiseReductionMs"] = int((time.perf_counter() - noise_started) * 1000)

    if model is None:
        model_started = time.perf_counter()
        model = WhisperModel(MODEL_NAME, device=DEVICE, compute_type=COMPUTE_TYPE)
        timings["modelLoadMs"] = int((time.perf_counter() - model_started) * 1000)
    else:
        timings["modelLoadMs"] = 0

    transcribe_started = time.perf_counter()
    segments, info = model.transcribe(str(clean_path), language=LANGUAGE, vad_filter=False)
    parsed_segments = [
        {"start": segment.start, "end": segment.end, "text": segment.text}
        for segment in segments
    ]
    timings["whisperMs"] = int((time.perf_counter() - transcribe_started) * 1000)
    raw_text = " ".join(segment["text"].strip() for segment in parsed_segments).strip()

    return {
        "rawText": raw_text,
        "segments": parsed_segments,
        "language": info.language,
        "model": MODEL_NAME,
        "device": DEVICE,
        "computeType": COMPUTE_TYPE,
        "durationMs": int((time.perf_counter() - started) * 1000),
        "timings": timings,
        "error": None,
    }


def serve() -> int:
    from faster_whisper import WhisperModel

    model_started = time.perf_counter()
    model = WhisperModel(MODEL_NAME, device=DEVICE, compute_type=COMPUTE_TYPE)
    print(
        json.dumps(
            {
                "ready": True,
                "model": MODEL_NAME,
                "device": DEVICE,
                "computeType": COMPUTE_TYPE,
                "modelLoadMs": int((time.perf_counter() - model_started) * 1000),
            }
        ),
        flush=True,
    )

    for line in sys.stdin:
        try:
            request = json.loads(line)
            audio = Path(request["audio"])
            result = transcribe(audio, model=model)
        except Exception as exc:
            result = {
                "rawText": "",
                "segments": [],
                "durationMs": 0,
                "timings": {},
                "error": str(exc),
            }

        print(json.dumps(result, ensure_ascii=True), flush=True)

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Voxx transcription sidecar")
    parser.add_argument("--audio", type=Path)
    parser.add_argument("--serve", action="store_true")
    args = parser.parse_args()

    if args.serve:
        return serve()

    if args.audio is None:
        print(json.dumps({"rawText": "", "segments": [], "durationMs": 0, "error": "Audio path required"}))
        return 2

    if not args.audio.exists():
        print(json.dumps({"rawText": "", "segments": [], "durationMs": 0, "error": "Audio file not found"}))
        return 2

    result = transcribe(args.audio)
    print(json.dumps(result, ensure_ascii=True))
    return 1 if result.get("error") else 0


if __name__ == "__main__":
    sys.exit(main())

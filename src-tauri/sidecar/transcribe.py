from __future__ import annotations

import argparse
import json
import os
import site
import sys
import time
from pathlib import Path
from typing import Any


MODEL_NAME = os.getenv("VOXX_WHISPER_MODEL", "large-v3-turbo")
# Use CUDA on the GTX 1650; fall back to CPU if CUDA is unavailable.
DEVICE = os.getenv("VOXX_WHISPER_DEVICE", "cuda")
# int8_float16 keeps VRAM usage low while retaining GPU speed.
COMPUTE_TYPE = os.getenv("VOXX_WHISPER_COMPUTE_TYPE", "int8_float16")
LANGUAGE = "en"
MIN_RMS = float(os.getenv("VOXX_MIN_RMS", "0.002"))
MAX_GAIN = float(os.getenv("VOXX_MAX_GAIN", "12"))
MIN_AVG_LOGPROB = float(os.getenv("VOXX_MIN_AVG_LOGPROB", "-0.75"))
NOISE_REDUCTION_MIN_RMS = float(os.getenv("VOXX_NOISE_REDUCTION_MIN_RMS", "0.01"))
FALLBACK_MODEL_NAME = os.getenv("VOXX_WHISPER_FALLBACK_MODEL", "small.en")
FALLBACK_DEVICE = os.getenv("VOXX_WHISPER_FALLBACK_DEVICE", "cpu")
FALLBACK_COMPUTE_TYPE = os.getenv("VOXX_WHISPER_FALLBACK_COMPUTE_TYPE", "int8")


def configure_cuda_dll_paths() -> None:
    if os.name != "nt":
        return

    candidates: list[Path] = []
    for root in map(Path, site.getsitepackages()):
        candidates.extend(
            [
                root / "nvidia" / "cublas" / "bin",
                root / "nvidia" / "cudnn" / "bin",
                root / "nvidia" / "cuda_nvrtc" / "bin",
                root / "ctranslate2",
            ]
        )

    for candidate in candidates:
        if candidate.exists():
            os.add_dll_directory(str(candidate))

    existing_path = os.environ.get("PATH", "")
    dll_path = os.pathsep.join(str(candidate) for candidate in candidates if candidate.exists())
    if dll_path:
        os.environ["PATH"] = f"{dll_path}{os.pathsep}{existing_path}"


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


def normalize_audio_for_whisper(audio):
    import numpy as np

    samples = np.asarray(audio, dtype=np.float32)
    if samples.size == 0:
        return samples

    peak = float(np.max(np.abs(samples)))
    if peak <= 0:
        return samples

    gain = min(1.0 / peak, MAX_GAIN)
    return np.clip(samples * gain, -1.0, 1.0).astype(np.float32, copy=False)


def audio_rms(audio) -> float:
    import numpy as np

    samples = np.asarray(audio, dtype=np.float32)
    if samples.size == 0:
        return 0.0
    return float(np.sqrt(np.mean(samples.astype(np.float64) ** 2)))


def filter_low_confidence_segments(segments):
    filtered = []
    for segment in segments:
        avg_logprob = getattr(segment, "avg_logprob", 0.0)
        text = segment.text.strip()
        if text and avg_logprob >= MIN_AVG_LOGPROB:
            filtered.append(segment)
    return filtered


def load_whisper_model(model_name: str = MODEL_NAME, device: str = DEVICE, compute_type: str = COMPUTE_TYPE):
    configure_cuda_dll_paths()
    from faster_whisper import WhisperModel

    return WhisperModel(model_name, device=device, compute_type=compute_type)


def transcribe(
    audio_path: Path,
    model: Any | None = None,
    model_name: str = MODEL_NAME,
    device: str = DEVICE,
    compute_type: str = COMPUTE_TYPE,
) -> dict[str, object]:
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
    rms_before = audio_rms(audio)
    if rms_before < MIN_RMS:
        return {
            "rawText": "",
            "segments": [],
            "language": LANGUAGE,
            "model": MODEL_NAME,
            "device": DEVICE,
            "computeType": COMPUTE_TYPE,
            "durationMs": int((time.perf_counter() - started) * 1000),
            "timings": timings,
            "error": None,
            "skipped": "audio_below_rms_threshold",
        }
    timings["readAudioMs"] = int((time.perf_counter() - read_started) * 1000)

    noise_started = time.perf_counter()
    if rms_before >= NOISE_REDUCTION_MIN_RMS:
        reduced = nr.reduce_noise(y=audio, sr=sample_rate)
        timings["noiseReductionApplied"] = 1
    else:
        reduced = audio
        timings["noiseReductionApplied"] = 0
    reduced = normalize_audio_for_whisper(reduced)
    timings["rmsBefore"] = int(rms_before * 1_000_000)
    timings["rmsAfter"] = int(audio_rms(reduced) * 1_000_000)
    timings["noiseReductionMs"] = int((time.perf_counter() - noise_started) * 1000)
    # Resample to 16 kHz in-memory if needed — Whisper's native sample rate.
    # This avoids writing a .clean.wav to disk and reading it back.
    if sample_rate != 16000:
        import scipy.signal
        num_samples = int(len(reduced) * 16000 / sample_rate)
        reduced = scipy.signal.resample(reduced, num_samples).astype(np.float32)
        sample_rate = 16000

    if model is None:
        model_started = time.perf_counter()
        model = load_whisper_model(model_name, device, compute_type)
        timings["modelLoadMs"] = int((time.perf_counter() - model_started) * 1000)
    else:
        timings["modelLoadMs"] = 0

    transcribe_started = time.perf_counter()
    try:
        # Pass the numpy array directly — no disk I/O for the cleaned audio.
        # beam_size=1 is greedy decoding: fastest possible, minimal quality loss
        # for short voice clips. best_of=1 disables sampling candidates.
        segments, info = model.transcribe(
            reduced,
            language=LANGUAGE,
            vad_filter=True,
            beam_size=1,
            best_of=1,
            temperature=0,
            condition_on_previous_text=False,
        )
        filtered_segments = filter_low_confidence_segments(segments)
    except RuntimeError:
        if device == FALLBACK_DEVICE and model_name == FALLBACK_MODEL_NAME:
            raise

        fallback_started = time.perf_counter()
        fallback_model = load_whisper_model(FALLBACK_MODEL_NAME, FALLBACK_DEVICE, FALLBACK_COMPUTE_TYPE)
        timings["fallbackModelLoadMs"] = int((time.perf_counter() - fallback_started) * 1000)
        timings["fallbackUsed"] = 1
        model_name = FALLBACK_MODEL_NAME
        device = FALLBACK_DEVICE
        compute_type = FALLBACK_COMPUTE_TYPE
        segments, info = fallback_model.transcribe(
            reduced,
            language=LANGUAGE,
            vad_filter=True,
            beam_size=1,
            best_of=1,
            temperature=0,
            condition_on_previous_text=False,
        )
        filtered_segments = filter_low_confidence_segments(segments)
    parsed_segments = [
        {
            "start": segment.start,
            "end": segment.end,
            "text": segment.text,
            "avgLogprob": getattr(segment, "avg_logprob", None),
            "noSpeechProb": getattr(segment, "no_speech_prob", None),
        }
        for segment in filtered_segments
    ]
    timings["whisperMs"] = int((time.perf_counter() - transcribe_started) * 1000)
    raw_text = " ".join(segment["text"].strip() for segment in parsed_segments).strip()

    return {
        "rawText": raw_text,
        "segments": parsed_segments,
        "language": info.language,
        "model": model_name,
        "device": device,
        "computeType": compute_type,
        "durationMs": int((time.perf_counter() - started) * 1000),
        "timings": timings,
        "error": None,
    }


def serve() -> int:
    model_started = time.perf_counter()
    if os.getenv("VOXX_FIXTURE_TRANSCRIPT"):
        model = None
        model_load_ms = 0
    else:
        model = load_whisper_model()
        model_load_ms = int((time.perf_counter() - model_started) * 1000)
    print(
        json.dumps(
            {
                "ready": True,
                "model": MODEL_NAME,
                "device": DEVICE,
                "computeType": COMPUTE_TYPE,
                "modelLoadMs": model_load_ms,
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

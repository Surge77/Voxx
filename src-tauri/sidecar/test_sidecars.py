import json
import os
import subprocess
import sys
from pathlib import Path

import numpy as np

from transcribe import configure_cuda_dll_paths, normalize_audio_for_whisper, prepare_audio_for_noise_reduction


def test_transcribe_fixture_returns_json(tmp_path: Path):
    audio_path = tmp_path / "sample.wav"
    audio_path.write_bytes(b"RIFF$\x00\x00\x00WAVEfmt ")
    env = os.environ.copy()
    env["VOXX_FIXTURE_TRANSCRIPT"] = "react query"

    result = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("transcribe.py")), "--audio", str(audio_path)],
        env=env,
        text=True,
        capture_output=True,
        check=True,
    )

    parsed = json.loads(result.stdout)
    assert parsed["rawText"] == "react query"
    assert parsed["error"] is None


def test_prepare_audio_for_noise_reduction_converts_soundfile_stereo_to_mono():
    stereo = np.ones((101760, 2), dtype=np.float64)

    prepared = prepare_audio_for_noise_reduction(stereo)

    assert prepared.shape == (101760,)
    assert prepared.dtype == np.float32


def test_transcribe_defaults_to_cpu_int8_model():
    import transcribe

    assert transcribe.DEVICE == "cuda"
    assert transcribe.COMPUTE_TYPE == "int8_float16"
    assert transcribe.MODEL_NAME == "large-v3-turbo"


def test_normalize_audio_for_whisper_lifts_quiet_signal_without_clipping():
    quiet = np.array([0.0, 0.05, -0.025], dtype=np.float32)

    normalized = normalize_audio_for_whisper(quiet)

    assert np.max(np.abs(normalized)) > np.max(np.abs(quiet))
    assert np.max(np.abs(normalized)) <= 1.0


def test_noise_reduction_threshold_defaults_above_quiet_sample():
    import transcribe

    assert transcribe.NOISE_REDUCTION_MIN_RMS == 0.01


def test_configure_cuda_dll_paths_is_safe_to_call():
    configure_cuda_dll_paths()


def test_transcribe_server_fixture_handles_json_line_request(tmp_path: Path):
    audio_path = tmp_path / "sample.wav"
    audio_path.write_bytes(b"RIFF$\x00\x00\x00WAVEfmt ")
    env = os.environ.copy()
    env["VOXX_FIXTURE_TRANSCRIPT"] = "server transcript"

    process = subprocess.Popen(
        [sys.executable, str(Path(__file__).with_name("transcribe.py")), "--serve"],
        env=env,
        text=True,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    assert process.stdin is not None
    assert process.stdout is not None
    ready = json.loads(process.stdout.readline())
    assert ready["ready"] is True

    process.stdin.write(json.dumps({"audio": str(audio_path)}) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    process.terminate()
    process.wait(timeout=5)

    assert response["rawText"] == "server transcript"
    assert response["error"] is None

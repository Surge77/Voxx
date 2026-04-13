import json
import os
import subprocess
import sys
from pathlib import Path

import numpy as np

from transcribe import prepare_audio_for_noise_reduction


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

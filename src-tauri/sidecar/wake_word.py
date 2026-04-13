from __future__ import annotations

import json
import os
import sys
import time


WAKE_WORD = "vox"


def emit(event: str, detail: str = "") -> None:
    print(json.dumps({"event": event, "wakeWord": WAKE_WORD, "detail": detail}), flush=True)


def main() -> int:
    if os.getenv("VOXX_WAKE_WORD_TEST") == "1":
        emit("wake_detected", "fixture")
        return 0

    try:
        import openwakeword  # noqa: F401
    except Exception as exc:
        emit("error", f"Missing OpenWakeWord dependency: {exc}")
        return 1

    emit("ready", "OpenWakeWord dependency loaded. Model wiring is the next integration step.")
    while True:
        time.sleep(1)


if __name__ == "__main__":
    sys.exit(main())


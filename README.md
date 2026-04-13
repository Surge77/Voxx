# Voxx

Voxx is a Windows desktop voice-to-text app built with Tauri 2, React, TypeScript, Tailwind CSS, Rust, Python sidecars, faster-whisper, and Ollama.

## Development

Install JavaScript dependencies:

```powershell
npm install
```

Run the frontend checks:

```powershell
npm test
npm run build
```

Run Python sidecar tests:

```powershell
python -m pytest src-tauri/sidecar
```

Run the desktop app:

```powershell
npm run tauri:dev
```

## Windows Rust Toolchain

This repo pins Rust to `stable-x86_64-pc-windows-msvc`. Install Visual Studio Build Tools with the C++ build tools workload and Windows SDK before running Cargo or Tauri builds.

If PowerShell resolves `link.exe` to Git or another Unix compatibility tool, open a Visual Studio Developer PowerShell or make sure the MSVC linker and Windows SDK libraries are earlier in `PATH` and `LIB`.

## Voice Pipeline

The intended release pipeline is:

1. Hold Ctrl+Space to start recording.
2. Release Ctrl+Space to stop recording.
3. Write the captured audio to WAV.
4. Run `src-tauri/sidecar/transcribe.py`.
5. Post-process the raw transcript with local Ollama `phi4-mini`.
6. Paste the processed text at the cursor and save the history row in SQLite.

For local sidecar smoke tests without Whisper installed, set:

```powershell
$env:VOXX_FIXTURE_TRANSCRIPT = "react query"
```

The default dev transcription model is `tiny.en` on CPU with int8 compute so the app stays responsive on machines without CUDA/cuBLAS installed. Override it when you want higher accuracy:

```powershell
$env:VOXX_WHISPER_MODEL = "large-v3-turbo"
$env:VOXX_WHISPER_DEVICE = "cuda"
$env:VOXX_WHISPER_COMPUTE_TYPE = "float16"
```

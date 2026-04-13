# Voxx Project Context

## What this is
Voxx is a Tauri 2.0 desktop app for Windows that does voice-to-text typing.
It works like Wispr Flow: hold Ctrl+Space to record, release to transcribe and paste.
It includes a full dashboard UI, transcription history, multiple dictation modes,
LLM-powered post-processing, context-aware mode switching, wake word support,
and a self-improving custom dictionary.

## App Identity
- Name: Voxx
- Identifier: com.voxx.app
- Tagline: "Your voice, typed instantly."

## Tech Stack
- Frontend: React + TypeScript + Tailwind CSS
- Desktop: Tauri 2.0 with Rust backend
- Speech-to-text: faster-whisper Python sidecar with whisper-large-v3-turbo
- Noise cancellation: noisereduce Python library before Whisper transcription
- Wake word detection: OpenWakeWord, local/offline, wake word "VOX"
- LLM post-processing: Ollama local API at localhost:11434 with phi4-mini, always active
- Local storage: SQLite for history, custom dictionary, correction memory, and preferences
- Hotkey: Tauri global shortcut plugin
- Clipboard: Tauri clipboard manager plugin

## System Specs
- CPU: Intel Core i5 12th Gen
- RAM: 16 GB
- GPU: NVIDIA GTX 1650 with 4 GB VRAM
- OS: Windows 11

## Architecture
- React frontend: dashboard window, floating overlay pill, audio visualizer
- Tauri Rust backend: hotkeys, window management, tray, clipboard, sidecar communication
- Python sidecar: noise cancellation -> faster-whisper -> JSON transcription output
- OpenWakeWord sidecar: always listening for "VOX" when enabled
- Ollama API: phi4-mini post-processes every transcription
- SQLite DB: transcription history, custom dictionary, correction memory, preferences

## App Views
1. Dashboard: transcription history feed, search bar, recent entries
2. Settings: hotkey, mic selector, wake word toggle, theme, sound, startup, Ollama status
3. Modes: switch dictation modes, configure context-aware switching rules
4. Dictionary: custom vocabulary, built organically from corrections, starts empty
5. Stats: words dictated today/week, most used mode, accuracy trends

## Dictation Modes
1. General Mode: natural conversational text with proper punctuation and capitalization.
   Best for messages, emails, and general writing.
2. Code Mode: camelCase for variables/functions, PascalCase for components, keeps acronyms
   such as API, URL, UI, SDK, and maps spoken punctuation like "open paren" and
   "arrow function". Best for coding and technical docs.
3. Command Mode: literal interpretation such as "new line" -> \n, "tab" -> \t,
   "open bracket" -> {. No extra punctuation. Best for terminal commands and config files.
4. Email/Message Mode: formal tone, proper structure, and proper noun capitalization.
   Best for emails, Slack, Discord, and messages.

## Context-Aware Mode Switching
- VSCode, Cursor, any IDE -> Code Mode
- Gmail, Outlook, browser compose window -> Email/Message Mode
- Windows Terminal, PowerShell, CMD -> Command Mode
- Everything else -> General Mode
- Users can override auto-switching per app in Settings -> Modes.

## Recording And Hotkey Behavior
- Hold Ctrl+Space -> start recording.
- Release Ctrl+Space -> strictly stop recording; do not auto-stop on silence.
- If held for less than 300ms -> ignore to prevent accidental triggers.
- Wake word flow: say "VOX" -> starts recording -> say "VOX" again or release any key to stop.
- Pipeline on release:
  1. Apply noise cancellation with noisereduce.
  2. Transcribe with faster-whisper using whisper-large-v3-turbo and language="en".
  3. Post-process with phi4-mini using the active mode prompt and dictionary entries.
  4. Paste result at the current cursor position through the clipboard.
  5. Save raw and processed text to SQLite history.

## Floating Overlay UI
- Bottom-center floating pill appears when recording starts.
- Contains animated audio bars, small teal Voxx icon, and live transcription preview text.
- Pill disappears automatically when Ctrl+Space is released and pasting is done.
- No close button.
- Partial preview appears in the overlay first; in-cursor partial replacement is an advanced
  behavior after final paste is stable.

## Tray Icon States
- Green: idle, ready
- Red: recording
- Yellow: processing
- Green flash: done, then return to idle after 500ms

## Audio And Microphone
- Default: laptop built-in microphone.
- Settings lists all available audio input devices detected by the OS.
- User can select any input device from Settings.
- Noise cancellation is always applied before Whisper.

## Wake Word
- Wake word: "VOX" in UI, "vox" in code.
- Powered by OpenWakeWord, local and offline.
- Enabled/disabled in Settings; disabled by default on first install.
- Wake word listener runs as a separate lightweight background process.

## Sound Feedback
- Optional soft click on recording start and stop.
- Disabled by default.
- Volume slider appears in Settings when enabled.

## Correction And Self-Improvement
- Every transcription stores raw_text, processed_text, mode, timestamp, and corrections.
- Voice command "Vox correct that" re-transcribes the last audio clip.
- Manual correction: edit any history entry.
- When a correction is saved, the wrong -> right pair is added to the custom dictionary.
- Future transcriptions with that term are fixed by phi4-mini through dynamic prompt entries.
- Dictionary starts empty and grows from real corrections.

## Transcription History
- Stored permanently in SQLite.
- Displayed newest first.
- Searchable by keyword, date range, and mode.
- Entry fields: processed text, raw Whisper text, mode, timestamp.
- Entry actions: Copy, Re-paste, Mark as corrected, Edit, Delete.
- No auto-delete.

## Startup Behavior
- Voxx launches automatically on Windows startup.
- Starts minimized to system tray.
- User opens dashboard from tray icon or taskbar.

## UI Design
- Dark theme by default, light mode toggle available.
- Main window available through tray or taskbar.
- Floating overlay pill bottom-center, auto show/hide.
- Clean minimal aesthetic inspired by Linear/Vercel dark UI.
- Accent color: electric teal #00BFA5.
- System tray right-click menu: Open Voxx, Switch Mode, Wake Word On/Off, Pause/Resume, Quit.

## Conventions
- camelCase for variables, PascalCase for components.
- All Tauri commands in src-tauri/src/commands/.
- All React components in src/components/.
- All database logic in src/lib/db.ts.
- Python sidecar code in src-tauri/sidecar/.
- Wake word sidecar in src-tauri/sidecar/wake_word.py.
- Transcription sidecar in src-tauri/sidecar/transcribe.py.
- Use Tailwind for styling, no inline styles.
- Always handle errors gracefully.
- Log all errors to a local file through the Tauri logger plugin.

## Performance Targets
- Noise cancellation: <100ms
- Transcription latency: <1.5 seconds for a 10-second clip
- LLM post-processing: <1 second with phi4-mini on GTX 1650 VRAM
- Total pipeline release -> paste: <3 seconds, target <2 seconds
- History search: <100ms for up to 10,000 entries
- Wake word detection latency: <200ms

## Language
- English only.
- Whisper language parameter hardcoded to "en".

## Build And Packaging
- Dev: npm run tauri dev
- Production: npm run tauri build -> NSIS installer
- Python sidecars bundled with the final app.
- Installer app name: Voxx.
- Auto-startup registry entry added during installation.

## Current Status
- Initial implementation in progress.

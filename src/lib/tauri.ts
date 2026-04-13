import { invoke } from "@tauri-apps/api/core";
import type {
  AppPreferences,
  DiagnosticResult,
  DictationMode,
  DictionaryEntry,
  HistoryEntry,
  PipelineResult
} from "../types";

export function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

const mockPreferences: AppPreferences = {
  activeMode: "general",
  wakeWordEnabled: false,
  soundEnabled: false,
  soundVolume: 0.4,
  theme: "dark",
  restoreClipboard: true,
  selectedAudioDevice: null
};

let mockHistory: HistoryEntry[] = [];
let mockDictionary: DictionaryEntry[] = [];
let mockStartedAt = 0;

function mockPipelineResult(): PipelineResult {
  const now = new Date().toISOString();
  const durationMs = mockStartedAt ? Date.now() - mockStartedAt : 750;
  if (durationMs < 300) {
    return {
      ignored: true,
      rawText: "",
      processedText: "",
      mode: mockPreferences.activeMode,
      durationMs
    };
  }

  const entry: HistoryEntry = {
    id: Date.now(),
    rawText: "react query",
    processedText: "React Query",
    mode: mockPreferences.activeMode,
    createdAt: now,
    durationMs
  };
  mockHistory = [entry, ...mockHistory];
  return {
    ignored: false,
    rawText: entry.rawText,
    processedText: entry.processedText,
    mode: entry.mode,
    durationMs,
    historyId: entry.id
  };
}

export const api = {
  startRecording: async () => {
    if (!isTauriRuntime()) {
      mockStartedAt = Date.now();
      return;
    }
    return invoke<void>("start_recording");
  },
  stopRecordingAndProcess: () => (isTauriRuntime() ? invoke<PipelineResult>("stop_recording_and_process") : Promise.resolve(mockPipelineResult())),
  cancelRecording: () => (isTauriRuntime() ? invoke<void>("cancel_recording") : Promise.resolve()),
  getHistory: (query = "") =>
    isTauriRuntime()
      ? invoke<HistoryEntry[]>("search_history", { query })
      : Promise.resolve(mockHistory.filter((entry) => JSON.stringify(entry).toLowerCase().includes(query.toLowerCase()))),
  updateHistoryEntry: (id: number, processedText: string) => {
    if (isTauriRuntime()) {
      return invoke<HistoryEntry>("update_history_entry", { id, processedText });
    }
    mockHistory = mockHistory.map((entry) => (entry.id === id ? { ...entry, processedText } : entry));
    return Promise.resolve(mockHistory.find((entry) => entry.id === id)!);
  },
  deleteHistoryEntry: (id: number) => {
    if (isTauriRuntime()) {
      return invoke<void>("delete_history_entry", { id });
    }
    mockHistory = mockHistory.filter((entry) => entry.id !== id);
    return Promise.resolve();
  },
  copyEntry: (id: number) => (isTauriRuntime() ? invoke<void>("copy_entry", { id }) : Promise.resolve()),
  repasteEntry: (id: number) => (isTauriRuntime() ? invoke<void>("repaste_entry", { id }) : Promise.resolve()),
  getDictionary: () => (isTauriRuntime() ? invoke<DictionaryEntry[]>("get_dictionary") : Promise.resolve(mockDictionary)),
  saveCorrection: (historyId: number, wrong: string, right: string) => {
    if (isTauriRuntime()) {
      return invoke<DictionaryEntry>("save_correction", { historyId, wrong, right });
    }
    const entry = { id: Date.now(), wrong, right, createdAt: new Date().toISOString() };
    mockDictionary = [entry, ...mockDictionary];
    return Promise.resolve(entry);
  },
  deleteDictionaryEntry: (id: number) => {
    if (isTauriRuntime()) {
      return invoke<void>("delete_dictionary_entry", { id });
    }
    mockDictionary = mockDictionary.filter((entry) => entry.id !== id);
    return Promise.resolve();
  },
  getPreferences: () => (isTauriRuntime() ? invoke<AppPreferences>("get_preferences") : Promise.resolve(mockPreferences)),
  setPreference: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => {
    if (isTauriRuntime()) {
      return invoke<AppPreferences>("set_preference", { key, value });
    }
    Object.assign(mockPreferences, { [key]: value });
    return Promise.resolve(mockPreferences);
  },
  setActiveMode: (mode: DictationMode) => {
    if (isTauriRuntime()) {
      return invoke<AppPreferences>("set_active_mode", { mode });
    }
    mockPreferences.activeMode = mode;
    return Promise.resolve(mockPreferences);
  },
  getDiagnostics: () =>
    isTauriRuntime()
      ? invoke<DiagnosticResult[]>("run_diagnostics")
      : Promise.resolve([
          { name: "Ollama", ok: false, detail: "Desktop runtime required" },
          { name: "SQLite", ok: true, detail: "Browser mock active" },
          { name: "Transcription sidecar", ok: false, detail: "Desktop runtime required" }
        ]),
  getAudioDevices: () => (isTauriRuntime() ? invoke<string[]>("get_audio_devices") : Promise.resolve(["Browser mock microphone"])),
  getOllamaStatus: () =>
    isTauriRuntime()
      ? invoke<DiagnosticResult>("get_ollama_status")
      : Promise.resolve({ name: "Ollama", ok: false, detail: "Desktop runtime required" })
};

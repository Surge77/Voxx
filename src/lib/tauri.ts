import { invoke } from "@tauri-apps/api/core";
import type {
  AppPreferences,
  DiagnosticResult,
  DictationMode,
  DictionaryEntry,
  HistoryEntry,
  PipelineResult
} from "../types";

export const api = {
  startRecording: () => invoke<void>("start_recording"),
  stopRecordingAndProcess: () => invoke<PipelineResult>("stop_recording_and_process"),
  cancelRecording: () => invoke<void>("cancel_recording"),
  getHistory: (query = "") => invoke<HistoryEntry[]>("search_history", { query }),
  updateHistoryEntry: (id: number, processedText: string) =>
    invoke<HistoryEntry>("update_history_entry", { id, processedText }),
  deleteHistoryEntry: (id: number) => invoke<void>("delete_history_entry", { id }),
  copyEntry: (id: number) => invoke<void>("copy_entry", { id }),
  repasteEntry: (id: number) => invoke<void>("repaste_entry", { id }),
  getDictionary: () => invoke<DictionaryEntry[]>("get_dictionary"),
  saveCorrection: (historyId: number, wrong: string, right: string) =>
    invoke<DictionaryEntry>("save_correction", { historyId, wrong, right }),
  deleteDictionaryEntry: (id: number) => invoke<void>("delete_dictionary_entry", { id }),
  getPreferences: () => invoke<AppPreferences>("get_preferences"),
  setPreference: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) =>
    invoke<AppPreferences>("set_preference", { key, value }),
  setActiveMode: (mode: DictationMode) => invoke<AppPreferences>("set_active_mode", { mode }),
  getDiagnostics: () => invoke<DiagnosticResult[]>("run_diagnostics"),
  getAudioDevices: () => invoke<string[]>("get_audio_devices"),
  getOllamaStatus: () => invoke<DiagnosticResult>("get_ollama_status")
};


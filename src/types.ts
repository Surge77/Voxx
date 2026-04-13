export type DictationMode = "general" | "code" | "command" | "email";

export type RecordingState = "idle" | "recording" | "processing" | "error";

export type TrayState = "idle" | "recording" | "processing" | "done";

export type HistoryEntry = {
  id: number;
  rawText: string;
  processedText: string;
  mode: DictationMode;
  createdAt: string;
  durationMs: number;
};

export type DictionaryEntry = {
  id: number;
  wrong: string;
  right: string;
  createdAt: string;
};

export type AppPreferences = {
  activeMode: DictationMode;
  wakeWordEnabled: boolean;
  soundEnabled: boolean;
  soundVolume: number;
  theme: "dark" | "light";
  restoreClipboard: boolean;
  selectedAudioDevice: string | null;
};

export type DiagnosticResult = {
  name: string;
  ok: boolean;
  detail: string;
};

export type PipelineResult = {
  ignored: boolean;
  rawText: string;
  processedText: string;
  mode: DictationMode;
  durationMs: number;
  historyId?: number;
  error?: string;
};


import { useEffect, useMemo, useState } from "react";
import { Dashboard } from "./components/Dashboard";
import { Dictionary } from "./components/Dictionary";
import { FloatingOverlay } from "./components/FloatingOverlay";
import { Modes } from "./components/Modes";
import { Settings } from "./components/Settings";
import { Stats } from "./components/Stats";
import { registerRecordingHotkey } from "./lib/hotkeys";
import { api } from "./lib/tauri";
import type {
  AppPreferences,
  DiagnosticResult,
  DictionaryEntry,
  HistoryEntry,
  RecordingState
} from "./types";

type View = "dashboard" | "settings" | "modes" | "dictionary" | "stats";

const defaultPreferences: AppPreferences = {
  activeMode: "general",
  wakeWordEnabled: false,
  soundEnabled: false,
  soundVolume: 0.4,
  theme: "dark",
  restoreClipboard: true,
  selectedAudioDevice: null
};

const navItems: Array<{ id: View; label: string }> = [
  { id: "dashboard", label: "Dashboard" },
  { id: "settings", label: "Settings" },
  { id: "modes", label: "Modes" },
  { id: "dictionary", label: "Dictionary" },
  { id: "stats", label: "Stats" }
];

export function App() {
  const [view, setView] = useState<View>("dashboard");
  const [recordingState, setRecordingState] = useState<RecordingState>("idle");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [dictionary, setDictionary] = useState<DictionaryEntry[]>([]);
  const [preferences, setPreferences] = useState<AppPreferences>(defaultPreferences);
  const [diagnostics, setDiagnostics] = useState<DiagnosticResult[]>([]);
  const [statusMessage, setStatusMessage] = useState("Ready");

  async function refreshData() {
    const [historyResult, dictionaryResult, preferencesResult, diagnosticsResult] =
      await Promise.allSettled([
        api.getHistory(),
        api.getDictionary(),
        api.getPreferences(),
        api.getDiagnostics()
      ]);

    if (historyResult.status === "fulfilled") setHistory(historyResult.value);
    if (dictionaryResult.status === "fulfilled") setDictionary(dictionaryResult.value);
    if (preferencesResult.status === "fulfilled") setPreferences(preferencesResult.value);
    if (diagnosticsResult.status === "fulfilled") setDiagnostics(diagnosticsResult.value);
  }

  useEffect(() => {
    void refreshData().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : "Unable to load Voxx data");
    });
  }, []);

  useEffect(() => {
    void registerRecordingHotkey({
      onRecordingStart: () => {
        setRecordingState("recording");
        setStatusMessage("Recording");
      },
      onProcessingStart: () => {
        setRecordingState("processing");
        setStatusMessage("Processing");
      },
      onError: (message) => {
        setRecordingState("error");
        setStatusMessage(message);
      },
      onDone: async () => {
        setRecordingState("idle");
        setStatusMessage("Pasted transcription");
        await refreshData();
      }
    }).catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : "Unable to register Ctrl+Space");
    });
  }, []);

  async function handleStartRecording() {
    try {
      setRecordingState("recording");
      setStatusMessage("Recording");
      await api.startRecording();
    } catch (error) {
      setRecordingState("error");
      setStatusMessage(error instanceof Error ? error.message : "Unable to start recording");
    }
  }

  async function handleStopRecording() {
    try {
      setRecordingState("processing");
      setStatusMessage("Processing");
      const result = await api.stopRecordingAndProcess();

      if (result.ignored) {
        setStatusMessage("Ignored short hold");
      } else {
        setStatusMessage(result.error ?? "Pasted transcription");
      }

      setRecordingState(result.error ? "error" : "idle");
      await refreshData();
    } catch (error) {
      setRecordingState("error");
      setStatusMessage(error instanceof Error ? error.message : "Unable to process recording");
    }
  }

  const activeView = useMemo(() => {
    if (view === "settings") {
      return (
        <Settings
          diagnostics={diagnostics}
          preferences={preferences}
          onRefresh={refreshData}
          onUpdatePreferences={setPreferences}
        />
      );
    }
    if (view === "modes") {
      return <Modes activeMode={preferences.activeMode} onModeChange={setPreferences} />;
    }
    if (view === "dictionary") {
      return <Dictionary entries={dictionary} onRefresh={refreshData} />;
    }
    if (view === "stats") {
      return <Stats history={history} />;
    }
    return <Dashboard history={history} onRefresh={refreshData} />;
  }, [diagnostics, dictionary, history, preferences, view]);

  return (
    <main className="min-h-screen bg-voxx-bg text-voxx-text">
      <div className="mx-auto flex min-h-screen w-full max-w-7xl">
        <aside className="hidden w-64 border-r border-voxx-line bg-voxx-panel px-4 py-6 md:block">
          <div className="mb-10">
            <p className="text-sm uppercase text-voxx-accent">Voxx</p>
            <h1 className="mt-2 text-2xl font-semibold">Your voice, typed instantly.</h1>
          </div>
          <nav className="flex flex-col gap-2">
            {navItems.map((item) => (
              <button
                key={item.id}
                type="button"
                className={`rounded-md px-3 py-2 text-left text-sm transition-colors ${
                  view === item.id
                    ? "bg-voxx-accent text-black"
                    : "text-voxx-muted hover:bg-voxx-panel-soft hover:text-voxx-text"
                }`}
                onClick={() => setView(item.id)}
              >
                {item.label}
              </button>
            ))}
          </nav>
        </aside>
        <section className="flex min-w-0 flex-1 flex-col">
          <header className="border-b border-voxx-line bg-voxx-panel/80 px-4 py-4 backdrop-blur md:px-8">
            <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
              <div>
                <p className="text-sm text-voxx-muted">Status</p>
                <h2 className="text-xl font-semibold">{statusMessage}</h2>
              </div>
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  className="rounded-md bg-voxx-accent px-4 py-2 text-sm font-semibold text-black"
                  onClick={handleStartRecording}
                >
                  Start recording
                </button>
                <button
                  type="button"
                  className="rounded-md border border-voxx-line px-4 py-2 text-sm text-voxx-text"
                  onClick={handleStopRecording}
                >
                  Stop and paste
                </button>
              </div>
            </div>
            <div className="mt-4 flex gap-2 overflow-x-auto md:hidden">
              {navItems.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className={`shrink-0 rounded-md px-3 py-2 text-sm ${
                    view === item.id ? "bg-voxx-accent text-black" : "bg-voxx-panel-soft text-voxx-muted"
                  }`}
                  onClick={() => setView(item.id)}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </header>
          <div className="min-h-0 flex-1 overflow-auto px-4 py-6 md:px-8">{activeView}</div>
        </section>
      </div>
      <FloatingOverlay state={recordingState} message={statusMessage} />
    </main>
  );
}

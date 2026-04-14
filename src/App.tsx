import { useEffect, useMemo, useState } from "react";
import { Dashboard } from "./components/Dashboard";
import { Dictionary } from "./components/Dictionary";
import { FloatingOverlay } from "./components/FloatingOverlay";
import { Modes } from "./components/Modes";
import { Settings } from "./components/Settings";
import { Stats } from "./components/Stats";
import { VoiceVisualizerCapsule } from "./components/VoiceVisualizerCapsule";
import { listen } from "@tauri-apps/api/event";
import { statusMessageForPipelineResult } from "./lib/pipelineStatus";
import { api, isTauriRuntime } from "./lib/tauri";
import type {
  AppPreferences,
  DiagnosticResult,
  DictionaryEntry,
  HistoryEntry,
  PipelineResult,
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
  const isOverlay = typeof window !== "undefined" && new URLSearchParams(window.location.search).has("overlay");
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
    document.body.classList.toggle("voxx-overlay-body", isOverlay);
    return () => document.body.classList.remove("voxx-overlay-body");
  }, [isOverlay]);

  useEffect(() => {
    void refreshData().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : "Unable to load Voxx data");
    });
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    const unlisten = Promise.all([
      listen<string>("voxx://recording-state", (event) => {
        if (event.payload === "recording") {
          setRecordingState("recording");
          setStatusMessage("Recording");
          return;
        }
        if (event.payload === "processing") {
          setRecordingState("idle");
          setStatusMessage("Processing");
          return;
        }
        if (event.payload === "error") {
          setRecordingState("error");
          return;
        }
        setRecordingState("idle");
      }),
      listen<PipelineResult>("voxx://pipeline-result", async (event) => {
        setRecordingState(event.payload.error ? "error" : "idle");
        setStatusMessage(statusMessageForPipelineResult(event.payload));
        await refreshData();
      }),
      listen<string>("voxx://recording-error", (event) => {
        setRecordingState("error");
        setStatusMessage(event.payload);
      }),
      listen<AppPreferences>("voxx://preferences-changed", (event) => {
        setPreferences(event.payload);
      })
    ]);

    return () => {
      void unlisten.then((callbacks) => callbacks.forEach((callback) => callback()));
    };
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
      setRecordingState("idle");
      setStatusMessage("Processing");
      const result = await api.stopRecordingAndProcess();

      setStatusMessage(statusMessageForPipelineResult(result));
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

  if (isOverlay) {
    return (
      <main className="voxx-overlay-root">
        <VoiceVisualizerCapsule />
      </main>
    );
  }

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

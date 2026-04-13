import { useEffect, useState } from "react";
import { api } from "../lib/tauri";
import type { AppPreferences, DiagnosticResult } from "../types";

type SettingsProps = {
  preferences: AppPreferences;
  diagnostics: DiagnosticResult[];
  onRefresh: () => Promise<void>;
  onUpdatePreferences: (preferences: AppPreferences) => void;
};

export function Settings({ preferences, diagnostics, onRefresh, onUpdatePreferences }: SettingsProps) {
  const [audioDevices, setAudioDevices] = useState<string[]>([]);

  useEffect(() => {
    void api.getAudioDevices().then(setAudioDevices).catch(() => setAudioDevices([]));
  }, []);

  async function updatePreference<K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) {
    const updated = await api.setPreference(key, value);
    onUpdatePreferences(updated);
    await onRefresh();
  }

  return (
    <section className="grid gap-6">
      <div>
        <p className="text-sm uppercase text-voxx-accent">Settings</p>
        <h2 className="mt-2 text-3xl font-semibold">Device and app controls</h2>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <label className="flex flex-col gap-2 rounded-md border border-voxx-line bg-voxx-panel p-5">
          <span className="text-sm text-voxx-muted">Microphone</span>
          <select
            className="rounded-md border border-voxx-line bg-voxx-panel-soft px-3 py-2"
            value={preferences.selectedAudioDevice ?? ""}
            onChange={(event) => updatePreference("selectedAudioDevice", event.currentTarget.value || null)}
          >
            <option value="">System default microphone</option>
            {audioDevices.map((device) => (
              <option key={device} value={device}>
                {device}
              </option>
            ))}
          </select>
        </label>

        <div className="rounded-md border border-voxx-line bg-voxx-panel p-5">
          <p className="text-sm text-voxx-muted">Wake word</p>
          <button
            className="mt-3 rounded-md bg-voxx-accent px-4 py-2 text-sm font-semibold text-black"
            onClick={() => updatePreference("wakeWordEnabled", !preferences.wakeWordEnabled)}
          >
            {preferences.wakeWordEnabled ? "Disable VOX" : "Enable VOX"}
          </button>
        </div>

        <div className="rounded-md border border-voxx-line bg-voxx-panel p-5">
          <p className="text-sm text-voxx-muted">Sound feedback</p>
          <label className="mt-3 flex items-center gap-3">
            <input
              type="checkbox"
              checked={preferences.soundEnabled}
              onChange={(event) => updatePreference("soundEnabled", event.currentTarget.checked)}
            />
            <span>Soft clicks on start and stop</span>
          </label>
          {preferences.soundEnabled ? (
            <input
              className="mt-4 w-full"
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={preferences.soundVolume}
              onChange={(event) => updatePreference("soundVolume", Number(event.currentTarget.value))}
            />
          ) : null}
        </div>

        <div className="rounded-md border border-voxx-line bg-voxx-panel p-5">
          <p className="text-sm text-voxx-muted">Diagnostics</p>
          <button className="mt-3 rounded-md border border-voxx-line px-4 py-2 text-sm" onClick={onRefresh}>
            Refresh checks
          </button>
          <div className="mt-4 grid gap-2">
            {diagnostics.map((diagnostic) => (
              <div key={diagnostic.name} className="flex items-center justify-between gap-4 rounded-md bg-voxx-panel-soft p-3 text-sm">
                <span>{diagnostic.name}</span>
                <span className={diagnostic.ok ? "text-voxx-accent" : "text-voxx-warning"}>{diagnostic.detail}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

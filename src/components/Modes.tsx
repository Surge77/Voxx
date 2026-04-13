import { api } from "../lib/tauri";
import { modeDescriptions, modeLabels, modes } from "../lib/modes";
import type { AppPreferences, DictationMode } from "../types";

type ModesProps = {
  activeMode: DictationMode;
  onModeChange: (preferences: AppPreferences) => void;
};

export function Modes({ activeMode, onModeChange }: ModesProps) {
  async function selectMode(mode: DictationMode) {
    const preferences = await api.setActiveMode(mode);
    onModeChange(preferences);
  }

  return (
    <section className="grid gap-6">
      <div>
        <p className="text-sm uppercase text-voxx-accent">Modes</p>
        <h2 className="mt-2 text-3xl font-semibold">Dictation behavior</h2>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        {modes.map((mode) => (
          <button
            key={mode}
            type="button"
            className={`rounded-md border p-5 text-left transition-colors ${
              activeMode === mode
                ? "border-voxx-accent bg-voxx-accent text-black"
                : "border-voxx-line bg-voxx-panel hover:bg-voxx-panel-soft"
            }`}
            onClick={() => selectMode(mode)}
          >
            <h3 className="text-xl font-semibold">{modeLabels[mode]}</h3>
            <p className={`mt-3 text-sm ${activeMode === mode ? "text-black" : "text-voxx-muted"}`}>{modeDescriptions[mode]}</p>
          </button>
        ))}
      </div>
      <div className="rounded-md border border-voxx-line bg-voxx-panel p-5">
        <h3 className="font-semibold">Context-aware rules</h3>
        <p className="mt-2 text-sm text-voxx-muted">
          IDEs switch to Code, terminals switch to Command, compose windows switch to Email, and everything else uses General.
        </p>
      </div>
    </section>
  );
}

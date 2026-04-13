import { api } from "../lib/tauri";
import type { DictionaryEntry } from "../types";

type DictionaryProps = {
  entries: DictionaryEntry[];
  onRefresh: () => Promise<void>;
};

export function Dictionary({ entries, onRefresh }: DictionaryProps) {
  return (
    <section className="grid gap-6">
      <div>
        <p className="text-sm uppercase text-voxx-accent">Dictionary</p>
        <h2 className="mt-2 text-3xl font-semibold">Correction memory</h2>
        <p className="mt-2 max-w-2xl text-voxx-muted">
          The dictionary starts empty and grows when you save corrections from history.
        </p>
      </div>
      <div className="grid gap-3">
        {entries.length === 0 ? (
          <div className="rounded-md border border-voxx-line bg-voxx-panel p-6 text-voxx-muted">No corrections saved yet.</div>
        ) : null}
        {entries.map((entry) => (
          <div key={entry.id} className="flex flex-col justify-between gap-3 rounded-md border border-voxx-line bg-voxx-panel p-4 md:flex-row md:items-center">
            <div>
              <p className="text-sm text-voxx-muted">Replace</p>
              <p>
                <span className="text-voxx-danger">{entry.wrong}</span>
                <span className="px-2 text-voxx-muted">with</span>
                <span className="text-voxx-accent">{entry.right}</span>
              </p>
            </div>
            <button
              className="rounded-md border border-voxx-danger px-3 py-2 text-sm text-voxx-danger"
              onClick={async () => {
                await api.deleteDictionaryEntry(entry.id);
                await onRefresh();
              }}
            >
              Delete
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}

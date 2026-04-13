import { useMemo, useState } from "react";
import { filterHistory } from "../lib/db";
import { api } from "../lib/tauri";
import type { HistoryEntry } from "../types";

type DashboardProps = {
  history: HistoryEntry[];
  onRefresh: () => Promise<void>;
};

export function Dashboard({ history, onRefresh }: DashboardProps) {
  const [query, setQuery] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [draft, setDraft] = useState("");

  const visibleHistory = useMemo(() => filterHistory(history, query), [history, query]);

  async function startEdit(entry: HistoryEntry) {
    setEditingId(entry.id);
    setDraft(entry.processedText);
  }

  async function saveEdit(entry: HistoryEntry) {
    await api.updateHistoryEntry(entry.id, draft);
    if (draft.trim() && draft.trim() !== entry.processedText.trim()) {
      await api.saveCorrection(entry.id, entry.processedText, draft);
    }
    setEditingId(null);
    setDraft("");
    await onRefresh();
  }

  return (
    <section className="flex flex-col gap-6">
      <div className="flex flex-col gap-3">
        <p className="text-sm uppercase text-voxx-accent">Dashboard</p>
        <h2 className="text-3xl font-semibold">Transcription history</h2>
        <input
          className="w-full rounded-md border border-voxx-line bg-voxx-panel px-4 py-3 text-voxx-text placeholder:text-voxx-muted"
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
          placeholder="Search text, mode, or raw transcript"
        />
      </div>

      <div className="grid gap-4">
        {visibleHistory.length === 0 ? (
          <div className="rounded-md border border-voxx-line bg-voxx-panel p-6 text-voxx-muted">
            No transcriptions yet. Hold Ctrl+Space, release, then check this feed.
          </div>
        ) : null}

        {visibleHistory.map((entry) => (
          <article key={entry.id} className="rounded-md border border-voxx-line bg-voxx-panel p-5">
            <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-sm uppercase text-voxx-accent">{entry.mode}</p>
                <p className="text-sm text-voxx-muted">{new Date(entry.createdAt).toLocaleString()}</p>
              </div>
              <div className="flex flex-wrap gap-2">
                <button className="rounded-md border border-voxx-line px-3 py-2 text-sm" onClick={() => api.copyEntry(entry.id)}>
                  Copy
                </button>
                <button className="rounded-md border border-voxx-line px-3 py-2 text-sm" onClick={() => api.repasteEntry(entry.id)}>
                  Re-paste
                </button>
                <button className="rounded-md border border-voxx-line px-3 py-2 text-sm" onClick={() => startEdit(entry)}>
                  Edit
                </button>
                <button
                  className="rounded-md border border-voxx-danger px-3 py-2 text-sm text-voxx-danger"
                  onClick={async () => {
                    await api.deleteHistoryEntry(entry.id);
                    await onRefresh();
                  }}
                >
                  Delete
                </button>
              </div>
            </div>

            {editingId === entry.id ? (
              <div className="flex flex-col gap-3">
                <textarea
                  className="min-h-28 rounded-md border border-voxx-line bg-voxx-panel-soft p-3 text-voxx-text"
                  value={draft}
                  onChange={(event) => setDraft(event.currentTarget.value)}
                />
                <div className="flex gap-2">
                  <button className="rounded-md bg-voxx-accent px-3 py-2 text-sm font-semibold text-black" onClick={() => saveEdit(entry)}>
                    Save correction
                  </button>
                  <button className="rounded-md border border-voxx-line px-3 py-2 text-sm" onClick={() => setEditingId(null)}>
                    Cancel
                  </button>
                </div>
              </div>
            ) : (
              <p className="text-base leading-7">{entry.processedText}</p>
            )}

            <details className="mt-4 text-sm text-voxx-muted">
              <summary className="cursor-pointer">Raw Whisper text</summary>
              <p className="mt-2 rounded-md bg-voxx-panel-soft p-3">{entry.rawText}</p>
            </details>
          </article>
        ))}
      </div>
    </section>
  );
}

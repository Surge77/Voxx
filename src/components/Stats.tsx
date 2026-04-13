import type { HistoryEntry } from "../types";

type StatsProps = {
  history: HistoryEntry[];
};

export function Stats({ history }: StatsProps) {
  const words = history.reduce((total, entry) => total + entry.processedText.trim().split(/\s+/).filter(Boolean).length, 0);
  const modeCounts = history.reduce<Record<string, number>>((counts, entry) => {
    counts[entry.mode] = (counts[entry.mode] ?? 0) + 1;
    return counts;
  }, {});
  const mostUsedMode = Object.entries(modeCounts).sort((a, b) => b[1] - a[1])[0]?.[0] ?? "general";

  return (
    <section className="grid gap-6">
      <div>
        <p className="text-sm uppercase text-voxx-accent">Stats</p>
        <h2 className="mt-2 text-3xl font-semibold">Dictation activity</h2>
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        <StatCard label="Total words" value={String(words)} />
        <StatCard label="Entries" value={String(history.length)} />
        <StatCard label="Most used mode" value={mostUsedMode} />
      </div>
    </section>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-voxx-line bg-voxx-panel p-5">
      <p className="text-sm text-voxx-muted">{label}</p>
      <p className="mt-3 text-3xl font-semibold capitalize">{value}</p>
    </div>
  );
}

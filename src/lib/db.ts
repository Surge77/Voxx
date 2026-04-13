import type { HistoryEntry } from "../types";

export function filterHistory(entries: HistoryEntry[], query: string): HistoryEntry[] {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return entries;
  }

  return entries.filter((entry) => {
    return (
      entry.processedText.toLowerCase().includes(normalized) ||
      entry.rawText.toLowerCase().includes(normalized) ||
      entry.mode.toLowerCase().includes(normalized)
    );
  });
}


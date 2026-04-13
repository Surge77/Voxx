import { describe, expect, it } from "vitest";
import { filterHistory } from "./db";
import type { HistoryEntry } from "../types";

const entries: HistoryEntry[] = [
  {
    id: 1,
    rawText: "react query",
    processedText: "React Query",
    mode: "code",
    createdAt: "2026-04-13T00:00:00Z",
    durationMs: 500
  },
  {
    id: 2,
    rawText: "hello there",
    processedText: "Hello there.",
    mode: "general",
    createdAt: "2026-04-13T00:00:01Z",
    durationMs: 600
  }
];

describe("filterHistory", () => {
  it("returns all entries when the query is empty", () => {
    expect(filterHistory(entries, "")).toHaveLength(2);
  });

  it("matches processed text, raw text, and mode", () => {
    expect(filterHistory(entries, "query")).toHaveLength(1);
    expect(filterHistory(entries, "hello")).toHaveLength(1);
    expect(filterHistory(entries, "code")).toHaveLength(1);
  });
});


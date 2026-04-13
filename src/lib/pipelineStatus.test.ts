import { describe, expect, it } from "vitest";
import { statusMessageForPipelineResult } from "./pipelineStatus";
import type { PipelineResult } from "../types";

const baseResult: PipelineResult = {
  ignored: false,
  rawText: "react query",
  processedText: "React Query",
  mode: "general",
  durationMs: 700
};

describe("statusMessageForPipelineResult", () => {
  it("surfaces pipeline errors instead of reporting a paste", () => {
    expect(statusMessageForPipelineResult({ ...baseResult, error: "Transcription failed" })).toBe("Transcription failed");
  });

  it("reports ignored short holds", () => {
    expect(statusMessageForPipelineResult({ ...baseResult, ignored: true })).toBe("Ignored short hold");
  });
});

import type { PipelineResult } from "../types";

export function statusMessageForPipelineResult(result: PipelineResult): string {
  if (result.error) {
    return result.error;
  }

  if (result.ignored) {
    return "Ignored short hold";
  }

  return "Pasted transcription";
}

import type { DictationMode } from "../types";

export const modeLabels: Record<DictationMode, string> = {
  general: "General",
  code: "Code",
  command: "Command",
  email: "Email"
};

export const modeDescriptions: Record<DictationMode, string> = {
  general: "Messages, notes, and natural writing.",
  code: "Technical writing with code-aware formatting.",
  command: "Literal commands, config, and terminal text.",
  email: "Structured email and chat messages."
};

export const modes = Object.keys(modeLabels) as DictationMode[];


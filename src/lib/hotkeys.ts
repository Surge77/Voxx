import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { api, isTauriRuntime } from "./tauri";
import type { PipelineResult } from "../types";

type HotkeyHandlers = {
  onRecordingStart: () => void;
  onProcessingStart: () => void;
  onError: (message: string) => void;
  onDone: (result: PipelineResult) => Promise<void>;
};

export async function registerRecordingHotkey(handlers: HotkeyHandlers) {
  if (!isTauriRuntime()) {
    return;
  }

  await unregisterAll();
  await register("CommandOrControl+Space", async (event) => {
    try {
      if (event.state === "Pressed") {
        handlers.onRecordingStart();
        await api.startRecording();
        return;
      }

      if (event.state === "Released") {
        handlers.onProcessingStart();
        const result = await api.stopRecordingAndProcess();
        await handlers.onDone(result);
      }
    } catch (error) {
      handlers.onError(error instanceof Error ? error.message : "Hotkey pipeline failed");
    }
  });
}

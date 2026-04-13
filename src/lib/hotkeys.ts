import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { api } from "./tauri";

type HotkeyHandlers = {
  onRecordingStart: () => void;
  onProcessingStart: () => void;
  onError: (message: string) => void;
  onDone: () => Promise<void>;
};

export async function registerRecordingHotkey(handlers: HotkeyHandlers) {
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
        await api.stopRecordingAndProcess();
        await handlers.onDone();
      }
    } catch (error) {
      handlers.onError(error instanceof Error ? error.message : "Hotkey pipeline failed");
    }
  });
}


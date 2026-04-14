import type { RecordingState } from "../types";
import { VoiceVisualizerCapsule } from "./VoiceVisualizerCapsule";

type FloatingOverlayProps = {
  state: RecordingState;
  message: string;
};

export function FloatingOverlay({ state, message }: FloatingOverlayProps) {
  if (state !== "recording") {
    return null;
  }

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-6 z-50 flex justify-center px-4">
      <div
        aria-label={message}
        aria-live="polite"
        role="status"
      >
        <VoiceVisualizerCapsule />
      </div>
    </div>
  );
}

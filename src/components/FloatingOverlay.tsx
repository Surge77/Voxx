import type { RecordingState } from "../types";

type FloatingOverlayProps = {
  state: RecordingState;
  message: string;
};

export function FloatingOverlay({ state, message }: FloatingOverlayProps) {
  if (state === "idle") {
    return null;
  }

  const bars = Array.from({ length: 12 }, (_, index) => index);
  const visualizerStateClass = state === "processing" ? "voxx-voice-capsule-processing" : "voxx-voice-capsule-recording";

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-6 z-50 flex justify-center px-4">
      <div
        aria-label={message}
        aria-live="polite"
        className={`voxx-voice-capsule ${visualizerStateClass}`}
        role="status"
      >
        <div className="voxx-voice-bars" aria-hidden="true">
          {bars.map((bar) => (
            <span key={bar} className="voxx-voice-bar" />
          ))}
        </div>
      </div>
    </div>
  );
}

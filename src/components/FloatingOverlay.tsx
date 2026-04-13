import type { RecordingState } from "../types";

type FloatingOverlayProps = {
  state: RecordingState;
  message: string;
};

export function FloatingOverlay({ state, message }: FloatingOverlayProps) {
  if (state !== "recording") {
    return null;
  }

  const bars = Array.from({ length: 12 }, (_, index) => index);

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-6 z-50 flex justify-center px-4">
      <div
        aria-label={message}
        aria-live="polite"
        className="voxx-voice-capsule"
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

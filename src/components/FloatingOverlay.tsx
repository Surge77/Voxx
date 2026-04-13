import type { RecordingState } from "../types";

type FloatingOverlayProps = {
  state: RecordingState;
  message: string;
};

export function FloatingOverlay({ state, message }: FloatingOverlayProps) {
  if (state === "idle") {
    return null;
  }

  const bars = ["h-5", "h-7", "h-4", "h-9", "h-6", "h-8", "h-5"];

  return (
    <div className="fixed inset-x-0 bottom-8 z-50 flex justify-center px-4">
      <div className="flex min-h-14 max-w-[min(92vw,720px)] items-center gap-4 rounded-md border border-voxx-line bg-voxx-panel px-5 py-3 shadow-2xl">
        <div className="flex h-9 w-9 items-center justify-center rounded-md bg-voxx-accent text-sm font-bold text-black shadow-[0_0_20px_rgba(0,191,165,0.45)]">
          VX
        </div>
        <div className="flex h-9 items-center gap-1" aria-label="Audio visualizer">
          {bars.map((height, index) => (
            <span
              key={`${height}-${index}`}
              className={`w-1.5 rounded-full bg-voxx-accent ${height} ${state === "recording" ? "animate-pulse" : ""}`}
            />
          ))}
        </div>
        <div className="min-w-0">
          <p className="truncate text-sm font-medium">{message}</p>
          <p className="truncate text-xs text-voxx-muted">{state === "processing" ? "Formatting with phi4-mini" : "Listening through Ctrl+Space"}</p>
        </div>
      </div>
    </div>
  );
}

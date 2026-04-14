export function VoiceVisualizerCapsule() {
  const bars = Array.from({ length: 12 }, (_, index) => index);

  return (
    <div className="voxx-voice-capsule" role="status" aria-label="Recording">
      <div className="voxx-voice-bars" aria-hidden="true">
        {bars.map((bar) => (
          <span key={bar} className="voxx-voice-bar" />
        ))}
      </div>
    </div>
  );
}


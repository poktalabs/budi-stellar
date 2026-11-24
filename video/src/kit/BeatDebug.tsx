import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { BeatGrid } from "./beat";

// Review-only overlay: bar.beat counter plus a flash on every downbeat, to check that moves
// land on the grid. Never ship with it on (master brief: no chrome unless SHOW_CHROME).
export const BeatDebug: React.FC<{ grid: BeatGrid; beatsPerBar?: number }> = ({ grid, beatsPerBar = 4 }) => {
  const frame = useCurrentFrame();
  const b = Math.floor(grid.beatAt(frame));
  const sinceBeat = frame - grid.beat(b);
  const flash = b >= 0 && sinceBeat < 4;
  const label = b < 0 ? "pre" : `${Math.floor(b / beatsPerBar) + 1}.${(b % beatsPerBar) + 1}`;
  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      <div
        style={{
          position: "absolute",
          right: 24,
          bottom: 24,
          padding: "8px 14px",
          borderRadius: 8,
          fontFamily: "monospace",
          fontSize: 22,
          color: "#fff",
          background: flash ? (b % beatsPerBar === 0 ? "#e5484d" : "#3e63dd") : "rgba(0,0,0,0.6)",
        }}
      >
        {label} f{frame}
      </div>
    </AbsoluteFill>
  );
};

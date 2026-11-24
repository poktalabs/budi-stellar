import React from "react";
import { EasingFunction, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { EASE } from "./tokens";

// Pointer that travels between keyframes (absolute px in its parent: world or screen) and
// clicks with a press + ripple. Moves use the destination key's ease (default EASE.inOut).
export type CursorKey = { frame: number; x: number; y: number; click?: boolean; ease?: EasingFunction };

const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

export const Cursor: React.FC<{ keys: CursorKey[]; size?: number; color?: string; ripple?: string }> = ({
  keys,
  size = 40,
  color = "#ffffff",
  ripple = "rgba(255,255,255,0.5)",
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const k = [...keys].sort((a, b) => a.frame - b.frame);
  let x = k[0].x;
  let y = k[0].y;
  const i = k.findIndex((key) => key.frame > frame);
  if (i === -1) {
    x = k[k.length - 1].x;
    y = k[k.length - 1].y;
  } else if (i > 0) {
    const t = interpolate(frame, [k[i - 1].frame, k[i].frame], [0, 1], { ...clamp, easing: k[i].ease ?? EASE.inOut });
    x = k[i - 1].x + (k[i].x - k[i - 1].x) * t;
    y = k[i - 1].y + (k[i].y - k[i - 1].y) * t;
  }
  const rippleLen = Math.round(fps * 0.4);
  const press = Math.round(fps * 0.1);
  const clicks = k.filter((key) => key.click);
  const active = clicks.find((c) => frame >= c.frame && frame < c.frame + rippleLen);
  const pressScale = clicks.reduce(
    (s, c) => Math.min(s, interpolate(frame, [c.frame - press, c.frame, c.frame + press], [1, 0.82, 1], clamp)),
    1,
  );
  const rp = active ? (frame - active.frame) / rippleLen : 0;
  return (
    <>
      {active && (
        <div
          style={{
            position: "absolute",
            left: active.x,
            top: active.y,
            width: size * 2.4,
            height: size * 2.4,
            marginLeft: -size * 1.2,
            marginTop: -size * 1.2,
            borderRadius: "50%",
            border: `3px solid ${ripple}`,
            scale: String(0.2 + 0.8 * EASE.out(rp)),
            opacity: 1 - rp,
          }}
        />
      )}
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        style={{
          position: "absolute",
          left: x,
          top: y,
          marginLeft: -size * 0.2,
          marginTop: -size * 0.12,
          scale: String(pressScale),
          transformOrigin: "20% 12%",
          filter: "drop-shadow(0 4px 8px rgba(0,0,0,0.45))",
          overflow: "visible",
        }}
      >
        <path d="M5 3 L19 12.5 L12.2 13.6 L8.6 20 Z" fill={color} stroke="#000" strokeWidth={1.4} strokeLinejoin="round" />
      </svg>
    </>
  );
};

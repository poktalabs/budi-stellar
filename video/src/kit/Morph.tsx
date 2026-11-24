import React from "react";
import { EasingFunction, interpolate, interpolateColors, useCurrentFrame } from "remotion";
import { EASE } from "./tokens";

// One shape becomes another (a button grows into a panel, a card into the end card): position,
// size, radius and fill interpolate together. Swap or crossfade the content yourself, ideally
// on the beat the morph lands.
export type MorphBox = { x: number; y: number; w: number; h: number; radius?: number; bg?: string };

export const Morph: React.FC<{
  from: MorphBox;
  to: MorphBox;
  at: number;
  duration: number;
  ease?: EasingFunction;
  style?: React.CSSProperties;
  children?: React.ReactNode;
}> = ({ from, to, at, duration, ease = EASE.inOut, style, children }) => {
  const frame = useCurrentFrame();
  const t = interpolate(frame, [at, at + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: ease,
  });
  const l = (a: number, b: number) => a + (b - a) * t;
  return (
    <div
      style={{
        position: "absolute",
        left: l(from.x, to.x),
        top: l(from.y, to.y),
        width: l(from.w, to.w),
        height: l(from.h, to.h),
        borderRadius: l(from.radius ?? 0, to.radius ?? 0),
        background: interpolateColors(t, [0, 1], [from.bg ?? "transparent", to.bg ?? from.bg ?? "transparent"]),
        overflow: "hidden",
        ...style,
      }}
    >
      {children}
    </div>
  );
};

import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { EASE, SPRING } from "./kit";
import { C, F, K, nbSm, RADIUS } from "./theme";

export const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

export const ramp = (frame: number, a: number, b: number, ease = EASE.out) =>
  interpolate(frame, [a, b], [0, 1], { ...clamp, easing: ease });

/** Calm entrance: a short rise and a fade (no shake, no bursts). */
export const useEnter = (at: number, dist = 36) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const p = spring({ frame: frame - at, fps, config: SPRING.soft });
  return { opacity: Math.min(1, p * 1.4), translate: `0 ${(1 - p) * dist}px` } as React.CSSProperties;
};

/** Scene exit: fades the whole scene in its last frames so cuts never pop. */
export const useExit = (dur: number, len = 10) => {
  const frame = useCurrentFrame();
  return { opacity: 1 - ramp(frame, dur - len, dur, EASE.in) };
};

export const Badge: React.FC<{ tone?: "default" | "info" | "success" | "violet" | "gold"; children: React.ReactNode; style?: React.CSSProperties }> = ({
  tone = "default",
  children,
  style,
}) => {
  const fill = { default: C.paperDark, info: C.info, success: C.success, violet: C.violet, gold: C.gold }[tone];
  const dark = tone === "default" || tone === "gold";
  return (
    <span
      style={{
        display: "inline-block",
        padding: `${2 * K}px ${8 * K}px`,
        borderRadius: RADIUS.badge,
        ...nbSm(),
        background: fill,
        color: dark ? C.ink : "#fff",
        fontFamily: F.body,
        fontSize: 26,
        fontWeight: 400,
        whiteSpace: "nowrap",
        ...style,
      }}
    >
      {children}
    </span>
  );
};

/** The coach's hand: Shantell 600, violet, tilted a few degrees. One note per scene at most. */
export const Hand: React.FC<{ children: React.ReactNode; style?: React.CSSProperties }> = ({ children, style }) => (
  <div style={{ fontFamily: F.hand, fontWeight: 600, color: C.violet, fontSize: 44, rotate: "-3deg", ...style }}>{children}</div>
);

/** Dotted whiteboard grid under everything (paper-dark dots on paper). */
export const DotGrid: React.FC = () => (
  <div
    style={{
      position: "absolute",
      inset: 0,
      backgroundColor: C.paper,
      backgroundImage: `radial-gradient(${C.paperDark} 3px, transparent 3.5px)`,
      backgroundSize: "36px 36px",
    }}
  />
);

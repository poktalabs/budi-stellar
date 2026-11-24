import React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { KineticLine } from "./KineticLine";
import { defaultTheme, EASE, SPRING, Theme } from "./tokens";

// Designed end card: mark, name (kinetic), tagline, CTA pill on the POP spring. Beats: mark at
// `at`, name half a second later, tagline and CTA after it. Hold it at least 1.5 s.
export const EndCard: React.FC<{
  at?: number;
  title: string;
  accent?: number;
  tagline?: string;
  cta?: string;
  mark?: React.ReactNode;
  theme?: Theme;
}> = ({ at = 0, title, accent, tagline, cta, mark, theme = defaultTheme }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const half = Math.round(fps * 0.5);
  const markP = spring({ frame: frame - at, fps, config: SPRING.pop });
  const tagO = interpolate(frame, [at + half * 2, at + half * 3], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASE.out,
  });
  const ctaP = spring({ frame: frame - at - half * 3, fps, config: SPRING.pop });
  return (
    <AbsoluteFill
      style={{ alignItems: "center", justifyContent: "center", gap: 36, background: theme.bg, fontFamily: theme.fonts.body }}
    >
      {mark && <div style={{ scale: String(markP), opacity: Math.min(1, markP) }}>{mark}</div>}
      <KineticLine text={title} at={at + half} accent={accent} size={140} theme={theme} style={{ justifyContent: "center" }} />
      {tagline && (
        <div style={{ color: theme.muted, fontSize: 44, opacity: tagO, translate: `0 ${(1 - tagO) * 16}px` }}>{tagline}</div>
      )}
      {cta && (
        <div
          style={{
            marginTop: 12,
            padding: "22px 48px",
            borderRadius: 999,
            background: theme.accent,
            color: "#fff",
            fontSize: 36,
            fontWeight: 600,
            scale: String(ctaP),
            opacity: Math.min(1, ctaP * 2),
          }}
        >
          {cta}
        </div>
      )}
    </AbsoluteFill>
  );
};

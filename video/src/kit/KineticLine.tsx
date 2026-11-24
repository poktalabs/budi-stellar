import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";
import { defaultTheme, SPRING, Theme } from "./tokens";

// Headline that rises word by word out of a mask. One accent word per line (master brief rule).
// `at` is the start frame; `stagger` defaults to 0.06 s per word.
export const KineticLine: React.FC<{
  text: string;
  at?: number;
  stagger?: number;
  accent?: number;
  size?: number;
  weight?: number;
  theme?: Theme;
  style?: React.CSSProperties;
}> = ({ text, at = 0, stagger, accent, size = 120, weight = 800, theme = defaultTheme, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const step = stagger ?? Math.round(fps * 0.06);
  const words = text.split(" ");
  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        columnGap: "0.25em",
        fontFamily: theme.fonts.display,
        fontSize: size,
        fontWeight: weight,
        letterSpacing: "-0.03em",
        lineHeight: 1.05,
        color: theme.ink,
        ...style,
      }}
    >
      {words.map((w, i) => {
        const p = spring({ frame: frame - at - i * step, fps, config: SPRING.soft });
        return (
          <span key={i} style={{ display: "inline-block", overflow: "hidden", paddingBottom: "0.08em" }}>
            <span
              style={{
                display: "inline-block",
                translate: `0 ${(1 - p) * 110}%`,
                color: i === accent ? theme.accent : undefined,
              }}
            >
              {w}
            </span>
          </span>
        );
      })}
    </div>
  );
};

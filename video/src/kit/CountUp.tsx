import React from "react";
import { EasingFunction, interpolate, useCurrentFrame } from "remotion";
import { EASE } from "./tokens";
import { Reserve } from "./Reserve";

const defaultFormat = (n: number) => Math.round(n).toLocaleString("en-US");

// Number that counts from `from` to `to` over `duration` frames starting at `at`. Tabular
// numerals plus a reserved width, so the layout never shifts while counting.
export const CountUp: React.FC<{
  to: number;
  from?: number;
  at?: number;
  duration: number;
  format?: (n: number) => string;
  ease?: EasingFunction;
  style?: React.CSSProperties;
}> = ({ to, from = 0, at = 0, duration, format = defaultFormat, ease = EASE.out, style }) => {
  const frame = useCurrentFrame();
  const v = interpolate(frame, [at, at + duration], [from, to], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: ease,
  });
  const widest = [format(from), format(to)].sort((a, b) => b.length - a.length)[0];
  return (
    <span style={{ fontVariantNumeric: "tabular-nums", ...style }}>
      <Reserve final={widest} align="right">
        {format(v)}
      </Reserve>
    </span>
  );
};

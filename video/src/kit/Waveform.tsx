import React from "react";
import { noise2D } from "@remotion/noise";
import { useAudioData, visualizeAudio } from "@remotion/media-utils";
import { useCurrentFrame, useVideoConfig } from "remotion";

// Voice bars for rebuilt audio UI (a speaker's waveform, a mic meter). Levels are 0..1 per bar,
// computed by the caller: `useAudioLevels` for a real clip, `syntheticLevels` for a stand-in.
// Bars grow from the centre line, so silence reads as a row of dots, never as nothing.

/** Levels from a real audio file at the current frame (`at` = the frame the clip starts). */
export const useAudioLevels = (src: string, bars: number, at = 0, gain = 3.2): number[] => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const audio = useAudioData(src);
  if (!audio || frame < at) return new Array(bars).fill(0);
  const raw = visualizeAudio({ fps, frame: frame - at, audioData: audio, numberOfSamples: 64, smoothing: true });
  // Voice energy sits in the low bins: spread the first `bars` bins and mirror for symmetry.
  const half = Math.ceil(bars / 2);
  const side = raw.slice(1, half + 1).map((v) => Math.min(1, Math.sqrt(v) * gain));
  return [...[...side].reverse(), ...side].slice(0, bars);
};

/** Speech-like stand-in levels: smooth noise, scaled by `energy` (0 = silent, 1 = talking). */
export const syntheticLevels = (frame: number, bars: number, energy: number, seed = "voice"): number[] =>
  Array.from({ length: bars }, (_, i) => {
    const n = (noise2D(seed, i * 0.35, frame * 0.09) + 1) / 2;
    const envelope = 1 - Math.abs(i - (bars - 1) / 2) / bars;
    return Math.max(0, Math.min(1, n * envelope * 1.5 * energy));
  });

export const Waveform: React.FC<{
  levels: number[];
  height: number;
  barWidth?: number;
  gap?: number;
  color?: string;
  style?: React.CSSProperties;
}> = ({ levels, height, barWidth = 10, gap = 6, color = "currentColor", style }) => (
  <div style={{ display: "flex", alignItems: "center", gap, height, ...style }}>
    {levels.map((l, i) => (
      <div
        key={i}
        style={{ width: barWidth, height: Math.max(barWidth, l * height), borderRadius: barWidth, background: color }}
      />
    ))}
  </div>
);

// Mic level pill: a mic glyph and a short bar meter, for "recording" states.
export const MicMeter: React.FC<{
  level: number;
  bars?: number;
  color?: string;
  idle?: string;
  size?: number;
  style?: React.CSSProperties;
}> = ({ level, bars = 12, color = "#15803d", idle = "rgba(0,0,0,0.12)", size = 40, style }) => (
  <div style={{ display: "flex", alignItems: "center", gap: size * 0.4, ...style }}>
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.4} strokeLinecap="round">
      <rect x="8.5" y="2.5" width="7" height="12" rx="3.5" />
      <path d="M5 11 a7 7 0 0 0 14 0 M12 18 v3.5" />
    </svg>
    <div style={{ display: "flex", alignItems: "flex-end", gap: size * 0.12, height: size }}>
      {Array.from({ length: bars }, (_, i) => {
        const on = i < Math.round(level * bars);
        return (
          <div
            key={i}
            style={{
              width: size * 0.18,
              height: size * (0.3 + (0.7 * i) / bars),
              borderRadius: 3,
              background: on ? color : idle,
            }}
          />
        );
      })}
    </div>
  </div>
);

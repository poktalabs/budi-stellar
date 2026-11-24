import React from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { Reserve } from "./Reserve";

// Types `text` from frame `at` at `cps` characters per second. The full text is reserved, so
// the line never reflows. The caret is solid while typing and blinks (0.5 s) when idle.
export const Typewriter: React.FC<{
  text: string;
  at?: number;
  cps?: number;
  caret?: boolean;
  caretColor?: string;
  style?: React.CSSProperties;
}> = ({ text, at = 0, cps = 30, caret = true, caretColor = "currentColor", style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const n = Math.max(0, Math.min(text.length, Math.floor(((frame - at) * cps) / fps)));
  const typing = frame >= at && n < text.length;
  const caretOn = typing || Math.floor(frame / (fps * 0.5)) % 2 === 0;
  const bar = (
    <span
      style={{
        display: "inline-block",
        width: "0.08em",
        height: "1em",
        marginLeft: "0.04em",
        verticalAlign: "-0.12em",
        background: caretColor,
        opacity: caret && caretOn ? 1 : 0,
      }}
    />
  );
  return (
    <span style={style}>
      <Reserve
        final={
          <>
            {text}
            {bar}
          </>
        }
      >
        {text.slice(0, n)}
        {bar}
      </Reserve>
    </span>
  );
};

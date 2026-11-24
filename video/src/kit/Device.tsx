import React from "react";
import { defaultTheme, Theme } from "./tokens";

// Device frames for rebuilt UI (never screenshots): the content area is exactly
// width x height, so UI components can be laid out against a known size. Frames add their
// bezel/toolbar outside it. Position with `style` (e.g. absolute left/top in world space).

type Common = {
  width: number;
  height: number;
  theme?: Theme;
  style?: React.CSSProperties;
  children?: React.ReactNode;
};

const shadow = "0 40px 120px rgba(0,0,0,0.45), 0 8px 24px rgba(0,0,0,0.25)";

const Dots: React.FC = () => (
  <div style={{ display: "flex", gap: 8 }}>
    {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
      <div key={c} style={{ width: 12, height: 12, borderRadius: 6, background: c }} />
    ))}
  </div>
);

export const BrowserFrame: React.FC<Common & { url?: string }> = ({
  width,
  height,
  theme = defaultTheme,
  url,
  style,
  children,
}) => (
  <div
    style={{
      width,
      borderRadius: theme.radius,
      overflow: "hidden",
      background: theme.surface,
      border: `1px solid ${theme.line}`,
      boxShadow: shadow,
      ...style,
    }}
  >
    <div
      style={{
        height: 52,
        display: "flex",
        alignItems: "center",
        gap: 20,
        padding: "0 18px",
        borderBottom: `1px solid ${theme.line}`,
      }}
    >
      <Dots />
      {url !== undefined && (
        <div
          style={{
            flex: 1,
            maxWidth: 560,
            margin: "0 auto",
            height: 30,
            borderRadius: 8,
            background: theme.bg,
            color: theme.muted,
            fontFamily: theme.fonts.body,
            fontSize: 15,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          {url}
        </div>
      )}
    </div>
    <div style={{ width, height, position: "relative", overflow: "hidden", background: theme.bg }}>
      {children}
    </div>
  </div>
);

export const WindowFrame: React.FC<Common & { title?: string }> = ({
  width,
  height,
  theme = defaultTheme,
  title,
  style,
  children,
}) => (
  <div
    style={{
      width,
      borderRadius: theme.radius,
      overflow: "hidden",
      background: theme.surface,
      border: `1px solid ${theme.line}`,
      boxShadow: shadow,
      ...style,
    }}
  >
    <div style={{ height: 40, display: "flex", alignItems: "center", padding: "0 16px", position: "relative" }}>
      <Dots />
      {title && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: theme.muted,
            fontFamily: theme.fonts.body,
            fontSize: 14,
            fontWeight: 500,
          }}
        >
          {title}
        </div>
      )}
    </div>
    <div style={{ width, height, position: "relative", overflow: "hidden", background: theme.bg }}>
      {children}
    </div>
  </div>
);

// Phone: content area is the screen (e.g. 390 x 844 at 1x, or scale the whole frame with CSS).
export const PhoneFrame: React.FC<Common & { island?: boolean }> = ({
  width,
  height,
  theme = defaultTheme,
  island = true,
  style,
  children,
}) => {
  const bezel = Math.round(width * 0.035);
  const r = Math.round(width * 0.14);
  return (
    <div
      style={{
        width: width + bezel * 2,
        height: height + bezel * 2,
        padding: bezel,
        borderRadius: r + bezel,
        background: "#050507",
        boxShadow: `${shadow}, inset 0 0 0 2px #2b2b30`,
        boxSizing: "border-box",
        ...style,
      }}
    >
      <div
        style={{
          width,
          height,
          borderRadius: r,
          overflow: "hidden",
          position: "relative",
          background: theme.bg,
        }}
      >
        {children}
        {island && (
          <div
            style={{
              position: "absolute",
              top: Math.round(height * 0.014),
              left: "50%",
              width: Math.round(width * 0.32),
              height: Math.round(width * 0.09),
              transform: "translateX(-50%)",
              borderRadius: 999,
              background: "#000",
            }}
          />
        )}
      </div>
    </div>
  );
};

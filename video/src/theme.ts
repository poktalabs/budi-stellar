import type React from "react";
import { loadFont as loadBricolage } from "@remotion/google-fonts/BricolageGrotesque";
import { loadFont as loadRubik } from "@remotion/google-fonts/Rubik";
import { loadFont as loadShantell } from "@remotion/google-fonts/ShantellSans";
import { loadFont as loadMono } from "@remotion/google-fonts/JetBrainsMono";
import type { Theme } from "./kit";

// Budi tokens from budi-lat-monorepo/DESIGN.md: three type voices (Bricolage headings, Rubik reading,
// Shantell the coach's hand), one tone per hue, one job per color, ink and paper ~85% of the frame.
const display = loadBricolage("normal", { weights: ["600", "700", "800"], subsets: ["latin"] });
const body = loadRubik("normal", { weights: ["400", "500", "600", "700"], subsets: ["latin"] });
const hand = loadShantell("normal", { weights: ["600"], subsets: ["latin"] });
const mono = loadMono("normal", { weights: ["400", "600"], subsets: ["latin"] });

export const C = {
  ink: "#0a0a0a",
  charcoal: "#3a3a3a",
  paper: "#fff9f5",
  paperDark: "#fff0e8",
  panel: "#ffffff",
  magenta: "#c4177a", // acts: the CTA and the one payoff line
  violet: "#7c3aed", // second voice and the coach's hand
  gold: "#ffb800", // header band, Budi's body
  success: "#15803d",
  info: "#2563eb", // status ("Pronto")
};

export const F = { display: display.fontFamily, body: body.fontFamily, hand: hand.fontFamily, mono: mono.fontFamily };

export const theme: Theme = {
  bg: C.paper,
  surface: C.panel,
  ink: C.ink,
  muted: C.charcoal,
  line: C.ink,
  accent: C.magenta,
  fonts: { display: F.display, body: F.body, mono: F.mono },
  radius: 12,
};

// The brand is specified at web sizes; the video draws ~2x, so borders, shadows and radii scale with it.
export const K = 2;
export const RADIUS = { badge: 4 * K, button: 6 * K, card: 8 * K };

// Borders always ink. Shadows only on pressables and the one hero object per scene (zero blur).
export const nb = (): React.CSSProperties => ({ border: `${3 * K}px solid ${C.ink}` });
export const nbSm = (): React.CSSProperties => ({ border: `${2 * K}px solid ${C.ink}` });
export const raised = (): React.CSSProperties => ({ ...nb(), boxShadow: `${6 * K}px ${6 * K}px 0 0 ${C.ink}` });
export const press = (): React.CSSProperties => ({ ...nbSm(), boxShadow: `${3 * K}px ${3 * K}px 0 0 ${C.ink}` });

// The brand SVGs read these CSS variables.
export const cssVars = {
  "--color-ink": C.ink,
  "--color-paper": C.paper,
  "--color-panel": C.panel,
  "--color-magenta": C.magenta,
  "--color-violet": C.violet,
  "--color-gold": C.gold,
} as React.CSSProperties;

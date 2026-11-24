import { loadFont as loadInter } from "@remotion/google-fonts/Inter";
import { loadFont as loadMono } from "@remotion/google-fonts/JetBrainsMono";
import { Easing, spring } from "remotion";

// One visual system per video: a theme is the only place colours, fonts and radii come from.
// Briefs fill these from the product's brand; defaultTheme is a neutral dark starting point.
export type Theme = {
  bg: string;
  surface: string;
  ink: string;
  muted: string;
  line: string;
  accent: string;
  fonts: { display: string; body: string; mono: string };
  radius: number;
};

const inter = loadInter("normal", {
  weights: ["400", "500", "600", "700", "800"],
  subsets: ["latin"],
});
const mono = loadMono("normal", { weights: ["400", "600"], subsets: ["latin"] });

export const defaultTheme: Theme = {
  bg: "#0b0b0f",
  surface: "#16161d",
  ink: "#f5f5f7",
  muted: "#8a8a96",
  line: "#2a2a35",
  accent: "#7c5cff",
  fonts: {
    display: inter.fontFamily,
    body: inter.fontFamily,
    mono: mono.fontFamily,
  },
  radius: 20,
};

type SpringConfig = NonNullable<Parameters<typeof spring>[0]["config"]>;

// Two springs only (master brief rule): SOFT for everything, POP reserved for the hero moment.
export const SPRING = {
  soft: { damping: 200 },
  pop: { damping: 12, stiffness: 170, mass: 0.7 },
} satisfies Record<string, SpringConfig>;

// Easing curves for interpolate() and Camera moves. `out` is the default for entrances,
// `inOut` for camera travel, `in` for exits.
export const EASE = {
  out: Easing.bezier(0.16, 1, 0.3, 1),
  inOut: Easing.bezier(0.65, 0, 0.35, 1),
  in: Easing.bezier(0.7, 0, 0.84, 0),
  linear: (t: number) => t,
};

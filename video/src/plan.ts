import { makeBeatGrid, planScenes } from "./kit";

// 30 fps, 120 BPM: 15 frames a beat. Silent cut (X autoplays muted), so every line is on screen.
export const FPS = 30;
export const grid = makeBeatGrid({ fps: FPS, bpm: 120 });

export const plan = planScenes(
  grid,
  [
    { id: "hook", beats: 9 },
    { id: "path", beats: 9 },
    { id: "workspace", beats: 20 },
    { id: "chain", beats: 10 },
    { id: "next", beats: 9 },
    { id: "end", beats: 9 },
  ],
  { hold: 45 },
);

export const S = plan.byId;

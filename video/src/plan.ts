import { makeBeatGrid, planScenes } from "./kit";
import vo from "./voiceover.json";

// 30 fps, 120 BPM: 15 frames a beat. Each scene is fitted to its narration clip in Mel's voice
// (src/voiceover.json, written by scripts/generate-voiceover.mjs) and snapped up to whole beats,
// with a minimum so the animation always has room. Narration starts LEAD frames into its scene.
export const FPS = 30;
export const grid = makeBeatGrid({ fps: FPS, bpm: 120 });
export const LEAD = 12;
const TAIL = 18;

type Id = "hook" | "path" | "workspace" | "chain" | "next" | "end";
const MIN: Record<Id, number> = { hook: 9, path: 9, workspace: 20, chain: 8, next: 9, end: 8 };
const voSec = vo as Partial<Record<Id, number>>;
const beatsFor = (id: Id) => Math.max(MIN[id], Math.ceil((LEAD + Math.ceil((voSec[id] ?? 0) * FPS) + TAIL) / grid.framesPerBeat));

export const plan = planScenes(
  grid,
  (["hook", "path", "workspace", "chain", "next", "end"] as Id[]).map((id) => ({ id, beats: beatsFor(id) })),
  { hold: 45 },
);

export const S = plan.byId;
export const hasVo = (id: Id) => voSec[id] !== undefined;

// Beat grid: every scene boundary and state change lands on a beat (beat = fps * 60 / BPM frames).
// Beat positions are rounded individually from the exact value, so long videos never drift.

export type BeatGrid = {
  fps: number;
  bpm: number;
  /** Frame of beat 0 (e.g. the first downbeat of the track). */
  offset: number;
  framesPerBeat: number;
  /** Frame where beat n starts; fractional n is allowed (0.5 = an eighth note). */
  beat: (n: number) => number;
  /** Frame where bar n starts. */
  bar: (n: number) => number;
  /** Duration of n beats in frames. */
  beats: (n: number) => number;
  /** Beat index (fractional) at a frame. */
  beatAt: (frame: number) => number;
  /** Nearest beat frame to a frame. */
  snap: (frame: number) => number;
};

export const makeBeatGrid = ({
  fps,
  bpm,
  offset = 0,
  beatsPerBar = 4,
}: {
  fps: number;
  bpm: number;
  offset?: number;
  beatsPerBar?: number;
}): BeatGrid => {
  const framesPerBeat = (fps * 60) / bpm;
  const beat = (n: number) => Math.round(offset + n * framesPerBeat);
  const beatAt = (frame: number) => (frame - offset) / framesPerBeat;
  return {
    fps,
    bpm,
    offset,
    framesPerBeat,
    beat,
    bar: (n) => beat(n * beatsPerBar),
    beats: (n) => Math.round(n * framesPerBeat),
    beatAt,
    snap: (frame) => beat(Math.round(beatAt(frame))),
  };
};

export type SceneSpec<Id extends string = string> = { id: Id; beats: number };
export type PlannedScene<Id extends string = string> = {
  id: Id;
  from: number;
  durationInFrames: number;
  /** Beat index the scene starts on. */
  startBeat: number;
};

// Turns a numbered scene list (in beats) into frame ranges on the grid. The first scene starts
// at frame 0 (covering any offset lead-in); every later boundary sits exactly on a beat.
// `hold` adds frames to the last scene (end-card hold). Pass `expectTotal` to enforce the
// brief's frame cap: a mismatch throws, so the studio shows the error instead of a long cut.
export const planScenes = <Id extends string>(
  grid: BeatGrid,
  specs: SceneSpec<Id>[],
  { hold = 0, expectTotal }: { hold?: number; expectTotal?: number } = {},
) => {
  let cum = 0;
  const scenes: PlannedScene<Id>[] = specs.map((s, i) => {
    const from = i === 0 ? 0 : grid.beat(cum);
    const startBeat = cum;
    cum += s.beats;
    const end = grid.beat(cum) + (i === specs.length - 1 ? hold : 0);
    return { id: s.id, from, durationInFrames: end - from, startBeat };
  });
  const total = scenes.length
    ? scenes[scenes.length - 1].from + scenes[scenes.length - 1].durationInFrames
    : 0;
  if (expectTotal !== undefined && total !== expectTotal) {
    throw new Error(
      `Scene plan is ${total} frames, brief expects ${expectTotal} (${(total - expectTotal) / grid.fps}s off)`,
    );
  }
  const byId = Object.fromEntries(scenes.map((s) => [s.id, s])) as Record<Id, PlannedScene<Id>>;
  return { scenes, byId, total, totalBeats: cum };
};

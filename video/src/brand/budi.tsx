// Budi, the character (DESIGN.md "Budi (signature)"): the flat cubist frog built from the brand's B,
// shared by the landing and the app. Moved here from apps/landing/src/bits.tsx on 2026-10-22 so the app
// can draw the real Budi (the live run) instead of the logo mark; the drawing is unchanged. The giant
// hero pose stays in the landing; it builds on LINE and Cheeks from here. Decorative throughout.
import type { CSSProperties } from "react";

export const LINE = 28;
/** The cheeks (Mel 2026-10-19, logo option 1c): flat magenta blush dots,
 *  out beside the mouth, where the face is widest (as the logo's sit in its outer corners), shared
 *  by every Budi. Magenta is the cheeks' (and the tongue's) alone; the body spots in the bottom
 *  corners are violet. Tucked under the eyes they crowded the eyes, nostrils and mouth. */
const CHEEKS = [100, 420];
export function Cheeks() {
  return (
    <>
      {CHEEKS.map((cx) => (
        <circle key={cx} cx={cx} cy={330} r={44} fill="var(--color-magenta)" />
      ))}
    </>
  );
}

/** The small Budis (B-eyes pass 2026-10-19, Mel): the giant's mark, whole. The same B-eyes, LINE,
 *  violet body spots, magenta cheeks, square nostrils and coordinates as BudiGiant; the head's sides stop at
 *  FROG_FLOOR, so he sits flat, and the body spots fill its bottom corners. Each pose sets the
 *  pupils (offsets from the eye centers), the lids and the mouth; arms and props are the pose's own,
 *  drawn as gold bands edged in LINE. `id` keeps each pose's clip paths unique on the page. */
const FROG_FLOOR = 560;
const PROP = LINE / 2; // props' fine detail: text lines, steam, the marker tip
type FrogFace = {
  id: string;
  /** pupil offset from each eye's center, or "happy" for squeezed-shut eyes */
  look: [number, number] | "happy";
  /** how far the lids come down over the eye (0 = open, 1 = shut) */
  lids?: number;
  mouth: "grin" | "smile" | "smirk";
  /** a happy pose can also carry open pupils (`.frog-look`), shown instead of the squint (`.frog-happy`) by CSS */
  alt?: boolean;
  /** a class on the pupils alone, so CSS can move them (reading, scanning) on top of `look` */
  pupils?: string;
};
function FrogBody({ id, look, lids = 0, mouth, alt, pupils }: FrogFace) {
  const ink = "var(--color-ink)";
  const gold = "var(--color-gold)";
  const magenta = "var(--color-magenta)";
  const head = `M10 ${FROG_FLOOR} V 400 A 250 250 0 0 1 510 400 V ${FROG_FLOOR} Z`;
  // quarter-circle body spots in the bottom corners, mirrored: the arcs, then the arcs closed into the corner
  const spotEdges = `M10 ${FROG_FLOOR - 130} A 130 130 0 0 1 140 ${FROG_FLOOR} M510 ${FROG_FLOOR - 130} A 130 130 0 0 0 380 ${FROG_FLOOR}`;
  const spots = `M10 ${FROG_FLOOR - 130} A 130 130 0 0 1 140 ${FROG_FLOOR} H 10 Z M510 ${FROG_FLOOR - 130} A 130 130 0 0 0 380 ${FROG_FLOOR} H 510 Z`;
  const B = "M74 244 V 120 A 100 100 0 0 1 274 120 V 244 Z M246 244 V 120 A 100 100 0 0 1 446 120 V 244 Z";
  const eyes = [
    { k: "l", counter: "M102 216 V 120 A 72 72 0 0 1 246 120 V 216 Z", cx: 174, cy: 150 },
    { k: "r", counter: "M274 216 V 120 A 72 72 0 0 1 418 120 V 216 Z", cx: 346, cy: 150 },
  ];
  const grin = "M170 336 H 350 A 90 90 0 0 1 170 336 Z";
  const lidEdge = 48 + lids * 168; // the counters run from y 48 to 216
  return (
    <g strokeLinejoin="round" strokeLinecap="round">
      <defs>
        {eyes.map((e) => (
          <clipPath key={e.k} id={`${id}-eye-${e.k}`}>
            <path d={e.counter} />
          </clipPath>
        ))}
        {mouth === "grin" && (
          <clipPath id={`${id}-mouth`}>
            <path d={grin} />
          </clipPath>
        )}
      </defs>
      <path d={head} fill={gold} />
      <path d={spots} fill="var(--color-violet)" />
      <path d={spotEdges} fill="none" stroke={ink} strokeWidth={LINE} />
      <Cheeks />
      <path d={head} fill="none" stroke={ink} strokeWidth={LINE} />
      <path d={B} fill={ink} />
      {eyes.map((e) => (
        <g key={e.k} clipPath={`url(#${id}-eye-${e.k})`}>
          <path d={e.counter} fill="var(--color-panel)" />
          {look === "happy" ? (
            <>
              <path className={alt ? "frog-happy" : undefined} d={`M${e.cx - 46} ${e.cy + 30} A 46 46 0 0 1 ${e.cx + 46} ${e.cy + 30}`} fill="none" stroke={ink} strokeWidth={LINE} />
              {alt && (
                <g className="frog-look">
                  <circle cx={e.cx} cy={e.cy} r={24} fill={ink} />
                  <rect x={e.cx - 13} y={e.cy - 15} width={9} height={9} fill="var(--color-panel)" transform={`rotate(45 ${e.cx - 8.5} ${e.cy - 10.5})`} />
                </g>
              )}
            </>
          ) : (
            <g transform={`translate(${look[0]} ${look[1]})`}>
              <g className={pupils}>
                <circle cx={e.cx} cy={e.cy} r={24} fill={ink} />
                <rect x={e.cx - 13} y={e.cy - 15} width={9} height={9} fill="var(--color-panel)" transform={`rotate(45 ${e.cx - 8.5} ${e.cy - 10.5})`} />
              </g>
            </g>
          )}
          {lids > 0 && (
            <>
              <rect x={96} y={40} width={328} height={lidEdge - 40} fill={gold} />
              <path d={`M96 ${lidEdge} H 424`} stroke={ink} strokeWidth={LINE / 2} />
            </>
          )}
        </g>
      ))}
      <rect x={226} y={278} width={16} height={16} fill={ink} transform="rotate(45 234 286)" />
      <rect x={278} y={278} width={16} height={16} fill={ink} transform="rotate(45 286 286)" />
      {mouth === "grin" && (
        <>
          <path d={grin} fill={ink} />
          <circle cx={260} cy={444} r={66} fill={magenta} clipPath={`url(#${id}-mouth)`} />
          <path d={grin} fill="none" stroke={ink} strokeWidth={LINE} />
        </>
      )}
      {/* closed smiles: the grin's own arc, as a line; the smirk lifts one corner */}
      {mouth === "smile" && <path d="M194 350 A 66 66 0 0 0 326 350" fill="none" stroke={ink} strokeWidth={LINE} />}
      {mouth === "smirk" && <path d="M200 362 Q 262 404 330 334" fill="none" stroke={ink} strokeWidth={LINE} />}
    </g>
  );
}

/** An arm: a gold band edged in LINE, drawn before the body so the shoulder tucks in. */
function FrogArm({ d }: { d: string }) {
  return (
    <g fill="none" strokeLinecap="round" strokeLinejoin="round">
      <path d={d} stroke="var(--color-ink)" strokeWidth={56 + 2 * LINE} />
      <path d={d} stroke="var(--color-gold)" strokeWidth={56} />
    </g>
  );
}
function FrogHand({ cx, cy }: { cx: number; cy: number }) {
  return <circle cx={cx} cy={cy} r={44} fill="var(--color-gold)" stroke="var(--color-ink)" strokeWidth={LINE} />;
}

/** Budi in coach mode, for the bridge into the spine: lids down, eyes on the heading, one arm up
 *  pointing the marker at it. The viewBox puts his center (x 260) at 38% of the width, where the
 *  spine's line hangs from. Decorative. */
export function FrogCoach({ className = "" }: { className?: string }) {
  const ink = "var(--color-ink)";
  return (
    <svg aria-hidden="true" viewBox="-46 -10 806 590" className={className}>
      <FrogArm d="M470 470 Q 600 460 600 280" />
      {/* the marker, pointing up and right at the heading: white barrel, violet tip */}
      <g transform="rotate(40 600 270)" strokeLinejoin="round">
        <rect x={568} y={110} width={64} height={160} rx={10} fill="var(--color-panel)" stroke={ink} strokeWidth={LINE} />
        {/* the tip edged in the fine PROP line: at LINE the violet disappears */}
        <path d="M570 110 L 600 30 L 630 110 Z" fill="var(--color-violet)" stroke={ink} strokeWidth={PROP} />
      </g>
      <FrogHand cx={600} cy={270} />
      <FrogBody id="coach" look={[24, 18]} lids={0.3} mouth="smirk" />
    </svg>
  );
}

/** Budi doing the talacha: both arms up, hauling a stack of CVs over his head, eyes squeezed in a
 *  happy grin. The closing's pose, so the page doesn't end on the same marker-up coach. Decorative. */
export function FrogHauling({ className = "" }: { className?: string }) {
  const ink = "var(--color-ink)";
  const sheets = [
    { x: 62, y: -66, r: -2 },
    { x: 88, y: -126, r: 2.5 },
    { x: 56, y: -186, r: -3 },
    { x: 80, y: -246, r: 1.5 },
  ];
  return (
    // The viewBox ends at FROG_FLOOR: the closing stands him on the footer's top border, which draws his floor line.
    // Delight pass (2026-10-20): when the closing scrolls in the sheets drop onto the stack one by one
    // (`data-haul`, armed by the page script only with motion allowed), and when the sign-up button is
    // hovered or focused his squint opens and he looks at it. Mel removed the sheets' job labels and checks.
    <svg aria-hidden="true" data-haul viewBox={`-100 -290 720 ${FROG_FLOOR + 290}`} className={`overflow-visible ${className}`}>
      <FrogArm d="M60 440 Q -40 220 40 -20" />
      <FrogArm d="M460 440 Q 560 220 480 -20" />
      {sheets.map((sh, i) => (
        <g key={i} className="haul-sheet" data-sheet={i} style={{ "--i": i } as CSSProperties}>
          <g transform={`rotate(${sh.r} ${sh.x + 190} ${sh.y + 28})`} strokeLinejoin="round">
            <rect x={sh.x} y={sh.y} width={380} height={64} rx={6} fill="var(--color-panel)" stroke={ink} strokeWidth={LINE} />
            <path d={`M${sh.x + 44} ${sh.y + 32} H ${sh.x + 200}`} stroke={ink} strokeWidth={PROP} strokeLinecap="round" />
          </g>
        </g>
      ))}
      <FrogHand cx={40} cy={-20} />
      <FrogHand cx={480} cy={-20} />
      <FrogBody id="hauling" look="happy" mouth="grin" alt />
    </svg>
  );
}

/** Budi on the Pro card: mug raised, steam rising, eyes on the coffee. Pairs with "menos que un café"
 *  and keeps the marker-up coach for the spine alone. Decorative. */
export function FrogCoffee({ className = "" }: { className?: string }) {
  const ink = "var(--color-ink)";
  return (
    <svg aria-hidden="true" viewBox="-20 -150 820 730" className={className}>
      <FrogArm d="M470 480 Q 600 470 600 300" />
      {/* the mug: white with a magenta band, handle on the far side, two curls of steam */}
      <g strokeLinejoin="round" strokeLinecap="round">
        <path d="M720 170 C 790 170, 790 260, 716 260" fill="none" stroke={ink} strokeWidth={LINE * 1.5} />
        <path d="M560 120 H 720 V 270 Q 720 310 680 310 H 600 Q 560 310 560 270 Z" fill="var(--color-panel)" stroke={ink} strokeWidth={LINE} />
        <path d="M574 172 H 706" stroke="var(--color-magenta)" strokeWidth={LINE} strokeLinecap="butt" />
        <path d="M610 80 C 584 40, 636 10, 610 -40 M670 80 C 644 40, 696 10, 670 -40" fill="none" stroke={ink} strokeWidth={PROP} />
      </g>
      <FrogHand cx={590} cy={290} />
      <FrogBody id="coffee" look={[30, -6]} mouth="smile" />
    </svg>
  );
}

/** Budi on the job search (Mel 2026-10-24: "I want to see Budi working"): lids half down, a magnifier
 *  up on his right, eyes on whatever sits beside him. `.frog-reading` on the pupils and `.frog-lens` on
 *  the magnifier are hooks for the app's motion (eyes reading across, the lens bobbing). Decorative. */
export function FrogSearching({ className = "" }: { className?: string }) {
  const ink = "var(--color-ink)";
  return (
    <svg aria-hidden="true" viewBox="-20 -60 860 640" className={`overflow-visible ${className}`}>
      <FrogArm d="M470 480 Q 600 470 610 330" />
      <g className="frog-lens" strokeLinejoin="round" strokeLinecap="round">
        {/* the handle from his hand to the lens, then the lens: a see-through glass with one glint, edged in LINE */}
        <path d="M610 310 L 668 214" stroke={ink} strokeWidth={LINE * 2.4} />
        <path d="M610 310 L 668 214" stroke="var(--color-violet)" strokeWidth={LINE * 1.1} />
        <circle cx={712} cy={130} r={98} fill="var(--color-panel)" fillOpacity={0.35} stroke={ink} strokeWidth={LINE} />
        <path d="M652 108 A 64 64 0 0 1 700 66" fill="none" stroke={ink} strokeWidth={PROP} />
      </g>
      <FrogHand cx={610} cy={320} />
      <FrogBody id="searching" look={[30, 14]} lids={0.35} mouth="smile" pupils="frog-reading" />
    </svg>
  );
}

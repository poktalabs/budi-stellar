// The cheeks logo (locked 2026-10-19, Mel), shared by the landing and the app. Styles live in logo.css
// (plain CSS, no Tailwind classes, so neither app has to scan this package); import both.
import type { ComponentPropsWithoutRef, ElementType } from "react";

/** The logo mark: Budi's eyes, the turned B, on a black face that runs below them, with his magenta
 *  cheeks in its bottom corners, the same cheeks every Budi wears. The B is heavier than the giant's
 *  (LOGO_LINE) so it holds at header size, and drawn as one outline (the two bowls meet at their circles'
 *  crossing), so an edge never shows a seam between the eyes. Alive without JS (logo.css): a blink every
 *  few seconds, a look up when the link is hovered, a happy squint while it is pressed; the lids are the
 *  counters, in the face's ink. On ink (dark) the face gets a paper edge so it reads. Decorative.
 *  docs/brand/icons-gen.mjs draws the same geometry for the favicons; keep the two in step. */
const LOGO_LINE = 40;
const LOGO_FOOT = 300; // the face's bottom: the B's own base (244) carried down under the eyes
export function LogoFace({ dark }: { dark?: boolean }) {
  const ink = "var(--color-ink)";
  const face = `M74 ${LOGO_FOOT} V 120 A 100 100 0 0 1 260 68.97 A 100 100 0 0 1 446 120 V ${LOGO_FOOT} Z`;
  const r = 100 - LOGO_LINE;
  const eyes = [174, 346].map((cx) => ({
    cx,
    cy: 150,
    counter: `M${cx - r} ${244 - LOGO_LINE} V 120 A ${r} ${r} 0 0 1 ${cx + r} 120 V ${244 - LOGO_LINE} Z`,
  }));
  return (
    <svg aria-hidden="true" viewBox="54 4 412 312" className="logo-eyes">
      <path d={face} fill={ink} {...(dark ? { stroke: "var(--color-paper)", strokeWidth: 16, strokeLinejoin: "round" as const } : {})} />
      {eyes.map((e) => (
        <g key={e.cx}>
          <path d={e.counter} fill="var(--color-panel)" />
          <g className="logo-pupil">
            <circle cx={e.cx} cy={e.cy} r={30} fill={ink} />
            <rect x={e.cx - 16} y={e.cy - 19} width={12} height={12} fill="var(--color-panel)" transform={`rotate(45 ${e.cx - 10} ${e.cy - 13})`} />
          </g>
          <path className="logo-lid" d={e.counter} fill={ink} />
        </g>
      ))}
      <circle cx={128} cy={252} r={30} fill="var(--color-magenta)" />
      <circle cx={392} cy={252} r={30} fill="var(--color-magenta)" />
    </svg>
  );
}

type LogoProps<C extends ElementType> = { as?: C; dark?: boolean } & Omit<ComponentPropsWithoutRef<C>, "as" | "children" | "className">;

/** The header (and footer) logo: the mark and the wordmark (Bricolage 800, 1.6rem on every viewport) in one
 *  link. A plain `<a>` by default; pass the app router's link as `as` (`<Logo as={Link} to="/" />`). No tile. */
export function Logo<C extends ElementType = "a">({ as, dark, ...rest }: LogoProps<C>) {
  const Root: ElementType = as ?? "a";
  return (
    <Root data-logo className={dark ? "ds-logo ds-logo-dark" : "ds-logo"} {...rest}>
      <LogoFace dark={dark} />
      <span>Budi</span>
    </Root>
  );
}

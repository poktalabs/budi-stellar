import React from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { CountUp, EASE, KineticLine, SPRING, Typewriter } from "./kit";
import { FrogCoach } from "./brand/budi";
import { LogoFace } from "./brand/logo";
import { C, F, K, nb, nbSm, press, RADIUS, raised, theme } from "./theme";
import { Badge, clamp, Hand, ramp, useEnter, useExit } from "./ui";

const M = 140; // safe margin on every side

const Title: React.FC<{ text: string; accent?: number; at?: number; size?: number }> = ({ text, accent, at = 4, size = 92 }) => (
  <div style={{ position: "absolute", left: M, top: 110, right: M }}>
    <KineticLine text={text} at={at} accent={accent} size={size} theme={{ ...theme, fonts: { ...theme.fonts, display: F.display } }} />
  </div>
);

// 1. The problem: learning Stellar today means a pile of open tabs.
const TABS = [
  "Stellar Learn",
  "Intro to Stellar",
  "Developer Docs",
  "Getting Started",
  "Stellar Lab",
  "Stellar Quest",
  "Stellar Skills",
  "Raven MCP",
  "Community Fund",
  "Stellar APEX",
];
const TAB_POS = [
  [150, 470, -3], [560, 440, 2], [990, 480, -2], [1400, 450, 3],
  [260, 640, 2], [700, 620, -2], [1120, 660, 2], [1490, 630, -3],
  [450, 810, -1], [930, 830, 2],
];
export const Hook: React.FC<{ dur: number }> = ({ dur }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const sub = useEnter(26);
  return (
    <AbsoluteFill style={useExit(dur)}>
      <Title text="¿Quieres construir en Stellar?" />
      <div style={{ position: "absolute", left: M, top: 240, fontFamily: F.body, fontSize: 46, color: C.charcoal, ...sub }}>
        Hoy empiezas con diez pestañas abiertas.
      </div>
      {TABS.map((t, i) => {
        const [x, y, r] = TAB_POS[i];
        const p = spring({ frame: frame - 40 - i * 5, fps, config: SPRING.soft });
        return (
          <div
            key={t}
            style={{
              position: "absolute",
              left: x,
              top: y,
              rotate: `${r}deg`,
              opacity: Math.min(1, p * 1.5),
              translate: `0 ${(1 - p) * -40}px`,
              padding: "18px 30px",
              background: C.panel,
              borderRadius: RADIUS.button,
              ...nbSm(),
              fontFamily: F.body,
              fontSize: 34,
              fontWeight: 500,
              color: C.ink,
              display: "flex",
              alignItems: "center",
              gap: 16,
              whiteSpace: "nowrap",
            }}
          >
            <span style={{ width: 18, height: 18, borderRadius: 4, border: `3px solid ${C.ink}` }} />
            {t}
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

// 2. The turn: Budi lays them out as one path.
const STEPS = ["Qué es Stellar", "Soroban y Rust", "Tu workspace", "Testnet", "Lo que sigue"];
export const Path: React.FC<{ dur: number }> = ({ dur }) => {
  const frame = useCurrentFrame();
  const frog = useEnter(10, 50);
  const lineW = ramp(frame, 24, 70, EASE.inOut);
  const cardW = 300;
  const gap = (1920 - 2 * M - STEPS.length * cardW) / (STEPS.length - 1);
  return (
    <AbsoluteFill style={useExit(dur)}>
      <Title text="Budi las convierte en un camino." />
      <div style={{ position: "absolute", left: M + cardW / 2, top: 486, height: 3 * K, width: (1920 - 2 * M - cardW) * lineW, background: C.ink }} />
      {STEPS.map((s, i) => (
        <Step key={s} n={i + 1} label={s} x={M + i * (cardW + gap)} w={cardW} at={24 + i * 10} />
      ))}
      <div style={{ position: "absolute", left: M - 20, top: 640, width: 400, ...frog }}>
        <FrogCoach />
      </div>
      <Hand style={{ position: "absolute", left: 600, top: 760, opacity: ramp(frame, 80, 95) }}>una lección, un ejercicio</Hand>
    </AbsoluteFill>
  );
};
const Step: React.FC<{ n: number; label: string; x: number; w: number; at: number }> = ({ n, label, x, w, at }) => {
  const enter = useEnter(at, 24);
  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: 380,
        width: w,
        height: 210,
        boxSizing: "border-box",
        padding: 26,
        background: C.panel,
        borderRadius: RADIUS.card,
        ...nb(),
        ...enter,
      }}
    >
      <div style={{ fontFamily: F.display, fontWeight: 800, fontSize: 64, lineHeight: 1, color: C.ink }}>{n}</div>
      <div style={{ fontFamily: F.body, fontWeight: 600, fontSize: 34, lineHeight: 1.2, marginTop: 18, color: C.ink }}>{label}</div>
    </div>
  );
};

// 3. The workspace, from the real first run (2026-11-23): every line and time is from that log.
const LOG: { s: number; ok?: boolean; text: string }[] = [
  { s: 2, text: "sesión lista en Tenki" },
  { s: 7, ok: true, text: "rustc 1.92.0 · stellar 30.0.1" },
  { s: 7, ok: true, text: "cargo test (7 pruebas)" },
  { s: 7, ok: true, text: "stellar contract build" },
  { s: 27, ok: true, text: "contrato desplegado en testnet" },
  { s: 63, ok: true, text: "VS Code abierto en el navegador" },
];
const T0 = 74; // frame the run starts
const SEC = 3; // frames per logged second (64 s plays in ~6.4 s)
export const Workspace: React.FC<{ dur: number }> = ({ dur }) => {
  const frame = useCurrentFrame();
  const card = useEnter(14);
  const timer = useEnter(T0);
  return (
    <AbsoluteFill style={useExit(dur)}>
      <Title text="Tu entorno Stellar, sin instalar nada." />
      <div
        style={{
          position: "absolute",
          left: M,
          top: 300,
          width: 1080,
          height: 560,
          boxSizing: "border-box",
          background: C.panel,
          borderRadius: RADIUS.card,
          overflow: "hidden",
          ...raised(),
          ...card,
        }}
      >
        <div style={{ height: 76, background: C.gold, borderBottom: `${3 * K}px solid ${C.ink}`, display: "flex", alignItems: "center", gap: 20, padding: "0 30px" }}>
          <span style={{ fontFamily: F.display, fontWeight: 800, fontSize: 36 }}>Budi workspace</span>
          <Badge>Tenki · Rust · Stellar CLI</Badge>
        </div>
        <div style={{ padding: "30px 36px", fontFamily: F.mono, fontSize: 32, lineHeight: 1.65, color: C.ink }}>
          <div>
            <span style={{ color: C.charcoal }}>$ </span>
            <Typewriter text="pnpm launch --testnet" at={34} cps={30} caret={frame < T0} />
          </div>
          {LOG.map((l, i) => {
            const at = T0 + l.s * SEC + i; // same-second lines land one frame apart
            const o = ramp(frame, at, at + 8);
            return (
              <div key={i} style={{ opacity: o, translate: `0 ${(1 - o) * 10}px`, display: "flex", gap: 22 }}>
                <span style={{ color: C.charcoal, width: 90 }}>[{l.s}s]</span>
                <span style={{ color: C.success, fontWeight: 600, width: 50 }}>{l.ok ? "ok" : ""}</span>
                <span>{l.text}</span>
              </div>
            );
          })}
        </div>
      </div>
      <div style={{ position: "absolute", left: 1300, top: 400, width: 480, ...timer }}>
        <div style={{ fontFamily: F.display, fontWeight: 800, fontSize: 240, lineHeight: 0.9, color: C.ink, letterSpacing: "-0.03em" }}>
          <CountUp to={64} at={T0} duration={64 * SEC} ease={(t) => t} />
          <span style={{ fontSize: 120 }}> s</span>
        </div>
        <div style={{ fontFamily: F.body, fontSize: 40, lineHeight: 1.3, color: C.charcoal, marginTop: 24 }}>de cero a un contrato en testnet</div>
      </div>
    </AbsoluteFill>
  );
};

// 4. The proof: Bitácora on testnet with the first milestone.
export const Chain: React.FC<{ dur: number }> = ({ dur }) => {
  const frame = useCurrentFrame();
  const card = useEnter(16);
  const row = useEnter(40, 20);
  return (
    <AbsoluteFill style={useExit(dur)}>
      <Title text="Y tu primer contrato, en testnet." />
      <div
        style={{
          position: "absolute",
          left: 360,
          top: 330,
          width: 1200,
          boxSizing: "border-box",
          padding: 48,
          background: C.panel,
          borderRadius: RADIUS.card,
          ...raised(),
          ...card,
        }}
      >
        <div style={{ fontFamily: F.body, fontWeight: 700, fontSize: 26, letterSpacing: "0.08em", textTransform: "uppercase", color: C.charcoal }}>
          Bitácora · contrato Soroban
        </div>
        <div style={{ fontFamily: F.mono, fontSize: 30, color: C.charcoal, marginTop: 12 }}>CCMAFZJHAC2V…T22ZUKDXSOP</div>
        <div style={{ height: 3 * K, background: C.ink, margin: "36px 0" }} />
        <div style={{ display: "flex", alignItems: "center", gap: 28, ...row }}>
          <span style={{ fontFamily: F.display, fontWeight: 800, fontSize: 64 }}>#1</span>
          <span style={{ fontFamily: F.body, fontWeight: 500, fontSize: 44, flex: 1 }}>Ship It: primer hito desde Budi</span>
          <Badge tone="success">En testnet</Badge>
        </div>
      </div>
      <Hand style={{ position: "absolute", left: 1180, top: 760, opacity: ramp(frame, 70, 85), translate: `0 ${(1 - ramp(frame, 70, 85)) * 12}px` }}>
        registrado onchain
      </Hand>
    </AbsoluteFill>
  );
};

// 5. What ships by Nov 27 (planned, so every row carries "Pronto").
const NEXT = ["Explicadores animados de Stellar y Soroban", "Un ejercicio en tu workspace por lección", "Budi como guía, de la primera lección a tu contrato"];
export const Next: React.FC<{ dur: number }> = ({ dur }) => (
  <AbsoluteFill style={useExit(dur)}>
    <Title text="Esta semana: la ruta, en español." />
    {NEXT.map((t, i) => (
      <NextRow key={t} text={t} y={340 + i * 180} at={24 + i * 12} />
    ))}
  </AbsoluteFill>
);
const NextRow: React.FC<{ text: string; y: number; at: number }> = ({ text, y, at }) => {
  const enter = useEnter(at, 24);
  return (
    <div
      style={{
        position: "absolute",
        left: M,
        top: y,
        width: 1920 - 2 * M,
        height: 140,
        boxSizing: "border-box",
        padding: "0 40px",
        display: "flex",
        alignItems: "center",
        gap: 30,
        background: C.panel,
        borderRadius: RADIUS.card,
        ...nb(),
        ...enter,
      }}
    >
      <span style={{ fontFamily: F.body, fontWeight: 500, fontSize: 44, flex: 1 }}>{text}</span>
      <Badge tone="info">Pronto</Badge>
    </div>
  );
};

// 6. Close: logo, the payoff line in magenta, the repo as the one action.
export const End: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const logo = useEnter(0, 24);
  const cta = spring({ frame: frame - 50, fps, config: SPRING.soft });
  const foot = interpolate(frame, [70, 90], [0, 1], clamp);
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", gap: 40 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 26, ...logo }}>
        <div style={{ width: 150 }}>
          <LogoFace />
        </div>
        <span style={{ fontFamily: F.display, fontWeight: 800, fontSize: 120, letterSpacing: "-0.025em", lineHeight: 1 }}>Budi para Stellar</span>
      </div>
      <KineticLine text="Aprende Stellar construyendo." at={18} accent={2} size={84} theme={{ ...theme, fonts: { ...theme.fonts, display: F.display } }} style={{ justifyContent: "center" }} />
      <div
        style={{
          marginTop: 10,
          padding: "22px 44px",
          borderRadius: RADIUS.button,
          background: C.magenta,
          color: "#fff",
          fontFamily: F.body,
          fontWeight: 700,
          fontSize: 38,
          opacity: Math.min(1, cta * 1.5),
          translate: `0 ${(1 - cta) * 20}px`,
          ...press(),
        }}
      >
        github.com/poktalabs/budi-stellar
      </div>
      <div style={{ fontFamily: F.body, fontSize: 32, color: C.charcoal, opacity: foot }}>#ShipItBuilder · BAF × Stellar México</div>
    </AbsoluteFill>
  );
};

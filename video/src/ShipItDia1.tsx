import React from "react";
import { Audio } from "@remotion/media";
import { AbsoluteFill, interpolate, Sequence, staticFile } from "remotion";
import { FPS, hasVo, LEAD, plan, S } from "./plan";
import { Chain, End, Hook, Next, Path, Workspace } from "./scenes";
import { cssVars } from "./theme";
import { DotGrid } from "./ui";

export const ShipItDia1: React.FC = () => (
  <AbsoluteFill style={cssVars}>
    {/* logo.css rest state, static so every frame renders the same: eyes open, pupils on the wordmark */}
    <style>{`.logo-pupil{transform:translate(16px,8px)}.logo-lid{transform-box:fill-box;transform-origin:50% 0;transform:scaleY(0)}`}</style>
    <DotGrid />
    <Sequence from={S.hook.from} durationInFrames={S.hook.durationInFrames}><Hook dur={S.hook.durationInFrames} /></Sequence>
    <Sequence from={S.path.from} durationInFrames={S.path.durationInFrames}><Path dur={S.path.durationInFrames} /></Sequence>
    <Sequence from={S.workspace.from} durationInFrames={S.workspace.durationInFrames}><Workspace dur={S.workspace.durationInFrames} /></Sequence>
    <Sequence from={S.chain.from} durationInFrames={S.chain.durationInFrames}><Chain dur={S.chain.durationInFrames} /></Sequence>
    <Sequence from={S.next.from} durationInFrames={S.next.durationInFrames}><Next dur={S.next.durationInFrames} /></Sequence>
    <Sequence from={S.end.from} durationInFrames={S.end.durationInFrames}><End /></Sequence>
    <Music />
    {plan.scenes.filter((s) => hasVo(s.id)).map((s) => (
      <Sequence key={s.id} from={s.from + LEAD}>
        <Audio src={staticFile(`voiceover/${s.id}.mp3`)} />
      </Sequence>
    ))}
    {/* two quiet hits, as in the Sage cut: the command typing and the contract card landing */}
    <Sequence from={S.workspace.from + 34}><Audio src={staticFile("sfx/paste.mp3")} volume={0.4} /></Sequence>
    <Sequence from={S.chain.from + 16}><Audio src={staticFile("sfx/card.mp3")} volume={0.35} /></Sequence>
  </AbsoluteFill>
);

// The Sage launch video's bed and mix: midnight-funk at one steady low level under the voice,
// no ducking, 0.4 s fade in, 2 s fade out.
const MUSIC = { volume: 0.15, fadeIn: Math.round(0.4 * FPS), fadeOut: 2 * FPS };
const Music: React.FC = () => {
  const len = plan.total;
  return (
    <Audio
      src={staticFile("music/midnight-funk.mp3")}
      volume={(f) => MUSIC.volume * interpolate(f, [0, MUSIC.fadeIn, len - MUSIC.fadeOut, len], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })}
    />
  );
};

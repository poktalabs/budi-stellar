import React from "react";
import { AbsoluteFill, Sequence } from "remotion";
import { S } from "./plan";
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
  </AbsoluteFill>
);

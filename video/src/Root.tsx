import React from "react";
import { Composition } from "remotion";
import { FPS, plan } from "./plan";
import { ShipItDia1 } from "./ShipItDia1";

export const RemotionRoot: React.FC = () => (
  <Composition id="ShipItDia1" component={ShipItDia1} durationInFrames={plan.total} fps={FPS} width={1920} height={1080} />
);

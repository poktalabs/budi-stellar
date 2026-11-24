import React from "react";
import { AbsoluteFill, EasingFunction, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { CameraMotionBlur } from "@remotion/motion-blur";
import { EASE } from "./tokens";

// A virtual camera over one large canvas ("world"). Each keyframe says which world point sits
// at the centre of the frame, at what zoom. Moves between keyframes use the *destination*
// keyframe's ease (default EASE.inOut); zoom interpolates in log space so 1x->4x feels as even
// as 4x->16x. Hold a shot by repeating a keyframe at a later frame.

export type CameraKey = {
  frame: number;
  x: number;
  y: number;
  scale?: number;
  rotate?: number;
  ease?: EasingFunction;
};

export type CameraState = { x: number; y: number; scale: number; rotate: number };

export const cameraAt = (keys: CameraKey[], frame: number): CameraState => {
  if (keys.length === 0) throw new Error("Camera needs at least one keyframe");
  const sorted = [...keys].sort((a, b) => a.frame - b.frame);
  const state = (k: CameraKey): CameraState => ({
    x: k.x,
    y: k.y,
    scale: k.scale ?? 1,
    rotate: k.rotate ?? 0,
  });
  if (frame <= sorted[0].frame) return state(sorted[0]);
  const last = sorted[sorted.length - 1];
  if (frame >= last.frame) return state(last);
  const i = sorted.findIndex((k) => k.frame > frame);
  const a = state(sorted[i - 1]);
  const b = state(sorted[i]);
  const t = interpolate(frame, [sorted[i - 1].frame, sorted[i].frame], [0, 1], {
    easing: sorted[i].ease ?? EASE.inOut,
  });
  const lerp = (p: number, q: number) => p + (q - p) * t;
  return {
    x: lerp(a.x, b.x),
    y: lerp(a.y, b.y),
    scale: Math.exp(lerp(Math.log(a.scale), Math.log(b.scale))),
    rotate: lerp(a.rotate, b.rotate),
  };
};

export const Camera: React.FC<{
  keys: CameraKey[];
  /** World size in px; defaults to the composition size. */
  worldWidth?: number;
  worldHeight?: number;
  /** Wrap in CameraMotionBlur (renders `blurSamples` sub-frames; costly, use for fast pushes). */
  motionBlur?: boolean;
  blurSamples?: number;
  background?: string;
  children: React.ReactNode;
}> = ({ keys, worldWidth, worldHeight, motionBlur = false, blurSamples = 8, background, children }) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const cam = cameraAt(keys, frame);
  const world = (
    <AbsoluteFill style={{ background, overflow: "hidden" }}>
      <div
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          width: worldWidth ?? width,
          height: worldHeight ?? height,
          transformOrigin: "0 0",
          transform: `translate(${width / 2}px, ${height / 2}px) rotate(${cam.rotate}deg) scale(${cam.scale}) translate(${-cam.x}px, ${-cam.y}px)`,
        }}
      >
        {children}
      </div>
    </AbsoluteFill>
  );
  return motionBlur ? (
    <CameraMotionBlur shutterAngle={180} samples={blurSamples}>
      {world}
    </CameraMotionBlur>
  ) : (
    world
  );
};

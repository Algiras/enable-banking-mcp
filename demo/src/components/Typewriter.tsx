import React from 'react';
import { interpolate, useCurrentFrame } from 'remotion';
export const Typewriter: React.FC<{
  text: string;
  start?: number;
  charsPerFrame?: number;
  style?: React.CSSProperties;
}> = ({ text, start = 0, charsPerFrame = 0.7, style }) => {
  const frame = useCurrentFrame();
  const chars = Math.floor(interpolate(frame, [start, start + text.length / charsPerFrame], [0, text.length], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  }));
  const visible = text.slice(0, chars);
  const cursor = chars < text.length ? '█' : '';
  return <span style={style}>{visible}<span style={{ opacity: 0.7 }}>{cursor}</span></span>;
};

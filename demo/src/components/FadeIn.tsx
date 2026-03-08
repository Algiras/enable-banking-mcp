import React from 'react';
import { interpolate, useCurrentFrame } from 'remotion';
export const FadeIn: React.FC<{
  delay?: number;
  duration?: number;
  children: React.ReactNode;
  style?: React.CSSProperties;
}> = ({ delay = 0, duration = 20, children, style }) => {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [delay, delay + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const translateY = interpolate(frame, [delay, delay + duration], [24, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <div style={{ opacity, transform: `translateY(${translateY}px)`, ...style }}>
      {children}
    </div>
  );
};

import React from 'react';
import { spring, useCurrentFrame, useVideoConfig, interpolate } from 'remotion';
import { THEME } from '../theme';
export const Badge: React.FC<{
  label: string;
  color?: string;
  delay?: number;
  size?: 'sm' | 'md';
}> = ({ label, color = THEME.accent, delay = 0, size = 'md' }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 15, stiffness: 200 } });
  const scale = interpolate(s, [0, 1], [0.5, 1]);
  const opacity = interpolate(s, [0, 0.3], [0, 1], { extrapolateRight: 'clamp' });
  const fs = size === 'sm' ? 11 : 13;
  const px = size === 'sm' ? 10 : 14;
  const py = size === 'sm' ? 3 : 5;
  return (
    <span style={{
      display: 'inline-block', transform: `scale(${scale})`, opacity,
      background: `${color}22`, border: `1px solid ${color}66`,
      color, borderRadius: 6, fontSize: fs, fontWeight: 600,
      padding: `${py}px ${px}px`, fontFamily: THEME.fontMono,
      letterSpacing: '0.02em',
    }}>
      {label}
    </span>
  );
};

import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
import { Badge } from '../components/Badge';
const BankIcon: React.FC<{ x: number; y: number; delay: number; emoji: string }> = ({ x, y, delay, emoji }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 12, stiffness: 150 } });
  const scale = interpolate(s, [0, 1], [0, 1]);
  const opacity = interpolate(s, [0, 0.4], [0, 0.3], { extrapolateRight: 'clamp' });
  return (
    <div style={{
      position: 'absolute', left: x, top: y,
      fontSize: 48, transform: `scale(${scale})`, opacity,
      filter: 'grayscale(30%)',
    }}>
      {emoji}
    </div>
  );
};
export const SceneHero: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const titleSpring = spring({ frame: frame - 5, fps, config: { damping: 200 } });
  const titleY = interpolate(titleSpring, [0, 1], [50, 0]);
  const titleOpacity = interpolate(titleSpring, [0, 1], [0, 1]);
  // Pulsing glow on accent line
  const glow = interpolate(Math.sin((frame / fps) * Math.PI * 2), [-1, 1], [0.5, 1.0]);
  // Fade out at end
  const fadeOut = interpolate(frame, [durationInFrames - 20, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, overflow: 'hidden', opacity: fadeOut }}>
      {/* Grid background */}
      <svg width="100%" height="100%" style={{ position: 'absolute', opacity: 0.04 }}>
        <defs>
          <pattern id="grid" width="60" height="60" patternUnits="userSpaceOnUse">
            <path d="M 60 0 L 0 0 0 60" fill="none" stroke={THEME.accent} strokeWidth="0.5" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#grid)" />
      </svg>
      {/* Floating bank icons */}
      <BankIcon x={80} y={120} delay={20} emoji="🏦" />
      <BankIcon x={1100} y={80} delay={25} emoji="💳" />
      <BankIcon x={60} y={550} delay={30} emoji="💰" />
      <BankIcon x={1150} y={500} delay={22} emoji="🏧" />
      <BankIcon x={900} y={580} delay={35} emoji="🔐" />
      {/* Radial glow behind title */}
      <div style={{
        position: 'absolute', width: 800, height: 400,
        left: '50%', top: '50%',
        transform: 'translate(-50%, -55%)',
        background: `radial-gradient(ellipse, ${THEME.accent}18 0%, transparent 70%)`,
        opacity: glow,
      }} />
      {/* Main title */}
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 0 }}>
        <div style={{
          transform: `translateY(${titleY}px)`, opacity: titleOpacity,
          textAlign: 'center',
        }}>
          <div style={{
            fontSize: 18, fontFamily: THEME.fontMono, color: THEME.accent,
            letterSpacing: '0.3em', textTransform: 'uppercase', marginBottom: 16,
          }}>
            Model Context Protocol
          </div>
          <div style={{
            fontSize: 80, fontWeight: 800, fontFamily: THEME.fontSans,
            color: THEME.textPrimary, lineHeight: 1.05,
            textShadow: `0 0 80px ${THEME.accent}44`,
          }}>
            Enable Banking
          </div>
          <div style={{
            width: 200, height: 3, margin: '20px auto',
            background: `linear-gradient(90deg, transparent, ${THEME.accent}, transparent)`,
            opacity: glow,
          }} />
          <div style={{
            fontSize: 28, color: THEME.textSecondary, fontFamily: THEME.fontSans,
            fontWeight: 300, marginTop: 8,
          }}>
            Connect AI to real banking data
          </div>
        </div>
        {/* Badges */}
        <div style={{ display: 'flex', gap: 12, marginTop: 40 }}>
          <Badge label="19 Tools" color={THEME.accent} delay={30} />
          <Badge label="6 Resources" color={THEME.purple} delay={38} />
          <Badge label="Rust + rmcp" color={THEME.yellow} delay={46} />
          <Badge label="OAuth 2.0" color={THEME.blue} delay={54} />
          <Badge label="Open Banking" color={THEME.red} delay={62} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

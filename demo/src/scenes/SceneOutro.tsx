import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
const SetupStep: React.FC<{ cmd: string; comment: string; delay: number }> = ({ cmd, comment, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity = interpolate(s, [0, 1], [0, 1]);
  const x = interpolate(s, [0, 1], [-20, 0]);
  return (
    <div style={{ opacity, transform: `translateX(${x}px)`, marginBottom: 10 }}>
      <div style={{
        display: 'flex', alignItems: 'center', gap: 12,
        background: THEME.bgCode, borderRadius: 8, padding: '10px 16px',
        border: `1px solid ${THEME.border}`,
      }}>
        <span style={{ color: THEME.accent, fontFamily: THEME.fontMono, fontSize: 13 }}>$</span>
        <span style={{ color: THEME.textPrimary, fontFamily: THEME.fontMono, fontSize: 13, flex: 1 }}>{cmd}</span>
        <span style={{ color: THEME.textSecondary, fontFamily: THEME.fontSans, fontSize: 12 }}>{comment}</span>
      </div>
    </div>
  );
};
export const SceneOutro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  // No fade out at end — hold final frame
  const fadeIn = interpolate(frame, [0, 15], [0, 1], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  const glow = interpolate(Math.sin((frame / fps) * Math.PI), [-1, 1], [0.6, 1.0]);
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeIn }}>
      {/* Radial glow */}
      <div style={{
        position: 'absolute', width: 600, height: 400, left: '50%', top: '40%',
        transform: 'translate(-50%, -50%)',
        background: `radial-gradient(ellipse, ${THEME.accent}14 0%, transparent 70%)`,
        opacity: glow,
      }} />
      <div style={{ maxWidth: 760 }}>
        <FadeIn delay={0} duration={15}>
          <div style={{
            fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent,
            letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 12,
          }}>
            Get Started
          </div>
        </FadeIn>
        <FadeIn delay={8} duration={20}>
          <div style={{
            fontSize: 52, fontWeight: 800, color: THEME.textPrimary,
            fontFamily: THEME.fontSans, lineHeight: 1.05, marginBottom: 8,
          }}>
            Start in 3 commands
          </div>
        </FadeIn>
        <FadeIn delay={20} duration={15}>
          <div style={{ fontSize: 18, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginBottom: 36 }}>
            From zero to banking AI in minutes
          </div>
        </FadeIn>
        <SetupStep cmd="enable-banking-mcp register" comment="Create app + generate RSA key pair" delay={32} />
        <SetupStep cmd="enable-banking-mcp init" comment="Connect your first bank account" delay={44} />
        <SetupStep cmd="enable-banking-mcp install" comment="Auto-install into Claude Desktop" delay={56} />
        <FadeIn delay={70} duration={15} style={{ marginTop: 28 }}>
          <div style={{
            background: `${THEME.accent}14`, border: `1px solid ${THEME.accent}55`,
            borderRadius: 14, padding: '18px 22px', display: 'flex', gap: 20, alignItems: 'center',
          }}>
            <div style={{ fontSize: 32 }}>🦀</div>
            <div>
              <div style={{ fontSize: 15, fontWeight: 700, color: THEME.accent, fontFamily: THEME.fontSans }}>
                Built with Rust + rmcp v1.1.0
              </div>
              <div style={{ fontSize: 13, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginTop: 4 }}>
                Official MCP Rust SDK · stdio + HTTP · 4.5 MB binary · zero dependencies at runtime
              </div>
            </div>
          </div>
        </FadeIn>
        <FadeIn delay={85} duration={15} style={{ marginTop: 20 }}>
          <div style={{ display: 'flex', gap: 12 }}>
            {['Sandbox & Production', '500+ European Banks', 'PSD2 Compliant', 'Session Persistence', 'MCP Apps UI'].map((feat, i) => (
              <div key={feat} style={{
                background: `${THEME.purple}12`, border: `1px solid ${THEME.purple}44`,
                borderRadius: 8, padding: '6px 12px', fontSize: 12,
                color: THEME.purple, fontFamily: THEME.fontSans,
              }}>✓ {feat}</div>
            ))}
          </div>
        </FadeIn>
      </div>
    </AbsoluteFill>
  );
};

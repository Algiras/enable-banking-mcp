import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
const Box: React.FC<{
  label: string;
  sublabel: string;
  emoji: string;
  color: string;
  delay: number;
  x: number;
}> = ({ label, sublabel, emoji, color, delay, x }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 15, stiffness: 180 } });
  const scale = interpolate(s, [0, 1], [0.6, 1]);
  const opacity = interpolate(s, [0, 0.3], [0, 1], { extrapolateRight: 'clamp' });
  return (
    <div style={{
      position: 'absolute', left: x, top: '50%',
      transform: `translateY(-50%) scale(${scale})`, opacity,
      width: 160, textAlign: 'center',
    }}>
      <div style={{
        background: `${color}18`, border: `2px solid ${color}66`,
        borderRadius: 16, padding: '20px 16px',
        boxShadow: `0 0 40px ${color}22`,
      }}>
        <div style={{ fontSize: 40, marginBottom: 8 }}>{emoji}</div>
        <div style={{ fontSize: 14, fontWeight: 700, color, fontFamily: THEME.fontSans }}>{label}</div>
        <div style={{ fontSize: 11, color: THEME.textSecondary, marginTop: 4, fontFamily: THEME.fontSans }}>{sublabel}</div>
      </div>
    </div>
  );
};
const Arrow: React.FC<{ x: number; delay: number; label: string }> = ({ x, delay, label }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const width = interpolate(s, [0, 1], [0, 80]);
  const opacity = interpolate(s, [0, 1], [0, 1]);
  return (
    <div style={{
      position: 'absolute', left: x, top: '50%',
      transform: 'translateY(-50%)', opacity,
      display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 6,
      width: 80,
    }}>
      <div style={{ fontSize: 11, color: THEME.accent, fontFamily: THEME.fontMono, whiteSpace: 'nowrap' }}>
        {label}
      </div>
      <div style={{
        height: 2,
        background: `linear-gradient(90deg, ${THEME.accent}88, ${THEME.accent})`,
        width, borderRadius: 1,
        position: 'relative',
      }}>
        <div style={{
          position: 'absolute', right: -6, top: -4,
          color: THEME.accent, fontSize: 12,
        }}>▶</div>
      </div>
    </div>
  );
};
export const SceneArchitecture: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeOut }}>
      <FadeIn delay={0} duration={15}>
        <div style={{
          fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent,
          letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8,
        }}>
          How It Works
        </div>
      </FadeIn>
      <FadeIn delay={8} duration={18}>
        <div style={{
          fontSize: 42, fontWeight: 800, color: THEME.textPrimary,
          fontFamily: THEME.fontSans, marginBottom: 8,
        }}>
          Architecture Overview
        </div>
      </FadeIn>
      <FadeIn delay={16} duration={15}>
        <div style={{
          fontSize: 17, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginBottom: 60,
        }}>
          A thin MCP bridge between your AI client and 500+ European banks
        </div>
      </FadeIn>
      {/* Architecture diagram */}
      <div style={{ position: 'relative', height: 220 }}>
        <Box label="AI Client" sublabel="Claude / GPT / Custom" emoji="🤖" color={THEME.purple} delay={25} x={60} />
        <Arrow x={230} delay={35} label="MCP tools" />
        <Box label="MCP Server" sublabel="enable-banking-mcp" emoji="⚙️" color={THEME.accent} delay={40} x={320} />
        <Arrow x={490} delay={50} label="JWT + REST" />
        <Box label="Enable Banking" sublabel="api.enablebanking.com" emoji="🌐" color={THEME.blue} delay={55} x={580} />
        <Arrow x={750} delay={65} label="PSD2 / Open" />
        <Box label="Your Banks" sublabel="500+ across Europe" emoji="🏦" color={THEME.yellow} delay={70} x={840} />
      </div>
      {/* Transport note */}
      <FadeIn delay={80} duration={15} style={{ marginTop: 24 }}>
        <div style={{
          display: 'flex', gap: 16, alignItems: 'center',
        }}>
          <div style={{
            background: `${THEME.accent}15`, border: `1px solid ${THEME.accent}44`,
            borderRadius: 10, padding: '10px 20px',
            fontSize: 13, color: THEME.accent, fontFamily: THEME.fontMono,
          }}>
            📡 stdio transport (Claude Desktop / VS Code)
          </div>
          <div style={{
            background: `${THEME.blue}15`, border: `1px solid ${THEME.blue}44`,
            borderRadius: 10, padding: '10px 20px',
            fontSize: 13, color: THEME.blue, fontFamily: THEME.fontMono,
          }}>
            🌐 HTTP + StreamableHTTP (remote hosting)
          </div>
          <div style={{
            background: `${THEME.yellow}15`, border: `1px solid ${THEME.yellow}44`,
            borderRadius: 10, padding: '10px 20px',
            fontSize: 13, color: THEME.yellow, fontFamily: THEME.fontMono,
          }}>
            🦀 Rust binary — 4.5 MB, zero overhead
          </div>
        </div>
      </FadeIn>
    </AbsoluteFill>
  );
};

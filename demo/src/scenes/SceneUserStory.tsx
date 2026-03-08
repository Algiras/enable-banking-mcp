import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';

const StoryCard: React.FC<{
  emoji: string; title: string; body: string; delay: number; accent: string;
}> = ({ emoji, title, body, delay, accent }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity = interpolate(s, [0, 1], [0, 1]);
  const y = interpolate(s, [0, 1], [20, 0]);
  return (
    <div style={{
      opacity, transform: "translateY(" + y + "px)",
      background: THEME.bgCard, border: "1px solid " + accent + "44",
      borderRadius: 12, padding: '14px 18px', flex: 1,
    }}>
      <div style={{ fontSize: 28, marginBottom: 6 }}>{emoji}</div>
      <div style={{ fontSize: 13, fontWeight: 700, color: accent, marginBottom: 5, fontFamily: THEME.fontSans }}>{title}</div>
      <div style={{ fontSize: 12, color: THEME.textSecondary, lineHeight: 1.55, fontFamily: THEME.fontSans }}>{body}</div>
    </div>
  );
};

export const SceneUserStory: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '44px 80px', opacity: fadeOut }}>
      <FadeIn delay={0} duration={15}>
        <div style={{ fontSize: 12, fontFamily: THEME.fontMono, color: THEME.accent, letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8 }}>User Story</div>
      </FadeIn>
      <FadeIn delay={8} duration={18}>
        <div style={{ fontSize: 38, fontWeight: 800, fontFamily: THEME.fontSans, color: THEME.textPrimary, lineHeight: 1.1, marginBottom: 10 }}>
          Meet Alex, an AI builder
        </div>
      </FadeIn>
      <FadeIn delay={16} duration={15}>
        <div style={{ fontSize: 16, color: THEME.textSecondary, fontFamily: THEME.fontSans, maxWidth: 720, lineHeight: 1.55, marginBottom: 28 }}>
          Alex wants a personal finance assistant. Instead of months of bank API plumbing,
          they plug in <span style={{ color: THEME.accent, fontWeight: 600 }}>Enable Banking MCP</span> and
          their AI accesses real accounts in minutes.
        </div>
      </FadeIn>
      <div style={{ display: 'flex', gap: 16 }}>
        <StoryCard emoji="💭" title="The Problem" body="Every bank has a different API. OAuth, token refresh, pagination — weeks before any real work." delay={28} accent={THEME.red} />
        <StoryCard emoji="⚡" title="The Solution" body="One MCP server, 500+ banks. The AI calls tools, the server handles everything else." delay={40} accent={THEME.accent} />
        <StoryCard emoji="🎯" title="The Result" body='Ask your AI: "What did I spend on food last month?" and get a real answer from real bank data.' delay={52} accent={THEME.purple} />
        <StoryCard emoji="🚀" title="As a Developer" body="I want to give my AI agent access to banking data so it can answer financial questions on my behalf." delay={64} accent={THEME.yellow} />
      </div>
    </AbsoluteFill>
  );
};

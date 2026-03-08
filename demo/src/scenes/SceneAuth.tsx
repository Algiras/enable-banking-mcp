import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
import { CodeLine } from '../components/CodeLine';
import { Badge } from '../components/Badge';
const Step: React.FC<{ num: number; label: string; desc: string; delay: number; done?: boolean }> = ({ num, label, desc, delay, done }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity = interpolate(s, [0, 1], [0, 1]);
  const x = interpolate(s, [0, 1], [-20, 0]);
  return (
    <div style={{ opacity, transform: `translateX(${x}px)`, display: 'flex', gap: 16, alignItems: 'flex-start', marginBottom: 20 }}>
      <div style={{
        width: 30, height: 30, borderRadius: '50%', flexShrink: 0,
        background: done ? THEME.accent : `${THEME.accent}22`,
        border: `2px solid ${THEME.accent}`,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontSize: 13, fontWeight: 700, color: done ? THEME.bg : THEME.accent,
        fontFamily: THEME.fontMono,
      }}>{done ? '✓' : num}</div>
      <div>
        <div style={{ fontSize: 15, fontWeight: 700, color: THEME.textPrimary, fontFamily: THEME.fontSans }}>{label}</div>
        <div style={{ fontSize: 13, color: THEME.textSecondary, marginTop: 3, fontFamily: THEME.fontSans }}>{desc}</div>
      </div>
    </div>
  );
};
export const SceneAuth: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeOut }}>
      <div style={{ display: 'flex', gap: 80 }}>
        {/* Left: feature info */}
        <div style={{ flex: 1 }}>
          <FadeIn delay={0} duration={15}>
            <div style={{
              fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent,
              letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8,
            }}>Feature 1</div>
          </FadeIn>
          <FadeIn delay={8} duration={18}>
            <div style={{ fontSize: 38, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 16 }}>
              Bank Authorization
            </div>
          </FadeIn>
          <FadeIn delay={16} duration={15}>
            <div style={{ fontSize: 15, color: THEME.textSecondary, fontFamily: THEME.fontSans, lineHeight: 1.6, marginBottom: 28 }}>
              One tool call starts the full OAuth 2.0 flow. A background listener automatically captures the authorization code — no manual URL pasting.
            </div>
          </FadeIn>
          <Step num={1} label="Call start_authorization" desc="Pass bank name, country, state UUID, redirect URL" delay={28} done />
          <Step num={2} label="User opens browser URL" desc="Bank login page — Nordea, OP, Swedbank, etc." delay={38} done />
          <Step num={3} label="Auto-captures callback code" desc="Background HTTPS listener on localhost:8080" delay={48} done />
          <Step num={4} label="Call create_session" desc="Exchange code → persistent session saved to disk" delay={58} />
          <FadeIn delay={70} duration={15} style={{ marginTop: 20 }}>
            <div style={{ display: 'flex', gap: 10 }}>
              <Badge label="OAuth 2.0" color={THEME.blue} delay={70} size="sm" />
              <Badge label="PKCE ready" color={THEME.accent} delay={75} size="sm" />
              <Badge label="Self-signed TLS" color={THEME.purple} delay={80} size="sm" />
            </div>
          </FadeIn>
        </div>
        {/* Right: code terminal */}
        <div style={{ width: 440 }}>
          <FadeIn delay={20} duration={15}>
            <div style={{
              background: THEME.bgCode, borderRadius: 12,
              border: `1px solid ${THEME.border}`,
              padding: '16px 20px',
              boxShadow: `0 8px 40px rgba(0,0,0,0.5)`,
            }}>
              <div style={{ display: 'flex', gap: 6, marginBottom: 16 }}>
                <div style={{ width: 12, height: 12, borderRadius: '50%', background: '#ff5f57' }} />
                <div style={{ width: 12, height: 12, borderRadius: '50%', background: '#febc2e' }} />
                <div style={{ width: 12, height: 12, borderRadius: '50%', background: '#28c840' }} />
                <span style={{ marginLeft: 8, fontSize: 12, color: THEME.textSecondary, fontFamily: THEME.fontMono }}>
                  AI → MCP tool call
                </span>
              </div>
              <CodeLine code={`// 1. Start authorization`} delay={25} />
              <CodeLine code={`start_authorization({`} delay={30} />
              <CodeLine code={`  bank_name: "Nordea",`} delay={35} />
              <CodeLine code={`  country: "FI",`} delay={40} />
              <CodeLine code={`  state: "uuid-v4",`} delay={45} highlight />
              <CodeLine code={`  redirect_url: "https://localhost:8080/callback",`} delay={50} />
              <CodeLine code={`})`} delay={55} />
              <div style={{ margin: '12px 0', borderTop: `1px solid ${THEME.border}` }} />
              <CodeLine code={`// 2. Get captured code`} delay={60} />
              <CodeLine code={`get_captured_code()  `} comment="→ { code: 'abc...' }" delay={65} highlight />
              <div style={{ margin: '12px 0', borderTop: `1px solid ${THEME.border}` }} />
              <CodeLine code={`// 3. Create session`} delay={70} />
              <CodeLine code={`create_session({ code, label: "Nordea FI" })`} delay={75} highlight />
            </div>
          </FadeIn>
        </div>
      </div>
    </AbsoluteFill>
  );
};

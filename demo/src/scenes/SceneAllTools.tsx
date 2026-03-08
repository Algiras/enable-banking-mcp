import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
const tools = [
  { name: 'setup_guide',            cat: 'Setup',    color: THEME.textSecondary },
  { name: 'configure_secrets',      cat: 'Setup',    color: THEME.textSecondary },
  { name: 'get_available_banks',    cat: 'Discovery',color: THEME.blue },
  { name: 'get_application',        cat: 'Discovery',color: THEME.blue },
  { name: 'start_authorization',    cat: 'Auth',     color: THEME.yellow },
  { name: 'get_captured_code',      cat: 'Auth',     color: THEME.yellow },
  { name: 'create_session',         cat: 'Sessions', color: THEME.accent },
  { name: 'list_sessions',          cat: 'Sessions', color: THEME.accent },
  { name: 'get_session',            cat: 'Sessions', color: THEME.accent },
  { name: 'delete_session',         cat: 'Sessions', color: THEME.accent },
  { name: 'list_accounts',          cat: 'Accounts', color: THEME.purple },
  { name: 'get_account_details',    cat: 'Accounts', color: THEME.purple },
  { name: 'get_account_balances',   cat: 'Accounts', color: THEME.purple },
  { name: 'get_account_transactions', cat: 'Accounts', color: THEME.purple },
  { name: 'get_transaction_details','cat': 'Accounts', color: THEME.purple },
  { name: 'spending_summary',       cat: 'Analytics',color: '#f97316' },
  { name: 'create_payment',         cat: 'Payments', color: THEME.red },
  { name: 'get_payment',            cat: 'Payments', color: THEME.red },
  { name: 'delete_payment',         cat: 'Payments', color: THEME.red },
];
const ToolPill: React.FC<{ tool: typeof tools[0]; delay: number }> = ({ tool, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 14, stiffness: 200 } });
  const scale = interpolate(s, [0, 1], [0.7, 1]);
  const opacity = interpolate(s, [0, 0.4], [0, 1], { extrapolateRight: 'clamp' });
  return (
    <div style={{
      transform: `scale(${scale})`, opacity,
      display: 'inline-flex', alignItems: 'center', gap: 6,
      background: `${tool.color}12`,
      border: `1px solid ${tool.color}44`,
      borderRadius: 8, padding: '5px 10px',
      fontSize: 12, fontFamily: THEME.fontMono, color: tool.color,
    }}>
      {tool.name}
    </div>
  );
};
export const SceneAllTools: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  // Counter for tool count
  const counted = Math.floor(interpolate(frame, [15, 50], [0, 19], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  }));
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '50px 100px', opacity: fadeOut }}>
      <FadeIn delay={0} duration={15}>
        <div style={{
          fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent,
          letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8,
        }}>
          Complete Toolset
        </div>
      </FadeIn>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 16, marginBottom: 8 }}>
        <FadeIn delay={8} duration={18}>
          <div style={{ fontSize: 42, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans }}>
            All{' '}
            <span style={{ color: THEME.accent, fontFamily: THEME.fontMono, fontSize: 50 }}>{counted}</span>
            {' '}Tools
          </div>
        </FadeIn>
      </div>
      <FadeIn delay={14} duration={15}>
        <div style={{ fontSize: 16, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginBottom: 30 }}>
          6 categories · from discovery to payment completion
        </div>
      </FadeIn>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
        {tools.map((tool, i) => (
          <ToolPill key={tool.name} tool={tool} delay={20 + i * 4} />
        ))}
      </div>
      {/* Stats row */}
      <FadeIn delay={100} duration={15} style={{ marginTop: 36 }}>
        <div style={{ display: 'flex', gap: 30 }}>
          {[
            { label: 'Tools', value: '19', color: THEME.accent },
            { label: 'UI Resources', value: '6', color: THEME.purple },
            { label: 'Banks', value: '500+', color: THEME.blue },
            { label: 'Countries', value: '30+', color: THEME.yellow },
            { label: 'Binary size', value: '4.5 MB', color: THEME.red },
          ].map(({ label, value, color }) => (
            <div key={label} style={{
              background: `${color}12`, border: `1px solid ${color}44`,
              borderRadius: 10, padding: '12px 20px', textAlign: 'center',
            }}>
              <div style={{ fontSize: 24, fontWeight: 800, color, fontFamily: THEME.fontMono }}>{value}</div>
              <div style={{ fontSize: 12, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginTop: 4 }}>{label}</div>
            </div>
          ))}
        </div>
      </FadeIn>
    </AbsoluteFill>
  );
};

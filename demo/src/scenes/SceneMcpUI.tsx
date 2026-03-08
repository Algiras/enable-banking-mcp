import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';

// Mock inline dashboard previews (no external image dependency)
const BalancePreview: React.FC = () => (
  <div style={{ padding: '10px 12px', background: THEME.bg }}>
    {[
      { bank: 'Nordea FI', bal: '4,250.00', color: THEME.blue },
      { bank: 'Swedbank LT', bal: '1,830.50', color: THEME.accent },
      { bank: 'OP Finland', bal: '980.20', color: THEME.purple },
    ].map(({ bank, bal, color }) => (
      <div key={bank} style={{ display: 'flex', justifyContent: 'space-between', padding: '6px 0', borderBottom: '1px solid ' + THEME.border, fontSize: 11 }}>
        <span style={{ color, fontFamily: THEME.fontSans }}>{bank}</span>
        <span style={{ color: THEME.textPrimary, fontFamily: THEME.fontMono, fontWeight: 700 }}>{bal} €</span>
      </div>
    ))}
  </div>
);

const TransactionPreview: React.FC = () => (
  <div style={{ padding: '8px 12px', background: THEME.bg }}>
    {[
      { desc: 'Lidl Helsinki', amt: '-42.30', color: '#ff4d6d' },
      { desc: 'Salary — Acme', amt: '+3500.00', color: THEME.accent },
      { desc: 'Netflix', amt: '-15.99', color: '#ff4d6d' },
      { desc: 'Bolt Ride', amt: '-8.40', color: '#ff4d6d' },
    ].map(({ desc, amt, color }) => (
      <div key={desc} style={{ display: 'flex', justifyContent: 'space-between', padding: '5px 0', borderBottom: '1px solid ' + THEME.border, fontSize: 11 }}>
        <span style={{ color: THEME.textPrimary, fontFamily: THEME.fontSans }}>{desc}</span>
        <span style={{ color, fontFamily: THEME.fontMono, fontWeight: 700 }}>{amt}</span>
      </div>
    ))}
  </div>
);

const SpendingPreview: React.FC = () => (
  <div style={{ padding: '10px 12px', background: THEME.bg }}>
    {[
      { cat: 'Groceries', pct: 100, color: THEME.accent },
      { cat: 'Shopping', pct: 68, color: '#f97316' },
      { cat: 'Dining', pct: 64, color: THEME.purple },
      { cat: 'Transport', pct: 46, color: THEME.blue },
    ].map(({ cat, pct, color }) => (
      <div key={cat} style={{ marginBottom: 8 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 10, marginBottom: 3 }}>
          <span style={{ color: THEME.textSecondary, fontFamily: THEME.fontSans }}>{cat}</span>
          <span style={{ color, fontFamily: THEME.fontMono }}>{pct}%</span>
        </div>
        <div style={{ height: 6, background: color + '22', borderRadius: 3 }}>
          <div style={{ width: pct + '%', height: '100%', background: color, borderRadius: 3 }} />
        </div>
      </div>
    ))}
  </div>
);

const SessionPreview: React.FC = () => (
  <div style={{ padding: '8px 12px', background: THEME.bg }}>
    {[
      { bank: 'Nordea FI', status: 'AUTHORIZED', color: THEME.accent },
      { bank: 'Swedbank LT', status: 'AUTHORIZED', color: THEME.accent },
    ].map(({ bank, status, color }) => (
      <div key={bank} style={{ padding: '8px 0', borderBottom: '1px solid ' + THEME.border }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: THEME.textPrimary, fontFamily: THEME.fontSans }}>{bank}</div>
        <div style={{ fontSize: 10, color, fontFamily: THEME.fontMono, marginTop: 2 }}>{status}</div>
      </div>
    ))}
  </div>
);

const UICard: React.FC<{
  label: string; desc: string; delay: number;
  children: React.ReactNode;
}> = ({ label, desc, delay, children }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity = interpolate(s, [0, 1], [0, 1]);
  const y = interpolate(s, [0, 1], [20, 0]);

  return (
    <div style={{ opacity, transform: "translateY(" + y + "px)", flex: 1 }}>
      <div style={{
        background: THEME.bgCard, border: '1px solid ' + THEME.border,
        borderRadius: 10, overflow: 'hidden',
        boxShadow: '0 4px 20px rgba(0,0,0,0.4)', marginBottom: 8,
      }}>
        <div style={{
          background: '#1a1f2e', padding: '6px 10px',
          display: 'flex', alignItems: 'center', gap: 5,
          borderBottom: '1px solid ' + THEME.border,
        }}>
          <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#ff5f57' }} />
          <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#febc2e' }} />
          <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#28c840' }} />
          <div style={{ flex: 1, marginLeft: 6, background: '#0d1117', borderRadius: 3, padding: '2px 8px', fontSize: 9, color: THEME.textSecondary, fontFamily: THEME.fontMono }}>
            ui://{label.toLowerCase()}
          </div>
        </div>
        {children}
      </div>
      <div style={{ fontSize: 12, fontWeight: 700, color: THEME.textPrimary, fontFamily: THEME.fontSans }}>{label}</div>
      <div style={{ fontSize: 10, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginTop: 2 }}>{desc}</div>
    </div>
  );
};

export const SceneMcpUI: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '44px 80px', opacity: fadeOut }}>
      <FadeIn delay={0} duration={15}>
        <div style={{ fontSize: 12, fontFamily: THEME.fontMono, color: THEME.accent, letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8 }}>
          Feature 6 · MCP Apps UI
        </div>
      </FadeIn>
      <FadeIn delay={8} duration={18}>
        <div style={{ fontSize: 38, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 8 }}>
          Visual Dashboards
        </div>
      </FadeIn>
      <FadeIn delay={16} duration={15}>
        <div style={{ fontSize: 15, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginBottom: 24, maxWidth: 700 }}>
          HTML apps embedded directly in AI clients. Hosted as{' '}
          <span style={{ color: THEME.accent, fontFamily: THEME.fontMono }}>ui://</span> MCP resources —
          no external URLs, no configuration.
        </div>
      </FadeIn>

      <div style={{ display: 'flex', gap: 16 }}>
        <UICard label="Balances" desc="Real-time balance cards" delay={28}><BalancePreview /></UICard>
        <UICard label="Transactions" desc="Sortable transaction table" delay={38}><TransactionPreview /></UICard>
        <UICard label="Spending" desc="Category bar chart" delay={48}><SpendingPreview /></UICard>
        <UICard label="Sessions" desc="Session status cards" delay={58}><SessionPreview /></UICard>
      </div>

      <FadeIn delay={70} duration={15} style={{ marginTop: 16 }}>
        <div style={{
          display: 'flex', gap: 12, alignItems: 'center',
          background: THEME.purple + '12', border: '1px solid ' + THEME.purple + '44',
          borderRadius: 8, padding: '10px 16px', fontSize: 13,
        }}>
          <span style={{ fontSize: 18 }}>💡</span>
          <span style={{ color: THEME.textSecondary, fontFamily: THEME.fontSans }}>
            Works in <span style={{ color: THEME.purple, fontWeight: 600 }}>Claude Desktop</span> · MIME type:{' '}
            <span style={{ color: THEME.accent, fontFamily: THEME.fontMono }}>text/html;profile=mcp-app</span>
          </span>
        </div>
      </FadeIn>
    </AbsoluteFill>
  );
};

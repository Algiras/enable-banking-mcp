import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
import { Badge } from '../components/Badge';
const categories = [
  { label: 'Groceries',     amount: 342.80, color: THEME.accent },
  { label: 'Dining',        amount: 218.50, color: THEME.purple },
  { label: 'Transport',     amount: 156.20, color: THEME.blue },
  { label: 'Subscriptions', amount: 89.97,  color: THEME.yellow },
  { label: 'Shopping',      amount: 234.10, color: '#f97316' },
  { label: 'Healthcare',    amount: 67.40,  color: THEME.red },
];
const maxAmount = Math.max(...categories.map(c => c.amount));
const Bar: React.FC<{ cat: typeof categories[0]; delay: number; rank: number }> = ({ cat, delay, rank }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const widthPct = (cat.amount / maxAmount) * 60;
  const width = interpolate(s, [0, 1], [0, widthPct]);
  const opacity = interpolate(s, [0, 0.3], [0, 1], { extrapolateRight: 'clamp' });
  const amountOpacity = interpolate(frame, [delay + 15, delay + 25], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  return (
    <div style={{ opacity, marginBottom: 14 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 5 }}>
        <div style={{ fontSize: 13, color: THEME.textPrimary, fontFamily: THEME.fontSans, fontWeight: 500 }}>
          {rank + 1}. {cat.label}
        </div>
        <div style={{ fontSize: 13, color: cat.color, fontFamily: THEME.fontMono, fontWeight: 700, opacity: amountOpacity }}>
          {cat.amount.toFixed(2)} €
        </div>
      </div>
      <div style={{ height: 10, background: `${cat.color}18`, borderRadius: 5, overflow: 'hidden' }}>
        <div style={{
          height: '100%', borderRadius: 5,
          background: `linear-gradient(90deg, ${cat.color}aa, ${cat.color})`,
          width: `${width}%`,
          boxShadow: `0 0 8px ${cat.color}66`,
        }} />
      </div>
    </div>
  );
};
export const SceneSpending: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  const totalCounted = interpolate(frame, [30, 60], [0, 1109.0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeOut }}>
      <div style={{ display: 'flex', gap: 70 }}>
        <div style={{ width: 340, flexShrink: 0 }}>
          <FadeIn delay={0} duration={15}>
            <div style={{ fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent, letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8 }}>
              Feature 4
            </div>
          </FadeIn>
          <FadeIn delay={8} duration={18}>
            <div style={{ fontSize: 38, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 16 }}>
              Spending Summary
            </div>
          </FadeIn>
          <FadeIn delay={16} duration={15}>
            <div style={{ fontSize: 15, color: THEME.textSecondary, fontFamily: THEME.fontSans, lineHeight: 1.6, marginBottom: 20 }}>
              Ask your AI "what am I spending on?" and get an instant breakdown. Aggregates all transactions across pages, grouped by category from remittance info.
            </div>
          </FadeIn>
          <FadeIn delay={24} duration={15}>
            <div style={{ fontFamily: THEME.fontMono, fontSize: 12, background: THEME.bgCode, borderRadius: 10, padding: '14px 18px', border: `1px solid ${THEME.border}`, marginBottom: 20 }}>
              <div style={{ color: THEME.textPrimary }}>spending_summary({'{'}</div>
              <div style={{ color: THEME.accent, paddingLeft: 16 }}>account_id, session_id,</div>
              <div style={{ color: THEME.blue, paddingLeft: 16 }}>date_from: "2025-01-01",</div>
              <div style={{ color: THEME.blue, paddingLeft: 16 }}>date_to: "2025-03-01"</div>
              <div style={{ color: THEME.textPrimary }}>{'}'}</div>
            </div>
          </FadeIn>
          {/* Total spend */}
          <FadeIn delay={28} duration={15}>
            <div style={{
              background: `${THEME.accent}12`, border: `1px solid ${THEME.accent}44`,
              borderRadius: 12, padding: '14px 18px',
            }}>
              <div style={{ fontSize: 12, color: THEME.textSecondary, fontFamily: THEME.fontSans }}>Total spend (Feb 2025)</div>
              <div style={{ fontSize: 32, fontWeight: 800, color: THEME.accent, fontFamily: THEME.fontMono }}>
                {totalCounted.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} €
              </div>
            </div>
          </FadeIn>
          <div style={{ display: 'flex', gap: 8, marginTop: 14, flexWrap: 'wrap' }}>
            <Badge label="Auto-aggregated" color={THEME.accent} delay={45} size="sm" />
            <Badge label="Visual chart UI" color={THEME.purple} delay={52} size="sm" />
          </div>
        </div>
        {/* Chart */}
        <div style={{ flex: 1, paddingTop: 20 }}>
          <FadeIn delay={20} duration={15}>
            <div style={{ fontSize: 15, fontWeight: 700, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 20 }}>
              Spending by Category
            </div>
          </FadeIn>
          {categories.map((cat, i) => (
            <Bar key={cat.label} cat={cat} delay={28 + i * 8} rank={i} />
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
};

import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
import { Badge } from '../components/Badge';
const BalanceCard: React.FC<{
  bank: string; iban: string; balance: string; currency: string;
  type: string; color: string; delay: number;
}> = ({ bank, iban, balance, currency, type, color, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 14, stiffness: 160 } });
  const scale = interpolate(s, [0, 1], [0.85, 1]);
  const opacity = interpolate(s, [0, 0.3], [0, 1], { extrapolateRight: 'clamp' });
  const y = interpolate(s, [0, 1], [20, 0]);
  // Animate balance counter
  const num = parseFloat(balance.replace(',', ''));
  const countedNum = interpolate(frame, [delay, delay + 30], [0, num], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  const displayBalance = countedNum.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return (
    <div style={{
      opacity, transform: `scale(${scale}) translateY(${y}px)`,
      background: THEME.bgCard, border: `1px solid ${color}55`,
      borderRadius: 14, padding: '18px 20px', minWidth: 220,
      boxShadow: `0 4px 24px ${color}18`,
      position: 'relative', overflow: 'hidden',
    }}>
      <div style={{
        position: 'absolute', top: 0, right: 0, width: 80, height: 80,
        background: `radial-gradient(circle, ${color}22, transparent 70%)`,
      }} />
      <div style={{ fontSize: 13, fontWeight: 700, color, fontFamily: THEME.fontSans, marginBottom: 6 }}>{bank}</div>
      <div style={{ fontSize: 11, color: THEME.textSecondary, fontFamily: THEME.fontMono, marginBottom: 14 }}>
        {iban.slice(0, 12)}···
      </div>
      <div style={{ fontSize: 28, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontMono }}>
        {displayBalance}
        <span style={{ fontSize: 16, color: THEME.textSecondary, marginLeft: 6 }}>{currency}</span>
      </div>
      <div style={{
        marginTop: 10, fontSize: 11, color: color,
        background: `${color}15`, borderRadius: 4, padding: '3px 8px',
        display: 'inline-block', fontFamily: THEME.fontMono,
      }}>{type}</div>
    </div>
  );
};
export const SceneAccountData: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeOut }}>
      <div style={{ display: 'flex', gap: 80 }}>
        <div style={{ flex: 1 }}>
          <FadeIn delay={0} duration={15}>
            <div style={{ fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent, letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8 }}>
              Feature 2
            </div>
          </FadeIn>
          <FadeIn delay={8} duration={18}>
            <div style={{ fontSize: 38, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 16 }}>
              Account Balances & Data
            </div>
          </FadeIn>
          <FadeIn delay={16} duration={15}>
            <div style={{ fontSize: 15, color: THEME.textSecondary, fontFamily: THEME.fontSans, lineHeight: 1.6, marginBottom: 28 }}>
              List all accounts in a session, then query real-time balances.
              The <span style={{ color: THEME.accent, fontFamily: THEME.fontMono }}>list_accounts</span> tool returns
              account UIDs needed for all subsequent calls.
            </div>
          </FadeIn>
          <FadeIn delay={26} duration={15}>
            <div style={{ fontFamily: THEME.fontMono, fontSize: 13, background: THEME.bgCode, borderRadius: 10, padding: '14px 18px', border: `1px solid ${THEME.border}`, marginBottom: 20 }}>
              <div style={{ color: THEME.textSecondary, marginBottom: 8 }}>{'// List accounts for a session'}</div>
              <div style={{ color: THEME.textPrimary }}>list_accounts({'{'} session_id {'}'})</div>
              <div style={{ color: THEME.textSecondary, margin: '8px 0' }}>{'// Get real-time balance'}</div>
              <div style={{ color: THEME.accent }}>get_account_balances({'{'} account_id, session_id {'}'})</div>
            </div>
          </FadeIn>
          <div style={{ display: 'flex', gap: 10, marginTop: 8 }}>
            <Badge label="CLBD balance" color={THEME.accent} delay={40} size="sm" />
            <Badge label="ITAV balance" color={THEME.blue} delay={46} size="sm" />
            <Badge label="Visual dashboard" color={THEME.purple} delay={52} size="sm" />
          </div>
        </div>
        {/* Balance cards preview */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 14, justifyContent: 'center' }}>
          <BalanceCard bank="Nordea FI" iban="FI2112345600000785" balance="4250.00" currency="EUR" type="CACC · Closing Ledger" color={THEME.blue} delay={30} />
          <BalanceCard bank="Swedbank LT" iban="LT647044001231234" balance="1830.50" currency="EUR" type="CACC · Available" color={THEME.accent} delay={42} />
          <BalanceCard bank="OP Finland" iban="FI5810773101900171" balance="980.20" currency="EUR" type="SVGS · Closing Ledger" color={THEME.purple} delay={54} />
        </div>
      </div>
    </AbsoluteFill>
  );
};

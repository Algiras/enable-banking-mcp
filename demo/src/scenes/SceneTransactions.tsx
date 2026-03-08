import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
import { Badge } from '../components/Badge';
const TxRow: React.FC<{
  date: string; desc: string; amount: string; status: string; delay: number;
}> = ({ date, desc, amount, status, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity = interpolate(s, [0, 1], [0, 1]);
  const x = interpolate(s, [0, 1], [30, 0]);
  const isDebit = amount.startsWith('-');
  return (
    <div style={{
      opacity, transform: `translateX(${x}px)`,
      display: 'grid', gridTemplateColumns: '90px 1fr 100px 70px',
      gap: 16, padding: '10px 16px', alignItems: 'center',
      borderBottom: `1px solid ${THEME.border}`,
      fontSize: 13, fontFamily: THEME.fontSans,
    }}>
      <div style={{ color: THEME.textSecondary, fontFamily: THEME.fontMono, fontSize: 12 }}>{date}</div>
      <div style={{ color: THEME.textPrimary }}>{desc}</div>
      <div style={{ color: isDebit ? THEME.red : THEME.accent, fontFamily: THEME.fontMono, fontWeight: 700, textAlign: 'right' }}>
        {amount}
      </div>
      <div style={{
        fontSize: 11, textAlign: 'center', padding: '3px 8px', borderRadius: 4,
        background: status === 'BOOK' ? `${THEME.accent}18` : `${THEME.yellow}18`,
        color: status === 'BOOK' ? THEME.accent : THEME.yellow,
        fontFamily: THEME.fontMono,
      }}>{status}</div>
    </div>
  );
};
export const SceneTransactions: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeOut }}>
      <div style={{ display: 'flex', gap: 60 }}>
        <div style={{ width: 340, flexShrink: 0 }}>
          <FadeIn delay={0} duration={15}>
            <div style={{ fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent, letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8 }}>
              Feature 3
            </div>
          </FadeIn>
          <FadeIn delay={8} duration={18}>
            <div style={{ fontSize: 38, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 16 }}>
              Transaction History
            </div>
          </FadeIn>
          <FadeIn delay={16} duration={15}>
            <div style={{ fontSize: 15, color: THEME.textSecondary, fontFamily: THEME.fontSans, lineHeight: 1.6, marginBottom: 20 }}>
              Automatic pagination across all pages. Filter by date range and status. Returns booked and pending transactions.
            </div>
          </FadeIn>
          <FadeIn delay={24} duration={15}>
            <div style={{ fontFamily: THEME.fontMono, fontSize: 12, background: THEME.bgCode, borderRadius: 10, padding: '14px 18px', border: `1px solid ${THEME.border}`, marginBottom: 20 }}>
              <div style={{ color: THEME.textSecondary }}>{'// Auto-paginates all pages'}</div>
              <div style={{ color: THEME.textPrimary, marginTop: 6 }}>get_account_transactions({'{'}</div>
              <div style={{ color: THEME.purple, paddingLeft: 16 }}>account_id, session_id,</div>
              <div style={{ color: THEME.accent, paddingLeft: 16 }}>date_from: "2025-01-01",</div>
              <div style={{ color: THEME.accent, paddingLeft: 16 }}>date_to: "2025-03-01",</div>
              <div style={{ color: THEME.textSecondary, paddingLeft: 16 }}>transaction_status: "BOOK"</div>
              <div style={{ color: THEME.textPrimary }}>{'}'}</div>
            </div>
          </FadeIn>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
            <Badge label="Auto-paginates" color={THEME.accent} delay={40} size="sm" />
            <Badge label="BOOKED + PENDING" color={THEME.yellow} delay={46} size="sm" />
            <Badge label="Date filtering" color={THEME.blue} delay={52} size="sm" />
          </div>
        </div>
        {/* Transaction table */}
        <div style={{ flex: 1 }}>
          <FadeIn delay={22} duration={15}>
            <div style={{
              background: THEME.bgCard, borderRadius: 12,
              border: `1px solid ${THEME.border}`,
              overflow: 'hidden',
            }}>
              {/* Header */}
              <div style={{
                display: 'grid', gridTemplateColumns: '90px 1fr 100px 70px',
                gap: 16, padding: '12px 16px',
                background: `${THEME.border}66`,
                fontSize: 11, fontFamily: THEME.fontMono, color: THEME.textSecondary,
                textTransform: 'uppercase', letterSpacing: '0.05em',
              }}>
                <div>Date</div><div>Description</div><div style={{ textAlign: 'right' }}>Amount</div><div style={{ textAlign: 'center' }}>Status</div>
              </div>
              <TxRow date="2025-02-28" desc="Supermarket Lidl Helsinki" amount="-42.30 €" status="BOOK" delay={32} />
              <TxRow date="2025-02-27" desc="Salary — Acme Corp" amount="+3500.00 €" status="BOOK" delay={38} />
              <TxRow date="2025-02-26" desc="Netflix Subscription" amount="-15.99 €" status="BOOK" delay={44} />
              <TxRow date="2025-02-25" desc="Bolt Ride Tallinn" amount="-8.40 €" status="BOOK" delay={50} />
              <TxRow date="2025-02-24" desc="Kaffila Coffee Roasters" amount="-6.80 €" status="BOOK" delay={56} />
              <TxRow date="2025-02-23" desc="Freelance Invoice #042" amount="+850.00 €" status="BOOK" delay={62} />
              <TxRow date="2025-02-22" desc="Wolt Food Delivery" amount="-24.50 €" status="PDNG" delay={68} />
            </div>
          </FadeIn>
          <FadeIn delay={75} duration={15} style={{ marginTop: 12 }}>
            <div style={{
              fontSize: 13, color: THEME.textSecondary, fontFamily: THEME.fontMono,
              textAlign: 'right',
            }}>
              ✓ 847 transactions · 3 pages fetched automatically
            </div>
          </FadeIn>
        </div>
      </div>
    </AbsoluteFill>
  );
};

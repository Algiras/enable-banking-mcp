import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
import { FadeIn } from '../components/FadeIn';
import { Badge } from '../components/Badge';
const PaymentStep: React.FC<{ step: number; label: string; desc: string; delay: number; active?: boolean }> = ({ step, label, desc, delay, active }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity = interpolate(s, [0, 1], [0, 1]);
  const x = interpolate(s, [0, 1], [20, 0]);
  return (
    <div style={{ opacity, transform: `translateX(${x}px)`, display: 'flex', gap: 14, marginBottom: 18, alignItems: 'flex-start' }}>
      <div style={{
        width: 28, height: 28, borderRadius: '50%', flexShrink: 0,
        background: active ? THEME.accent : `${THEME.accent}22`,
        border: `2px solid ${THEME.accent}`,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontSize: 12, fontWeight: 700, fontFamily: THEME.fontMono,
        color: active ? THEME.bg : THEME.accent,
      }}>{step}</div>
      <div>
        <div style={{ fontSize: 14, fontWeight: 700, color: THEME.textPrimary, fontFamily: THEME.fontSans }}>{label}</div>
        <div style={{ fontSize: 12, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginTop: 2 }}>{desc}</div>
      </div>
    </div>
  );
};
export const ScenePayments: React.FC = () => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const fadeOut = interpolate(frame, [durationInFrames - 15, durationInFrames], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill style={{ background: THEME.bg, padding: '60px 100px', opacity: fadeOut }}>
      <div style={{ display: 'flex', gap: 70 }}>
        <div style={{ flex: 1 }}>
          <FadeIn delay={0} duration={15}>
            <div style={{ fontSize: 13, fontFamily: THEME.fontMono, color: THEME.accent, letterSpacing: '0.25em', textTransform: 'uppercase', marginBottom: 8 }}>
              Feature 5
            </div>
          </FadeIn>
          <FadeIn delay={8} duration={18}>
            <div style={{ fontSize: 38, fontWeight: 800, color: THEME.textPrimary, fontFamily: THEME.fontSans, marginBottom: 16 }}>
              Payment Initiation
            </div>
          </FadeIn>
          <FadeIn delay={16} duration={15}>
            <div style={{ fontSize: 15, color: THEME.textSecondary, fontFamily: THEME.fontSans, lineHeight: 1.6, marginBottom: 24 }}>
              Ask your AI to send a payment. It calls <span style={{ color: THEME.accent, fontFamily: THEME.fontMono }}>create_payment</span>,
              you authorize in your bank, and the AI polls status automatically.
            </div>
          </FadeIn>
          <PaymentStep step={1} label="Call create_payment" desc="Bank name, amount, recipient IBAN, reference" delay={26} active />
          <PaymentStep step={2} label="Open authorization URL" desc="User approves payment in their bank app/web" delay={36} />
          <PaymentStep step={3} label="get_payment polls status" desc="ACSP → ACCC (accepted → completed)" delay={46} />
          <PaymentStep step={4} label="get_payment_transaction" desc="Link back to the booked bank transaction" delay={56} />
          <FadeIn delay={65} duration={15} style={{ marginTop: 16 }}>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
              <Badge label="SEPA" color={THEME.accent} delay={65} size="sm" />
              <Badge label="INST_SEPA" color={THEME.blue} delay={70} size="sm" />
              <Badge label="Domestic" color={THEME.purple} delay={75} size="sm" />
              <Badge label="Webhooks" color={THEME.yellow} delay={80} size="sm" />
              <Badge label="Future-dated" color={THEME.red} delay={85} size="sm" />
            </div>
          </FadeIn>
        </div>
        {/* Payment card mockup */}
        <div style={{ width: 380, flexShrink: 0, paddingTop: 30 }}>
          <FadeIn delay={20} duration={18}>
            <div style={{
              background: THEME.bgCard, border: `1px solid ${THEME.border}`,
              borderRadius: 16, padding: '24px', boxShadow: `0 8px 40px rgba(0,0,0,0.4)`,
            }}>
              <div style={{ fontSize: 13, color: THEME.textSecondary, fontFamily: THEME.fontSans, marginBottom: 16 }}>
                Payment Request
              </div>
              {[
                { label: 'To', value: 'Acme Corp Ltd' },
                { label: 'IBAN', value: 'LT64 7044 0012 3123 4567' },
                { label: 'Amount', value: '€ 1,250.00' },
                { label: 'Reference', value: 'INV-2025-042' },
                { label: 'Type', value: 'SEPA Credit Transfer' },
                { label: 'Bank', value: 'Nordea Finland' },
              ].map(({ label, value }, i) => (
                <FadeIn key={label} delay={28 + i * 6} duration={10}>
                  <div style={{
                    display: 'flex', justifyContent: 'space-between',
                    padding: '8px 0', borderBottom: `1px solid ${THEME.border}`,
                    fontSize: 13,
                  }}>
                    <span style={{ color: THEME.textSecondary, fontFamily: THEME.fontSans }}>{label}</span>
                    <span style={{ color: THEME.textPrimary, fontFamily: label === 'Amount' ? THEME.fontMono : THEME.fontSans, fontWeight: label === 'Amount' ? 700 : 400 }}>
                      {value}
                    </span>
                  </div>
                </FadeIn>
              ))}
              <FadeIn delay={70} duration={15} style={{ marginTop: 18 }}>
                <div style={{
                  background: `${THEME.accent}22`, border: `1px solid ${THEME.accent}66`,
                  borderRadius: 8, padding: '10px 16px', textAlign: 'center',
                  fontSize: 14, fontWeight: 700, color: THEME.accent, fontFamily: THEME.fontSans,
                }}>
                  ⚡ Authorize in your bank →
                </div>
              </FadeIn>
            </div>
          </FadeIn>
        </div>
      </div>
    </AbsoluteFill>
  );
};

import React from 'react';
import { interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { THEME } from '../theme';
export const CodeLine: React.FC<{
  code: string;
  comment?: string;
  delay?: number;
  highlight?: boolean;
}> = ({ code, comment, delay = 0, highlight = false }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const progress = spring({ frame: frame - delay, fps, config: { damping: 200 } });
  const opacity  = interpolate(progress, [0, 1], [0, 1]);
  const x        = interpolate(progress, [0, 1], [-20, 0]);
  const parts = code.split(/(\$[\w_]+|"[^"]*"|\b(fn|let|async|await|pub|use|match|Ok|Err|Some|None)\b)/g)
    .filter(Boolean);
  return (
    <div style={{
      opacity, transform: `translateX(${x}px)`,
      fontFamily: THEME.fontMono, fontSize: 14, lineHeight: '1.7',
      padding: '3px 0',
      background: highlight ? 'rgba(0,200,150,0.07)' : 'transparent',
      borderLeft: highlight ? `2px solid ${THEME.accent}` : '2px solid transparent',
      paddingLeft: 12,
    }}>
      {parts.map((part, i) => {
        if (/^\$[\w_]+$/.test(part)) return <span key={i} style={{ color: '#f97316' }}>{part}</span>;
        if (/^"/.test(part)) return <span key={i} style={{ color: '#a3e635' }}>{part}</span>;
        if (/^(fn|let|async|await|pub|use|match|Ok|Err|Some|None)$/.test(part)) return <span key={i} style={{ color: THEME.purple }}>{part}</span>;
        return <span key={i} style={{ color: THEME.textPrimary }}>{part}</span>;
      })}
      {comment && <span style={{ color: THEME.textSecondary, marginLeft: 8 }}>{comment}</span>}
    </div>
  );
};

import { globalStyle, style } from '@vanilla-extract/css';
import { accent, bg, border, monoFont, muted, surface } from '@/styles/tokens';

export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(3, minmax(0, 1fr))',
  gap: 1,
  background: border,
  border: `1px solid ${border}`,
  borderRadius: 14,
  overflow: 'hidden',
  '@media': {
    '(max-width: 900px)': { gridTemplateColumns: 'repeat(2, minmax(0, 1fr))' },
    '(max-width: 600px)': { gridTemplateColumns: 'minmax(0, 1fr)' },
  },
});

export const feature = style({
  background: bg,
  padding: 28,
  display: 'grid',
  gap: 10,
  alignContent: 'start',
  minWidth: 0,
});

export const key = style({
  fontFamily: monoFont,
  fontSize: '0.8rem',
  color: muted,
});

globalStyle(`${key} kbd`, {
  color: accent,
  border: `1px solid ${border}`,
  borderRadius: 4,
  padding: '0 6px',
  background: surface,
});

export const body = style({
  color: muted,
  fontSize: '0.98rem',
});

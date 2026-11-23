import { style } from '@vanilla-extract/css';
import { bg, border, gutter, maxContentWidth, muted, text } from '@/styles/tokens';

export const header = style({
  position: 'sticky',
  top: 'env(safe-area-inset-top, 0px)',
  zIndex: 5,
  background: `color-mix(in srgb, ${bg} 88%, transparent)`,
  backdropFilter: 'blur(10px)',
  borderBottom: `1px solid ${border}`,
});

export const bar = style({
  maxWidth: maxContentWidth,
  height: 64,
  margin: '0 auto',
  paddingInline: gutter,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: 24,
});

export const links = style({
  display: 'flex',
  alignItems: 'center',
  gap: 28,
  fontSize: '0.95rem',
});

export const link = style({
  color: muted,
  textDecoration: 'none',
  ':hover': { color: text },
});

export const wide = style({
  '@media': { '(max-width: 760px)': { display: 'none' } },
});

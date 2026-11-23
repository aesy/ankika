import { style } from '@vanilla-extract/css';
import { border, gutter, maxContentWidth, muted, text } from '@/styles/tokens';

export const footer = style({
  borderTop: `1px solid ${border}`,
  paddingBlock: 40,
  color: muted,
  fontSize: '0.9rem',
});

export const inner = style({
  maxWidth: maxContentWidth,
  margin: '0 auto',
  paddingInline: gutter,
  display: 'grid',
  gap: 20,
});

export const row = style({
  display: 'flex',
  flexWrap: 'wrap',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: 16,
});

export const links = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: 20,
});

export const link = style({
  color: muted,
  textDecoration: 'none',
  ':hover': { color: text },
});

export const notice = style({ margin: 0 });

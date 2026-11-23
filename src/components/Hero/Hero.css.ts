import { globalStyle, style } from '@vanilla-extract/css';
import {
  displayFont,
  glow,
  gutter,
  maxContentWidth,
  monoFont,
  muted,
  reviewCount,
} from '@/styles/tokens';

export const section = style({
  position: 'relative',
  overflow: 'hidden',
  paddingBlock: '80px 96px',
  '::before': {
    content: '""',
    position: 'absolute',
    inset: '-20% -10% auto 40%',
    height: '120%',
    background: `radial-gradient(closest-side, ${glow}, transparent)`,
    pointerEvents: 'none',
  },
});

export const inner = style({
  position: 'relative',
  maxWidth: maxContentWidth,
  margin: '0 auto',
  paddingInline: gutter,
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 5fr) minmax(0, 7fr)',
  gap: 56,
  alignItems: 'center',
  '@media': { '(max-width: 960px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
});

export const copy = style({
  display: 'grid',
  gap: 24,
  minWidth: 0,
});

export const title = style({
  margin: 0,
  fontFamily: displayFont,
  fontSize: 'clamp(2.8rem, 6.4vw, 4.9rem)',
  lineHeight: 0.98,
  fontWeight: 800,
  letterSpacing: '-0.02em',
  textWrap: 'balance',
});

export const dim = style({
  color: muted,
  fontWeight: 500,
});

export const lead = style({
  margin: 0,
  color: muted,
  fontSize: '1.12rem',
  maxWidth: '60ch',
});

export const buttons = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: 12,
});

export const notes = style({
  margin: 0,
  padding: 0,
  listStyle: 'none',
  display: 'flex',
  flexWrap: 'wrap',
  gap: '6px 18px',
  fontSize: '0.9rem',
  color: muted,
});

export const note = style({
  '::before': { content: '"\\2713"', color: reviewCount, marginRight: 8 },
});

export const hint = style({
  margin: '10px 0 0',
  color: muted,
  fontSize: '0.85rem',
  textAlign: 'center',
});

globalStyle(`${hint} kbd`, { fontFamily: monoFont });

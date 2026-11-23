import { globalStyle, style } from '@vanilla-extract/css';
import {
  accent,
  border,
  displayFont,
  gutter,
  maxContentWidth,
  monoFont,
  muted,
} from '@/styles/tokens';

export const band = style({
  selectors: {
    '& + &': { borderTop: `1px solid ${border}` },
  },
});

export const inner = style({
  maxWidth: maxContentWidth,
  margin: '0 auto',
  paddingBlock: 96,
  paddingInline: gutter,
});

export const split = style([
  inner,
  {
    display: 'grid',
    gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 1fr)',
    gap: 64,
    alignItems: 'center',
    '@media': { '(max-width: 900px)': { gridTemplateColumns: 'minmax(0, 1fr)', gap: 40 } },
  },
]);

export const head = style({
  display: 'grid',
  gap: 16,
  maxWidth: 720,
  marginBottom: 48,
});

export const copy = style({
  display: 'grid',
  gap: 20,
  minWidth: 0,
});

export const eyebrow = style({
  fontFamily: monoFont,
  fontSize: '0.8rem',
  letterSpacing: '0.08em',
  textTransform: 'uppercase',
  color: accent,
});

export const title = style({
  fontFamily: displayFont,
  fontSize: 'clamp(2rem, 4vw, 2.9rem)',
  lineHeight: 1.08,
  fontWeight: 700,
});

export const lead = style({
  margin: 0,
  color: muted,
  fontSize: '1.12rem',
  maxWidth: '60ch',
});

globalStyle(`${lead} code`, { fontFamily: monoFont });

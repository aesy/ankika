import { style } from '@vanilla-extract/css';
import {
  accent,
  border,
  monoFont,
  muted,
  surface,
  termAccent,
  termBorder,
  termMuted,
  termText,
  text,
} from '@/styles/tokens';
import { inner } from '@/styles/section.css';

export const layout = style([
  inner,
  {
    display: 'grid',
    gridTemplateColumns: 'minmax(0, 4fr) minmax(0, 7fr)',
    gap: 56,
    alignItems: 'start',
    '@media': { '(max-width: 900px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
  },
]);

export const tabs = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: 6,
  marginBottom: 14,
});

export const tab = style({
  fontFamily: monoFont,
  fontSize: '0.85rem',
  padding: '8px 14px',
  borderRadius: 8,
  border: `1px solid ${border}`,
  background: 'transparent',
  color: muted,
  cursor: 'pointer',
  selectors: {
    '&[aria-selected="true"]': { background: surface, color: text, borderColor: accent },
  },
});

export const copyButton = style({
  marginLeft: 'auto',
  fontFamily: monoFont,
  fontSize: 11,
  color: termMuted,
  background: 'transparent',
  border: `1px solid ${termBorder}`,
  borderRadius: 6,
  padding: '2px 8px',
  cursor: 'pointer',
  ':hover': { color: termText },
});

export const link = style({
  color: termText,
  textUnderlineOffset: 3,
  ':hover': { color: termAccent },
});

export const hint = style({
  marginTop: 10,
  color: muted,
  fontSize: '0.85rem',
});

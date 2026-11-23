import { globalStyle, style } from '@vanilla-extract/css';
import {
  accent,
  monoFont,
  termAccent,
  termBorder,
  termFiltered,
  termMuted,
  termReview,
  termSurface,
} from '@/styles/tokens';

export const head = style({
  display: 'flex',
  justifyContent: 'space-between',
  gap: 16,
  paddingBottom: 10,
  borderBottom: `1px solid ${termBorder}`,
  whiteSpace: 'nowrap',
});

export const headDeck = style({ color: termAccent, fontWeight: 700 });
export const headSync = style({ color: termReview });

export const table = style({
  width: '100%',
  marginTop: 10,
  borderCollapse: 'collapse',
  whiteSpace: 'nowrap',
});

globalStyle(`${table} th`, {
  textAlign: 'right',
  fontWeight: 500,
  color: termMuted,
  padding: '4px 0 8px 18px',
});

globalStyle(`${table} td`, {
  textAlign: 'right',
  padding: '3px 0 3px 18px',
  fontVariantNumeric: 'tabular-nums',
});

globalStyle(`${table} th:first-child, ${table} td:first-child`, {
  textAlign: 'left',
  paddingLeft: 0,
});

export const selected = style({});

globalStyle(`${selected} td`, { background: termSurface });
globalStyle(`${selected} td:first-child`, { boxShadow: `inset 3px 0 0 ${termAccent}` });

export const arrow = style({ color: termMuted, display: 'inline-block', width: '1.4em' });
export const indent = style({ paddingLeft: '1.4em', display: 'inline-block' });
export const filtered = style({ color: termFiltered });

export const foot = style({
  marginTop: 12,
  paddingTop: 10,
  borderTop: `1px solid ${termBorder}`,
  color: termMuted,
  fontSize: 12,
  display: 'flex',
  flexWrap: 'wrap',
  gap: '4px 16px',
});

globalStyle(`${foot} kbd`, { color: termAccent });

export const checks = style({
  listStyle: 'none',
  padding: 0,
  margin: 0,
  display: 'grid',
  gap: 12,
});

globalStyle(`${checks} li`, { display: 'grid', gridTemplateColumns: '22px 1fr', gap: 8 });
globalStyle(`${checks} li::before`, {
  content: '"\\203A"',
  fontFamily: monoFont,
  color: accent,
  fontWeight: 700,
});

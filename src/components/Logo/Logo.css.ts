import { style } from '@vanilla-extract/css';
import { accent, monoFont, text } from '@/styles/tokens';

export const brand = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: 10,
  minHeight: 44,
  fontFamily: monoFont,
  fontWeight: 700,
  fontSize: '1.25rem',
  color: text,
  textDecoration: 'none',
});

export const mark = style({
  width: 32,
  height: 32,
  flex: 'none',
  color: accent,
});

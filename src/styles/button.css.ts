import { style } from '@vanilla-extract/css';
import { accent, bg, border, surface, text } from '@/styles/tokens';

export const buttonRoot = style({
  fontWeight: 600,
  transition: 'border-color 0.15s ease-out, filter 0.15s ease-out',
});

export const primary = style({
  background: accent,
  borderColor: accent,
  color: bg,
  ':hover': { background: accent, filter: 'brightness(1.08)' },
});

export const secondary = style({
  background: surface,
  borderColor: border,
  color: text,
  ':hover': { background: surface, borderColor: accent },
});

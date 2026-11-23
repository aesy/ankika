import { style } from '@vanilla-extract/css';
import { muted } from '@/styles/tokens';

export const scripts = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
  gap: 24,
  '@media': { '(max-width: 900px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
});

export const script = style({
  display: 'grid',
  gap: 14,
  minWidth: 0,
});

export const note = style({ color: muted });

import { style } from '@vanilla-extract/css';
import {
  monoFont,
  termAccent,
  termBg,
  termBorder,
  termLearn,
  termMuted,
  termNew,
  termReview,
  termSurface,
  termText,
} from '@/styles/tokens';

export const frame = style({
  background: termBg,
  color: termText,
  border: `1px solid ${termBorder}`,
  borderRadius: 12,
  fontFamily: monoFont,
  fontSize: 14,
  lineHeight: 1.5,
  boxShadow: '0 30px 80px -30px rgba(0, 0, 0, 0.55)',
  overflow: 'hidden',
  minWidth: 0,
});

export const bar = style({
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '10px 14px',
  borderBottom: `1px solid ${termBorder}`,
  background: termSurface,
});

export const dot = style({
  width: 11,
  height: 11,
  borderRadius: '50%',
  background: termBorder,
});

export const title = style({
  marginLeft: 10,
  color: termMuted,
  fontSize: 12,
});

export const body = style({
  padding: '14px 16px',
  overflowX: 'auto',
});

export const shell = style([body, { whiteSpace: 'pre', margin: 0, fontFamily: monoFont }]);

export const prompt = style({ color: termAccent, userSelect: 'none' });
export const output = style({ color: termMuted });
export const string = style({ color: termReview });
export const number = style({ color: termNew });

export const countNew = style({ color: termNew });
export const countLearn = style({ color: termLearn });
export const countReview = style({ color: termReview });
export const countZero = style({ color: termMuted, opacity: 0.6 });

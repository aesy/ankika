import { globalStyle, style, styleVariants } from '@vanilla-extract/css';
import type { Look } from '@/lib/screen';
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

const LINE_HEIGHT = 1.32;

/* Plain monospace text, centered as one block; the box is drawn in the same text, so
   whatever is centered inside it stays centered whatever the font */
export const screen = style({
  display: 'block',
  width: 'fit-content',
  maxWidth: '100%',
  margin: '0 auto',
  padding: '10px 0',
  background: termBg,
  color: termText,
  fontFamily: monoFont,
  lineHeight: LINE_HEIGHT,
  whiteSpace: 'pre',
  overflow: 'hidden',
  outline: 'none',
});

const line = { content: '""', position: 'absolute', border: `0 solid ${termBorder}` } as const;
const drawn = {
  display: 'inline-block',
  position: 'relative',
  height: `${LINE_HEIGHT}em`,
  verticalAlign: 'top',
} as const;
const button = { background: termBorder, color: termText } as const;
const focused = { background: termAccent, color: termBg, fontWeight: 700 } as const;

export const look = styleVariants<Record<Look, Parameters<typeof style>[0]>>({
  accent: { color: termAccent },
  bold: { fontWeight: 700 },
  muted: { color: termMuted },
  current: { textDecoration: 'underline', textUnderlineOffset: 3, fontWeight: 700 },
  new: { color: termNew },
  learn: { color: termLearn },
  review: { color: termReview },
  button,
  focused,
  hint: {},
  vbar: { ...drawn, '::after': { ...line, left: '50%', top: -1, bottom: -1, borderLeftWidth: 1 } },
  hbar: { ...drawn, '::after': { ...line, left: 0, right: 0, top: '50%', borderTopWidth: 1 } },
  cornerTopLeft: {
    ...drawn,
    '::after': {
      ...line,
      left: '50%',
      top: '50%',
      right: 0,
      bottom: 0,
      borderLeftWidth: 1,
      borderTopWidth: 1,
      borderTopLeftRadius: 6,
    },
  },
  cornerTopRight: {
    ...drawn,
    '::after': {
      ...line,
      left: 0,
      top: '50%',
      right: '50%',
      bottom: 0,
      borderRightWidth: 1,
      borderTopWidth: 1,
      borderTopRightRadius: 6,
    },
  },
  cornerBottomLeft: {
    ...drawn,
    '::after': {
      ...line,
      left: '50%',
      top: 0,
      right: 0,
      bottom: '50%',
      borderLeftWidth: 1,
      borderBottomWidth: 1,
      borderBottomLeftRadius: 6,
    },
  },
  cornerBottomRight: {
    ...drawn,
    '::after': {
      ...line,
      left: 0,
      top: 0,
      right: '50%',
      bottom: '50%',
      borderRightWidth: 1,
      borderBottomWidth: 1,
      borderBottomRightRadius: 6,
    },
  },
});

globalStyle(`${screen} [style*="left"]`, { position: 'relative' });
globalStyle(`${screen} [data-action]`, { cursor: 'pointer' });

// A button or key hint lights up as a whole, like the TUI's hover
for (const action of ['show', 'again', 'hard', 'good', 'easy', 'restart']) {
  const target = `[data-action="${action}"]`;

  globalStyle(`${screen}:has(.${look.button}${target}:hover) .${look.button}${target}`, focused);
  globalStyle(`${screen}:has(.${look.hint}${target}:hover) .${look.hint}${target}`, {
    background: termSurface,
  });
}

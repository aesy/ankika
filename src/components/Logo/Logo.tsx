import type { FC } from 'react';

/** A star on a card: Anki's mark, drawn with the TUI's rounded borders. */
export const Logo: FC<{ className?: string }> = ({ className }) => (
  <svg className={className} viewBox="0 0 32 32" aria-hidden="true">
    <rect
      x="2"
      y="2"
      width="28"
      height="28"
      rx="7"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    />
    <polygon
      transform="translate(0 1.4)"
      points="16.00,6.40 18.06,11.77 23.80,12.07 19.33,15.68 20.82,21.23 16.00,18.10 11.18,21.23 12.67,15.68 8.20,12.07 13.94,11.77"
      fill="currentColor"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinejoin="round"
    />
  </svg>
);

import { createTheme, Button } from '@mantine/core';
import { buttonRoot } from '@/styles/button.css';
import { bodyFont, displayFont, monoFont } from '@/styles/tokens';

export const theme = createTheme({
  fontFamily: bodyFont,
  fontFamilyMonospace: monoFont,
  headings: { fontFamily: displayFont, fontWeight: '700' },
  radius: { xs: '4px', sm: '6px', md: '8px', lg: '12px', xl: '14px' },
  defaultRadius: 'md',
  respectReducedMotion: true,
  components: {
    Button: Button.extend({ classNames: { root: buttonRoot } }),
  },
});

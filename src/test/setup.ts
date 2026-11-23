import '@testing-library/jest-dom/vitest';
import { afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';

afterEach(cleanup);

vi.stubGlobal(
  'ResizeObserver',
  class {
    observe = () => undefined;
    disconnect = () => undefined;
  }
);

vi.stubGlobal(
  'matchMedia',
  (query: string) =>
    ({
      matches: false,
      media: query,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    }) as unknown as MediaQueryList
);

Object.defineProperty(document, 'fonts', {
  value: { ready: Promise.resolve() },
  configurable: true,
});

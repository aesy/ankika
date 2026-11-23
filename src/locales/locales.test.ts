import en from '@/locales/en.json';

function flatten(value: object, prefix = ''): Record<string, string> {
  return Object.entries(value).reduce<Record<string, string>>((flat, [key, child]) => {
    const path = prefix ? `${prefix}.${key}` : key;

    return typeof child === 'string'
      ? { ...flat, [path]: child }
      : { ...flat, ...flatten(child as object, path) };
  }, {});
}

describe('the English text', () => {
  it('uses plain punctuation', () => {
    for (const [key, text] of Object.entries(flatten(en))) {
      expect([key, /[–—‘’“”−]/.test(text)]).toEqual([key, false]);
    }
  });

  it('has no empty strings', () => {
    for (const [key, text] of Object.entries(flatten(en))) {
      expect([key, text.trim()]).not.toEqual([key, '']);
    }
  });
});

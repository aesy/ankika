import i18next from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from '@/locales/en.json';

/** Values are not HTML-escaped: React escapes them when rendering. */
export async function initI18n() {
  await i18next.use(initReactI18next).init({
    lng: 'en',
    fallbackLng: 'en',
    resources: { en: { translation: en } },
    interpolation: { escapeValue: false },
  });
}

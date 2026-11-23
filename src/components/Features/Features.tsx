import type { FC } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { body, feature, grid, key as keyClass } from '@/components/Features/Features.css';
import { band, eyebrow, head, inner, lead, title } from '@/styles/section.css';

const FEATURES = [
  'images',
  'audio',
  'typeIn',
  'notes',
  'browser',
  'stats',
  'palette',
  'packages',
  'options',
] as const;

// Anki's field syntax, kept out of the text so i18next doesn't read it as a placeholder
const KEYS: Partial<Record<(typeof FEATURES)[number], string>> = { typeIn: '{{type:Answer}}' };

export const Features: FC = () => {
  const { t } = useTranslation();

  return (
    <section className={band} id="features" aria-labelledby="features-title">
      <div className={inner}>
        <div className={head}>
          <span className={eyebrow}>{t('features.eyebrow')}</span>
          <h2 id="features-title" className={title}>
            {t('features.title')}
          </h2>
          <p className={lead}>{t('features.lead')}</p>
        </div>
        <div className={grid}>
          {FEATURES.map(name => (
            <div key={name} className={feature}>
              <span className={keyClass}>
                {KEYS[name] ?? (
                  <Trans i18nKey={`features.items.${name}.key`} components={{ kbd: <kbd /> }} />
                )}
              </span>
              <h3>{t(`features.items.${name}.title`)}</h3>
              <p className={body}>
                <Trans i18nKey={`features.items.${name}.body`} components={{ kbd: <kbd /> }} />
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

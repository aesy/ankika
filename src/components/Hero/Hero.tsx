import type { FC } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { Button } from '@mantine/core';
import {
  buttons,
  copy,
  dim,
  hint,
  inner,
  lead,
  note,
  notes,
  section,
  title,
} from '@/components/Hero/Hero.css';
import { ReviewDemo } from '@/components/Hero/ReviewDemo';
import { REPOSITORY } from '@/lib/links';
import { primary, secondary } from '@/styles/button.css';

const NOTES = ['platforms', 'license', 'input'] as const;

export const Hero: FC = () => {
  const { t } = useTranslation();

  return (
    <section className={section} aria-labelledby="hero-title">
      <div className={inner}>
        <div className={copy}>
          <h1 id="hero-title" className={title}>
            {t('hero.title')} <span className={dim}>{t('hero.titleRest')}</span>
          </h1>
          <p className={lead}>{t('hero.lead')}</p>
          <div className={buttons}>
            <Button component="a" href="#install" size="lg" fz="md" className={primary}>
              {t('hero.install')}
            </Button>
            <Button component="a" href={REPOSITORY} size="lg" fz="md" className={secondary}>
              {t('hero.source')}
            </Button>
          </div>
          <ul className={notes}>
            {NOTES.map(key => (
              <li key={key} className={note}>
                {t(`hero.notes.${key}`)}
              </li>
            ))}
          </ul>
        </div>
        <div>
          <ReviewDemo />
          <p className={hint}>
            <Trans i18nKey="hero.tryIt" components={{ kbd: <kbd /> }} />
          </p>
        </div>
      </div>
    </section>
  );
};

import type { FC } from 'react';
import { useTranslation } from 'react-i18next';
import { footer, inner, link, links, notice, row } from '@/components/Footer/Footer.css';
import { brand, mark } from '@/components/Logo/Logo.css';
import { Logo } from '@/components/Logo/Logo';
import { ISSUES, RELEASES, REPOSITORY } from '@/lib/links';

const LINKS = [
  ['source', REPOSITORY],
  ['releases', RELEASES],
  ['issues', ISSUES],
] as const;

export const Footer: FC = () => {
  const { t } = useTranslation();

  return (
    <footer className={footer}>
      <div className={inner}>
        <div className={row}>
          <a className={brand} href="#top" aria-label={t('nav.home')}>
            <Logo className={mark} />
            ankika
          </a>
          <nav className={links} aria-label={t('footer.label')}>
            {LINKS.map(([key, href]) => (
              <a key={key} className={link} href={href}>
                {t(`footer.${key}`)}
              </a>
            ))}
          </nav>
        </div>
        <p className={notice}>{t('footer.notice')}</p>
      </div>
    </footer>
  );
};

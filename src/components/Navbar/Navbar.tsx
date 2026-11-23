import type { FC } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@mantine/core';
import { brand, mark } from '@/components/Logo/Logo.css';
import { Logo } from '@/components/Logo/Logo';
import { bar, header, link, links, wide } from '@/components/Navbar/Navbar.css';
import { REPOSITORY } from '@/lib/links';
import { primary } from '@/styles/button.css';

export const Navbar: FC = () => {
  const { t } = useTranslation();

  return (
    <header className={header}>
      <div className={bar}>
        <a className={brand} href="#top" aria-label={t('nav.home')}>
          <Logo className={mark} />
          ankika
        </a>
        <nav className={links} aria-label={t('nav.label')}>
          <a className={`${link} ${wide}`} href="#features">
            {t('nav.features')}
          </a>
          <a className={`${link} ${wide}`} href="#scripting">
            {t('nav.scripting')}
          </a>
          <a className={`${link} ${wide}`} href={REPOSITORY}>
            {t('nav.github')}
          </a>
          <Button component="a" href="#install" size="md" fz="0.9rem" className={primary}>
            {t('nav.install')}
          </Button>
        </nav>
      </div>
    </header>
  );
};

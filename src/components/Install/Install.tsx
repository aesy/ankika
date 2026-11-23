import { useState, type FC } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { copyButton, hint, layout, link, tab, tabs } from '@/components/Install/Install.css';
import { Terminal } from '@/components/Terminal/Terminal';
import { prompt, shell } from '@/components/Terminal/Terminal.css';
import { RELEASES, REPOSITORY } from '@/lib/links';
import { band, copy, eyebrow, lead, title } from '@/styles/section.css';

type Method = 'release' | 'arch' | 'cargo';

const METHODS: readonly Method[] = ['release', 'arch', 'cargo'];

const COMMANDS: Record<Exclude<Method, 'release'>, readonly string[]> = {
  arch: [`git clone ${REPOSITORY}`, 'cd ankika/pkg/arch', 'makepkg -si'],
  cargo: [
    `DESCRIPTORS_BIN=/tmp/anki_descriptors.bin cargo install --locked --git ${REPOSITORY} ankika`,
  ],
};

export const Install: FC = () => {
  const { t } = useTranslation();
  const [method, setMethod] = useState<Method>('release');
  const [copied, setCopied] = useState(false);
  const commands = method === 'release' ? null : COMMANDS[method];

  const choose = (next: Method) => {
    setMethod(next);
    setCopied(false);
  };

  const copyCommands = () => {
    if (commands) {
      void navigator.clipboard.writeText(commands.join('\n')).then(() => setCopied(true));
    }
  };

  return (
    <section className={band} id="install" aria-labelledby="install-title">
      <div className={layout}>
        <div className={copy}>
          <span className={eyebrow}>{t('install.eyebrow')}</span>
          <h2 id="install-title" className={title}>
            {t('install.title')}
          </h2>
          <p className={lead}>
            <Trans i18nKey="install.lead" components={{ code: <code /> }} />
          </p>
        </div>
        <div>
          <div className={tabs} role="tablist" aria-label={t('install.tabsLabel')}>
            {METHODS.map(name => (
              <button
                key={name}
                id={`install-tab-${name}`}
                type="button"
                role="tab"
                className={tab}
                aria-selected={method === name}
                aria-controls="install-panel"
                onClick={() => choose(name)}
              >
                {t(`install.${name}.tab`)}
              </button>
            ))}
          </div>
          <Terminal
            title={t(`install.${method}.tab`)}
            actions={
              commands ? (
                <button type="button" className={copyButton} onClick={copyCommands}>
                  {copied ? t('install.copied') : t('install.copy')}
                </button>
              ) : null
            }
          >
            <div
              id="install-panel"
              role="tabpanel"
              aria-labelledby={`install-tab-${method}`}
              className={shell}
            >
              {commands ? (
                commands.map((command, i) => (
                  <span key={command}>
                    <span className={prompt}>$ </span>
                    {command}
                    {i < commands.length - 1 ? '\n' : null}
                  </span>
                ))
              ) : (
                <>
                  <span className={prompt}>$ </span>
                  open{' '}
                  <a className={link} href={RELEASES}>
                    {RELEASES}
                  </a>
                </>
              )}
            </div>
          </Terminal>
          <p className={hint}>{t(`install.${method}.note`)}</p>
        </div>
      </div>
    </section>
  );
};

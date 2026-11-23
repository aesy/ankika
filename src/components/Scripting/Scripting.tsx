import type { FC, ReactNode } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { note, script, scripts } from '@/components/Scripting/Scripting.css';
import { Terminal } from '@/components/Terminal/Terminal';
import { number, output, prompt, shell, string } from '@/components/Terminal/Terminal.css';
import { band, eyebrow, head, inner, lead, title } from '@/styles/section.css';

const Command: FC<{ children: string }> = ({ children }) => (
  <>
    <span className={prompt}>$ </span>
    {children}
    {'\n'}
  </>
);

const Output: FC<{ children: ReactNode; last?: boolean }> = ({ children, last }) => (
  <>
    <span className={output}>{children}</span>
    {last ? null : '\n'}
  </>
);

const Num: FC<{ children: number }> = ({ children }) => <span className={number}>{children}</span>;

const Str: FC<{ children: string }> = ({ children }) => (
  <span className={string}>{`"${children}"`}</span>
);

export const Scripting: FC = () => {
  const { t } = useTranslation();
  const code = { code: <code /> };

  return (
    <section className={band} id="scripting" aria-labelledby="scripting-title">
      <div className={inner}>
        <div className={head}>
          <span className={eyebrow}>{t('scripting.eyebrow')}</span>
          <h2 id="scripting-title" className={title}>
            {t('scripting.title')}
          </h2>
          <p className={lead}>
            <Trans i18nKey="scripting.lead" components={code} />
          </p>
        </div>
        <div className={scripts}>
          <div className={script}>
            <h3>{t('scripting.due.title')}</h3>
            <Terminal title={t('terminal.title')}>
              <pre className={shell}>
                <Command>ankika due</Command>
                <Output>117</Output>
                <Command>ankika due --verbose-counts</Command>
                <Output>117 (37 new, 4 learning, 76 review)</Output>
                <Command>ankika due --json</Command>
                <Output last>
                  {'{"new":'}
                  <Num>{37}</Num>
                  {',"learning":'}
                  <Num>{4}</Num>
                  {',"review":'}
                  <Num>{76}</Num>
                  {',"total":'}
                  <Num>{117}</Num>
                  {'}'}
                </Output>
              </pre>
            </Terminal>
            <p className={note}>
              <Trans i18nKey="scripting.due.body" components={code} />
            </p>
          </div>
          <div className={script}>
            <h3>{t('scripting.sync.title')}</h3>
            <Terminal title={t('terminal.title')}>
              <pre className={shell}>
                <Command>ankika sync --json</Command>
                <Output>
                  {'{"collection":'}
                  <Str>synced</Str>
                  {',"media":'}
                  <Str>synced</Str>
                  {',"message":'}
                  <Str>{''}</Str>
                  {'}'}
                </Output>
                <Command>ankika export backup.colpkg</Command>
                <span className={prompt}>$ </span>
                ankika completions fish &gt; ~/.config/fish/completions/ankika.fish
              </pre>
            </Terminal>
            <p className={note}>
              <Trans i18nKey="scripting.sync.body" components={code} />
            </p>
          </div>
        </div>
      </div>
    </section>
  );
};

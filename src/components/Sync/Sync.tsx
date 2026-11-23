import type { FC } from 'react';
import { useTranslation } from 'react-i18next';
import {
  arrow,
  checks,
  filtered,
  foot,
  head,
  headDeck,
  headSync,
  indent,
  selected,
  table,
} from '@/components/Sync/Sync.css';
import { Terminal } from '@/components/Terminal/Terminal';
import {
  body,
  countLearn,
  countNew,
  countReview,
  countZero,
} from '@/components/Terminal/Terminal.css';
import { QUEUES, type Queue } from '@/lib/screen';
import { band, copy, eyebrow, lead, split, title } from '@/styles/section.css';

type Deck = {
  name: 'spanish' | 'grammar' | 'vocabulary' | 'pharmacology' | 'rust' | 'leeches';
  arrow: string;
  child?: boolean;
  selected?: boolean;
  filtered?: boolean;
  counts: Record<Queue, number>;
};

const DECKS: readonly Deck[] = [
  { name: 'spanish', arrow: '▾', counts: { new: 12, learn: 3, review: 47 } },
  { name: 'grammar', arrow: '·', child: true, counts: { new: 0, learn: 0, review: 6 } },
  {
    name: 'vocabulary',
    arrow: '·',
    child: true,
    selected: true,
    counts: { new: 12, learn: 3, review: 41 },
  },
  { name: 'pharmacology', arrow: '▸', counts: { new: 20, learn: 0, review: 18 } },
  { name: 'rust', arrow: '·', counts: { new: 5, learn: 1, review: 7 } },
  { name: 'leeches', arrow: '·', filtered: true, counts: { new: 0, learn: 0, review: 4 } },
];

const COUNT_CLASSES: Record<Queue, string> = {
  new: countNew,
  learn: countLearn,
  review: countReview,
};
const POINTS = ['when', 'media', 'keyring'] as const;
const KEYS = [
  ['Enter', 'study'],
  ['s', 'sync'],
  ['Ctrl+P', 'commands'],
  ['q', 'quit'],
] as const;

export const Sync: FC = () => {
  const { t } = useTranslation();

  return (
    <section className={band} id="decks" aria-labelledby="sync-title">
      <div className={split}>
        <Terminal title={t('terminal.title')}>
          <div className={body}>
            <div className={head}>
              <span className={headDeck}>{t('sync.screen.decks')}</span>
              <span>{t('sync.screen.due')}</span>
              <span className={headSync}>{t('sync.screen.synced')}</span>
            </div>
            <table className={table}>
              <thead>
                <tr>
                  <th>{t('sync.screen.deck')}</th>
                  <th>{t('sync.screen.new')}</th>
                  <th>{t('sync.screen.learn')}</th>
                  <th>{t('sync.screen.dueColumn')}</th>
                </tr>
              </thead>
              <tbody>
                {DECKS.map(deck => {
                  const name = t(`sync.screen.names.${deck.name}`);
                  const label = (
                    <>
                      <span className={arrow}>{deck.arrow}</span>
                      {deck.filtered ? <span className={filtered}>{name}</span> : name}
                    </>
                  );

                  return (
                    <tr key={deck.name} className={deck.selected ? selected : undefined}>
                      <td>{deck.child ? <span className={indent}>{label}</span> : label}</td>
                      {QUEUES.map(queue => (
                        <td
                          key={queue}
                          className={deck.counts[queue] === 0 ? countZero : COUNT_CLASSES[queue]}
                        >
                          {deck.counts[queue]}
                        </td>
                      ))}
                    </tr>
                  );
                })}
              </tbody>
            </table>
            <div className={foot}>
              {KEYS.map(([key, label]) => (
                <span key={key}>
                  <kbd>{key}</kbd> {t(`sync.screen.${label}`)}
                </span>
              ))}
            </div>
          </div>
        </Terminal>
        <div className={copy}>
          <span className={eyebrow}>{t('sync.eyebrow')}</span>
          <h2 id="sync-title" className={title}>
            {t('sync.title')}
          </h2>
          <p className={lead}>{t('sync.lead')}</p>
          <ul className={checks}>
            {POINTS.map(key => (
              <li key={key}>{t(`sync.points.${key}`)}</li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
};

import {
  Fragment,
  useEffect,
  useMemo,
  useReducer,
  useState,
  type FC,
  type KeyboardEvent,
  type MouseEvent,
} from 'react';
import { useTranslation } from 'react-i18next';
import { look, screen as screenClass } from '@/components/Hero/ReviewDemo.css';
import { Terminal } from '@/components/Terminal/Terminal';
import { SAMPLE_CARDS, START_COUNTS } from '@/lib/deck';
import {
  buildScreen,
  MIN_WIDTH,
  RATINGS,
  stageOf,
  toRuns,
  type Action,
  type Card,
  type Counts,
  type Hint,
  type Labels,
} from '@/lib/screen';

const FONT_SIZE = 13;
const SIDE_PADDING = 24;

type State = { index: number; revealed: boolean; counts: Counts; studied: number };
type Event = { type: 'show' } | { type: 'answer' } | { type: 'restart' };

const initial: State = { index: 0, revealed: false, counts: START_COUNTS, studied: 0 };

function reduce(state: State, event: Event): State {
  const done = state.index >= SAMPLE_CARDS.length;

  switch (event.type) {
    case 'restart':
      return initial;
    case 'show':
      return done ? state : { ...state, revealed: true };
    case 'answer': {
      if (done || !state.revealed) {
        return state;
      }

      const { queue } = SAMPLE_CARDS[state.index];

      return {
        index: state.index + 1,
        revealed: false,
        studied: state.studied + 1,
        counts: { ...state.counts, [queue]: Math.max(0, state.counts[queue] - 1) },
      };
    }
  }
}

type Layout = { columns: number; fontSize: number };

/** Columns follow the space available, like the TUI follows the terminal's size. */
function useLayout(screen: HTMLPreElement | null): Layout {
  const [layout, setLayout] = useState<Layout>({ columns: MIN_WIDTH, fontSize: FONT_SIZE });

  useEffect(() => {
    const frame = screen?.parentElement;

    if (!screen || !frame) {
      return;
    }

    const measure = () => {
      const probe = document.createElement('span');

      probe.style.fontSize = `${FONT_SIZE}px`;
      probe.textContent = 'M'.repeat(40);
      screen.appendChild(probe);

      const cell = probe.getBoundingClientRect().width / 40 || FONT_SIZE * 0.6;

      probe.remove();

      const available = frame.clientWidth - SIDE_PADDING;

      if (available <= 0) {
        return;
      }

      // Smaller text only when the narrowest screen wouldn't fit
      const scale = Math.min(1, available / ((MIN_WIDTH + 1) * cell));
      const fontSize = Math.floor(FONT_SIZE * scale * 10) / 10;
      // One column spare, so a slightly wider font never overflows
      const columns = Math.max(
        MIN_WIDTH,
        Math.floor(available / (cell * (fontSize / FONT_SIZE))) - 1
      );

      setLayout(current =>
        current.columns === columns && current.fontSize === fontSize
          ? current
          : { columns, fontSize }
      );
    };

    const observer = new ResizeObserver(measure);

    observer.observe(frame);
    void document.fonts.ready.then(measure);

    return () => observer.disconnect();
  }, [screen]);

  return layout;
}

export const ReviewDemo: FC = () => {
  const { t } = useTranslation();
  const [state, dispatch] = useReducer(reduce, initial);
  const [screen, setScreen] = useState<HTMLPreElement | null>(null);
  const { columns, fontSize } = useLayout(screen);

  const labels = useMemo<Labels>(() => {
    const hint = (key: string, label: string, action?: Action): Hint => ({
      key,
      label: t(`demo.hints.${label}`),
      action,
    });
    const common = [hint('?', 'help'), hint(':', 'commands'), hint('Ctrl+C', 'quit')];

    return {
      appName: 'ankika',
      deck: t('demo.deck'),
      synced: t('demo.synced'),
      account: t('demo.account'),
      showAnswer: t('demo.showAnswer'),
      back: t('demo.back'),
      ratings: {
        again: t('demo.ratings.again'),
        hard: t('demo.ratings.hard'),
        good: t('demo.ratings.good'),
        easy: t('demo.ratings.easy'),
      },
      congratulations: t('demo.congratulations'),
      studied: t('demo.studied', { count: state.studied }),
      hints: {
        question: [
          hint('Space', 'showAnswer', 'show'),
          hint('r', 'replayAudio'),
          hint('Ctrl+Z', 'undo'),
          hint('Esc', 'back'),
          ...common,
          hint('a', 'add'),
          hint('b', 'browse'),
          hint('e', 'edit'),
          hint('*', 'markNote'),
          hint('-', 'buryCard'),
        ],
        answer: [
          ...RATINGS.map((rating, i) => ({
            key: String(i + 1),
            label: t(`demo.ratings.${rating}`),
            action: rating,
          })),
          hint('r', 'replayAudio'),
          hint('Ctrl+Z', 'undo'),
          hint('Esc', 'back'),
          ...common,
          hint('a', 'add'),
          hint('b', 'browse'),
        ],
        finished: [hint('Esc', 'back', 'restart'), hint('Ctrl+Z', 'undo'), ...common],
      },
    };
  }, [t, state.studied]);

  const sample = SAMPLE_CARDS[state.index] as (typeof SAMPLE_CARDS)[number] | undefined;
  const card: Card | null = sample
    ? {
        front: t(`demo.cards.${sample.id}.front`),
        back: t(`demo.cards.${sample.id}.back`),
        sound: sample.sound,
        queue: sample.queue,
        intervals: sample.intervals,
      }
    : null;
  const screenState = { card, revealed: state.revealed, counts: state.counts };
  const rows = buildScreen(screenState, labels, columns).map((cells, line) => ({
    line,
    runs: toRuns(cells),
  }));

  const act = (action: Action) => {
    if (action === 'show') {
      dispatch({ type: 'show' });
    } else if (action === 'restart') {
      dispatch({ type: 'restart' });
    } else {
      dispatch({ type: 'answer' });
    }
  };

  const onClick = (event: MouseEvent<HTMLPreElement>) => {
    const target = (event.target as HTMLElement).closest<HTMLElement>('[data-action]');
    const action = target?.dataset.action as Action | undefined;

    if (action) {
      act(action);
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLPreElement>) => {
    const stage = stageOf(screenState);

    if (event.key === ' ' || event.key === 'Enter') {
      event.preventDefault();
      act({ question: 'show', answer: 'good', finished: 'restart' }[stage] as Action);
    } else if (stage === 'answer' && /^[1-4]$/.test(event.key)) {
      act(RATINGS[Number(event.key) - 1]);
    } else if (stage === 'finished' && event.key === 'Escape') {
      act('restart');
    }
  };

  return (
    <Terminal title={t('terminal.title')}>
      <pre
        ref={setScreen}
        className={screenClass}
        style={{ fontSize }}
        tabIndex={0}
        aria-label={t('demo.label')}
        onClick={onClick}
        onKeyDown={onKeyDown}
      >
        {rows.map(({ line, runs }) => (
          <Fragment key={line}>
            {runs.map(run => (
              <span
                key={run.start}
                className={run.looks.map(name => look[name]).join(' ') || undefined}
                data-action={run.action}
                style={run.nudge ? { left: `${run.nudge * 0.5}ch` } : undefined}
              >
                {run.text}
              </span>
            ))}
            {line < rows.length - 1 ? '\n' : null}
          </Fragment>
        ))}
      </pre>
    </Terminal>
  );
};

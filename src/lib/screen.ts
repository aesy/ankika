// The review screen laid out cell by cell, like the TUI draws it with ratatui
// (app/ankika/src/tui/view/review.rs, header.rs and footer.rs).

export type Queue = 'new' | 'learn' | 'review';
export type Rating = 'again' | 'hard' | 'good' | 'easy';
export type Action = 'show' | Rating | 'restart';
export type Stage = 'question' | 'answer' | 'finished';
export type Counts = Record<Queue, number>;

export const RATINGS: readonly Rating[] = ['again', 'hard', 'good', 'easy'];
export const QUEUES: readonly Queue[] = ['new', 'learn', 'review'];

/** How a cell looks; a cell can have several. */
export type Look =
  | 'accent'
  | 'bold'
  | 'muted'
  | 'current'
  | Queue
  | 'button'
  | 'focused'
  | 'hint'
  | 'vbar'
  | 'hbar'
  | 'cornerTopLeft'
  | 'cornerTopRight'
  | 'cornerBottomLeft'
  | 'cornerBottomRight';

export type Card = {
  front: string;
  back: string;
  sound: string;
  queue: Queue;
  intervals: Record<Rating, string>;
};

export type Hint = { key: string; label: string; action?: Action };

export type Labels = {
  appName: string;
  deck: string;
  synced: string;
  account: string;
  showAnswer: string;
  back: string;
  ratings: Record<Rating, string>;
  congratulations: string;
  studied: string;
  hints: Record<Stage, readonly Hint[]>;
};

export type ScreenState = { card: Card | null; revealed: boolean; counts: Counts };

/** `nudge` moves a cell right by half cells, to center an odd remainder. */
export type Cell = { char: string; looks: readonly Look[]; action?: Action; nudge: number };

export type Run = {
  start: number;
  text: string;
  looks: readonly Look[];
  action?: Action;
  nudge: number;
};

export const HEIGHT = 20;
export const MIN_WIDTH = 46;

const BOX_MARGIN = 3;
const BUTTON_GAP = 2;

export function stageOf(state: ScreenState): Stage {
  if (state.card === null) {
    return 'finished';
  }

  return state.revealed ? 'answer' : 'question';
}

export function buildScreen(state: ScreenState, labels: Labels, width: number): Cell[][] {
  const grid: Cell[][] = Array.from({ length: HEIGHT }, () =>
    Array.from({ length: width }, () => ({ char: ' ', looks: [], nudge: 0 }))
  );

  const put = (
    row: number,
    col: number,
    text: string,
    looks: readonly Look[] = [],
    action?: Action
  ) => {
    Array.from(text).forEach((char, i) => {
      const x = col + i;

      if (x >= 0 && x < width) {
        grid[row][x] = { char, looks, action, nudge: 0 };
      }
    });
  };

  // Whole cells can't center an odd remainder, so those cells move right by half a cell
  const shiftHalf = (row: number, from: number, to: number) => {
    for (let x = from; x < to; x++) {
      grid[row][x].nudge += 1;
    }
  };

  const centered = (
    row: number,
    x0: number,
    span: number,
    text: string,
    looks: readonly Look[] = [],
    action?: Action
  ) => {
    const chars = Array.from(text).slice(0, span);
    const rest = span - chars.length;
    const start = x0 + Math.floor(rest / 2);

    put(row, start, chars.join(''), looks, action);

    if (rest % 2 === 1) {
      shiftHalf(row, start, start + chars.length);
    }
  };

  const stage = stageOf(state);
  const { card } = state;

  // Header: title, remaining counts with the current card's queue underlined, sync status, account
  put(0, 0, ` ${labels.appName}`, ['accent', 'bold']);

  let x = labels.appName.length + 3;

  for (const queue of QUEUES) {
    const count = String(state.counts[queue]);

    put(0, x, count, card?.queue === queue ? [queue, 'current'] : [queue]);
    x += count.length + 1;
  }

  const account = `${labels.account} `;
  const synced = ` ${labels.synced} `;

  put(0, width - account.length, account, ['muted']);
  put(0, width - account.length - 1 - synced.length, synced, ['muted']);

  // The review box, with the deck name in its top border
  put(1, 0, ' ', ['cornerTopLeft']);
  put(1, 1, ' '.repeat(width - 2), ['hbar']);
  put(1, width - 1, ' ', ['cornerTopRight']);
  put(1, 1, ` ${labels.deck} `);

  for (let row = 2; row < HEIGHT - 2; row++) {
    put(row, 0, ' ', ['vbar']);
    put(row, width - 1, ' ', ['vbar']);
  }

  put(HEIGHT - 2, 0, ' ', ['cornerBottomLeft']);
  put(HEIGHT - 2, 1, ' '.repeat(width - 2), ['hbar']);
  put(HEIGHT - 2, width - 1, ' ', ['cornerBottomRight']);

  const x0 = BOX_MARGIN;
  const span = width - 2 * BOX_MARGIN;
  const buttonRow = HEIGHT - 4;
  const lines = { question: 2, answer: 4, finished: 3 }[stage];
  // The card's lines sit in the middle of the box's inner rows
  const top = 2 + Math.floor((HEIGHT - 4 - lines) / 2);

  if (card === null) {
    centered(top, x0, span, labels.congratulations, ['accent', 'bold']);
    centered(top + 2, x0, span, labels.studied, ['muted']);
    centered(buttonRow, x0, span, ` ${labels.back} `, ['button', 'focused'], 'restart');
  } else {
    centered(top, x0, span, card.front);
    centered(top + 1, x0, span, `[sound: ${card.sound}]`, ['muted']);

    if (state.revealed) {
      put(top + 2, x0, ' '.repeat(span), ['hbar']);
      centered(top + 3, x0, span, card.back);

      const widths = RATINGS.map(
        rating => Math.max(labels.ratings[rating].length, card.intervals[rating].length) + 2
      );
      const total = widths.reduce((sum, w) => sum + w, 0) + BUTTON_GAP * (RATINGS.length - 1);
      const groupStart = x0 + Math.floor((span - total) / 2);
      let bx = groupStart;

      RATINGS.forEach((rating, i) => {
        const w = widths[i];
        const looks: Look[] = rating === 'good' ? ['button', 'focused'] : ['button'];

        centered(buttonRow - 1, bx, w, card.intervals[rating], ['muted']);
        put(buttonRow, bx, ' '.repeat(w), looks, rating);
        centered(buttonRow, bx, w, ` ${labels.ratings[rating]} `, looks, rating);
        bx += w + BUTTON_GAP;
      });

      if ((span - total) % 2 === 1) {
        shiftHalf(buttonRow - 1, groupStart, groupStart + total);
        shiftHalf(buttonRow, groupStart, groupStart + total);
      }
    } else {
      centered(buttonRow, x0, span, ` ${labels.showAnswer} `, ['button', 'focused'], 'show');
    }
  }

  // Footer: whole hints only, the rest are in the help overlay
  let fx = 1;

  for (const hint of labels.hints[stage]) {
    const key = ` ${hint.key} `;
    const label = `${hint.label} `;

    if (fx + key.length + label.length > width) {
      break;
    }

    put(HEIGHT - 1, fx, key, ['accent', 'bold', 'hint'], hint.action);
    put(HEIGHT - 1, fx + key.length, label, ['muted', 'hint'], hint.action);
    fx += key.length + label.length + 1;
  }

  return grid;
}

/** Neighbouring cells that look the same, as one piece of text. */
export function toRuns(row: readonly Cell[]): Run[] {
  const runs: Run[] = [];

  row.forEach((cell, start) => {
    const last = runs.at(-1);

    if (
      last?.looks.join(' ') === cell.looks.join(' ') &&
      last.action === cell.action &&
      last.nudge === cell.nudge
    ) {
      last.text += cell.char;
    } else {
      runs.push({
        start,
        text: cell.char,
        looks: cell.looks,
        action: cell.action,
        nudge: cell.nudge,
      });
    }
  });

  return runs;
}

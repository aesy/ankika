import {
  buildScreen,
  HEIGHT,
  toRuns,
  type Cell,
  type Labels,
  type ScreenState,
} from '@/lib/screen';

const labels: Labels = {
  appName: 'ankika',
  deck: 'Spanish::Vocabulary',
  synced: 'Synced',
  account: 'you@example.com',
  showAnswer: 'Show Answer',
  back: 'Back',
  ratings: { again: 'Again', hard: 'Hard', good: 'Good', easy: 'Easy' },
  congratulations: 'Congratulations! You have finished this deck for now.',
  studied: 'Studied 4 cards today.',
  hints: {
    question: [
      { key: 'Space', label: 'Show Answer', action: 'show' },
      { key: 'r', label: 'Replay Audio' },
      { key: 'Ctrl+Z', label: 'Undo' },
      { key: 'Esc', label: 'Back' },
    ],
    answer: [{ key: '1', label: 'Again', action: 'again' }],
    finished: [{ key: 'Esc', label: 'Back', action: 'restart' }],
  },
};

const card = {
  front: 'el gato',
  back: 'the cat',
  sound: 'gato.mp3',
  queue: 'review' as const,
  intervals: { again: '<10m', hard: '2d', good: '5d', easy: '11d' },
};

const question: ScreenState = { card, revealed: false, counts: { new: 12, learn: 3, review: 41 } };

const text = (row: readonly Cell[]) => row.map(cell => cell.char).join('');

/** Space on each side of `needle` inside the box, counting half-cell nudges. */
function margins(row: readonly Cell[], needle: string) {
  const line = text(row);
  const start = line.indexOf(needle);
  const nudge = row[start].nudge / 2;
  const left = start - 1 + nudge;
  const right = row.length - 1 - (start + needle.length) - nudge;

  return { left, right };
}

describe('buildScreen', () => {
  it('fills every row to the width', () => {
    const grid = buildScreen(question, labels, 60);

    expect(grid).toHaveLength(HEIGHT);
    for (const row of grid) {
      expect(row).toHaveLength(60);
    }
  });

  it.each([60, 61, 76, 95])('centers the card and its button at %i columns', width => {
    const grid = buildScreen(question, labels, width);

    for (const needle of ['el gato', '[sound: gato.mp3]', ' Show Answer ']) {
      const row = grid.find(cells => text(cells).includes(needle))!;
      const { left, right } = margins(row, needle);

      expect([needle, left]).toEqual([needle, right]);
    }
  });

  it('centers the card lines vertically in the box', () => {
    const grid = buildScreen(question, labels, 60);
    const front = grid.findIndex(row => text(row).includes('el gato'));
    const sound = front + 1;
    // The box's inner rows are 2 to HEIGHT - 3
    expect(front - 2).toBe(HEIGHT - 3 - sound);
  });

  it('shows the four answers with their intervals', () => {
    const grid = buildScreen({ ...question, revealed: true }, labels, 76);
    const buttons = grid.find(row => text(row).includes(' Again '))!;
    const intervals = grid[grid.indexOf(buttons) - 1];

    expect(text(buttons)).toMatch(/ Again +Hard +Good +Easy /);
    expect(text(intervals)).toMatch(/<10m.*2d.*5d.*11d/);
    expect(grid.some(row => text(row).includes('the cat'))).toBe(true);
  });

  it('underlines the current card queue in the header', () => {
    const header = buildScreen(question, labels, 60)[0];
    const review = header[text(header).indexOf('41')];

    expect(review.looks).toEqual(['review', 'current']);
  });

  it('shows only whole key hints in the footer', () => {
    const footer = text(buildScreen(question, labels, 46)[HEIGHT - 1]);

    expect(footer).toContain(' Space Show Answer ');
    expect(footer).not.toContain('Ctrl+Z');
  });

  it('shows the finished screen when no card is left', () => {
    const grid = buildScreen({ ...question, card: null }, labels, 76);

    expect(grid.some(row => text(row).includes('Congratulations!'))).toBe(true);
    expect(grid.some(row => row.some(cell => cell.action === 'restart'))).toBe(true);
  });
});

describe('toRuns', () => {
  it('joins neighbouring cells that look the same', () => {
    const runs = toRuns(buildScreen(question, labels, 60)[0]);

    expect(runs[0]).toMatchObject({ start: 0, text: ' ankika' });
    expect(runs.map(run => run.text).join('')).toHaveLength(60);
  });
});

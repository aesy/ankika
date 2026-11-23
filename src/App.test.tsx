import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MantineProvider } from '@mantine/core';
import App from '@/App';
import { initI18n } from '@/i18n';
import { theme } from '@/theme';

beforeAll(async () => {
  await initI18n();
});

function renderApp() {
  return render(
    <MantineProvider theme={theme}>
      <App />
    </MantineProvider>
  );
}

const demo = () => screen.getByLabelText(/ankika review screen/);

describe('App', () => {
  it('shows the headline and every section', () => {
    renderApp();

    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Anki in the terminal.');
    for (const name of [
      'Syncs with AnkiWeb',
      'What it covers',
      'Commands for scripts',
      'Install',
    ]) {
      expect(screen.getByRole('heading', { level: 2, name })).toBeInTheDocument();
    }
  });

  it('reviews a card with the keyboard', async () => {
    const user = userEvent.setup();

    renderApp();
    expect(demo()).toHaveTextContent('el gato');
    expect(demo()).toHaveTextContent('Show Answer');

    demo().focus();
    await user.keyboard(' ');
    expect(demo()).toHaveTextContent('the cat');
    expect(demo()).toHaveTextContent('Again');

    await user.keyboard('3');
    expect(demo()).toHaveTextContent('la estación');
    expect(demo()).not.toHaveTextContent('the cat');
  });

  it('finishes the deck and starts again', async () => {
    const user = userEvent.setup();

    renderApp();
    demo().focus();
    for (let i = 0; i < 4; i++) {
      await user.keyboard(' 1');
    }
    expect(demo()).toHaveTextContent('Congratulations!');

    await user.keyboard('{Escape}');
    expect(demo()).toHaveTextContent('el gato');
  });

  it('answers with a click on a button', async () => {
    const user = userEvent.setup();

    renderApp();
    await user.click(within(demo()).getAllByText(/Show Answer/)[0]);
    await user.click(within(demo()).getAllByText(/^\s*Easy\s*$/)[0]);
    expect(demo()).toHaveTextContent('la estación');
  });

  it('switches install method', async () => {
    const user = userEvent.setup();

    renderApp();
    expect(
      within(screen.getByRole('tabpanel')).getByRole('link', {
        name: 'https://github.com/aesy/ankika/releases',
      })
    ).toBeInTheDocument();

    await user.click(screen.getByRole('tab', { name: 'Cargo' }));
    expect(screen.getByRole('tabpanel')).toHaveTextContent(
      'DESCRIPTORS_BIN=/tmp/anki_descriptors.bin cargo install --locked --git https://github.com/aesy/ankika ankika'
    );
    expect(screen.getByRole('tab', { name: 'Cargo' })).toHaveAttribute('aria-selected', 'true');
  });
});

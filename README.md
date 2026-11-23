# ankika landing page

Built with React, TypeScript, Vite, Mantine, vanilla-extract and i18next. Pushing the `site` branch deploys it to
GitHub Pages.

```sh
npm install
npm run dev
```

## Checks

```sh
npm run lint
npm run format:check
npm run typecheck
npm test
npm run build
```

## Text

All text lives in `src/locales/en.json`. `locales.test.ts` fails on typographic dashes and quotes.

## Review demo

The terminal in the hero is drawn from `src/lib/screen.ts`, which lays out the review screen cell by cell
like the TUI does (`app/ankika/src/tui/view/review.rs`, `header.rs`, `footer.rs` on `main`). Keep the two in step
when the TUI's review screen changes.

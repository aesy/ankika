# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### Added

- `ankika` TUI for Anki, built on Anki's own Rust core, with a command palette and dark and light themes.
- AnkiWeb login and logout, collection sync with a prompt before full syncs, sync on start and quit, and background
  media sync.
- Deck list with due counts and collapsible decks, and a deck overview with custom study, unbury and filtered decks.
- Review screen with card rendering, type-in answers, undo, inline images (terminal graphics with a half-block
  fallback) and audio with autoplay and replay.
- Bury, suspend, flag, mark, delete and note editing during review.
- Add dialog with note type and deck pickers, tag completion and Anki's duplicate and empty checks.
- Card browser with search, a sortable table, preview and bulk actions.
- Statistics with today, future due, reviews, calendar, card counts and retention.
- Settings file and screen, and deck options.
- Profiles with their own collection and login, a startup picker and switching.
- `.apkg` and `.colpkg` import and export, Check Database, Check Media and Empty Cards.
- CLI commands for scripts: `due`, `sync`, `login`, `logout`, `import`, `export`, `check-database`, `check-media`,
  `empty-cards`, `profile`, `completions` and `man`, with JSON output for `due`, `sync` and `profile list`.
- Prebuilt binaries for Linux (x86_64 and ARM64), macOS (Apple silicon) and Windows (x86_64), with shell completions
  and the man page, and an Arch Linux PKGBUILD.

[Unreleased]: https://github.com/aesy/ankika/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aesy/ankika/releases/tag/v0.1.0

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./img/logo-dark.svg">
  <img height="52" alt="ankika" src="./img/logo-light.svg">
</picture>

[![Release][release-image]][release-url]
[![Build Status][github-actions-image]][github-actions-url]
[![Test coverage][codecov-image]][codecov-url]
[![AGPL license][license-image]][license-url]

[release-image]: https://img.shields.io/github/v/release/aesy/ankika?style=flat-square
[release-url]: https://github.com/aesy/ankika/releases

[github-actions-image]: https://img.shields.io/github/actions/workflow/status/aesy/ankika/ci.yml?branch=main&style=flat-square
[github-actions-url]: https://github.com/aesy/ankika/actions

[codecov-image]: https://img.shields.io/codecov/c/github/aesy/ankika?style=flat-square
[codecov-url]: https://codecov.io/github/aesy/ankika

[license-image]: https://img.shields.io/github/license/aesy/ankika?style=flat-square
[license-url]: https://github.com/aesy/ankika/blob/main/LICENSE

Terminal client for [Anki](https://apps.ankiweb.net) that syncs with AnkiWeb, without Anki installed. It's built on
Anki's own Rust core, so cards are scheduled the same way as in Anki. `ankika` is available as a native Linux/macOS/Windows
binary.

[Visit the website](https://aesy.github.io/ankika/) to see what it can do.

## Usage

Run `ankika` to open the TUI, and log in from the command palette to sync with AnkiWeb.

| Flag                  | Environment variable | Default                                      |
|-----------------------|----------------------|----------------------------------------------|
| `--data-dir <DIR>`    | `ANKIKA_DATA_DIR`    | Platform data directory                      |
| `--config-dir <DIR>`  | `ANKIKA_CONFIG_DIR`  | Platform config directory                    |
| `--log-file <FILE>`   | `ANKIKA_LOG_FILE`    | `ankika.log` in the platform cache           |
| `--sync-server <URL>` | `ANKIKA_SYNC_SERVER` | `sync_server` in `config.toml`, else AnkiWeb |
| `--images <MODE>`     | `ANKIKA_IMAGES`      | `images` in `config.toml`, else `auto`       |
| `-v`, `-vv`           | `RUST_LOG`           | `info`                                       |

`ankika login` and `ankika logout` manage the AnkiWeb login without opening the TUI. For scripts:

| Command                      | Prints                                                                          |
|------------------------------|---------------------------------------------------------------------------------|
| `ankika due`                 | Cards due today, for status bars; `--verbose-counts`, `--deck`                  |
| `ankika sync`                | Syncs the collection and media, for timers                                      |
| `ankika import <FILE>`       | Imports an `.apkg`, or replaces the collection with a `.colpkg`                 |
| `ankika export <FILE>`       | Exports the collection or `--deck` as `.apkg`, or a `.colpkg`                   |
| `ankika check-database`      | Anki's Check Database report                                                    |
| `ankika check-media`         | Missing and unused media; `--delete-unused`, `--empty-trash`, `--restore-trash` |
| `ankika empty-cards`         | Cards with an empty front; `--delete`, `--no-keep-notes`                        |
| `ankika completions <SHELL>` | A completion script for bash, zsh, fish, elvish or PowerShell                   |
| `ankika man`                 | The man page                                                                    |

`--json` makes `due`, `sync` and `profile list` print JSON.

### Profiles

Each profile has its own collection and AnkiWeb login. `--profile <NAME>` (or `ANKIKA_PROFILE`) picks one;
otherwise ankika opens the one used last. `ankika profile list`, `add`, `rename` and `remove` manage them.

### Settings

`config.toml` in the config directory holds ankika's own settings; the Settings screen (`,`) edits it and keeps
your comments. Every key is optional:

```toml
theme = "dark"          # or "light"
sync_on_start = true
sync_on_quit = true
images = "auto"         # kitty, sixel, iterm2, halfblocks or off
sync_server = "http://localhost:8080/"
```

Flags and environment variables override the file for that run only.

## Installation

### Binaries

The latest `ankika` releases are available as [prebuilt binaries under GitHub
releases](https://github.com/aesy/ankika/releases) for Linux (x86_64 and ARM64), macOS (Apple silicon) and Windows
(x86_64). Each archive holds the binary, shell completions for bash, zsh, fish and PowerShell, and the man page.
Binaries for other platforms have to be built from source.

#### Linux

Example on how to download and install an `ankika` binary using [curl](https://curl.se/):

```shell
curl -L https://github.com/aesy/ankika/releases/download/v0.1.0/ankika-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo install ankika-v0.1.0-x86_64-unknown-linux-gnu/ankika /usr/local/bin
```

On ARM64, download `ankika-v0.1.0-aarch64-unknown-linux-gnu.tar.gz` instead. Audio playback needs ALSA's shared library
(`alsa-lib` or `libasound2`), which most desktop distributions already have.

#### macOS

Example on how to download and install an `ankika` binary using [curl](https://curl.se/):

```shell
curl -L https://github.com/aesy/ankika/releases/download/v0.1.0/ankika-v0.1.0-aarch64-apple-darwin.tar.gz | tar -xz
sudo install ankika-v0.1.0-aarch64-apple-darwin/ankika /usr/local/bin
```

#### Windows

Download `ankika-v0.1.0-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/aesy/ankika/releases)
and put `ankika.exe` somewhere on your `PATH`.

### Arch Linux

Build the package in [`pkg/arch`](/pkg/arch/PKGBUILD), which also installs the completions and the man page:

```shell
git clone https://github.com/aesy/ankika
cd ankika/pkg/arch
makepkg -si
```

### Cargo

`ankika` depends on Anki's core straight from its Git repository, so it isn't on crates.io. Cargo can install it from
this repository instead, with the [build dependencies](#prerequisites) installed. `cargo install` doesn't read the
repository's Cargo config, so `DESCRIPTORS_BIN` has to be set by hand to a file Anki's build scripts can share:

```shell
DESCRIPTORS_BIN=/tmp/anki_descriptors.bin cargo install --locked --git https://github.com/aesy/ankika ankika
```

### Build from source

To build the binary, run the following command:

```shell
cargo build --release
```

The resulting binary can be found in the repository's root directory under `target/release/` called `ankika`.

## Development

#### Prerequisites

* [Rust Stable 1.100.0+](https://www.rust-lang.org/tools/install)
* [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* `protoc` (Protocol Buffers compiler), which Anki's core needs at build time
* ALSA's development files, on Linux only, for audio playback

| Platform   | Install                                        |
|------------|------------------------------------------------|
| Arch Linux | `pacman -S protobuf alsa-lib`                  |
| Debian     | `apt install protobuf-compiler libasound2-dev` |
| macOS      | `brew install protobuf`                        |
| Windows    | `winget install protobuf`                      |

#### Build

To compile the project, simply issue the following command:

```shell
$ cargo build
```

#### Test

##### Linting

This project uses [rustfmt](https://github.com/rust-lang/rustfmt) for formatting and
[clippy](https://github.com/rust-lang/rust-clippy) for linting:

```shell
$ cargo fmt
$ cargo clippy --all-targets
```

Dependency licenses and sources are checked with [cargo-deny](https://github.com/EmbarkStudios/cargo-deny):

```shell
$ cargo deny check
```

##### Unit/Integration Testing

All code that goes into main must pass all tests. To run all tests, use:

```shell
$ cargo test --workspace
```

#### Website

The website lives on the [`site`](https://github.com/aesy/ankika/tree/site) branch and is deployed to GitHub Pages
whenever that branch is pushed.

## Contribute

Use the [issue tracker](https://github.com/aesy/ankika/issues) to report bugs or make feature requests. Pull requests
are welcome, but it may be a good idea to create an issue to discuss any changes beforehand.

## License

AGPL-3.0-or-later, as required by Anki's core. See [LICENSE](/LICENSE) file.

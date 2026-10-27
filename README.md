# ankika

Terminal client for [Anki](https://apps.ankiweb.net) that syncs with AnkiWeb, without Anki installed.
Built on Anki's own Rust core.

## Building

Requires the latest stable Rust and `protoc` (Protocol Buffers compiler), which Anki's core needs at build time.
On Linux, audio playback also needs ALSA's development files.

| Platform    | Install                                            |
|-------------|----------------------------------------------------|
| Arch Linux  | `pacman -S protobuf alsa-lib`                      |
| Debian      | `apt install protobuf-compiler libasound2-dev`     |
| macOS       | `brew install protobuf`                            |
| Windows     | `winget install protobuf`                          |

```sh
cargo build --release
```

## Usage

| Flag                  | Environment variable | Default                              |
|-----------------------|----------------------|--------------------------------------|
| `--data-dir <DIR>`    | `ANKIKA_DATA_DIR`    | Platform data directory              |
| `--config-dir <DIR>`  | `ANKIKA_CONFIG_DIR`  | Platform config directory            |
| `--log-file <FILE>`   | `ANKIKA_LOG_FILE`    | `ankika.log` in the platform cache   |
| `--sync-server <URL>` | `ANKIKA_SYNC_SERVER` | `sync_server` in `config.toml`, else AnkiWeb |
| `--images <MODE>`     | `ANKIKA_IMAGES`      | `images` in `config.toml`, else `auto` |
| `-v`, `-vv`           | `RUST_LOG`           | `info`                               |

`ankika login` and `ankika logout` manage the AnkiWeb login without opening the TUI.

## Settings

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

## License

AGPL-3.0-or-later, as required by Anki's core. See [LICENSE](LICENSE).

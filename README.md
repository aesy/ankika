# ankika

Terminal client for [Anki](https://apps.ankiweb.net) that syncs with AnkiWeb, without Anki installed.
Built on Anki's own Rust core.

## Building

Requires the latest stable Rust and `protoc` (Protocol Buffers compiler), which Anki's core needs at build time.

| Platform    | Install `protoc`                |
|-------------|---------------------------------|
| Arch Linux  | `pacman -S protobuf`            |
| Debian      | `apt install protobuf-compiler` |
| macOS       | `brew install protobuf`         |
| Windows     | `winget install protobuf`       |

```sh
cargo build --release
```

## Usage

| Flag                  | Environment variable | Default                              |
|-----------------------|----------------------|--------------------------------------|
| `--data-dir <DIR>`    | `ANKIKA_DATA_DIR`    | Platform data directory              |
| `--config-dir <DIR>`  | `ANKIKA_CONFIG_DIR`  | Platform config directory            |
| `--log-file <FILE>`   | `ANKIKA_LOG_FILE`    | `ankika.log` in the platform cache   |
| `-v`, `-vv`           | `RUST_LOG`           | `info`                               |

## License

AGPL-3.0-or-later, as required by Anki's core. See [LICENSE](LICENSE).

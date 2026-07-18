# Symphony

Symphony is a terminal music player built with Rust, Ratatui, and Crossterm. Search and stream songs from YouTube without an account, or play local MP3, FLAC, M4A/AAC, Ogg Vorbis, and WAV files through the system's default audio output.

## Quick start

Install Symphony on Linux with one command:

```sh
curl -fsSL https://raw.githubusercontent.com/Richuchusave/symphony/main/install.sh | sh
```

The installer downloads the correct prebuilt binary for x86-64 or ARM64, verifies its checksum, installs `yt-dlp`, and installs `mpv` through a supported Linux package manager when necessary. No Rust toolchain is required. Open a new terminal if the installer adds `~/.local/bin` to your `PATH`, then run:

```sh
symphony
```

On first launch, Symphony creates its configuration and data directories using the platform defaults. YouTube is the default provider and does not require authentication.

To install somewhere else or pin a release, set `SYMPHONY_INSTALL_DIR` or `SYMPHONY_VERSION`:

```sh
curl -fsSL https://raw.githubusercontent.com/Richuchusave/symphony/main/install.sh | SYMPHONY_VERSION=v0.1.0 sh
```

## Controls

| Key | Action |
| --- | --- |
| `/` | Focus search |
| `Enter` | Submit search or play the selected track |
| `Esc` | Leave search or navigate back |
| `j` / `k` | Select the next / previous item |
| `Space` | Play or pause |
| `h` / `l` | Previous or next track |
| `Left` / `Right` | Seek backward or forward |
| `Up` / `Down` | Change volume |
| `z` / `x` | Toggle shuffle or repeat |
| `Ctrl+Q` | Open the queue |
| `Ctrl+B` | Toggle the sidebar |
| `?` | Show in-app help |
| `q` | Quit |

To use the local catalog, set `general.default_provider = "local"` and configure `providers.local.music_directories` in the generated `config.toml`.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --locked
```

CI runs the same checks. Build artifacts belong in `target/` and are intentionally excluded from version control.

Pushing a `v*` tag builds checksum-verified Linux x86-64 and ARM64 archives and publishes them as a GitHub release. The public installer always selects the latest release unless `SYMPHONY_VERSION` is set.

## Project structure

- `src/app.rs` coordinates events, providers, playback, and persistence.
- `src/provider/` contains the demo and local-file providers.
- `src/playback/` contains the playback abstraction and queue behavior.
- `src/ui/` contains input handling, layout, components, and screens.
- `src/db/` stores the queue, library models, and settings in SQLite.

## License

MIT. See [LICENSE](LICENSE).

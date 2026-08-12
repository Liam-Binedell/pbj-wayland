# pbj-wayland

A background daemon that watches whatever's currently playing over MPRIS (Spotify, Cider, or anything else exposing the standard Linux media-player D-Bus interface) and sets your Wayland wallpaper to a processed version of the current track's album art.

On every track change, it:

1. Downloads the new track's album art and decodes it straight from memory — no intermediate file.
2. Builds a 1920×1080 wallpaper from it: a blurred copy of the artwork is scaled up and center-cropped to fill the whole canvas (`resize_cover`), then the original sharp artwork is composited on top, scaled down to fit while preserving aspect ratio, and centered (`resize_contain`).
3. Writes the result to a fixed path under the system temp directory and hands it off to `awww` to apply it as the wallpaper.

This is a live-updating tool, not a one-shot script — it holds an open MPRIS event subscription and reacts to events as they happen, and idles and retries if no player is active yet rather than requiring one at startup.

## How it works

- **Player discovery & retry** — `mpris::PlayerFinder` looks for an active player in a loop; if none is found, it prints a message and retries every 2 seconds rather than exiting, so the daemon can be started before anything is playing.
- **Events** — once a player is found, its blocking event stream is consumed and dispatched by type: `TrackChanged` triggers an album art lookup and wallpaper update, `Playing`/`Paused` are logged, and `PlayerShutDown` is logged and falls back to the retry loop to wait for the next active player.
- **Album art fetch** — a blocking `reqwest` client downloads the art URL's response bytes directly into memory; `image`'s `load_from_memory` decodes it into a `DynamicImage` without ever touching disk.
- **Compositing** — two distinct resize passes, not one:
  - `resize_cover` scales the blurred copy up so it fully covers the 1920×1080 target, then center-crops the overflow — this becomes the background.
  - `resize_contain` scales the sharp copy down to fit within the same target without cropping, then composites it centered over the cover'd background.
- **Applying the wallpaper** — the finished JPEG is written to `<tmp>/pbj-wayland.jpg` (`std::env::temp_dir()`) and handed off to [`awww`](https://codeberg.org/LGFae/awww) (`awww img <path>`) via `std::process::Command`, the actively-maintained successor to `swww`.

## Requirements

- A Wayland compositor with [`awww`](https://codeberg.org/LGFae/awww) installed and running as the wallpaper daemon
- A media player exposing an MPRIS D-Bus interface (Spotify, Cider, etc.) — doesn't need to be running yet when the daemon starts
- Rust toolchain, edition 2024 (uses `imageproc`, `mpris`, and `reqwest` with the `blocking` feature)

## Building

```sh
cargo build --release
```

## Running

```sh
cargo run --release
```

No arguments needed — the daemon prints its progress (waiting for a player, player found, album art found) as it runs, and idles until a player appears rather than requiring one up front.

## Installing

`deploy.sh` builds a release binary and installs it system-wide:

```sh
./deploy.sh
```

This runs `cargo build --release`, removes any existing `/usr/bin/pbj-wayland`, and copies the fresh binary into place — `sudo` is required for the install step.

## Known issues / still to fix

- The 1920×1080 target resolution is hardcoded in `process_album_art` — not yet derived from the actual output/screen resolution, so non-1080p or multi-monitor setups won't be framed correctly.
- Failures from the `awww` command (e.g. it isn't installed, or errors on a given image) are silently swallowed rather than logged — worth surfacing for debugging.

## Roadmap

Part of a broader plan to build a music-reactive wallpaper engine — this is the Wayland-oriented piece, following on from an initial KDE Plasma-focused version.

# pbj-wayland

A background daemon that watches whatever's currently playing over MPRIS (Spotify, Cider, or anything else exposing the standard Linux media-player D-Bus interface) and sets your Wayland wallpaper to a processed version of the current track's album art.

On every track change, it:

1. Finds the active MPRIS player and reads the new track's `art_url` from its metadata.
2. Downloads the album art.
3. Builds a 1920×1080 wallpaper from it: a heavily blurred, scaled-up copy of the artwork as the background, with the original artwork composited on top, centered and scaled down (`resize_contain`).
4. Writes the result to disk and hands it off to the system's wallpaper tool to actually apply it.

This is a live-updating, "wallpaper reacts to what you're listening to" tool rather than a one-shot script — it holds an open MPRIS event subscription and reacts to `TrackChanged` events as they happen, skipping redundant work if the same track/art URL comes through twice.

## How it works

- **Player discovery & events** — `mpris::PlayerFinder` locates the active player and exposes a blocking event stream; the daemon matches on `Event::TrackChanged` and pulls the `art_url` out of the track metadata.
- **Album art fetch** — `reqwest` (blocking client) downloads the art URL to a local file.
- **Compositing** — `imageproc`/`image` do the actual pixel work: the artwork is cloned and blurred for the background, the foreground copy is Lanczos3-resized to fit within the target dimensions while preserving aspect ratio, and the two are composited with the foreground centered over the blurred backdrop.
- **Applying the wallpaper** — the finished JPEG is handed off to `awww` (`awww img <path>`) via `std::process::Command`, the wlroots wallpaper daemon that succeeded `swww`.

## Requirements

- A Wayland compositor and [`awww`](https://codeberg.org/LGFae/awww) (the actively-maintained successor to `swww`, same CLI) installed and running as the wallpaper daemon
- A media player currently running that exposes an MPRIS D-Bus interface (Spotify, Cider, etc.)
- Rust toolchain (uses `imageproc`, `mpris`, and `reqwest` with the `blocking` feature)

## Building

```sh
cargo build --release
```

## Running

```sh
cargo run --release
```

The daemon prints which player it attached to, then blocks listening for track changes until killed.

## Things to fix before relying on this day-to-day

- The output path is hardcoded to `/home/lee/Pictures/temp.jpg` in `main.rs` — this needs to be parameterized (e.g. via `$HOME` or a config file) before it'll work on another machine or user account.
- No `.unwrap()`/`.expect()` hardening yet around player discovery — if no active MPRIS player is found at startup, the program will panic rather than retry or wait.

## Roadmap

Part of a broader plan to build a music-reactive wallpaper engine — this is the Wayland/`wlr-layer-shell`-oriented piece, following on from an initial KDE Plasma-focused version.

# pbj-wayland

A background daemon that watches your currently-playing media over MPRIS (Spotify, Cider, or anything else exposing the standard Linux media-player D-Bus interface) and turns the current track's album art into a Wayland wallpaper.

On every track change, it:

1. Finds the active MPRIS player and reads the new track's `art_url` from its metadata.
2. Downloads the album art.
3. Builds a 1920×1080 wallpaper from it: a blurred copy of the artwork is scaled and cropped to fill the whole canvas, then the original sharp artwork is composited on top, centered and scaled down to fit.
4. Writes the result to disk and hands it off to `awww` to apply it.

This is a live-updating tool, not a one-shot script — it holds an open MPRIS event subscription and reacts to `TrackChanged` events as they come in, skipping redundant work if the same track/art URL fires twice in a row.

## How it works

- **Player discovery & events** — `mpris::PlayerFinder` locates the active player and exposes a blocking event stream; the daemon matches on `Event::TrackChanged` and pulls `art_url` out of the track metadata. If no active player is found, it reports that and exits cleanly rather than panicking.
- **Album art fetch** — a blocking `reqwest` client downloads the art to the path passed on the command line.
- **Compositing** — two separate resize passes, not one:
  - `resize_cover` scales the blurred copy up so it fully covers the 1920×1080 target, then center-crops off the overflow — this is the background layer.
  - `resize_contain` scales the sharp copy down to fit within the same target while preserving aspect ratio, then composites it centered over the cover'd background.
- **Applying the wallpaper** — the finished JPEG is handed off to [`awww`](https://codeberg.org/LGFae/awww) (the actively-maintained successor to `swww`) via `std::process::Command`.

## Requirements

- A Wayland compositor with [`awww`](https://codeberg.org/LGFae/awww) installed and running as the wallpaper daemon
- A media player running that exposes an MPRIS D-Bus interface (Spotify, Cider, etc.)
- Rust toolchain (uses `imageproc`, `mpris`, and `reqwest` with the `blocking` feature)

## Building

```sh
cargo build --release
```

## Running

The temp file path (where downloaded/processed album art is written) is passed as a required argument:

```sh
cargo run --release -- /path/to/temp.jpg
```

Running with no argument (or more than one) prints a usage message and exits.

## Known issues / still to fix

- The 1920×1080 target resolution is still hardcoded in `process_album_art` — it isn't yet derived from the actual output/screen resolution, so multi-monitor or non-1080p setups won't be framed correctly.
- `meta.art_url()` and the `awww` command's output are still unwrapped — a track with no art, or a failed `awww` invocation, will panic the whole daemon rather than logging and continuing.

## Roadmap

Part of a broader plan to build a music-reactive wallpaper engine — this is the Wayland / `wlr-layer-shell`-oriented piece, following on from an earlier KDE Plasma-focused version.

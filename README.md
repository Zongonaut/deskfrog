# DeskFrog 🐸

A tiny desktop companion in the tradition of Oneko that wanders around your screen: a 🐸 that takes roguelike-style grid steps, naps, gets startled, and flees when your cursor gets too close, commenting in a small DOS-style speech bubble along the way. No image or sound assets: the frog is your system's own emoji font, and the speech bubble uses an embedded 8x8 bitmap font.

Windows only for now.

## Tech stack
- Minimalistic by design.
- **Rust**, talking to the Win32 API directly via the [`windows`](https://crates.io/crates/windows) crate; 
  no windowing framework.
- Rendering is CPU-composited and pushed to screen with `UpdateLayeredWindow`, which is
  what gives the frog real per-pixel alpha transparency and click-through.
- [`swash`](https://crates.io/crates/swash) rasterizes the 🐸 emoji straight out of the
  system's color emoji font (Segoe UI Emoji). It's a real glyph, not a drawn sprite.
- The speech-bubble font is the public-domain
  [font8x8](https://github.com/dhepper/font8x8) bitmap font, embedded directly in
  the source and integer-scaled for a crisp retro-terminal look.

## Building

**Prerequisites:**
- Rust via [rustup](https://rustup.rs), targeting `x86_64-pc-windows-msvc`.
- The MSVC linker and resource compiler (`rc.exe`). These come with the
  [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio)
  ("Desktop development with C++" workload), or a full Visual Studio install. Most Rust
  setups on Windows already have this.

```sh
# Debug build -> target/debug/deskfrog.exe
cargo build

# Release build -> target/release/deskfrog.exe
cargo build --release

# Or run directly
cargo run
cargo run --release
```

## Usage

Launch `deskfrog.exe`: the frog appears somewhere on your primary monitor and starts
wandering. It runs from the system tray; there's no main window.

- **Right-click the tray icon** (in the notification area, click the `^` "show hidden
  icons" arrow if you don't see it right away) for **About** and **Quit**.
- **Quit** from that menu is how you close DeskFrog, there's nothing to click on the
  frog itself.
- Move your mouse near the frog and it'll bolt; get close while it's sleeping and it'll
  wake up startled.

## Configuration

DeskFrog reads an optional `deskfrog.toml` from next to the `.exe`. If it's missing, or
fails to parse, DeskFrog just falls back to its built-in defaults; a bad config file is
never a reason it won't start. A documented example lives at
[`deskfrog.toml`](deskfrog.toml) in this repo.

Configurable: frog size, speech-bubble text scale, how often the frog's brain "ticks"
(movement/behavior timing), how close the cursor has to get before it flees, and the
list of things it says.

> **Heads up:** in TOML, a plain `key = value` line belongs to whichever `[section]`
> table appears *above* it in the file. So the top-level `messages` list needs to stay
> above the `[appearance]`/`[behavior]` headers. Move it below one and it silently
> becomes part of that section instead, and DeskFrog falls back to its built-in
> messages. The shipped example is already ordered correctly.

## License

DeskFrog v1.0
Copyright (c) 2026 Dominic Lenz

Open source software licensed under the **GNU General Public License v3.0**. See
[LICENSE](LICENSE) for the full text.

Speech-bubble font: **Font8x8** by Daniel Hepper ([Public Domain](https://github.com/dhepper/font8x8)).

Project home: <https://github.com/Zongonaut/deskfrog/>

> This software was created with AI support.
# DeskFrog 🐸

A tiny desktop companion in the tradition of Oneko that wanders around your screen: a 🐸 that takes roguelike-style grid steps, naps, gets startled, and flees when your cursor gets too close, commenting in a small DOS-style speech bubble along the way. No image or sound assets: the frog is your system's own emoji font, and the speech bubble uses an embedded 8x8 bitmap font.

Windows and Linux (X11) are both supported.

## Tech stack
- Minimalistic by design.
- **Rust**, talking to the platform's native windowing API directly; no windowing
  framework, on either platform.
- **Windows**: the [`windows`](https://crates.io/crates/windows) crate. Rendering is
  CPU-composited and pushed to screen with `UpdateLayeredWindow`, which is what gives
  the frog real per-pixel alpha transparency and click-through.
- **Linux**: [`x11rb`](https://crates.io/crates/x11rb) (a pure-Rust X11 client; no
  libX11/libxcb needed to build or run). The window is an X11 *override-redirect*
  window (bypassing the window manager entirely, in the same style as classic desktop
  pets like `oneko`/`xpenguins`) on a 32-bit ARGB visual, composited by whatever
  compositor your desktop already runs. Click-through uses the XShape extension's input
  shape, set to empty once at startup. The tray icon uses
  [`ksni`](https://crates.io/crates/ksni) (StatusNotifierItem over D-Bus).
- [`swash`](https://crates.io/crates/swash) rasterizes the 🐸 emoji straight out of the
  system's color emoji font (Segoe UI Emoji on Windows, Noto Color Emoji on Linux). It's
  a real glyph, not a drawn sprite.
- The speech-bubble font is the public-domain
  [font8x8](https://github.com/dhepper/font8x8) bitmap font, embedded directly in
  the source and integer-scaled for a crisp retro-terminal look.

## Building

### Windows

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

### Linux

**Prerequisites:**
- Rust via [rustup](https://rustup.rs).
- An X11 session, or a Wayland session with XWayland (the default on most
  distros. DeskFrog is an X11 client either way; see [Known limitations](#known-limitations)
  for the one Wayland-adjacent caveat that isn't about this).
- No extra system packages needed to *build*: both `x11rb` and `ksni` are pure Rust,
  with no libX11/libxcb/libdbus linking required. `fontconfig` (specifically the
  `fc-match` command) is used at *runtime* to find a color emoji font, but that's
  present on essentially every desktop Linux install already.

```sh
# Debug build -> target/debug/deskfrog
cargo build

# Release build -> target/release/deskfrog
cargo build --release

# Or run directly
cargo run
cargo run --release
```

## Usage

Launch the executable (`deskfrog.exe` on Windows, `deskfrog` on Linux): the frog appears
somewhere on your primary monitor and starts wandering. It runs from the system tray;
there's no main window.

- **Right-click the tray icon** (in the notification area, on Windows, click the `^`
  "show hidden icons" arrow if you don't see it right away; on Linux, this needs a
  StatusNotifierItem host, which GNOME requires the *AppIndicator/KStatusNotifierItem*
  extension for; KDE and Cinnamon support it out of the box) for **About** and **Quit**.
- **Quit** from that menu is how you close DeskFrog, there's nothing to click on the
  frog itself.
- Move your mouse near the frog and it'll bolt; get close while it's sleeping and it'll
  wake up startled.

## Configuration

DeskFrog reads an optional `deskfrog.toml` from next to the executable. If it's missing,
or fails to parse, DeskFrog just falls back to its built-in defaults; a bad config file
is never a reason it won't start. A documented example lives at
[`deskfrog.toml`](deskfrog.toml) in this repo. This works identically on both platforms.

Configurable: frog size, speech-bubble text scale, how often the frog's brain "ticks"
(movement/behavior timing), how close the cursor has to get before it flees, and the
list of things it says.

> **Heads up:** in TOML, a plain `key = value` line belongs to whichever `[section]`
> table appears *above* it in the file. So the top-level `messages` list needs to stay
> above the `[appearance]`/`[behavior]` headers. Move it below one and it silently
> becomes part of that section instead, and DeskFrog falls back to its built-in
> messages. The shipped example is already ordered correctly.

## Known limitations

- **Fedora (and any distro whose default emoji font is COLRv1)**: DeskFrog currently
  fails to start with a "no color emoji font found" message, even if a Noto Color Emoji
  package is installed. Google's Noto Emoji font ships in two different formats: an
  older bitmap (CBDT) build and a newer vector (COLRv1) build, and
  [`swash`](https://crates.io/crates/swash), the font rasterizer DeskFrog uses, can't
  render the COLRv1 format yet. Fedora packages the COLRv1 build by default; Ubuntu,
  Debian, and Linux Mint package the older bitmap build, which works fine.
  **Workaround:** install a CBDT-format Noto Color Emoji font alongside (or instead of)
  the system default. This is a font-format gap, not a distro-specific bug - it's
  possible other distros default to COLRv1 too, or will in the future, since it's
  Google's own recommended direction for emoji fonts going forward.
- Linux support targets **X11 only** (including via XWayland, which is what makes it
  work under most default Wayland sessions too). A native Wayland backend isn't
  planned, since GNOME's and KDE's own Wayland compositors don't support the
  protocol (`wlr-layer-shell`) a borderless always-on-top overlay like this would need
  anyway.

## License

DeskFrog v1.0
Copyright (c) 2026 Dominic Lenz

Open source software licensed under the **GNU General Public License v3.0**. See
[LICENSE](LICENSE) for the full text.

DeskFrog is built on a number of Rust crates, each under its own permissive
license (MIT, Apache-2.0, or Unlicense); see
[THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt) for the full notices,
generated with [`cargo-about`](https://crates.io/crates/cargo-about).

Speech-bubble font: **Font8x8** by Daniel Hepper ([Public Domain](https://github.com/dhepper/font8x8)).

Project home: <https://github.com/Zongonaut/deskfrog/>

> This software was created with AI support.

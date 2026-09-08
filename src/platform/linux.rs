use crate::brain::movement::{MonitorBounds, Point};
use crate::render::Frame;
use ksni::TrayMethods;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use x11rb::connection::Connection;
use x11rb::protocol::randr;
use x11rb::protocol::shape;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

static CONN: OnceLock<(RustConnection, usize)> = OnceLock::new();

fn conn_and_screen() -> (&'static RustConnection, usize) {
    let (conn, screen_num) = CONN.get_or_init(|| {
        x11rb::rust_connection::RustConnection::connect(None).expect("connect to X server")
    });
    (conn, *screen_num)
}

const FROG_CODEPOINT: char = '\u{1F438}';

/// Path to a color emoji font, discovered via fontconfig rather than a
/// hardcoded path, since both the location AND the meaning of the generic
/// "emoji" alias vary by distro — on Fedora KDE, `fc-match emoji` resolves to
/// a font with no COLOR glyph for our codepoint, where Ubuntu/Mint resolve it
/// to Noto Color Emoji. Every candidate is validated by actually attempting
/// the same rasterization the caller will do, rather than trusting a name.
///
/// If nothing validates, there's a real, distro-level gap (confirmed on a
/// stock Fedora KDE spin, which ships no color emoji font at all) rather than
/// a bug in this search — exits with an actionable message instead of
/// returning a guessed path that would just fail a few lines later as a
/// confusing "file not found".
pub fn emoji_font_path() -> std::path::PathBuf {
    emoji_font_candidates()
        .into_iter()
        .find(|path| has_color_glyph(path, FROG_CODEPOINT))
        .unwrap_or_else(|| {
            eprintln!(
                "DeskFrog needs a color emoji font, and couldn't find one installed.\n\
                 Try installing one and running DeskFrog again:\n\
                 \u{20}   Fedora:             sudo dnf install google-noto-emoji-color-fonts   (or: dnf search noto emoji)\n\
                 \u{20}   Ubuntu/Debian/Mint: sudo apt install fonts-noto-color-emoji\n\
                 \u{20}   Arch:               sudo pacman -S noto-fonts-emoji"
            );
            std::process::exit(1);
        })
}

fn emoji_font_candidates() -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();

    // Ask fontconfig directly which installed font covers this exact
    // codepoint — more reliable across distros than the generic "emoji"
    // alias, which is a convention some fontconfig setups honor but isn't
    // guaranteed to point at a color-capable font everywhere.
    if let Some(path) = fc_match(":charset=1F438") {
        candidates.push(path);
    }
    if let Some(path) = fc_match("emoji") {
        candidates.push(path);
    }

    // Common per-distro install locations, in case fontconfig itself can't
    // find anything (missing fc-match, broken cache, etc).
    for path in [
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf", // Debian/Ubuntu/Mint
        "/usr/share/fonts/google-noto-emoji-color-fonts/NotoColorEmoji.ttf", // Fedora
        "/usr/share/fonts/google-noto-color-emoji-fonts/NotoColorEmoji.ttf", // Fedora (alt name)
        "/usr/share/fonts/noto/NotoColorEmoji.ttf",          // Arch
        "/usr/share/fonts/noto-emoji/NotoColorEmoji.ttf",    // generic
    ] {
        candidates.push(std::path::PathBuf::from(path));
    }
    candidates
}

fn fc_match(pattern: &str) -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("fc-match")
        .args([pattern, "-f", "%{file}"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!path.is_empty()).then(|| std::path::PathBuf::from(path))
}

fn has_color_glyph(path: &std::path::Path, ch: char) -> bool {
    std::fs::read(path)
        .ok()
        .and_then(|data| crate::render::glyph::rasterize_color_emoji(&data, ch, 32.0))
        .is_some()
}

/// Current mouse position in root-window (virtual-desktop) coordinates.
pub fn cursor_pos() -> Point {
    let (conn, screen_num) = conn_and_screen();
    let root = conn.setup().roots[screen_num].root;
    match conn.query_pointer(root).ok().and_then(|c| c.reply().ok()) {
        Some(reply) => Point {
            x: reply.root_x as i32,
            y: reply.root_y as i32,
        },
        None => Point { x: 0, y: 0 },
    }
}

/// Enumerates monitors via RandR, in root-window (virtual-desktop) coordinates.
pub fn enumerate_monitors() -> Vec<MonitorBounds> {
    let (conn, screen_num) = conn_and_screen();
    let root = conn.setup().roots[screen_num].root;
    match randr::get_monitors(conn, root, true).ok().and_then(|c| c.reply().ok()) {
        Some(reply) => reply
            .monitors
            .iter()
            .map(|m| MonitorBounds {
                x: m.x as i32,
                y: m.y as i32,
                width: m.width as i32,
                height: m.height as i32,
            })
            .collect(),
        None => Vec::new(),
    }
}

/// An override-redirect, click-through, always-on-top ARGB window — the X11
/// equivalent of the Windows layered window. Click-through is a single empty
/// input shape set once at creation (the whole window always passes input
/// through), not recomputed per frame.
pub struct FrogWindow {
    win: Window,
    gc: Gcontext,
    width: u32,
    height: u32,
    tick_interval_ms: Cell<u32>,
    quit_flag: Arc<AtomicBool>,
}

impl FrogWindow {
    pub fn new(width: u32, height: u32) -> Result<Self, BoxError> {
        let (conn, screen_num) = conn_and_screen();
        let screen = &conn.setup().roots[screen_num];

        let visual_id = screen
            .allowed_depths
            .iter()
            .find(|d| d.depth == 32)
            .and_then(|d| d.visuals.iter().find(|v| v.class == VisualClass::TRUE_COLOR))
            .map(|v| v.visual_id)
            .ok_or("no 32-bit ARGB visual available on this display")?;

        let colormap = conn.generate_id()?;
        conn.create_colormap(ColormapAlloc::NONE, colormap, screen.root, visual_id)?
            .check()?;

        let win = conn.generate_id()?;
        let aux = CreateWindowAux::new()
            .background_pixel(0)
            .border_pixel(0)
            .colormap(colormap)
            .override_redirect(1);
        conn.create_window(
            32,
            win,
            screen.root,
            0,
            0,
            width as u16,
            height as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            visual_id,
            &aux,
        )?
        .check()?;

        let gc = conn.generate_id()?;
        conn.create_gc(gc, win, &CreateGCAux::new())?.check()?;

        // Whole window is permanently click-through.
        shape::rectangles(
            conn,
            shape::SO::SET,
            shape::SK::INPUT,
            ClipOrdering::UNSORTED,
            win,
            0,
            0,
            &[],
        )?
        .check()?;

        Ok(Self {
            win,
            gc,
            width,
            height,
            tick_interval_ms: Cell::new(500),
            quit_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn show(&self) {
        let (conn, _) = conn_and_screen();
        let _ = conn.map_window(self.win);
        let _ = conn.flush();
    }

    /// Adds a StatusNotifierItem tray icon (About/Quit) via `ksni`. This needs a
    /// StatusNotifierWatcher running (provided by the desktop shell — GNOME,
    /// KDE, most DEs with the AppIndicator extension) to actually display
    /// anywhere; if none is available, or D-Bus itself isn't reachable, this
    /// logs and continues without a tray rather than taking the app down —
    /// the frog itself doesn't depend on the tray to work.
    pub fn enable_tray_icon(&mut self, tooltip: &str) {
        let quit_flag = self.quit_flag.clone();
        let tooltip = tooltip.to_string();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt,
                Err(err) => {
                    eprintln!("deskfrog: tray disabled, couldn't start async runtime: {err}");
                    return;
                }
            };
            rt.block_on(async move {
                let icon = tray_icon_pixmap();
                let tray = DeskFrogTray {
                    quit_flag,
                    icon,
                    tooltip,
                };
                match tray.spawn().await {
                    Ok(_handle) => std::future::pending::<()>().await,
                    Err(err) => eprintln!("deskfrog: tray disabled, no StatusNotifierWatcher? ({err})"),
                }
            });
        });
    }

    /// Repositions and redraws in one round-trip-free pair of requests, and
    /// re-asserts topmost stacking (an override-redirect window's z-order
    /// isn't otherwise maintained as later windows get created).
    pub fn present(&self, frame: &Frame, x: i32, y: i32) {
        debug_assert_eq!(frame.width, self.width);
        debug_assert_eq!(frame.height, self.height);
        let (conn, _) = conn_and_screen();
        let _ = conn.configure_window(
            self.win,
            &ConfigureWindowAux::new().x(x).y(y).stack_mode(StackMode::ABOVE),
        );
        let _ = conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.win,
            self.gc,
            self.width as u16,
            self.height as u16,
            0,
            0,
            0,
            32,
            &frame.bgra,
        );
        let _ = conn.flush();
    }

    pub fn start_timer(&self, interval_ms: u32) {
        self.tick_interval_ms.set(interval_ms);
    }

    /// X11 has no built-in timer primitive like Win32's SetTimer — this is a
    /// plain sleep loop instead of a message pump, draining any pending X
    /// events each tick just to stay responsive, and polling the quit flag
    /// (set by the tray's Quit handler, once tray support lands) to exit.
    pub fn run_message_loop_with(&self, mut on_tick: impl FnMut() -> (Frame, i32, i32)) {
        let (conn, _) = conn_and_screen();
        let interval = std::time::Duration::from_millis(self.tick_interval_ms.get() as u64);
        loop {
            if self.quit_flag.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(interval);
            if self.quit_flag.load(Ordering::Relaxed) {
                break;
            }
            while let Ok(Some(_ev)) = conn.poll_for_event() {}
            let (frame, x, y) = on_tick();
            self.present(&frame, x, y);
        }
    }
}

struct DeskFrogTray {
    quit_flag: Arc<AtomicBool>,
    icon: Option<ksni::Icon>,
    tooltip: String,
}

impl ksni::Tray for DeskFrogTray {
    fn id(&self) -> String {
        "deskfrog".into()
    }

    fn title(&self) -> String {
        self.tooltip.clone()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icon.iter().cloned().collect()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        vec![
            StandardItem {
                label: "About".into(),
                activate: Box::new(|_this: &mut Self| show_about()),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| this.quit_flag.store(true, Ordering::Relaxed)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Rasterizes the same 🐸 glyph the frog itself uses, converted to the ARGB32
/// network-byte-order pixmap format the StatusNotifierItem spec wants —
/// reusing the existing rasterizer rather than shipping a separate asset.
fn tray_icon_pixmap() -> Option<ksni::Icon> {
    let font_data = std::fs::read(emoji_font_path()).ok()?;
    let glyph = crate::render::glyph::rasterize_color_emoji(&font_data, '\u{1F438}', 32.0)?;
    let mut argb = Vec::with_capacity((glyph.width * glyph.height * 4) as usize);
    for px in glyph.rgba.chunks_exact(4) {
        argb.extend_from_slice(&[px[3], px[0], px[1], px[2]]);
    }
    Some(ksni::Icon {
        width: glyph.width as i32,
        height: glyph.height as i32,
        data: argb,
    })
}

/// A plain, WM-decorated (not override-redirect, not click-through) window
/// showing the About text, reusing the same embedded 8x8 font the speech
/// bubbles use rather than pulling in a GTK/Qt dependency for a native dialog.
/// Closing is left entirely to the window manager's own decorations (no
/// WM_DELETE_WINDOW handling) — acceptable for a rarely-opened, short-lived
/// utility window.
fn show_about() {
    let (conn, screen_num) = conn_and_screen();
    let screen = conn.setup().roots[screen_num].clone();

    let lines = [
        "DeskFrog v1.0",
        "",
        "Copyright (c) 2026 Dominic Lenz",
        "",
        "Open Source software licensed under GNU GPL v3",
        "",
        "https://github.com/Zongonaut/deskfrog/",
        "",
        "Font8x8 by Daniel Hepper (Public Domain)",
    ];

    let scale: u32 = 2;
    let padding: i32 = 12;
    let line_h = (crate::render::font::GLYPH_HEIGHT * scale) as i32;
    let max_cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u32;
    let width = max_cols * crate::render::font::GLYPH_WIDTH * scale + padding as u32 * 2;
    let height = line_h as u32 * lines.len() as u32 + padding as u32 * 2;

    let Ok(win) = conn.generate_id() else { return };
    let aux = CreateWindowAux::new()
        .background_pixel(screen.black_pixel)
        .event_mask(EventMask::EXPOSURE);
    if conn
        .create_window(
            screen.root_depth,
            win,
            screen.root,
            0,
            0,
            width as u16,
            height as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &aux,
        )
        .is_err()
    {
        return;
    }

    let _ = conn.change_property8(
        PropMode::REPLACE,
        win,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        b"About DeskFrog",
    );

    // Opt into the WM_DELETE_WINDOW protocol so the window manager sends us a
    // polite ClientMessage on close instead of its fallback for
    // non-participating windows: forcibly killing this X connection outright
    // — which took the frog window down with it, since they share one
    // connection (the bug reported after the first Linux build).
    let Some(wm_protocols) = conn.intern_atom(false, b"WM_PROTOCOLS").ok().and_then(|c| c.reply().ok()).map(|r| r.atom) else {
        return;
    };
    let Some(wm_delete_window) = conn.intern_atom(false, b"WM_DELETE_WINDOW").ok().and_then(|c| c.reply().ok()).map(|r| r.atom) else {
        return;
    };
    let _ = conn.change_property32(PropMode::REPLACE, win, wm_protocols, AtomEnum::ATOM, &[wm_delete_window]);

    let Ok(gc) = conn.generate_id() else { return };
    let _ = conn.create_gc(
        gc,
        win,
        &CreateGCAux::new().foreground(0x00ffffff).background(screen.black_pixel),
    );

    let draw = || {
        for (row, line) in lines.iter().enumerate() {
            let y = padding + row as i32 * line_h;
            for (col, ch) in line.chars().enumerate() {
                let x = padding + col as i32 * (crate::render::font::GLYPH_WIDTH * scale) as i32;
                let bits = crate::render::font::glyph_for(ch);
                let mut rects = Vec::new();
                for r in 0..8u32 {
                    for c in 0..8u32 {
                        if (bits[r as usize] >> c) & 1 == 1 {
                            rects.push(Rectangle {
                                x: (x + (c * scale) as i32) as i16,
                                y: (y + (r * scale) as i32) as i16,
                                width: scale as u16,
                                height: scale as u16,
                            });
                        }
                    }
                }
                if !rects.is_empty() {
                    let _ = conn.poly_fill_rectangle(win, gc, &rects);
                }
            }
        }
        let _ = conn.flush();
    };

    let _ = conn.map_window(win);
    let _ = conn.flush();
    std::thread::sleep(std::time::Duration::from_millis(300));
    draw();

    // Block this (dedicated tray-handler) thread until the dialog is closed,
    // redrawing on Expose (e.g. after being uncovered) and watching for the
    // WM_DELETE_WINDOW message to know when to close — never touching the
    // frog window or anything else on the shared connection.
    loop {
        match conn.poll_for_event() {
            Ok(Some(x11rb::protocol::Event::Expose(ev))) if ev.window == win => draw(),
            Ok(Some(x11rb::protocol::Event::ClientMessage(ev)))
                if ev.window == win && ev.data.as_data32()[0] == wm_delete_window =>
            {
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let _ = conn.destroy_window(win);
    let _ = conn.free_gc(gc);
    let _ = conn.flush();
}

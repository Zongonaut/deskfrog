use deskfrog::brain::movement::{monitor_containing, Point};
use deskfrog::brain::{Frog, WorldInput};
use deskfrog::platform::windows::{cursor_pos, enumerate_monitors, FrogWindow};
use deskfrog::render::bubble::{self, BubbleStyle};
use deskfrog::render::font::GLYPH_HEIGHT;
use deskfrog::render::glyph::GlyphBitmap;
use deskfrog::render;

const FROG_SIZE_PX: f32 = 24.0;
const STEP_PX: i32 = FROG_SIZE_PX as i32;
const STEP_INTERVAL_MS: u32 = 500;
const MAX_BUBBLE_LINES: u32 = 3;

fn window_origin(pos: Point, glyph_center: (i32, i32)) -> (i32, i32) {
    (pos.x - glyph_center.0, pos.y - glyph_center.1)
}

/// A bubble drawn "above" the frog needs room between the frog and the top of
/// its monitor; if there isn't enough, flip the layout so it draws below instead.
fn prefer_below(pos: Point, monitors: &[deskfrog::brain::movement::MonitorBounds], canvas_h: u32) -> bool {
    match monitor_containing(pos, monitors) {
        Some(m) => (pos.y - m.y) < (canvas_h as i32 / 2),
        None => false,
    }
}

fn main() {
    let font_data = std::fs::read(r"C:\Windows\Fonts\seguiemj.ttf").expect("failed to read emoji font");
    let glyph: GlyphBitmap = render::glyph::rasterize_color_emoji(&font_data, '\u{1F438}', FROG_SIZE_PX)
        .expect("failed to rasterize frog emoji");

    let monitors = enumerate_monitors();
    let primary = monitors.first().expect("no monitors detected");
    let start = Point {
        x: primary.x + primary.width / 2,
        y: primary.y + primary.height / 2,
    };

    let mut rng = rand::rng();
    let mut frog = Frog::new(start, STEP_PX, &mut rng);
    let style = BubbleStyle::default();

    // Canvas must comfortably fit the widest/tallest possible bubble plus the
    // frog and its margins/gap, on either the "above" or "below" side of it —
    // otherwise the bubble box clips against the canvas edge.
    let bubble_gap_and_margins = 16;
    let canvas_w = bubble::max_width(&style) + 16;
    let canvas_h = MAX_BUBBLE_LINES * GLYPH_HEIGHT * style.scale
        + (style.padding as u32) * 2
        + glyph.height
        + bubble_gap_and_margins;

    let window = FrogWindow::new(canvas_w, canvas_h).expect("failed to create window");
    window.show();

    let render_frog = |frog: &Frog, monitors: &[deskfrog::brain::movement::MonitorBounds]| {
        let layout = bubble::compose(
            canvas_w,
            canvas_h,
            &glyph,
            frog.bubble().as_deref(),
            &style,
            prefer_below(frog.pos, monitors, canvas_h),
        );
        let (x, y) = window_origin(frog.pos, layout.glyph_center);
        (layout.frame, x, y)
    };

    let (frame, x0, y0) = render_frog(&frog, &monitors);
    window.present(&frame, x0, y0);

    window.start_timer(STEP_INTERVAL_MS);
    window.run_message_loop_with(|| {
        frog.step(
            &WorldInput {
                monitors: &monitors,
                cursor: cursor_pos(),
            },
            &mut rng,
        );
        render_frog(&frog, &monitors)
    });
}

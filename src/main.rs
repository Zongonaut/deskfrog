mod brain;
mod platform;
mod render;

use brain::movement::Point;
use brain::{Frog, WorldInput};
use platform::windows::{enumerate_monitors, FrogWindow};
use render::glyph::GlyphBitmap;
use render::Frame;

const FROG_SIZE_PX: f32 = 24.0;
const WINDOW_SIZE: u32 = 60;
const STEP_PX: i32 = FROG_SIZE_PX as i32;
const STEP_INTERVAL_MS: u32 = 500;

fn draw_frame(glyph: &GlyphBitmap) -> Frame {
    let mut frame = Frame::new(WINDOW_SIZE, WINDOW_SIZE);
    let x = ((WINDOW_SIZE - glyph.width) / 2) as i32;
    let y = ((WINDOW_SIZE - glyph.height) / 2) as i32;
    frame.blit_rgba_straight(x, y, glyph.width, glyph.height, &glyph.rgba);
    frame
}

fn window_origin(pos: Point) -> (i32, i32) {
    let half = (WINDOW_SIZE / 2) as i32;
    (pos.x - half, pos.y - half)
}

fn main() {
    let font_data = std::fs::read(r"C:\Windows\Fonts\seguiemj.ttf").expect("failed to read emoji font");
    let glyph = render::glyph::rasterize_color_emoji(&font_data, '\u{1F438}', FROG_SIZE_PX)
        .expect("failed to rasterize frog emoji");

    let monitors = enumerate_monitors();
    let primary = monitors.first().expect("no monitors detected");
    let start = Point {
        x: primary.x + primary.width / 2,
        y: primary.y + primary.height / 2,
    };

    let mut frog = Frog::new(start, STEP_PX);
    let mut rng = rand::rng();

    let window = FrogWindow::new(WINDOW_SIZE, WINDOW_SIZE).expect("failed to create window");
    window.show();

    let (x0, y0) = window_origin(frog.pos);
    window.present(&draw_frame(&glyph), x0, y0);

    window.start_timer(STEP_INTERVAL_MS);
    window.run_message_loop_with(|| {
        frog.step(&WorldInput { monitors: &monitors }, &mut rng);
        let (x, y) = window_origin(frog.pos);
        (draw_frame(&glyph), x, y)
    });
}

mod platform;
mod render;

use platform::windows::FrogWindow;
use render::Frame;

const FROG_SIZE_PX: f32 = 48.0;
const WINDOW_SIZE: u32 = 120;

fn main() {
    let font_path = r"C:\Windows\Fonts\seguiemj.ttf";
    let font_data = std::fs::read(font_path).expect("failed to read emoji font");

    let glyph = render::glyph::rasterize_color_emoji(&font_data, '\u{1F438}', FROG_SIZE_PX)
        .expect("failed to rasterize frog emoji");

    let mut frame = Frame::new(WINDOW_SIZE, WINDOW_SIZE);
    let x = ((WINDOW_SIZE - glyph.width) / 2) as i32;
    let y = ((WINDOW_SIZE - glyph.height) / 2) as i32;
    frame.blit_rgba_straight(x, y, glyph.width, glyph.height, &glyph.rgba);

    let window = FrogWindow::new(WINDOW_SIZE, WINDOW_SIZE).expect("failed to create window");
    window.show();
    window.present(&frame, 800, 400);
    window.run_message_loop();
}

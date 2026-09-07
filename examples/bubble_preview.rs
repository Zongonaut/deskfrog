//! Dev-only visual check for `render::bubble::compose`. Not part of the app;
//! run with `cargo run --example bubble_preview` and inspect the PNGs it writes.
use deskfrog::render::bubble::{compose, BubbleStyle};
use deskfrog::render::glyph::rasterize_color_emoji;

fn save_frame(frame: &deskfrog::render::Frame, path: &str) {
    // Frame is premultiplied BGRA; convert to straight RGBA over black for viewing.
    let mut rgba = vec![0u8; (frame.width * frame.height * 4) as usize];
    for i in 0..(frame.width * frame.height) as usize {
        let b = frame.bgra[i * 4];
        let g = frame.bgra[i * 4 + 1];
        let r = frame.bgra[i * 4 + 2];
        let a = frame.bgra[i * 4 + 3];
        rgba[i * 4] = r;
        rgba[i * 4 + 1] = g;
        rgba[i * 4 + 2] = b;
        rgba[i * 4 + 3] = a;
    }
    image::RgbaImage::from_raw(frame.width, frame.height, rgba)
        .unwrap()
        .save(path)
        .unwrap();
}

fn main() {
    let font_data = std::fs::read(r"C:\Windows\Fonts\seguiemj.ttf").expect("read emoji font");
    let glyph = rasterize_color_emoji(&font_data, '\u{1F438}', 24.0).expect("rasterize frog");
    let style = BubbleStyle::default();

    let w = deskfrog::render::bubble::max_width(&style) + 16;
    let h = 3 * deskfrog::render::font::GLYPH_HEIGHT * style.scale + (style.padding as u32) * 2 + glyph.height + 16;

    let above = compose(w, h, &glyph, Some("I have important frog business."), &style, false);
    save_frame(&above.frame, "bubble_above.png");

    let below = compose(w, h, &glyph, Some("zZzZz"), &style, true);
    save_frame(&below.frame, "bubble_below.png");

    let none = compose(w, h, &glyph, None, &style, false);
    save_frame(&none.frame, "bubble_none.png");

    let diag = compose(w, h, &glyph, Some("AaBbCc123 Fp"), &style, false);
    save_frame(&diag.frame, "bubble_diag.png");

    println!("wrote bubble_above.png, bubble_below.png, bubble_none.png, bubble_diag.png");
}

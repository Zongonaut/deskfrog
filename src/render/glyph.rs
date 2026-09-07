use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::FontRef;

pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    /// Straight (non-premultiplied) RGBA, row-major, top-to-bottom.
    pub rgba: Vec<u8>,
}

/// Rasterizes a single color emoji glyph from raw font bytes.
/// Returns `None` if the font has no glyph for `ch`, or the glyph is not a color bitmap.
pub fn rasterize_color_emoji(font_data: &[u8], ch: char, size_px: f32) -> Option<GlyphBitmap> {
    let font = FontRef::from_index(font_data, 0)?;
    let glyph_id = font.charmap().map(ch);
    if glyph_id == 0 {
        return None;
    }

    let mut context = ScaleContext::new();
    let mut scaler = context.builder(font).size(size_px).hint(false).build();

    let image = Render::new(&[
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::ColorOutline(0),
    ])
    .render(&mut scaler, glyph_id)?;

    if image.content != Content::Color {
        return None;
    }

    Some(GlyphBitmap {
        width: image.placement.width,
        height: image.placement.height,
        rgba: image.data,
    })
}

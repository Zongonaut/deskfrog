use super::font::{self, GLYPH_HEIGHT, GLYPH_WIDTH};
use super::glyph::GlyphBitmap;
use super::Frame;

const EDGE_MARGIN: i32 = 4;
const BUBBLE_GAP: i32 = 3;

pub struct BubbleStyle {
    pub scale: u32,
    pub padding: i32,
    pub fg: (u8, u8, u8, u8),
    pub bg: (u8, u8, u8, u8),
    pub border: (u8, u8, u8, u8),
    pub max_cols: usize,
}

impl Default for BubbleStyle {
    fn default() -> Self {
        Self {
            scale: 1,
            padding: 4,
            fg: (230, 230, 230, 255),
            bg: (10, 10, 10, 235),
            border: (230, 230, 230, 255),
            max_cols: 16,
        }
    }
}

/// Greedy word wrap to at most `max_cols` characters per line.
pub fn wrap_text(text: &str, max_cols: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= max_cols {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// The widest a bubble can ever get for this style — a full line of `max_cols`
/// characters. Callers need this to size a canvas that never clips the bubble.
pub fn max_width(style: &BubbleStyle) -> u32 {
    style.max_cols as u32 * GLYPH_WIDTH * style.scale + (style.padding as u32) * 2
}

fn measure(lines: &[String], style: &BubbleStyle) -> (u32, u32) {
    let cols = lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
        .max(1) as u32;
    let rows = lines.len() as u32;
    let w = cols * GLYPH_WIDTH * style.scale + (style.padding as u32) * 2;
    let h = rows * GLYPH_HEIGHT * style.scale + (style.padding as u32) * 2;
    (w, h)
}

fn draw_glyph(frame: &mut Frame, x: i32, y: i32, glyph: &[u8; 8], scale: u32, color: (u8, u8, u8, u8)) {
    for row in 0..8u32 {
        let bits = glyph[row as usize];
        for col in 0..8u32 {
            if (bits >> col) & 1 == 1 {
                frame.fill_rect(
                    x + (col * scale) as i32,
                    y + (row * scale) as i32,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}

fn draw_bubble(frame: &mut Frame, x: i32, y: i32, lines: &[String], style: &BubbleStyle) {
    let (w, h) = measure(lines, style);
    frame.fill_rect(x, y, w, h, style.bg);
    frame.stroke_rect(x, y, w, h, style.border);

    for (row_idx, line) in lines.iter().enumerate() {
        let line_y = y + style.padding + (row_idx as u32 * GLYPH_HEIGHT * style.scale) as i32;
        for (col_idx, ch) in line.chars().enumerate() {
            let char_x = x + style.padding + (col_idx as u32 * GLYPH_WIDTH * style.scale) as i32;
            draw_glyph(frame, char_x, line_y, &font::glyph_for(ch), style.scale, style.fg);
        }
    }
}

/// A composed frame plus the on-screen offset (within the frame) of the frog
/// glyph's center — the platform layer positions the window so this point
/// lands exactly on the frog's logical world position.
pub struct Layout {
    pub frame: Frame,
    pub glyph_center: (i32, i32),
}

/// Lays out the frog glyph and an optional speech bubble onto a fixed-size canvas.
/// When `prefer_below` is set (the frog is too close to the top edge of its
/// monitor for a bubble to fit above it), the frog anchors near the top of the
/// canvas and the bubble is drawn beneath it instead.
pub fn compose(
    canvas_w: u32,
    canvas_h: u32,
    glyph: &GlyphBitmap,
    bubble_text: Option<&str>,
    style: &BubbleStyle,
    prefer_below: bool,
) -> Layout {
    let mut frame = Frame::new(canvas_w, canvas_h);

    let frog_x = ((canvas_w - glyph.width) / 2) as i32;
    let frog_y = if prefer_below {
        EDGE_MARGIN
    } else {
        canvas_h as i32 - EDGE_MARGIN - glyph.height as i32
    };

    if let Some(text) = bubble_text {
        let lines = wrap_text(text, style.max_cols);
        let (bw, bh) = measure(&lines, style);
        let bubble_x = frog_x + glyph.width as i32 / 2 - bw as i32 / 2;
        let bubble_y = if prefer_below {
            frog_y + glyph.height as i32 + BUBBLE_GAP
        } else {
            frog_y - BUBBLE_GAP - bh as i32
        };
        draw_bubble(&mut frame, bubble_x, bubble_y, &lines, style);
    }

    frame.blit_rgba_straight(frog_x, frog_y, glyph.width, glyph.height, &glyph.rgba);

    let glyph_center = (
        frog_x + glyph.width as i32 / 2,
        frog_y + glyph.height as i32 / 2,
    );
    Layout { frame, glyph_center }
}

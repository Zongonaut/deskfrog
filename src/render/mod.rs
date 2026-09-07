pub mod bubble;
pub mod font;
pub mod glyph;

/// A CPU-composited frame in BGRA, premultiplied-alpha, top-down row order —
/// the exact layout `UpdateLayeredWindow` (ULW_ALPHA) expects on Windows.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl Frame {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bgra: vec![0u8; (width * height * 4) as usize],
        }
    }

    /// Blits a straight-alpha RGBA source onto this frame at (x, y), alpha-composited
    /// "over" whatever is already there (so it can safely overlap other drawing).
    pub fn blit_rgba_straight(&mut self, x: i32, y: i32, w: u32, h: u32, src_rgba: &[u8]) {
        for row in 0..h {
            let dy = y + row as i32;
            if dy < 0 || dy >= self.height as i32 {
                continue;
            }
            for col in 0..w {
                let dx = x + col as i32;
                if dx < 0 || dx >= self.width as i32 {
                    continue;
                }
                let si = ((row * w + col) * 4) as usize;
                self.blend_pixel(
                    dx as u32,
                    dy as u32,
                    src_rgba[si],
                    src_rgba[si + 1],
                    src_rgba[si + 2],
                    src_rgba[si + 3],
                );
            }
        }
    }

    /// Fills an axis-aligned rectangle with a straight-alpha color, composited "over".
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, rgba: (u8, u8, u8, u8)) {
        let (r, g, b, a) = rgba;
        for row in 0..h {
            let dy = y + row as i32;
            if dy < 0 || dy >= self.height as i32 {
                continue;
            }
            for col in 0..w {
                let dx = x + col as i32;
                if dx < 0 || dx >= self.width as i32 {
                    continue;
                }
                self.blend_pixel(dx as u32, dy as u32, r, g, b, a);
            }
        }
    }

    /// Draws a 1px rectangle outline with a straight-alpha color.
    pub fn stroke_rect(&mut self, x: i32, y: i32, w: u32, h: u32, rgba: (u8, u8, u8, u8)) {
        if w == 0 || h == 0 {
            return;
        }
        self.fill_rect(x, y, w, 1, rgba);
        self.fill_rect(x, y + h as i32 - 1, w, 1, rgba);
        self.fill_rect(x, y, 1, h, rgba);
        self.fill_rect(x + w as i32 - 1, y, 1, h, rgba);
    }

    /// Alpha-composites one straight-alpha RGBA pixel "over" the premultiplied
    /// pixel already at (x, y).
    fn blend_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        let a = a as u32;
        let inv_a = 255 - a;
        let pr = (r as u32 * a) / 255;
        let pg = (g as u32 * a) / 255;
        let pb = (b as u32 * a) / 255;

        let di = ((y * self.width + x) * 4) as usize;
        self.bgra[di] = (pb + (self.bgra[di] as u32 * inv_a) / 255) as u8;
        self.bgra[di + 1] = (pg + (self.bgra[di + 1] as u32 * inv_a) / 255) as u8;
        self.bgra[di + 2] = (pr + (self.bgra[di + 2] as u32 * inv_a) / 255) as u8;
        self.bgra[di + 3] = (a + (self.bgra[di + 3] as u32 * inv_a) / 255) as u8;
    }
}

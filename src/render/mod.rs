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

    /// Blits a straight-alpha RGBA source onto this frame at (x, y), premultiplying
    /// and converting channel order as it goes.
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
                let r = src_rgba[si] as u32;
                let g = src_rgba[si + 1] as u32;
                let b = src_rgba[si + 2] as u32;
                let a = src_rgba[si + 3] as u32;

                let pr = (r * a) / 255;
                let pg = (g * a) / 255;
                let pb = (b * a) / 255;

                let di = ((dy as u32 * self.width + dx as u32) * 4) as usize;
                self.bgra[di] = pb as u8;
                self.bgra[di + 1] = pg as u8;
                self.bgra[di + 2] = pr as u8;
                self.bgra[di + 3] = a as u8;
            }
        }
    }
}

use std::fs;

use ab_glyph::{point, Font, FontArc, FontVec, PxScale, ScaleFont};

pub struct TextRenderer {
    font: Option<FontArc>,
    serif_font: Option<FontArc>,
}

impl TextRenderer {
    pub fn load() -> Self {
        let mut regular = None;
        let candidates = [
            r"C:\Windows\Fonts\Deng.ttf",
            r"C:\Windows\Fonts\simhei.ttf",
        ];
        for path in candidates {
            if let Ok(data) = fs::read(path) {
                if let Ok(font) = FontArc::try_from_vec(data) {
                    regular = Some(font);
                    break;
                }
            }
        }
        if regular.is_none() {
            if let Ok(data) = fs::read(r"C:\Windows\Fonts\msyh.ttc") {
            if let Ok(font) = FontVec::try_from_vec_and_index(data, 0) {
                    regular = Some(font.into());
                }
            }
        }

        let serif_font = [
            r"C:\Windows\Fonts\georgiab.ttf",
            r"C:\Windows\Fonts\timesbd.ttf",
            r"C:\Windows\Fonts\cambria.ttc",
        ]
        .iter()
        .find_map(|path| {
            let data = fs::read(path).ok()?;
            FontArc::try_from_vec(data).ok()
        });

        Self {
            font: regular,
            serif_font,
        }
    }

    pub fn text_width(&self, text: &str, size: f32) -> f32 {
        if let Some(font) = &self.font {
            let scale = font.as_scaled(PxScale::from(size));
            text.chars()
                .map(|c| scale.h_advance(scale.glyph_id(c)))
                .sum()
        } else {
            text.len() as f32 * size * 0.6
        }
    }

    pub fn draw_text(
        &self,
        buf: &mut [u32],
        w: usize,
        h: usize,
        x: f32,
        baseline_y: f32,
        text: &str,
        color: u32,
        size: f32,
    ) {
        let Some(font) = &self.font else {
            return;
        };
        let scale = font.as_scaled(PxScale::from(size));
        let mut pen_x = x;
        for ch in text.chars() {
            let id = scale.glyph_id(ch);
            let glyph = id.with_scale_and_position(PxScale::from(size), point(pen_x, baseline_y));
            if let Some(outlined) = scale.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        let alpha = ((cov * 255.0).round() as u32).min(255);
                        if alpha > 0 {
                            let idx = py as usize * w + px as usize;
                            buf[idx] = blend(buf[idx], color, alpha);
                        }
                    }
                });
            }
            pen_x += scale.h_advance(id);
        }
    }

    pub fn serif_text_width(
        &self,
        text: &str,
        size: f32,
        horizontal_stretch: f32,
        tracking: f32,
    ) -> f32 {
        let font = self.serif_font.as_ref().or(self.font.as_ref());
        if let Some(font) = font {
            let scale = font.as_scaled(PxScale {
                x: size * horizontal_stretch,
                y: size,
            });
            text.chars()
                .map(|c| scale.h_advance(scale.glyph_id(c)) + tracking)
                .sum::<f32>()
                .max(0.0)
        } else {
            text.len() as f32 * (size * horizontal_stretch * 0.6 + tracking)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_serif_text(
        &self,
        buf: &mut [u32],
        w: usize,
        h: usize,
        x: f32,
        baseline_y: f32,
        text: &str,
        color: u32,
        size: f32,
        horizontal_stretch: f32,
        tracking: f32,
    ) {
        let Some(font) = self.serif_font.as_ref().or(self.font.as_ref()) else {
            return;
        };
        let px_scale = PxScale {
            x: size * horizontal_stretch,
            y: size,
        };
        let scale = font.as_scaled(px_scale);
        let mut pen_x = x;
        for ch in text.chars() {
            let id = scale.glyph_id(ch);
            let glyph = id.with_scale_and_position(px_scale, point(pen_x, baseline_y));
            if let Some(outlined) = scale.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        let alpha = ((cov * 255.0).round() as u32).min(255);
                        if alpha > 0 {
                            let idx = py as usize * w + px as usize;
                            buf[idx] = blend(buf[idx], color, alpha);
                        }
                    }
                });
            }
            pen_x += scale.h_advance(id) + tracking;
        }
    }
}

fn blend(dst: u32, src: u32, alpha: u32) -> u32 {
    if alpha == 0 {
        return dst;
    }
    if alpha >= 255 {
        return src & 0x00ff_ffff;
    }
    let dr = (dst >> 16) & 0xff;
    let dg = (dst >> 8) & 0xff;
    let db = dst & 0xff;
    let sr = (src >> 16) & 0xff;
    let sg = (src >> 8) & 0xff;
    let sb = src & 0xff;
    let r = (sr * alpha + dr * (255 - alpha)) / 255;
    let g = (sg * alpha + dg * (255 - alpha)) / 255;
    let b = (sb * alpha + db * (255 - alpha)) / 255;
    (r << 16) | (g << 8) | b
}

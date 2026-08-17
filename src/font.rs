use std::fs;

use ab_glyph::{Font, FontArc, FontVec, PxScale, ScaleFont, point};

pub struct TextRenderer {
    font: Option<FontArc>,
    fallback_fonts: Vec<FontArc>,
    serif_font: Option<FontArc>,
}

fn load_face(path: &str) -> Option<FontArc> {
    let data = fs::read(path).ok()?;
    if path.to_ascii_lowercase().ends_with(".ttc") {
        FontVec::try_from_vec_and_index(data, 0)
            .ok()
            .map(Into::into)
    } else {
        FontArc::try_from_vec(data).ok()
    }
}

fn load_first_face(paths: &[&str]) -> Option<FontArc> {
    paths.iter().find_map(|path| load_face(path))
}

fn ui_font_size(size: f32) -> f32 {
    size * if size <= 18.0 { 1.12 } else { 1.08 }
}

fn coverage_alpha(coverage: f32) -> u32 {
    ((coverage.clamp(0.0, 1.0).powf(0.78) * 255.0).round() as u32).min(255)
}

impl TextRenderer {
    pub fn load() -> Self {
        let regular = load_face(r"C:\Windows\Fonts\segoeui.ttf")
            .or_else(|| load_face(r"C:\Windows\Fonts\Deng.ttf"))
            .or_else(|| load_face(r"C:\Windows\Fonts\simhei.ttf"));

        let fallback_fonts = [
            load_first_face(&[
                r"C:\Windows\Fonts\msyh.ttc",
                r"C:\Windows\Fonts\Deng.ttf",
                r"C:\Windows\Fonts\simhei.ttf",
            ]),
            load_first_face(&[r"C:\Windows\Fonts\msjh.ttc", r"C:\Windows\Fonts\msyh.ttc"]),
            load_first_face(&[
                r"C:\Windows\Fonts\YuGothM.ttc",
                r"C:\Windows\Fonts\meiryo.ttc",
            ]),
            load_first_face(&[r"C:\Windows\Fonts\malgun.ttf"]),
        ]
        .into_iter()
        .flatten()
        .collect();

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
            fallback_fonts,
            serif_font,
        }
    }

    fn font_for_char(&self, ch: char) -> Option<&FontArc> {
        if let Some(font) = &self.font {
            if font.glyph_id(ch).0 != 0 {
                return Some(font);
            }
        }
        self.fallback_fonts
            .iter()
            .find(|font| font.glyph_id(ch).0 != 0)
            .or(self.font.as_ref())
    }

    pub fn text_width(&self, text: &str, size: f32) -> f32 {
        let size = ui_font_size(size);
        text.chars()
            .map(|ch| {
                self.font_for_char(ch)
                    .map(|font| {
                        let scale = font.as_scaled(PxScale::from(size));
                        scale.h_advance(scale.glyph_id(ch))
                    })
                    .unwrap_or(size * 0.6)
            })
            .sum()
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
        let size = ui_font_size(size);
        let mut pen_x = x;
        for ch in text.chars() {
            let Some(font) = self.font_for_char(ch) else {
                pen_x += size * 0.6;
                continue;
            };
            let scale = font.as_scaled(PxScale::from(size));
            let id = scale.glyph_id(ch);
            let glyph = id.with_scale_and_position(
                PxScale::from(size),
                point(pen_x.round(), baseline_y.round()),
            );
            if let Some(outlined) = scale.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        let alpha = coverage_alpha(cov);
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
        vertical_stretch: f32,
        tracking: f32,
    ) {
        let Some(font) = self.serif_font.as_ref().or(self.font.as_ref()) else {
            return;
        };
        let px_scale = PxScale {
            x: size * horizontal_stretch,
            y: size * vertical_stretch,
        };
        let scale = font.as_scaled(px_scale);
        let mut pen_x = x;
        for ch in text.chars() {
            let id = scale.glyph_id(ch);
            let glyph =
                id.with_scale_and_position(px_scale, point(pen_x.round(), baseline_y.round()));
            if let Some(outlined) = scale.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        let alpha = coverage_alpha(cov);
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

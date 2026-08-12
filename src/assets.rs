use std::io::Cursor;

use png::{ColorType, Decoder, Transformations};
use shakmaty::{Color, Role};

#[derive(Debug)]
pub struct PieceTexture {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u32>, // 0xAARRGGBB
}

impl PieceTexture {
    fn from_bytes(data: &[u8], name: &str) -> Self {
        let mut decoder = Decoder::new(Cursor::new(data));
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
        let mut reader = decoder
            .read_info()
            .unwrap_or_else(|e| panic!("PNG 解码失败 ({name}): {e}"));
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buf)
            .unwrap_or_else(|e| panic!("PNG 读取失败 ({name}): {e}"));
        let (w, h) = (info.width as usize, info.height as usize);
        let bytes = &buf[..info.buffer_size()];
        let mut pixels = Vec::with_capacity(w * h);

        match info.color_type {
            ColorType::Rgba => {
                for px in bytes.chunks_exact(4) {
                    pixels.push(
                        ((px[3] as u32) << 24)
                            | ((px[0] as u32) << 16)
                            | ((px[1] as u32) << 8)
                            | px[2] as u32,
                    );
                }
            }
            ColorType::Rgb => {
                for px in bytes.chunks_exact(3) {
                    pixels.push(
                        0xff00_0000
                            | ((px[0] as u32) << 16)
                            | ((px[1] as u32) << 8)
                            | px[2] as u32,
                    );
                }
            }
            ColorType::GrayscaleAlpha => {
                for px in bytes.chunks_exact(2) {
                    let g = px[0] as u32;
                    pixels.push(((px[1] as u32) << 24) | (g << 16) | (g << 8) | g);
                }
            }
            ColorType::Grayscale => {
                for &b in bytes {
                    let g = b as u32;
                    pixels.push(0xff00_0000 | (g << 16) | (g << 8) | g);
                }
            }
            ColorType::Indexed => {
                let pal = reader.info().palette.clone().unwrap_or_default();
                for &idx in bytes {
                    let i = idx as usize * 3;
                    if i + 2 < pal.len() {
                        let (r, g, b) = (
                            pal[i] as u32,
                            pal[i + 1] as u32,
                            pal[i + 2] as u32,
                        );
                        pixels.push(0xff00_0000 | (r << 16) | (g << 8) | b);
                    } else {
                        pixels.push(0xff00_0000);
                    }
                }
            }
        }

        PieceTexture { w, h, pixels }
    }
}

pub struct PieceImages {
    white: [PieceTexture; 6],
    black: [PieceTexture; 6],
}

const W_P: &[u8] = include_bytes!("../assets/pieces/wP.png");
const W_N: &[u8] = include_bytes!("../assets/pieces/wN.png");
const W_B: &[u8] = include_bytes!("../assets/pieces/wB.png");
const W_R: &[u8] = include_bytes!("../assets/pieces/wR.png");
const W_Q: &[u8] = include_bytes!("../assets/pieces/wQ.png");
const W_K: &[u8] = include_bytes!("../assets/pieces/wK.png");
const B_P: &[u8] = include_bytes!("../assets/pieces/bP.png");
const B_N: &[u8] = include_bytes!("../assets/pieces/bN.png");
const B_B: &[u8] = include_bytes!("../assets/pieces/bB.png");
const B_R: &[u8] = include_bytes!("../assets/pieces/bR.png");
const B_Q: &[u8] = include_bytes!("../assets/pieces/bQ.png");
const B_K: &[u8] = include_bytes!("../assets/pieces/bK.png");

impl PieceImages {
    pub fn load() -> Self {
        let assets: [(&str, &[u8]); 12] = [
            ("wP", W_P),
            ("wN", W_N),
            ("wB", W_B),
            ("wR", W_R),
            ("wQ", W_Q),
            ("wK", W_K),
            ("bP", B_P),
            ("bN", B_N),
            ("bB", B_B),
            ("bR", B_R),
            ("bQ", B_Q),
            ("bK", B_K),
        ];
        let mut white = Vec::new();
        let mut black = Vec::new();
        for (name, data) in assets {
            let tex = PieceTexture::from_bytes(data, name);
            if name.starts_with('w') {
                white.push(tex);
            } else {
                black.push(tex);
            }
        }
        PieceImages {
            white: white.try_into().expect("6 white pieces"),
            black: black.try_into().expect("6 black pieces"),
        }
    }

    pub fn get(&self, color: Color, role: Role) -> &PieceTexture {
        let set = match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        };
        let idx = match role {
            Role::Pawn => 0,
            Role::Knight => 1,
            Role::Bishop => 2,
            Role::Rook => 3,
            Role::Queen => 4,
            Role::King => 5,
        };
        &set[idx]
    }
}

pub fn fill_rect(
    buf: &mut [u32],
    w: usize,
    h: usize,
    x: i32,
    y: i32,
    rw: i32,
    rh: i32,
    color: u32,
) {
    let color = color & 0x00ff_ffff;
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + rw).min(w as i32);
    let y1 = (y + rh).min(h as i32);
    for py in y0..y1 {
        let row = py as usize * w;
        for px in x0..x1 {
            buf[row + px as usize] = color;
        }
    }
}

pub fn fill_rect_alpha(
    buf: &mut [u32],
    w: usize,
    h: usize,
    x: i32,
    y: i32,
    rw: i32,
    rh: i32,
    color: u32,
    alpha: u32,
) {
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + rw).min(w as i32);
    let y1 = (y + rh).min(h as i32);
    for py in y0..y1 {
        let row = py as usize * w;
        for px in x0..x1 {
            let idx = row + px as usize;
            buf[idx] = blend_color(buf[idx], color, alpha);
        }
    }
}

pub fn fill_circle(
    buf: &mut [u32],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    r: f32,
    color: u32,
    alpha: u32,
) {
    let x0 = (cx - r).floor().max(0.0) as i32;
    let y0 = (cy - r).floor().max(0.0) as i32;
    let x1 = (cx + r).ceil().min(w as f32 - 1.0) as i32;
    let y1 = (cy + r).ceil().min(h as f32 - 1.0) as i32;
    for py in y0..=y1 {
        let row = py as usize * w;
        for px in x0..=x1 {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                buf[row + px as usize] = blend_color(buf[row + px as usize], color, alpha);
            }
        }
    }
}

pub fn draw_ring(
    buf: &mut [u32],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    r: f32,
    color: u32,
    alpha: u32,
) {
    let x0 = (cx - r - 2.0).floor().max(0.0) as i32;
    let y0 = (cy - r - 2.0).floor().max(0.0) as i32;
    let x1 = (cx + r + 2.0).ceil().min(w as f32 - 1.0) as i32;
    let y1 = (cy + r + 2.0).ceil().min(h as f32 - 1.0) as i32;
    for py in y0..=y1 {
        let row = py as usize * w;
        for px in x0..=x1 {
            let dist = ((px as f32 - cx).powi(2) + (py as f32 - cy).powi(2)).sqrt();
            if (dist - r).abs() <= 2.5 {
                buf[row + px as usize] = blend_color(buf[row + px as usize], color, alpha);
            }
        }
    }
}

pub fn draw_arrow_down(
    buf: &mut [u32],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    size: f32,
    color: u32,
) {
    let half = size / 2.0;
    let x0 = (cx - half).floor().max(0.0) as i32;
    let y0 = (cy - size).floor().max(0.0) as i32;
    let x1 = (cx + half).ceil().min(w as f32 - 1.0) as i32;
    let y1 = (cy + size).ceil().min(h as f32 - 1.0) as i32;
    for py in y0..=y1 {
        let t = ((py as f32 - (cy - size)) / (size * 2.0)).clamp(0.0, 1.0);
        let row_half = (t * size).min(half);
        for px in x0..=x1 {
            let dx = (px as f32 - cx).abs();
            if dx <= row_half {
                buf[py as usize * w + px as usize] = color;
            }
        }
    }
}

pub fn draw_arrow_up(
    buf: &mut [u32],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    size: f32,
    color: u32,
) {
    let half = size / 2.0;
    let x0 = (cx - half).floor().max(0.0) as i32;
    let y0 = (cy - size).floor().max(0.0) as i32;
    let x1 = (cx + half).ceil().min(w as f32 - 1.0) as i32;
    let y1 = (cy + size).ceil().min(h as f32 - 1.0) as i32;
    for py in y0..=y1 {
        let t = ((y1 as f32 - py as f32) / (size * 2.0)).clamp(0.0, 1.0);
        let row_half = (t * size).min(half);
        for px in x0..=x1 {
            let dx = (px as f32 - cx).abs();
            if dx <= row_half {
                buf[py as usize * w + px as usize] = color;
            }
        }
    }
}

fn blend_color(dst: u32, src: u32, alpha: u32) -> u32 {
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
    let out_r = (sr * alpha + dr * (255 - alpha)) / 255;
    let out_g = (sg * alpha + dg * (255 - alpha)) / 255;
    let out_b = (sb * alpha + db * (255 - alpha)) / 255;
    (out_r << 16) | (out_g << 8) | out_b
}

fn lerp_channel(c0: u32, c1: u32, c2: u32, c3: u32, shift: u32, fx: f32, fy: f32) -> u32 {
    let a = ((c0 >> shift) & 0xff) as f32;
    let b = ((c1 >> shift) & 0xff) as f32;
    let c = ((c2 >> shift) & 0xff) as f32;
    let d = ((c3 >> shift) & 0xff) as f32;
    let top = a * (1.0 - fx) + b * fx;
    let bottom = c * (1.0 - fx) + d * fx;
    (top * (1.0 - fy) + bottom * fy).round() as u32
}

fn lerp_pixel(c00: u32, c01: u32, c10: u32, c11: u32, fx: f32, fy: f32) -> u32 {
    (lerp_channel(c00, c01, c10, c11, 24, fx, fy) << 24)
        | (lerp_channel(c00, c01, c10, c11, 16, fx, fy) << 16)
        | (lerp_channel(c00, c01, c10, c11, 8, fx, fy) << 8)
        | lerp_channel(c00, c01, c10, c11, 0, fx, fy)
}

pub fn draw_scaled(
    buf: &mut [u32],
    w: usize,
    h: usize,
    tex: &PieceTexture,
    x: f32,
    y: f32,
    dw: f32,
    dh: f32,
) {
    let (dw, dh) = (dw.round() as i32, dh.round() as i32);
    let (x, y) = (x.round() as i32, y.round() as i32);
    if dw <= 0 || dh <= 0 {
        return;
    }
    let tex_w = tex.w as f32;
    let tex_h = tex.h as f32;
    for dy in 0..dh {
        let sy = ((dy as f32 + 0.5) * tex_h / dh as f32) - 0.5;
        let y0 = sy.max(0.0).floor() as usize;
        let y1 = (y0 + 1).min(tex.h - 1);
        let fy = sy.max(0.0) - y0 as f32;
        for dx in 0..dw {
            let sx = ((dx as f32 + 0.5) * tex_w / dw as f32) - 0.5;
            let x0 = sx.max(0.0).floor() as usize;
            let x1 = (x0 + 1).min(tex.w - 1);
            let fx = sx.max(0.0) - x0 as f32;
            let c00 = tex.pixels[y0 * tex.w + x0];
            let c01 = tex.pixels[y0 * tex.w + x1];
            let c10 = tex.pixels[y1 * tex.w + x0];
            let c11 = tex.pixels[y1 * tex.w + x1];
            let color = lerp_pixel(c00, c01, c10, c11, fx, fy);
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                let idx = py as usize * w + px as usize;
                let alpha = (color >> 24) & 0xff;
                buf[idx] = blend_color(buf[idx], color, alpha);
            }
        }
    }
}

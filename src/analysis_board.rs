//! Read-only overlays. Geometry comes from the renderer; no input or game mutation.
use crate::{
    assets::{blend_color, fill_rect},
    font::TextRenderer,
    render::Layout,
    theme::Palette,
    uci_protocol::{Bound, Score},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Visual {
    pub active: bool,
    pub score: Option<(Score, Bound)>,
    pub arrow: Option<(shakmaty::Square, shakmaty::Square)>,
}
/// A bounded monotonic visualization, NOT a win probability.
pub fn white_fraction(score: Option<(Score, Bound)>) -> f64 {
    match score.map(|s| s.0) {
        Some(Score::Cp(cp)) => {
            0.5 + f64::from(cp.clamp(-4000, 4000)).atan2(400.0) / std::f64::consts::PI
        }
        Some(Score::Mate(n)) => {
            if n > 0 {
                1.0
            } else {
                0.0
            }
        }
        Some(Score::Mated(side)) => {
            if side == shakmaty::Color::White {
                0.0
            } else {
                1.0
            }
        }
        None => 0.5,
    }
}
pub fn score_label(score: Option<(Score, Bound)>) -> String {
    score
        .map(|(s, b)| {
            format!(
                "{}{}",
                match b {
                    Bound::Exact => "",
                    Bound::Lower => ">=",
                    Bound::Upper => "<=",
                },
                match s {
                    Score::Cp(cp) => format!("{:+.2}", f64::from(cp) / 100.0),
                    Score::Mate(m) => format!("M{m:+}"),
                    Score::Mated(side) =>
                        if side == shakmaty::Color::White {
                            "M-0".into()
                        } else {
                            "M+0".into()
                        },
                }
            )
        })
        .unwrap_or_else(|| "—".into())
}
pub fn draw(
    buf: &mut [u32],
    size: (usize, usize),
    font: &TextRenderer,
    visual: Visual,
    layout: &Layout,
    flipped: bool,
    pal: &Palette,
) {
    if !visual.active {
        return;
    }
    let x = (layout.board_left - 16.0) as i32;
    let y = layout.board_top as i32;
    let height = layout.board_size as i32;
    let white = (f64::from(height) * white_fraction(visual.score)).round() as i32;
    fill_rect(buf, size, x - 1, y - 1, 12, height + 2, pal.border);
    fill_rect(buf, size, x, y, 10, height, 0x00222222);
    fill_rect(buf, size, x, y + height - white, 10, white, 0x00eeeeee);
    // Unknown score is explicitly marked; bounds and mate retain distinct labels.
    font.draw_text(
        buf,
        size,
        (x as f32, y as f32 - 8.0),
        &score_label(visual.score),
        pal.text,
        12.0,
    );
    let Some((from, to)) = visual.arrow else {
        return;
    };
    let (fx, fy) = layout.square_center(from, flipped);
    let (tx, ty) = layout.square_center(to, flipped);
    let (dx, dy) = (tx - fx, ty - fy);
    let length = dx.hypot(dy);
    if length < 1.0 {
        return;
    }
    let (ux, uy) = (dx / length, dy / length);
    let start = layout.sq * 0.22;
    let end = length - layout.sq * 0.12;
    let head = layout.sq * 0.30;
    let shaft = layout.sq * 0.065;
    // Thin translucent shaft + triangular tip leave most of each piece visible.
    let padding = layout.sq * 0.24;
    let min_x = (fx.min(tx) - padding).max(layout.board_left).max(0.0) as usize;
    let max_x = (fx.max(tx) + padding)
        .min(layout.board_left + layout.board_size)
        .min(size.0 as f32) as usize;
    let min_y = (fy.min(ty) - padding).max(layout.board_top).max(0.0) as usize;
    let max_y = (fy.max(ty) + padding)
        .min(layout.board_top + layout.board_size)
        .min(size.1 as f32) as usize;
    for py in min_y..max_y {
        for px in min_x..max_x {
            let (rx, ry) = (px as f32 + 0.5 - fx, py as f32 + 0.5 - fy);
            let along = rx * ux + ry * uy;
            let across = (-rx * uy + ry * ux).abs();
            let inside = along >= start
                && along <= end
                && if along < end - head {
                    across <= shaft
                } else {
                    across <= (end - along) * 0.65
                };
            if inside {
                let p = &mut buf[py * size.0 + px];
                *p = blend_color(*p, pal.accent, 170);
            }
        }
    }
}

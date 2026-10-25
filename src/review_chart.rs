//! Bounded white-view advantage chart. Missing/non-exact evaluations stay gaps.
use crate::{
    assets::fill_rect,
    font::TextRenderer,
    game::GameResult,
    game_review::{Job, Point},
    review_scoring,
    theme::Palette,
    uci_protocol::{Bound, Score},
};
pub type Rect = (f32, f32, f32, f32);
pub fn rect(size: (usize, usize)) -> Rect {
    (
        size.0 as f32 - 280.0,
        225.0,
        252.0,
        (size.1 as f32 - 360.0).clamp(90.0, 230.0),
    )
}
pub fn value(p: &Point) -> Option<(f64, bool)> {
    if let Some(t) = p.terminal {
        return Some(match t {
            GameResult::Checkmate { winner } => (
                if winner == shakmaty::Color::White {
                    1.0
                } else {
                    -1.0
                },
                true,
            ),
            _ => (0.0, false),
        });
    }
    let (score, bound) = p.score?;
    if bound != Bound::Exact {
        return None;
    }
    Some((
        review_scoring::advantage(score),
        !matches!(score, Score::Cp(_)),
    ))
}
pub fn point(rect: Rect, total: usize, index: usize, value: f64) -> (f32, f32) {
    (
        rect.0 + index as f32 / total.max(1) as f32 * rect.2,
        rect.1 + rect.3 / 2.0 - value.clamp(-1.0, 1.0) as f32 * (rect.3 / 2.0 - 7.0),
    )
}
pub fn hit(job: &Job, rect: Rect, mouse: (f32, f32)) -> Option<usize> {
    let mut best = None;
    let mut distance = 81.0;
    for (i, p) in job.points[..=job.limit].iter().enumerate() {
        let Some((v, _)) = p.as_ref().and_then(value) else {
            continue;
        };
        let (x, y) = point(rect, job.game.moves.len(), i, v);
        let d = (mouse.0 - x).powi(2) + (mouse.1 - y).powi(2);
        if d < distance {
            distance = d;
            best = Some(i);
        }
    }
    best
}
fn line(buf: &mut [u32], size: (usize, usize), a: (f32, f32), b: (f32, f32), color: u32) {
    let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().max(1.0) as usize;
    for n in 0..=steps {
        let t = n as f32 / steps as f32;
        fill_rect(
            buf,
            size,
            (a.0 + (b.0 - a.0) * t) as i32,
            (a.1 + (b.1 - a.1) * t) as i32,
            1,
            1,
            color,
        );
    }
}
pub fn draw(
    buf: &mut [u32],
    size: (usize, usize),
    font: &TextRenderer,
    job: &Job,
    cursor: usize,
    pal: &Palette,
) {
    let r = rect(size);
    let zero = r.1 + r.3 / 2.0;
    fill_rect(
        buf,
        size,
        r.0 as i32,
        r.1 as i32,
        r.2 as i32 + 1,
        r.3 as i32,
        pal.bg,
    );
    line(buf, size, (r.0, zero), (r.0 + r.2, zero), pal.muted);
    let mut previous = None;
    for (i, p) in job.points[..=job.limit].iter().enumerate() {
        let Some((v, mate)) = p.as_ref().and_then(value) else {
            previous = None;
            continue;
        };
        let xy = point(r, job.game.moves.len(), i, v);
        if let Some(prev) = previous {
            line(buf, size, prev, xy, pal.accent);
        }
        previous = Some(xy);
        fill_rect(
            buf,
            size,
            xy.0 as i32 - 1,
            xy.1 as i32 - 1,
            3,
            3,
            pal.accent,
        );
        if mate {
            font.draw_text(
                buf,
                size,
                (xy.0.min(r.0 + r.2 - 12.0), xy.1 + 4.0),
                "M",
                pal.text,
                11.0,
            );
        }
        if i > 0
            && let Some(mark) = match job.moves[i - 1].quality {
                review_scoring::Quality::Mistake => Some("?"),
                review_scoring::Quality::Blunder => Some("??"),
                _ => None,
            }
        {
            font.draw_text(
                buf,
                size,
                (xy.0.min(r.0 + r.2 - 12.0), xy.1 + 15.0),
                mark,
                pal.check_ring,
                12.0,
            );
        }
    }
    let x = point(r, job.game.moves.len(), cursor, 0.0).0;
    line(buf, size, (x, r.1), (x, r.1 + r.3), pal.text);
    font.draw_text(buf, size, (r.0, r.1 + r.3 + 13.0), "0", pal.muted, 12.0);
    font.draw_text(
        buf,
        size,
        (r.0 + r.2 - 38.0, r.1 + r.3 + 13.0),
        &job.game.moves.len().to_string(),
        pal.muted,
        12.0,
    );
}

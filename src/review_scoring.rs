//! Deterministic heuristic, not calibrated win probability or objective chess truth.
use crate::{
    analysis_board,
    game_review::Point,
    uci_protocol::{Bound, Score},
};
use shakmaty::Color;

pub const ALGORITHM: u32 = 1;
pub const MIN_DEPTH: u32 = 10;
pub const MAX_DEPTH_GAP: u32 = 6;
pub const GOOD_LOSS: f64 = 0.05;
pub const INACCURACY_LOSS: f64 = 0.15;
pub const MISTAKE_LOSS: f64 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Best,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
    Unrated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Heuristic,
    Incomplete,
    Terminal,
    MissingScore,
    NonExact,
    Mate,
    LowDepth,
    DepthGap,
    InconsistentBest,
}
#[derive(Clone, Debug)]
pub struct MoveReview {
    pub ply: usize,
    pub side: Color,
    pub san: String,
    pub uci: String,
    pub before: Option<(Score, Bound)>,
    pub after: Option<(Score, Bound)>,
    pub recommended: Option<String>,
    pub recommended_san: Option<String>,
    pub depths: (Option<u32>, Option<u32>),
    pub loss: Option<f64>,
    pub quality: Quality,
    pub reliable: bool,
    pub matched: bool,
    pub reason: Reason,
}
pub fn advantage(score: Score) -> f64 {
    2.0 * analysis_board::white_fraction(Some((score, Bound::Exact))) - 1.0
}
pub fn loss(side: Color, before: f64, after: f64) -> f64 {
    (if side == Color::White {
        before - after
    } else {
        after - before
    })
    .max(0.0)
}
pub fn classify(value: f64) -> Quality {
    if !value.is_finite() || value < 0.0 {
        return Quality::Unrated;
    }
    if value <= GOOD_LOSS {
        Quality::Good
    } else if value <= INACCURACY_LOSS {
        Quality::Inaccuracy
    } else if value <= MISTAKE_LOSS {
        Quality::Mistake
    } else {
        Quality::Blunder
    }
}
pub fn evaluate(m: &mut MoveReview, before: Option<&Point>, after: Option<&Point>) {
    m.before = before.and_then(|p| p.score);
    m.after = after.and_then(|p| p.score);
    m.recommended = before.and_then(|p| p.best.clone());
    m.recommended_san = before.and_then(|p| p.best_san.clone());
    m.matched = m.recommended.as_ref() == Some(&m.uci);
    m.depths = (before.and_then(|p| p.depth), after.and_then(|p| p.depth));
    m.loss = None;
    m.reliable = false;
    m.quality = Quality::Unrated;
    m.reason = (|| {
        let (Some(a), Some(b)) = (before, after) else {
            return Reason::Incomplete;
        };
        if a.terminal.is_some() || b.terminal.is_some() {
            return Reason::Terminal;
        }
        let (Some((sa, ba)), Some((sb, bb))) = (a.score, b.score) else {
            return Reason::MissingScore;
        };
        if ba != Bound::Exact || bb != Bound::Exact {
            return Reason::NonExact;
        }
        if !matches!((sa, sb), (Score::Cp(_), Score::Cp(_))) {
            return Reason::Mate;
        }
        let (Some(da), Some(db)) = m.depths else {
            return Reason::LowDepth;
        };
        if da.min(db) < MIN_DEPTH {
            return Reason::LowDepth;
        }
        if da.abs_diff(db) > MAX_DEPTH_GAP {
            return Reason::DepthGap;
        }
        let value = loss(m.side, advantage(sa), advantage(sb));
        m.loss = Some(value);
        if m.matched && value > GOOD_LOSS {
            return Reason::InconsistentBest;
        }
        m.quality = if m.matched {
            Quality::Best
        } else {
            classify(value)
        };
        m.reliable = true;
        Reason::Heuristic
    })();
}

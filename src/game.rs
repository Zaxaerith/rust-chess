//! Deterministic game-state values; no window, threads or wall-clock reads.
use shakmaty::{Chess, Color, EnPassantMode, Position, fen::Fen};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameResult {
    Checkmate {
        #[serde(with = "crate::persistence::color_serde")]
        winner: Color,
    },
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FivefoldRepetition,
    SeventyFiveMoves,
    Timeout {
        #[serde(with = "crate::persistence::color_serde")]
        loser: Color,
        draw: bool,
    },
    TimeoutUnresolved {
        #[serde(with = "crate::persistence::color_serde")]
        loser: Color,
    },
}

impl GameResult {
    pub fn from_position(pos: &Chess, repetitions: usize) -> Option<Self> {
        // A completed mating move takes priority over automatic draw rules.
        if pos.is_checkmate() {
            Some(Self::Checkmate {
                winner: !pos.turn(),
            })
        } else if pos.is_stalemate() {
            Some(Self::Stalemate)
        } else if pos.is_insufficient_material() {
            Some(Self::InsufficientMaterial)
        } else if repetitions >= 5 {
            Some(Self::FivefoldRepetition)
        } else if pos.halfmoves() >= 150 {
            Some(Self::SeventyFiveMoves)
        } else {
            None
        }
    }

    pub fn timeout(pos: &Chess) -> Self {
        Self::timeout_with(pos, chasolver::winnability)
    }

    fn timeout_with(
        pos: &Chess,
        solve: impl FnOnce(&chess::Board, chess::Color) -> Option<chasolver::Winnability>,
    ) -> Self {
        use chasolver::Winnability;
        let loser = pos.turn();
        if pos.has_insufficient_material(!loser) {
            return Self::Timeout { loser, draw: true };
        }
        let verdict = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let fen = Fen::from_position(pos.clone(), EnPassantMode::Legal).to_string();
            let board = fen.parse::<chess::Board>().ok()?;
            let winner = if loser == Color::White {
                chess::Color::Black
            } else {
                chess::Color::White
            };
            solve(&board, winner)
        }))
        .ok()
        .flatten();
        match verdict {
            Some(Winnability::Unwinnable) => Self::Timeout { loser, draw: true },
            Some(Winnability::Winnable { .. }) => Self::Timeout { loser, draw: false },
            None => {
                eprintln!(
                    "Timeout adjudication failed or exhausted its node budget; no winner declared"
                );
                Self::TimeoutUnresolved { loser }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClockState {
    pub white: Duration,
    pub black: Duration,
    pub warned: [bool; 2],
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            white: Duration::from_secs(600),
            black: Duration::from_secs(600),
            warned: [false; 2],
        }
    }
}

impl ClockState {
    pub fn remaining(&self, color: Color) -> Duration {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }
    /// Returns (expired, newly crossed warning threshold).
    pub fn elapse(&mut self, turn: Color, elapsed: Duration) -> (bool, bool) {
        let (remaining, index) = match turn {
            Color::White => (&mut self.white, 0),
            Color::Black => (&mut self.black, 1),
        };
        let before = *remaining;
        *remaining = remaining.saturating_sub(elapsed);
        let warning = before > Duration::from_secs(30)
            && *remaining <= Duration::from_secs(30)
            && !self.warned[index];
        self.warned[index] |= warning;
        (remaining.is_zero(), warning)
    }

    pub fn add_time(&mut self, color: Color) {
        let remaining = match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        };
        *remaining = remaining.saturating_add(Duration::from_secs(15));
    }
}

/// Repetition identity includes only legal en passant rights and no counters.
pub fn position_key(position: &Chess) -> String {
    Fen::from_position(position.clone(), EnPassantMode::Legal)
        .to_string()
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::CastlingMode;

    fn position(fen: &str) -> Chess {
        fen.parse::<Fen>()
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    #[test]
    fn timeout_forced_capture_of_opponents_last_piece_is_draw() {
        let pos = position("8/8/8/8/8/2k5/r7/KQ6 w - - 0 1");
        assert!(!pos.has_insufficient_material(Color::Black));
        assert!(!pos.legal_moves().is_empty());
        for mv in pos.legal_moves() {
            let next = pos.clone().play(&mv).unwrap();
            assert!(next.has_insufficient_material(Color::Black));
        }
        assert_eq!(
            GameResult::timeout(&pos),
            GameResult::Timeout {
                loser: Color::White,
                draw: true
            }
        );
    }

    #[test]
    fn locked_pawn_wall_is_timeout_draw_despite_material() {
        let pos = position("7k/8/8/p1p1p1p1/P1P1P1P1/8/8/K7 w - - 0 1");
        assert!(!pos.has_insufficient_material(Color::Black));
        assert_eq!(
            GameResult::timeout(&pos),
            GameResult::Timeout {
                loser: Color::White,
                draw: true
            }
        );
    }

    #[test]
    fn asymmetric_locked_bishop_position_uses_correct_intended_winner() {
        // CHA-Solver's documented real-game fixture: only White can helpmate.
        let pos = position("5b2/p7/Pp3k2/1Pp1pBp1/2P1P1P1/5K2/8/8 w - - 0 1");
        assert_eq!(
            GameResult::timeout(&pos),
            GameResult::Timeout {
                loser: Color::White,
                draw: true
            }
        );
        let pos = position("5b2/p7/Pp3k2/1Pp1pBp1/2P1P1P1/5K2/8/8 b - - 0 1");
        assert_eq!(
            GameResult::timeout(&pos),
            GameResult::Timeout {
                loser: Color::Black,
                draw: false
            }
        );
    }

    #[test]
    fn unresolved_or_panicking_adjudication_never_guesses_a_winner() {
        let pos = Chess::default();
        let expected = GameResult::TimeoutUnresolved {
            loser: Color::White,
        };
        assert_eq!(GameResult::timeout_with(&pos, |_, _| None), expected);
        assert_eq!(
            GameResult::timeout_with(&pos, |_, _| panic!("injected failure")),
            expected
        );
    }
}

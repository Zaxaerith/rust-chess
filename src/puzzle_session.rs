//! Independent, deterministic training state. No App/game/engine mutation.
use crate::puzzle_data::Validated;
use serde::{Deserialize, Serialize};
use shakmaty::{Chess, Color, Move, Position, Square};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Waiting,
    Responding,
    Solved,
    Answer,
    Skipped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    FirstTry,
    Assisted,
    Revealed,
    Skipped,
}
pub struct Session {
    pub puzzle: Arc<Validated>,
    pub pos: Chess,
    pub state: State,
    pub next: usize,
    pub played: Vec<Move>,
    pub hints: u8,
    hint_level: u8,
    pub errors: u32,
    pub hint_square: Option<Square>,
    pub answer_cursor: usize,
    pub outcome: Option<Outcome>,
    pub recorded: bool,
    pub started: Instant,
    pub elapsed: Duration,
}
impl Session {
    pub fn new(puzzle: Arc<Validated>, now: Instant) -> Self {
        Self {
            pos: puzzle.positions[1].clone(),
            puzzle,
            state: State::Waiting,
            next: 1,
            played: vec![],
            hints: 0,
            hint_level: 0,
            errors: 0,
            hint_square: None,
            answer_cursor: 1,
            outcome: None,
            recorded: false,
            started: now,
            elapsed: Duration::ZERO,
        }
    }
    pub fn solver(&self) -> Color {
        self.puzzle.solver
    }
    pub fn display(&self) -> &Chess {
        if self.state == State::Answer {
            &self.puzzle.positions[self.answer_cursor]
        } else {
            &self.pos
        }
    }
    fn finish(&mut self, outcome: Outcome, now: Instant) {
        if self.outcome.is_none() {
            self.outcome = Some(outcome);
            self.elapsed = now.saturating_duration_since(self.started);
        }
    }
    pub fn attempt(&mut self, mv: Move, now: Instant) -> Result<bool, String> {
        if self.state != State::Waiting {
            return Err("Not waiting for a player move".into());
        }
        if !self.pos.is_legal(&mv) {
            return Err("Illegal move".into());
        }
        let mut after = self.pos.clone();
        after.play_unchecked(&mv);
        let alternative =
            self.puzzle.data.themes.iter().any(|s| s == "mateIn1") && after.is_checkmate();
        if mv != self.puzzle.moves[self.next] && !alternative {
            self.errors = self.errors.saturating_add(1);
            return Ok(false);
        }
        self.pos = after;
        self.played.push(mv);
        self.next += 1;
        self.hint_square = None;
        self.hint_level = 0;
        if alternative || self.next == self.puzzle.moves.len() {
            self.state = State::Solved;
            self.finish(
                if self.hints == 0 && self.errors == 0 {
                    Outcome::FirstTry
                } else {
                    Outcome::Assisted
                },
                now,
            );
        } else {
            self.state = State::Responding;
        }
        Ok(true)
    }
    pub fn respond(&mut self) {
        if self.state == State::Responding {
            let mv = self.puzzle.moves[self.next].clone();
            self.pos.play_unchecked(&mv);
            self.played.push(mv);
            self.next += 1;
            self.state = State::Waiting;
        }
    }
    pub fn hint(&mut self) {
        if self.state == State::Waiting {
            self.hints = self.hints.saturating_add(1);
            self.hint_level = self.hint_level.saturating_add(1);
            let mv = &self.puzzle.moves[self.next];
            self.hint_square = if self.hint_level == 1 {
                mv.from()
            } else {
                crate::uci_protocol::move_arrow(mv).map(|(_, to)| to)
            };
        }
    }
    pub fn reveal(&mut self, now: Instant) {
        self.finish(Outcome::Revealed, now);
        self.state = State::Answer;
        self.hint_square = None;
        self.answer_cursor = 1;
    }
    pub fn skip(&mut self, now: Instant) {
        self.finish(Outcome::Skipped, now);
        self.state = State::Skipped;
    }
    pub fn answer_step(&mut self, delta: isize) {
        if self.state == State::Answer {
            self.answer_cursor = self
                .answer_cursor
                .saturating_add_signed(delta)
                .clamp(1, self.puzzle.moves.len());
        }
    }
}

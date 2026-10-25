//! Immutable game target + one outstanding UCI request; no second engine client.
use crate::{
    fen,
    game::{GameResult, position_key},
    game_record::ValidatedRecord,
    review_scoring::{self, MoveReview, Quality, Reason},
    uci::{Report, Status},
    uci_protocol::{self, Bound, Score},
};
use serde::{Deserialize, Serialize};
use shakmaty::{Position, uci::UciMove};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Idle,
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Options {
    pub movetime: u32,
    pub max_plies: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            movetime: 500,
            max_plies: 160,
        }
    }
}
impl Options {
    pub fn clamp(&mut self) {
        self.movetime = self.movetime.clamp(100, 2000);
        self.max_plies = self.max_plies.clamp(1, 512);
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub fen: String,
    pub score: Option<(Score, Bound)>,
    pub best: Option<String>,
    pub best_san: Option<String>,
    pub depth: Option<u32>,
    pub elapsed_ms: u64,
    pub reported_ms: Option<u64>,
    pub terminal: Option<GameResult>,
}
struct Pending {
    id: u64,
    index: usize,
    fen: String,
    started: Instant,
}
pub struct Job {
    pub game: ValidatedRecord,
    pub game_key: String,
    pub cache_key: String,
    pub engine: String,
    pub options: Options,
    pub limit: usize,
    pub state: State,
    pub points: Vec<Option<Point>>,
    pub moves: Vec<MoveReview>,
    pub error: Option<String>,
    pub context: u64,
    pub dirty: usize,
    pending: Option<Pending>,
}
impl Job {
    pub fn new(
        game: ValidatedRecord,
        game_key: String,
        cache_key: String,
        engine: String,
        mut options: Options,
        context: u64,
    ) -> Self {
        options.clamp();
        let limit = game.moves.len().min(options.max_plies);
        let moves = (0..game.moves.len())
            .map(|i| MoveReview {
                ply: i + 1,
                side: game.positions[i].turn(),
                san: game.sans[i].clone(),
                uci: UciMove::from_standard(&game.moves[i]).to_string(),
                before: None,
                after: None,
                recommended: None,
                recommended_san: None,
                depths: (None, None),
                loss: None,
                quality: Quality::Unrated,
                reliable: false,
                matched: false,
                reason: Reason::Incomplete,
            })
            .collect();
        let count = game.positions.len();
        Self {
            game,
            game_key,
            cache_key,
            engine,
            options,
            limit,
            state: State::Idle,
            points: vec![None; count],
            moves,
            error: None,
            context,
            dirty: 0,
            pending: None,
        }
    }
    pub fn owns_engine(&self) -> bool {
        matches!(self.state, State::Running | State::Paused)
    }
    pub fn done(&self) -> usize {
        self.points[..=self.limit].iter().flatten().count()
    }
    pub fn next(&self) -> Option<usize> {
        self.points[..=self.limit].iter().position(Option::is_none)
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn begin(&mut self, id: u64, index: usize) {
        self.pending = Some(Pending {
            id,
            index,
            fen: fen::write(&self.game.positions[index]),
            started: Instant::now(),
        });
    }
    pub fn pause(&mut self) {
        if self.state == State::Running {
            self.state = State::Paused;
            self.pending = None;
        }
    }
    pub fn cancel(&mut self) {
        if self.owns_engine() {
            self.state = State::Cancelled;
            self.pending = None;
        }
    }
    pub fn fail(&mut self, error: String) {
        self.state = State::Failed;
        self.pending = None;
        self.error = Some(error);
    }
    pub fn refresh(&mut self) {
        for i in 0..self.moves.len() {
            review_scoring::evaluate(
                &mut self.moves[i],
                self.points[i].as_ref(),
                self.points[i + 1].as_ref(),
            );
        }
    }
    pub fn terminal(&self, index: usize) -> Option<Point> {
        let pos = &self.game.positions[index];
        let key = position_key(pos);
        let repetitions = self.game.positions[..=index]
            .iter()
            .filter(|p| position_key(p) == key)
            .count();
        GameResult::from_position(pos, repetitions).map(|terminal| Point {
            fen: fen::write(pos),
            score: None,
            best: None,
            best_san: None,
            depth: None,
            elapsed_ms: 0,
            reported_ms: None,
            terminal: Some(terminal),
        })
    }
    pub fn store(&mut self, index: usize, point: Point) {
        self.points[index] = Some(point);
        self.dirty += 1;
        for i in index.saturating_sub(1)..=index.min(self.moves.len().saturating_sub(1)) {
            if let Some(m) = self.moves.get_mut(i) {
                review_scoring::evaluate(m, self.points[i].as_ref(), self.points[i + 1].as_ref());
            }
        }
    }
    pub fn accept(&mut self, report: &Report) -> Result<bool, String> {
        if self.state != State::Running {
            return Ok(false);
        }
        if report.status == Status::Failed || report.status == Status::Off {
            return Err(report
                .error
                .clone()
                .unwrap_or_else(|| "Review engine disconnected".into()));
        }
        let Some(p) = &self.pending else {
            return Ok(false);
        };
        if report.request != Some(p.id) || report.status != Status::Complete {
            return Ok(false);
        }
        let index = p.index;
        let pos = &self.game.positions[index];
        if p.fen != fen::write(pos) {
            return Err("Review position identity mismatch".into());
        }
        let text = report
            .best
            .as_ref()
            .ok_or("Review missing final bestmove")?;
        let best_san = uci_protocol::legal_move(pos, text)?
            .ok_or("Review returned null move in nonterminal position")?;
        let mv = text
            .parse::<UciMove>()
            .map_err(|_| "Invalid review bestmove")?
            .to_move(pos)
            .map_err(|_| "Illegal review bestmove")?;
        if report.info.score.is_none() {
            return Err("Review completed without a final score".into());
        }
        if let Some(pv) = &report.info.pv
            && let Some(error) = uci_protocol::variation(pos, pv).error
        {
            return Err(error);
        }
        let point = Point {
            fen: p.fen.clone(),
            score: report
                .info
                .score
                .map(|s| uci_protocol::white_score(s, pos.turn())),
            best: Some(UciMove::from_standard(&mv).to_string()),
            best_san: Some(best_san),
            depth: report.info.depth,
            elapsed_ms: p.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            reported_ms: report.info.time,
            terminal: None,
        };
        self.pending = None;
        self.store(index, point);
        Ok(true)
    }
}

#[derive(Default)]
pub struct Review {
    pub panel: bool,
    pub options: Options,
    pub job: Option<Job>,
    pub error: Option<String>,
    pub cache_error: Option<String>,
    pub cache_blocked: bool,
}
impl Review {
    pub fn owns_engine(&self) -> bool {
        self.job.as_ref().is_some_and(Job::owns_engine)
    }
}

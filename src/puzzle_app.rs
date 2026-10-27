//! Puzzle-owned input target and bounded worker. Never calls App::apply_move.
use crate::{
    App, Duration, Instant, Move, Position, Role, Screen, Square,
    puzzle_data::{self, Validated},
    puzzle_session::{Outcome, Session, State},
    puzzle_store::{self, Progress},
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enter,
    Back,
    Next,
    Retry,
    Hint,
    Reveal,
    Previous,
    Forward,
    Theme,
    Difficulty,
    Mode,
    Import,
    Cancel,
    Themes,
    Details,
}
pub const THEMES: [&str; 13] = [
    "",
    "mateIn1",
    "mateIn2",
    "mateIn3",
    "fork",
    "pin",
    "skewer",
    "discoveredAttack",
    "backRankMate",
    "hangingPiece",
    "sacrifice",
    "promotion",
    "endgame",
];
pub const RANGES: [(u16, u16); 5] = [
    (0, 4000),
    (0, 1199),
    (1200, 1599),
    (1600, 1999),
    (2000, 4000),
];
type WorkerResult = Result<puzzle_store::Imported, String>;
struct Task {
    join: JoinHandle<WorkerResult>,
    cancel: Arc<AtomicBool>,
}
pub struct Puzzles {
    pub bank: Vec<Arc<Validated>>,
    pub session: Option<Session>,
    pub progress: Progress,
    pub totals: [u64; 5],
    pub completed: usize,
    pub root: Option<PathBuf>,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub selected: Option<Square>,
    pub dragging: Option<Square>,
    pub promotion: Option<(Square, Square)>,
    pub theme: usize,
    pub difficulty: usize,
    pub mode: usize,
    pub show_themes: bool,
    pub visited: std::collections::BTreeSet<String>,
    pub work: Arc<AtomicUsize>,
    pub started: Instant,
    task: Option<Task>,
    pub progress_error: bool,
}
impl Default for Puzzles {
    fn default() -> Self {
        Self {
            bank: vec![],
            session: None,
            progress: Progress::default(),
            totals: [0; 5],
            completed: 0,
            root: None,
            error: None,
            notice: None,
            selected: None,
            dragging: None,
            promotion: None,
            theme: 0,
            difficulty: 0,
            mode: 0,
            show_themes: false,
            visited: Default::default(),
            work: Arc::new(AtomicUsize::new(0)),
            started: Instant::now(),
            task: None,
            progress_error: false,
        }
    }
}
pub fn target(mv: &Move) -> Square {
    crate::uci_protocol::move_arrow(mv)
        .map(|(_, to)| to)
        .unwrap_or_else(|| mv.to())
}

impl Puzzles {
    pub fn refresh_totals(&mut self) {
        self.totals = [0; 5];
        self.completed = 0;
        for e in self.progress.entries.values() {
            self.completed += usize::from(e.first > 0 || e.assisted > 0);
            for (slot, value) in self
                .totals
                .iter_mut()
                .zip([e.attempts, e.first, e.assisted, e.revealed, e.skipped])
            {
                *slot += u64::from(value);
            }
        }
    }
    pub fn busy(&self) -> bool {
        self.task.is_some()
    }
    pub fn select_next(&mut self, now: Instant) {
        let (lo, hi) = RANGES[self.difficulty];
        let theme = THEMES[self.theme];
        let mut choices: Vec<_> = self
            .bank
            .iter()
            .filter(|p| {
                p.data.rating >= lo
                    && p.data.rating <= hi
                    && (theme.is_empty() || p.data.themes.iter().any(|s| s == theme))
                    && (self.mode != 2
                        || self
                            .progress
                            .entries
                            .get(&p.data.id)
                            .is_some_and(|e| e.errors > 0 || e.revealed > 0))
            })
            .cloned()
            .collect();
        choices.sort_by_key(|p| {
            (
                if self.mode == 1 {
                    self.progress.entries.contains_key(&p.data.id)
                } else {
                    false
                },
                p.data.id.clone(),
            )
        });
        if choices.is_empty() {
            self.session = None;
            return;
        }
        if choices.iter().all(|p| self.visited.contains(&p.data.id)) {
            self.visited.clear();
        }
        let current = self.session.as_ref().map(|s| s.puzzle.data.id.clone());
        let pick = choices
            .iter()
            .find(|p| !self.visited.contains(&p.data.id) && current.as_ref() != Some(&p.data.id))
            .or_else(|| choices.iter().find(|p| !self.visited.contains(&p.data.id)))
            .unwrap();
        self.visited.insert(pick.data.id.clone());
        self.session = Some(Session::new(pick.clone(), now));
        self.selected = None;
        self.dragging = None;
        self.promotion = None;
        self.notice = None;
    }
    pub fn shutdown(&mut self, timeout: Duration) -> bool {
        let Some(task) = self.task.take() else {
            return true;
        };
        task.cancel.store(true, Ordering::Relaxed);
        let until = Instant::now() + timeout;
        while !task.join.is_finished() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !task.join.is_finished() {
            self.task = Some(task);
            return false;
        }
        let _ = task.join.join();
        true
    }
}
impl Drop for Puzzles {
    fn drop(&mut self) {
        if let Some(t) = &self.task {
            t.cancel.store(true, Ordering::Relaxed);
        }
    }
}
impl App {
    pub(crate) fn puzzle_root(&self) -> Result<PathBuf, String> {
        if let Some(r) = &self.puzzles.root {
            return Ok(r.clone());
        }
        Ok(self
            .library_root()?
            .parent()
            .ok_or("Puzzle data root missing")?
            .to_path_buf())
    }
    pub(crate) fn enter_puzzles(&mut self) -> Result<(), String> {
        if self.defer_replay_loss(crate::advanced_app::Intent::PuzzlePage) {
            return Ok(());
        }
        self.tick_clock();
        self.cancel_analysis();
        self.cancel_review();
        self.analysis.panel = false;
        self.review.panel = false;
        let root = self.puzzle_root()?;
        self.puzzles.root = Some(root.clone());
        match puzzle_store::progress(&root.join("puzzle-progress-v1.json")) {
            Ok(p) => {
                self.puzzles.progress = p;
                self.puzzles.refresh_totals();
                self.puzzles.progress_error = false;
            }
            Err(e) => {
                self.puzzles.error = Some(e);
                self.puzzles.progress_error = true;
            }
        }
        self.puzzles.started = Instant::now();
        self.screen = Screen::Puzzle;
        if self
            .puzzles
            .session
            .as_ref()
            .is_some_and(|s| s.state == State::Skipped)
        {
            self.puzzles.select_next(Instant::now());
        }
        if self.puzzles.bank.is_empty() && !self.puzzles.busy() {
            self.start_puzzle_load(None)?;
        }
        Ok(())
    }
    pub(crate) fn start_puzzle_load(&mut self, source: Option<PathBuf>) -> Result<(), String> {
        if self.puzzles.busy() {
            return Err("Puzzle worker already running".into());
        }
        let root = self.puzzle_root()?;
        self.puzzles.notice = None;
        if !self.puzzles.progress_error {
            self.puzzles.error = None;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let progress = self.puzzles.work.clone();
        progress.store(0, Ordering::Relaxed);
        let join = std::thread::spawn(move || {
            if let Some(source) = source {
                puzzle_store::import(&root, &source, &flag, &progress)
            } else {
                let mut bank = puzzle_data::builtin()?;
                match puzzle_store::load_pack(&root, &flag, &progress) {
                    Ok(extra) => bank.extend(extra),
                    Err(e) => return Err(e),
                };
                Ok(puzzle_store::Imported {
                    bank,
                    added: 0,
                    duplicates: 0,
                })
            }
        });
        self.puzzles.task = Some(Task { join, cancel });
        Ok(())
    }
    pub(crate) fn poll_puzzles(&mut self) {
        if self
            .puzzles
            .task
            .as_ref()
            .is_some_and(|t| t.join.is_finished())
        {
            let task = self.puzzles.task.take().unwrap();
            match task.join.join() {
                Ok(Ok(out)) => {
                    self.puzzles.bank = out.bank;
                    if self.puzzles.session.is_none() {
                        self.puzzles.select_next(Instant::now());
                    }
                    self.puzzles.notice =
                        Some(format!("+{} / ={} / ×0", out.added, out.duplicates));
                }
                Ok(Err(e)) => {
                    self.puzzles.error = Some(e);
                    if self.puzzles.bank.is_empty() {
                        match puzzle_data::builtin() {
                            Ok(b) => self.puzzles.bank = b,
                            Err(e) => self.puzzles.error = Some(e),
                        };
                        self.puzzles.select_next(Instant::now());
                    }
                }
                Err(_) => self.puzzles.error = Some("Puzzle worker failed".into()),
            }
        }
        if self.screen == Screen::Puzzle
            && let Some(s) = &mut self.puzzles.session
        {
            s.respond();
        }
        if self.screen == Screen::Puzzle {
            self.record_puzzle();
        }
    }
    pub(crate) fn record_puzzle(&mut self) {
        let Some(s) = &self.puzzles.session else {
            return;
        };
        let Some(outcome) = s.outcome else { return };
        if s.recorded || self.puzzles.progress_error {
            return;
        }
        let mut p = self.puzzles.progress.clone();
        let e = p.entries.entry(s.puzzle.data.id.clone()).or_default();
        e.attempts = e.attempts.saturating_add(1);
        match outcome {
            Outcome::FirstTry => e.first = e.first.saturating_add(1),
            Outcome::Assisted => e.assisted = e.assisted.saturating_add(1),
            Outcome::Revealed => e.revealed = e.revealed.saturating_add(1),
            Outcome::Skipped => e.skipped = e.skipped.saturating_add(1),
        };
        e.hints = e.hints.saturating_add(u32::from(s.hints));
        e.errors = e.errors.saturating_add(s.errors);
        e.seconds = e.seconds.saturating_add(s.elapsed.as_secs());
        e.last = Some(outcome);
        e.last_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let result = self
            .puzzle_root()
            .and_then(|r| puzzle_store::save_progress(&r.join("puzzle-progress-v1.json"), &p));
        match result {
            Ok(()) => {
                self.puzzles.progress = p;
                self.puzzles.refresh_totals();
                self.puzzles.session.as_mut().unwrap().recorded = true;
            }
            Err(e) => {
                self.puzzles.error = Some(e);
                self.puzzles.progress_error = true;
            }
        }
    }
    pub(crate) fn finish_puzzle(&mut self) {
        if let Some(s) = &mut self.puzzles.session {
            s.skip(Instant::now());
        }
        self.record_puzzle();
    }
    pub(crate) fn handle_puzzle_action(&mut self, a: Action) {
        let result = (|| -> Result<(), String> {
            if a == Action::Enter {
                return self.enter_puzzles();
            }
            if self.screen != Screen::Puzzle {
                return Ok(());
            }
            match a {
                Action::Details => {
                    self.records.message = Some(format!(
                        "{}\n{}\n{}\n{}",
                        self.puzzles.error.as_deref().unwrap_or(""),
                        self.puzzles.notice.as_deref().unwrap_or(""),
                        crate::puzzle_ui::words(self.settings.language)[35],
                        self.puzzles
                            .session
                            .as_ref()
                            .filter(|s| s.state == State::Answer)
                            .map(|s| s.puzzle.sans.join(" "))
                            .unwrap_or_default()
                    ));
                }
                Action::Enter => {}
                Action::Back => {
                    self.finish_puzzle();
                    self.screen = Screen::Menu;
                    self.clock_last_tick = Instant::now();
                }
                Action::Next => {
                    self.finish_puzzle();
                    self.puzzles.select_next(Instant::now());
                }
                Action::Retry => {
                    self.finish_puzzle();
                    if let Some(s) = &self.puzzles.session {
                        self.puzzles.session = Some(Session::new(s.puzzle.clone(), Instant::now()));
                    }
                    self.puzzles.selected = None;
                    self.puzzles.promotion = None;
                    self.puzzles.dragging = None;
                }
                Action::Hint => {
                    if let Some(s) = &mut self.puzzles.session {
                        s.hint();
                    }
                }
                Action::Reveal => {
                    if let Some(s) = &mut self.puzzles.session {
                        s.reveal(Instant::now());
                    }
                    self.record_puzzle();
                    self.puzzles.selected = None;
                    self.puzzles.promotion = None;
                }
                Action::Previous | Action::Forward => {
                    if let Some(s) = &mut self.puzzles.session {
                        s.answer_step(if a == Action::Previous { -1 } else { 1 });
                    }
                }
                Action::Theme | Action::Difficulty | Action::Mode => {
                    self.finish_puzzle();
                    match a {
                        Action::Theme => {
                            self.puzzles.theme = (self.puzzles.theme + 1) % THEMES.len()
                        }
                        Action::Difficulty => {
                            self.puzzles.difficulty = (self.puzzles.difficulty + 1) % RANGES.len()
                        }
                        _ => self.puzzles.mode = (self.puzzles.mode + 1) % 3,
                    };
                    self.puzzles.visited.clear();
                    self.puzzles.select_next(Instant::now());
                }
                Action::Import => {
                    if !self.puzzles.busy()
                        && let Some(path) = crate::native_files::choose_csv(self.records.owner)?
                    {
                        self.start_puzzle_load(Some(path))?;
                    }
                }
                Action::Cancel => {
                    if let Some(t) = &self.puzzles.task {
                        t.cancel.store(true, Ordering::Relaxed);
                    }
                }
                Action::Themes => self.puzzles.show_themes = !self.puzzles.show_themes,
            }
            Ok(())
        })();
        if let Err(e) = result {
            if self.screen != Screen::Puzzle {
                self.records.message = Some(e.clone());
            }
            self.puzzles.error = Some(e);
        }
    }
    pub(crate) fn puzzle_move(&mut self, mv: Move) {
        if let Some(s) = &mut self.puzzles.session {
            match s.attempt(mv, Instant::now()) {
                Ok(true) => self.puzzles.notice = Some("✓".into()),
                Ok(false) => {
                    self.puzzles.notice =
                        Some(crate::puzzle_ui::words(self.settings.language)[20].into())
                }
                Err(_) => {
                    self.puzzles.notice =
                        Some(crate::puzzle_ui::words(self.settings.language)[21].into())
                }
            };
        }
        self.puzzles.selected = None;
        self.puzzles.dragging = None;
        self.puzzles.promotion = None;
        self.record_puzzle();
    }
    fn puzzle_to(&mut self, from: Square, to: Square) {
        let Some(s) = &self.puzzles.session else {
            return;
        };
        if s.state != State::Waiting {
            return;
        }
        let moves: Vec<_> = s
            .pos
            .legal_moves()
            .into_iter()
            .filter(|m| m.from() == Some(from) && target(m) == to)
            .collect();
        if moves.iter().any(|m| m.promotion().is_some()) {
            self.puzzles.promotion = Some((from, to));
            return;
        }
        if let Some(mv) = moves.first() {
            self.puzzle_move(mv.clone());
        } else {
            self.puzzles.notice = Some(crate::puzzle_ui::words(self.settings.language)[21].into());
            self.puzzles.selected = None;
            self.puzzles.dragging = None;
        }
    }
    pub(crate) fn puzzle_down(&mut self, sq: Square) {
        let Some(s) = &self.puzzles.session else {
            return;
        };
        if s.state != State::Waiting || self.puzzles.promotion.is_some() {
            return;
        }
        if let Some(from) = self.puzzles.selected
            && from != sq
            && s.pos
                .board()
                .piece_at(sq)
                .is_none_or(|p| p.color != s.solver())
        {
            self.puzzle_to(from, sq);
            return;
        }
        if s.pos
            .board()
            .piece_at(sq)
            .is_some_and(|p| p.color == s.solver())
        {
            self.puzzles.selected = Some(sq);
            self.puzzles.dragging = Some(sq);
        }
    }
    pub(crate) fn puzzle_up(&mut self, sq: Option<Square>) {
        if let Some(from) = self.puzzles.dragging.take()
            && let Some(to) = sq
            && from != to
        {
            self.puzzle_to(from, to);
        }
    }
    pub(crate) fn puzzle_promote(&mut self, role: Role) {
        if let Some((from, to)) = self.puzzles.promotion
            && let Some(s) = &self.puzzles.session
            && let Some(mv) =
                s.pos.legal_moves().iter().find(|m| {
                    m.from() == Some(from) && target(m) == to && m.promotion() == Some(role)
                })
        {
            self.puzzle_move(mv.clone());
        }
    }
    pub(crate) fn puzzle_view(&self, v: &mut crate::render::ViewState) {
        if self.screen != Screen::Puzzle {
            return;
        }
        v.analysis = Default::default();
        v.animations.clear();
        v.history_sans.clear();
        v.settings.board.zen_mode = crate::preferences::ZenMode::No;
        v.settings.board.immersive_mode = false;
        v.pos = self
            .puzzles
            .session
            .as_ref()
            .map(|s| s.display().clone())
            .unwrap_or_default();
        v.settings.flip_for_black = true;
        v.settings.mode = self.puzzles.session.as_ref().map(|s| !s.solver());
        v.selected = self
            .puzzles
            .selected
            .or_else(|| self.puzzles.session.as_ref().and_then(|s| s.hint_square));
        v.dragging_from = self.puzzles.dragging;
        v.legal_targets = self
            .puzzles
            .selected
            .map(|sq| {
                v.pos
                    .legal_moves()
                    .iter()
                    .filter(|m| m.from() == Some(sq))
                    .map(target)
                    .collect()
            })
            .unwrap_or_default();
        v.last_move = self.puzzles.session.as_ref().and_then(|s| {
            let mv = if s.state == State::Answer {
                &s.puzzle.moves[s.answer_cursor - 1]
            } else {
                s.played.last().unwrap_or(&s.puzzle.moves[0])
            };
            mv.from().map(|from| (from, target(mv)))
        });
        v.promotion = self.puzzles.promotion;
        v.move_confirmation_pending = false;
        v.result = None;
        v.game_over_progress = 0.0;
        v.ai_thinking = false;
        v.hint_thinking = false;
        v.suggestion = None;
        v.search_failed = false;
    }
}

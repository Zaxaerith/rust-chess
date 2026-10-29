use crate::{
    App, Chess, Instant, Screen, fen,
    game::{ClockState, position_key},
    game_record::{GameRecord, ValidatedRecord},
    persistence::{self, Snapshot},
    replay::Replay,
};
use std::path::PathBuf;

pub struct RecordsState {
    pub library_id: Option<String>,
    pub initial: Chess,
    pub metadata: GameRecord,
    pub replay: Option<Replay>,
    pub replay_return: Screen,
    pub fen_input: Option<String>,
    pub fen_cursor: usize,
    pub save_error: Option<String>,
    pub message: Option<String>,
    pub resume: Option<Snapshot>,
    pub resume_prompt: bool,
    pub save_path: Option<PathBuf>,
    pub save_last: Instant,
    pub has_game: bool,
    pub owner: usize,
}
impl Default for RecordsState {
    fn default() -> Self {
        let initial = Chess::default();
        Self {
            library_id: None,
            metadata: GameRecord::new(&initial, None),
            initial,
            replay: None,
            replay_return: Screen::Menu,
            fen_input: None,
            fen_cursor: 0,
            save_error: None,
            message: None,
            resume: None,
            resume_prompt: false,
            save_path: None,
            save_last: Instant::now(),
            has_game: false,
            owner: 0,
        }
    }
}

impl App {
    pub(crate) fn record(&self) -> GameRecord {
        GameRecord::capture(
            &self.records.initial,
            &self.history,
            self.records.metadata.clone(),
            self.result,
        )
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            schema_version: 1,
            record: self.record(),
            current_fen: fen::write(&self.pos),
            clocks: self.clocks,
            clock_history: self.clock_history.clone(),
            ai_color: self.ai_mode,
            ai_depth: self.settings.ai_depth,
            board_preferences: self.settings.board,
            result: self.result,
            saved_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    pub(crate) fn init_persistence(&mut self) {
        let path = std::env::var_os("RUST_CHESS_DATA_DIR")
            .map(|p| PathBuf::from(p).join("autosave-v1.json"))
            .map(Ok)
            .unwrap_or_else(persistence::data_path);
        match path {
            Ok(path) => {
                match persistence::load(&path) {
                    Ok(Some(snapshot)) => {
                        if snapshot.result.is_none() {
                            self.records.resume = Some(snapshot);
                            self.records.resume_prompt = true;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => self.records.message = Some(e),
                }
                self.records.save_path = Some(path);
            }
            Err(e) => self.records.message = Some(e),
        }
    }
    pub(crate) fn save_session(&mut self) {
        if !self.records.has_game {
            return;
        }
        if let Some(path) = &self.records.save_path {
            match persistence::save(path, &self.snapshot()) {
                Ok(()) => {
                    self.records.save_error = None;
                }
                Err(e) => {
                    eprintln!("Autosave: {e}");
                    if self.records.save_error.as_ref() != Some(&e) {
                        self.records.message = Some(e.clone());
                    }
                    self.records.save_error = Some(e);
                }
            }
        }
        self.records.save_last = Instant::now();
    }
    pub(crate) fn periodic_save(&mut self) {
        if self.screen == Screen::Game
            && !self.game_over()
            && self.records.fen_input.is_none()
            && self.records.save_last.elapsed() >= std::time::Duration::from_secs(5)
        {
            self.save_session();
        }
    }
    pub(crate) fn restore_snapshot(&mut self, snapshot: Snapshot) -> Result<(), String> {
        let validated = snapshot.validate()?;
        if self.needs_replay_save() {
            self.defer_replay_loss(crate::advanced_app::Intent::Restore(Box::new(snapshot)));
            return Ok(());
        }
        self.analysis_context(false);
        let initial = validated.positions[0].clone();
        self.advanced = crate::advanced_app::AdvancedState::default();
        let pos = validated
            .positions
            .last()
            .expect("initial position")
            .clone();
        // All validation completes before touching the live session.
        self.invalidate_searches();
        self.clear_input();
        self.animations.clear();
        self.records.initial = initial;
        self.records.metadata = snapshot.record;
        self.pos = pos;
        self.history = validated.moves;
        self.history_sans = validated.sans;
        self.position_keys = validated.positions.iter().map(position_key).collect();
        self.clocks = snapshot.clocks;
        self.clock_history = snapshot.clock_history;
        self.ai_mode = snapshot.ai_color;
        self.settings.mode = snapshot.ai_color;
        self.settings.ai_depth = snapshot.ai_depth;
        self.settings.board = snapshot.board_preferences;
        self.result = snapshot.result;
        self.game_over_at = self.result.map(|_| Instant::now());
        self.last_move = self
            .history
            .last()
            .and_then(|m| m.from().map(|f| (f, m.to())));
        self.clock_last_tick = Instant::now();
        self.records.has_game = true;
        self.records.replay = None;
        self.records.library_id = None;
        self.records.resume = None;
        self.records.resume_prompt = false;
        self.screen = Screen::Game;
        self.maybe_spawn_ai();
        Ok(())
    }
    pub(crate) fn import_fen(&mut self, text: &str) -> Result<(), String> {
        let pos = fen::parse(text)?;
        if self.defer_replay_loss(crate::advanced_app::Intent::Fen(text.to_owned())) {
            return Ok(());
        }
        self.advanced = crate::advanced_app::AdvancedState::default();
        self.analysis_context(false);
        self.invalidate_searches();
        self.clear_input();
        self.animations.clear();
        self.ai_mode = self.settings.mode;
        self.pos = pos.clone();
        self.records.initial = pos.clone();
        self.records.metadata = GameRecord::new(&pos, self.ai_mode);
        self.history.clear();
        self.history_sans.clear();
        self.position_keys = vec![position_key(&pos)];
        self.clocks = ClockState::default();
        self.clock_history = vec![self.clocks];
        self.clock_last_tick = Instant::now();
        self.result = crate::game::GameResult::from_position(&self.pos, 0);
        self.game_over_at = self.result.map(|_| Instant::now());
        self.last_move = None;
        self.screen = Screen::Game;
        self.records.replay = None;
        self.records.library_id = None;
        self.records.has_game = true;
        self.records.fen_input = None;
        self.records.resume_prompt = false;
        self.records.resume = None;
        self.save_session();
        self.maybe_spawn_ai();
        Ok(())
    }
    pub(crate) fn enter_replay(&mut self, game: ValidatedRecord, external: bool) {
        self.open_annotated_replay(
            crate::annotated_game::AnnotatedGame::from_linear(game),
            external,
        );
    }

    pub(crate) fn leave_replay(&mut self) {
        if self.defer_replay_loss(crate::advanced_app::Intent::Back) {
            return;
        }
        self.cancel_review();
        self.review.panel = false;
        self.cancel_analysis();
        self.records.replay = None;
        self.records.library_id = None;
        self.advanced = crate::advanced_app::AdvancedState::default();
        self.screen = self.records.replay_return;
        self.clock_last_tick = Instant::now();
        // Retain existing receivers and generation. A queued reply is polled only
        // after returning to the real live position.
    }
    pub(crate) fn display_fen(&self) -> String {
        if self.screen == Screen::PositionWorkspace {
            return self.workspace.text();
        }
        if let Some(r) = &self.records.replay {
            fen::write(r.position())
        } else {
            fen::write(&self.pos)
        }
    }
    pub(crate) fn export_pgn_dialog(&mut self) -> Result<(), String> {
        if self.records.replay.is_some() {
            self.export_annotated_dialog()?;
            return Ok(());
        }
        let record = self
            .records
            .replay
            .as_ref()
            .map(|r| r.game.record.clone())
            .unwrap_or_else(|| self.record());
        let text = crate::pgn::export(&record)?;
        let name = format!("rust-chess-{}.pgn", record.created_unix);
        let path = crate::native_files::choose_pgn(true, self.records.owner, &name);
        self.clock_last_tick = Instant::now();
        self.advanced.last_tick = Instant::now();
        if let Some(path) = path? {
            persistence::atomic_write(&path, text.as_bytes())?;
        }
        Ok(())
    }
    pub(crate) fn import_pgn_dialog(&mut self) -> Result<(), String> {
        let path = crate::native_files::choose_pgn(false, self.records.owner, "");
        self.clock_last_tick = Instant::now();
        self.advanced.last_tick = Instant::now();
        if let Some(path) = path? {
            self.import_annotated_bytes(&persistence::read_limited(&path)?)?;
        }
        Ok(())
    }
    pub(crate) fn reset_record(&mut self) {
        self.advanced = crate::advanced_app::AdvancedState::default();
        self.analysis_context(false);
        self.records.initial = Chess::default();
        self.records.metadata = GameRecord::new(&self.records.initial, self.ai_mode);
        self.records.replay = None;
        self.records.library_id = None;
        self.records.resume = None;
        self.records.resume_prompt = false;
        self.records.has_game = true;
    }
}

#![windows_subsystem = "windows"]

mod ai;
mod assets;
mod display;
mod font;
mod i18n;
mod preferences;
mod render;
mod settings;
mod theme;

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use shakmaty::san::San;
use shakmaty::{Chess, Color, File, Move, Position, Role, Square};

use assets::PieceSet;
use preferences::{
    AccessPolicy, AutoPromotion, AutoThreefold, CastlingMethod, ClockPosition, ClockTenths,
    DragTarget, PieceNotation, ZenMode,
};
use render::{
    DropdownKind, PieceAnimView, PreferenceItem, Renderer, Screen, Settings, SettingsPage,
    UiAction, ViewState,
};
use theme::BoardStyle;

struct PieceAnim {
    color: Color,
    role: Role,
    from: Square,
    to: Square,
    start: Instant,
    duration: f32,
}

impl PieceAnim {
    fn new(piece: shakmaty::Piece, from: Square, to: Square, start: Instant) -> Self {
        Self {
            color: piece.color,
            role: piece.role,
            from,
            to,
            start,
            duration: 0.22,
        }
    }

    fn progress(&self, now: Instant) -> f32 {
        let p = (now.duration_since(self.start).as_secs_f32() / self.duration).clamp(0.0, 1.0);
        p * p * (3.0 - 2.0 * p)
    }

    fn done(&self, now: Instant) -> bool {
        now.duration_since(self.start).as_secs_f32() >= self.duration
    }

    fn view(&self, now: Instant) -> PieceAnimView {
        PieceAnimView {
            color: self.color,
            role: self.role,
            from: self.from,
            to: self.to,
            progress: self.progress(now),
        }
    }
}

struct App {
    pos: Chess,
    history: Vec<Move>,
    history_sans: Vec<String>,
    position_keys: Vec<String>,
    claimed_draw: bool,
    selected: Option<Square>,
    dragging_from: Option<Square>,
    premove_from: Option<Square>,
    premove: Option<(Square, Square)>,
    last_move: Option<(Square, Square)>,
    ai_mode: Option<Color>,
    ai_rx: Option<Receiver<Move>>,
    ai_thinking: bool,
    hint_rx: Option<Receiver<Move>>,
    hint_thinking: bool,
    suggestion: Option<(Square, Square, String)>,
    promotion: Option<(Square, Square)>,
    pending_move: Option<Move>,
    screen: Screen,
    settings: Settings,
    exit_requested: bool,
    window_resize_requested: Option<(u32, u32)>,
    window_title_update_requested: bool,
    animations: Vec<PieceAnim>,
    ai_reply_at: Option<Instant>,
    resolutions: Vec<(u32, u32)>,
    refreshes: Vec<u32>,
    open_dropdown: Option<DropdownKind>,
    dropdown_scroll: usize,
    settings_page: SettingsPage,
    settings_scroll: usize,
    game_over_at: Option<Instant>,
    started_at: Instant,
    white_clock: f32,
    black_clock: f32,
    clock_last_tick: Instant,
    clock_warning_played: [bool; 2],
}

impl App {
    fn new() -> Self {
        let mut loaded = settings::load().unwrap_or_default();
        let modes = display::display_modes();
        let resolutions = display::unique_resolutions(&modes);
        let refreshes = display::unique_refreshes(&modes);
        if !resolutions.contains(&loaded.resolution) {
            if let Some(desktop) = display::desktop_resolution() {
                loaded.resolution = desktop;
            }
            if !resolutions.contains(&loaded.resolution) {
                loaded.resolution = resolutions[0];
            }
        }
        if !refreshes.contains(&loaded.fps) {
            loaded.fps = 60;
        }
        Self {
            pos: Chess::default(),
            history: Vec::new(),
            history_sans: Vec::new(),
            position_keys: vec![position_key(&Chess::default())],
            claimed_draw: false,
            selected: None,
            dragging_from: None,
            premove_from: None,
            premove: None,
            last_move: None,
            ai_mode: None,
            ai_rx: None,
            ai_thinking: false,
            hint_rx: None,
            hint_thinking: false,
            suggestion: None,
            promotion: None,
            pending_move: None,
            screen: Screen::Menu,
            settings: loaded,
            exit_requested: false,
            window_resize_requested: None,
            window_title_update_requested: false,
            animations: Vec::new(),
            ai_reply_at: None,
            resolutions,
            refreshes,
            open_dropdown: None,
            dropdown_scroll: 0,
            settings_page: SettingsPage::Root,
            settings_scroll: 0,
            game_over_at: None,
            started_at: Instant::now(),
            white_clock: 600.0,
            black_clock: 600.0,
            clock_last_tick: Instant::now(),
            clock_warning_played: [false; 2],
        }
    }

    fn game_over(&self) -> bool {
        self.pos.outcome().is_some() || self.claimed_draw
    }

    fn tick_clock(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.clock_last_tick).as_secs_f32();
        self.clock_last_tick = now;
        if self.screen != Screen::Game
            || !self.settings.board.chess_clock_enabled
            || self.game_over()
            || self.promotion.is_some()
            || self.pending_move.is_some()
        {
            return;
        }
        let (clock, warning_index) = match self.pos.turn() {
            Color::White => (&mut self.white_clock, 0),
            Color::Black => (&mut self.black_clock, 1),
        };
        let before = *clock;
        *clock = (*clock - elapsed).max(0.0);
        if self.settings.board.clock_warning
            && before > 30.0
            && *clock <= 30.0
            && !self.clock_warning_played[warning_index]
        {
            self.clock_warning_played[warning_index] = true;
            play_clock_warning();
        }
    }

    fn poll_ai(&mut self) {
        let now = Instant::now();
        self.animations.retain(|a| !a.done(now));
        let Some(reply_at) = self.ai_reply_at else {
            return;
        };
        if now < reply_at {
            return;
        }
        if self.ai_rx.is_none() {
            self.spawn_ai();
            return;
        }
        let rx = self.ai_rx.take().expect("ai_rx checked");
        match rx.try_recv() {
            Ok(mv) => {
                self.ai_thinking = false;
                self.ai_reply_at = None;
                self.apply_move(mv);
                self.try_apply_premove();
                self.maybe_spawn_ai();
            }
            Err(TryRecvError::Empty) => self.ai_rx = Some(rx),
            Err(TryRecvError::Disconnected) => {
                self.ai_thinking = false;
                self.ai_reply_at = None;
            }
        }
    }

    fn poll_hint(&mut self) {
        let Some(rx) = self.hint_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(mv) => {
                self.hint_thinking = false;
                let san = San::from_move(&self.pos, &mv).to_string();
                if let Some(from) = mv.from() {
                    self.suggestion = Some((from, mv.to(), san));
                }
            }
            Err(TryRecvError::Empty) => self.hint_rx = Some(rx),
            Err(TryRecvError::Disconnected) => self.hint_thinking = false,
        }
    }

    fn request_hint(&mut self) {
        if self.game_over() || self.ai_thinking || self.hint_thinking || self.promotion.is_some() {
            return;
        }
        self.suggestion = None;
        let pos = self.pos.clone();
        let depth = self.settings.ai_depth.max(3);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mv = ai::best_move(&pos, depth);
            let _ = tx.send(mv);
        });
        self.hint_rx = Some(rx);
        self.hint_thinking = true;
    }

    fn spawn_ai(&mut self) {
        if self.game_over() || self.ai_rx.is_some() {
            return;
        }
        let depth = self.settings.ai_depth;
        let pos = self.pos.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mv = ai::best_move(&pos, depth);
            let _ = tx.send(mv);
        });
        self.ai_rx = Some(rx);
        self.ai_thinking = true;
    }

    fn maybe_spawn_ai(&mut self) {
        if let Some(color) = self.ai_mode {
            if !self.game_over() && self.pos.turn() == color {
                let base = match self.settings.ai_depth {
                    1 => 350,
                    2 => 500,
                    3 => 700,
                    4 => 900,
                    _ => 1200,
                };
                let jitter = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos()
                    % 250;
                self.ai_rx = None;
                self.ai_thinking = true;
                self.ai_reply_at =
                    Some(Instant::now() + Duration::from_millis(base + jitter as u64));
            }
        }
    }

    fn apply_move(&mut self, mv: Move) {
        self.hint_rx = None;
        self.hint_thinking = false;
        self.suggestion = None;
        let san = San::from_move(&self.pos, &mv).to_string();
        let now = Instant::now();
        let mut anims = Vec::new();
        match &mv {
            Move::Normal {
                role: _, from, to, ..
            } => {
                if let Some(piece) = self.pos.board().piece_at(*from) {
                    anims.push(PieceAnim::new(piece, *from, *to, now));
                }
            }
            Move::EnPassant { from, to } => {
                if let Some(piece) = self.pos.board().piece_at(*from) {
                    anims.push(PieceAnim::new(piece, *from, *to, now));
                }
            }
            Move::Castle { king, rook } => {
                let king_to = Square::from_coords(
                    if king.file() == File::E {
                        File::G
                    } else {
                        File::C
                    },
                    king.rank(),
                );
                let rook_to = Square::from_coords(
                    if rook.file() == File::H {
                        File::F
                    } else {
                        File::D
                    },
                    rook.rank(),
                );
                if let Some(piece) = self.pos.board().piece_at(*king) {
                    anims.push(PieceAnim::new(piece, *king, king_to, now));
                }
                if let Some(piece) = self.pos.board().piece_at(*rook) {
                    anims.push(PieceAnim::new(piece, *rook, rook_to, now));
                }
            }
            Move::Put { .. } => {}
        }
        if let Ok(new_pos) = self.pos.clone().play(&mv) {
            self.last_move = Some((mv.from().expect("standard move has origin"), mv.to()));
            self.history.push(mv);
            self.history_sans.push(san);
            self.pos = new_pos;
            let key = position_key(&self.pos);
            self.position_keys.push(key.clone());
            let repetitions = self
                .position_keys
                .iter()
                .filter(|entry| **entry == key)
                .count();
            let should_claim = match self.settings.board.auto_threefold {
                AutoThreefold::Always => repetitions >= 3,
                AutoThreefold::Never => false,
                AutoThreefold::UnderThirtySeconds => {
                    repetitions >= 3 && self.white_clock.min(self.black_clock) < 30.0
                }
            };
            if should_claim {
                self.claimed_draw = true;
            }
            self.selected = None;
            self.promotion = None;
            if self.settings.board.piece_animation {
                self.animations.extend(anims);
            } else {
                self.animations.clear();
            }
            if self.game_over() && self.game_over_at.is_none() {
                self.game_over_at = Some(Instant::now());
            }
        }
    }

    fn handle_action(&mut self, action: UiAction) {
        match action {
            UiAction::StartGame => self.start_game(),
            UiAction::OpenSettings => self.open_settings(),
            UiAction::BackToMenu => self.back_to_menu(),
            UiAction::ExitGame => {
                settings::save(&self.settings);
                self.exit_requested = true;
            }
            UiAction::SetDifficulty(depth) => {
                self.settings.ai_depth = depth;
                self.open_dropdown = None;
                settings::save(&self.settings);
            }
            UiAction::SetResolution((w, h)) => {
                if self.settings.resolution != (w, h) {
                    self.settings.resolution = (w, h);
                    self.window_resize_requested = Some((w, h));
                }
                self.open_dropdown = None;
                settings::save(&self.settings);
            }
            UiAction::SetFps(fps) => {
                self.settings.fps = fps;
                self.open_dropdown = None;
                settings::save(&self.settings);
            }
            UiAction::SetTheme(theme) => {
                self.settings.theme = theme;
                self.open_dropdown = None;
                settings::save(&self.settings);
            }
            UiAction::SetLanguage(language) => {
                self.settings.language = language;
                self.open_dropdown = None;
                self.window_title_update_requested = true;
                settings::save(&self.settings);
            }
            UiAction::SetFlipForBlack(enabled) => {
                self.settings.flip_for_black = enabled;
                self.open_dropdown = None;
                settings::save(&self.settings);
            }
            UiAction::ToggleDropdown(kind) => {
                let opening = self.open_dropdown != Some(kind);
                self.open_dropdown = if opening { Some(kind) } else { None };
                if opening {
                    self.dropdown_scroll = 0;
                }
            }
            UiAction::OpenSettingsPage(page) => {
                self.settings_page = page;
                self.settings_scroll = 0;
                self.open_dropdown = None;
            }
            UiAction::SettingsBack => {
                self.settings_page = match self.settings_page {
                    SettingsPage::Root => {
                        self.back_to_menu();
                        SettingsPage::Root
                    }
                    SettingsPage::Board => SettingsPage::Root,
                    SettingsPage::Display | SettingsPage::Behavior | SettingsPage::Clock => {
                        SettingsPage::Board
                    }
                };
                self.settings_scroll = 0;
                self.open_dropdown = None;
            }
            UiAction::SetPreference(item, value) => {
                self.set_preference(item, value);
                self.open_dropdown = None;
                settings::save(&self.settings);
            }
            UiAction::NewGame => self.new_game(),
            UiAction::Undo => self.undo(),
            UiAction::Hint => self.request_hint(),
            UiAction::Mode(mode) => {
                self.settings.mode = mode;
                self.open_dropdown = None;
                settings::save(&self.settings);
                if self.screen != Screen::Game {
                    return;
                }
                self.ai_rx = None;
                self.ai_reply_at = None;
                self.ai_thinking = false;
                self.promotion = None;
                self.selected = None;
                self.hint_rx = None;
                self.hint_thinking = false;
                self.suggestion = None;
                self.ai_mode = mode;
                self.maybe_spawn_ai();
            }
            UiAction::PointerDown(sq) => self.handle_pointer_down(sq),
            UiAction::PointerUp(sq) => self.handle_pointer_up(sq),
            UiAction::ConfirmMove => {
                if let Some(mv) = self.pending_move.take() {
                    self.apply_move(mv);
                    self.maybe_spawn_ai();
                }
            }
            UiAction::CancelMove => {
                self.pending_move = None;
                self.selected = None;
            }
            UiAction::AddClockTime(color) => {
                if self.settings.board.give_more_time != AccessPolicy::Never {
                    match color {
                        Color::White => self.white_clock += 15.0,
                        Color::Black => self.black_clock += 15.0,
                    }
                }
            }
            UiAction::Promote(role) => self.handle_promotion(role),
        }
    }

    fn start_game(&mut self) {
        self.ai_mode = self.settings.mode;
        self.new_game();
        self.screen = Screen::Game;
    }

    fn open_settings(&mut self) {
        self.settings_page = SettingsPage::Root;
        self.settings_scroll = 0;
        self.screen = Screen::Settings;
    }

    fn back_to_menu(&mut self) {
        self.ai_rx = None;
        self.ai_reply_at = None;
        self.ai_thinking = false;
        self.hint_rx = None;
        self.hint_thinking = false;
        self.suggestion = None;
        self.game_over_at = None;
        self.dragging_from = None;
        self.premove_from = None;
        self.premove = None;
        self.pending_move = None;
        self.screen = Screen::Menu;
    }

    fn handle_pointer_down(&mut self, sq: Square) {
        if self.game_over() || self.promotion.is_some() {
            return;
        }
        if self.ai_thinking {
            if !self.settings.board.premoves {
                return;
            }
            if let Some(from) = self.premove_from.take() {
                if from != sq {
                    self.premove = Some((from, sq));
                } else {
                    self.premove = None;
                }
                self.selected = None;
                return;
            }
            if let Some(piece) = self.pos.board().piece_at(sq) {
                if piece.color != self.pos.turn() {
                    self.premove_from = Some(sq);
                    self.selected = Some(sq);
                }
            }
            return;
        }
        if let Some(piece) = self.pos.board().piece_at(sq) {
            if piece.color == self.pos.turn() {
                self.selected = Some(sq);
                self.dragging_from = Some(sq);
                return;
            }
        }
        self.dragging_from = None;
        self.handle_board_click(sq);
    }

    fn handle_pointer_up(&mut self, target: Option<Square>) {
        let Some(from) = self.dragging_from.take() else {
            return;
        };
        let Some(to) = target else {
            self.selected = Some(from);
            return;
        };
        if from != to {
            self.selected = Some(from);
            self.handle_board_click(to);
        }
    }

    fn try_apply_premove(&mut self) {
        let Some((from, to)) = self.premove.take() else {
            return;
        };
        self.premove_from = None;
        let moves: Vec<Move> = self
            .pos
            .legal_moves()
            .iter()
            .filter(|mv| mv.from() == Some(from) && mv.to() == to)
            .cloned()
            .collect();
        if moves.is_empty() {
            self.selected = None;
            return;
        }
        if moves.iter().any(|mv| mv.promotion().is_some())
            && self.settings.board.auto_promotion == AutoPromotion::Never
        {
            self.promotion = Some((from, to));
            self.selected = None;
            return;
        }
        let mv = if moves.iter().any(|mv| mv.promotion().is_some()) {
            moves
                .iter()
                .find(|mv| mv.promotion() == Some(Role::Queen))
                .cloned()
                .unwrap_or_else(|| moves[0].clone())
        } else {
            moves[0].clone()
        };
        self.apply_move(mv);
    }

    fn submit_player_move(&mut self, mv: Move) {
        if self.settings.board.move_confirmation {
            self.pending_move = Some(mv);
            self.selected = None;
            self.dragging_from = None;
        } else {
            self.apply_move(mv);
            self.maybe_spawn_ai();
        }
    }

    fn handle_board_click(&mut self, sq: Square) {
        if self.ai_thinking || self.game_over() || self.promotion.is_some() {
            return;
        }
        if let Some(sel) = self.selected {
            if sel == sq {
                self.selected = None;
                return;
            }
            if self.settings.board.castling_method == CastlingMethod::KingOntoRook
                && self.pos.board().role_at(sel) == Some(Role::King)
                && self.pos.board().role_at(sq) == Some(Role::Rook)
                && self.pos.board().color_at(sel) == self.pos.board().color_at(sq)
            {
                let castle = self.pos.legal_moves().iter().find_map(|mv| match mv {
                    Move::Castle { king, rook } if *king == sel && *rook == sq => Some(mv.clone()),
                    _ => None,
                });
                if let Some(mv) = castle {
                    self.submit_player_move(mv);
                    return;
                }
            }
            let moves: Vec<Move> = self
                .pos
                .legal_moves()
                .iter()
                .cloned()
                .filter(|m| m.from() == Some(sel) && m.to() == sq)
                .collect();
            if moves.is_empty() {
                if let Some(piece) = self.pos.board().piece_at(sq) {
                    if piece.color == self.pos.turn() {
                        self.selected = Some(sq);
                    }
                } else {
                    self.selected = None;
                }
                return;
            }
            if moves.iter().any(|m| m.promotion().is_some())
                && self.settings.board.auto_promotion == AutoPromotion::Always
            {
                if let Some(mv) = moves
                    .iter()
                    .find(|m| m.promotion() == Some(Role::Queen))
                    .cloned()
                {
                    self.submit_player_move(mv);
                }
            } else if moves.iter().any(|m| m.promotion().is_some()) {
                self.promotion = Some((sel, sq));
            } else {
                self.submit_player_move(moves[0].clone());
            }
        } else if let Some(piece) = self.pos.board().piece_at(sq) {
            if piece.color == self.pos.turn() {
                self.selected = Some(sq);
            }
        }
    }

    fn handle_promotion(&mut self, role: Role) {
        if let Some((from, to)) = self.promotion {
            let mv =
                self.pos.legal_moves().iter().cloned().find(|m| {
                    m.from() == Some(from) && m.to() == to && m.promotion() == Some(role)
                });
            if let Some(mv) = mv {
                self.submit_player_move(mv);
            }
        }
        self.promotion = None;
    }

    fn undo(&mut self) {
        if self.settings.board.takebacks == AccessPolicy::Never {
            return;
        }
        self.ai_rx = None;
        self.ai_reply_at = None;
        self.ai_thinking = false;
        self.hint_rx = None;
        self.hint_thinking = false;
        self.suggestion = None;
        self.promotion = None;
        self.selected = None;
        self.dragging_from = None;
        self.premove_from = None;
        self.premove = None;
        self.pending_move = None;
        if self.history.is_empty() {
            return;
        }
        self.history.pop();
        self.history_sans.pop();
        if let Some(ai_color) = self.ai_mode {
            while !self.history.is_empty() {
                let last_idx = self.history.len() - 1;
                let mover = if last_idx % 2 == 0 {
                    Color::White
                } else {
                    Color::Black
                };
                if mover != ai_color {
                    break;
                }
                self.history.pop();
                self.history_sans.pop();
            }
        }
        self.rebuild();
        self.game_over_at = None;
        self.maybe_spawn_ai();
    }

    fn new_game(&mut self) {
        self.pos = Chess::default();
        self.history.clear();
        self.history_sans.clear();
        self.position_keys.clear();
        self.position_keys.push(position_key(&self.pos));
        self.claimed_draw = false;
        self.selected = None;
        self.dragging_from = None;
        self.premove_from = None;
        self.premove = None;
        self.pending_move = None;
        self.last_move = None;
        self.white_clock = 600.0;
        self.black_clock = 600.0;
        self.clock_last_tick = Instant::now();
        self.clock_warning_played = [false; 2];
        self.promotion = None;
        self.ai_rx = None;
        self.ai_reply_at = None;
        self.ai_thinking = false;
        self.hint_rx = None;
        self.hint_thinking = false;
        self.suggestion = None;
        self.game_over_at = None;
        self.maybe_spawn_ai();
    }

    fn rebuild(&mut self) {
        let mut pos = Chess::default();
        self.position_keys.clear();
        self.position_keys.push(position_key(&pos));
        for mv in &self.history {
            pos = pos.play(mv).expect("棋谱回放失败");
            self.position_keys.push(position_key(&pos));
        }
        self.pos = pos;
        self.claimed_draw = false;
        self.last_move = self
            .history
            .last()
            .map(|m| (m.from().expect("standard move has origin"), m.to()));
    }

    fn view_state(
        &self,
        mouse: Option<(f32, f32)>,
        mouse_pressed: bool,
        mouse_down: bool,
        mouse_released: bool,
    ) -> ViewState {
        let now = Instant::now();
        let animations: Vec<PieceAnimView> = self
            .animations
            .iter()
            .filter(|a| !a.done(now))
            .map(|a| a.view(now))
            .collect();
        let game_over_progress = if self.game_over() {
            if let Some(start) = self.game_over_at {
                let raw = (now.duration_since(start).as_secs_f32() / 0.6).clamp(0.0, 1.0);
                raw * raw * (3.0 - 2.0 * raw)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let legal_targets = if let Some(sel) = self.selected {
            self.pos
                .legal_moves()
                .iter()
                .filter(|m| m.from() == Some(sel))
                .map(|m| m.to())
                .collect()
        } else {
            Vec::new()
        };
        ViewState {
            pos: self.pos.clone(),
            selected: self.selected,
            last_move: self.last_move,
            legal_targets,
            history_sans: self.history_sans.clone(),
            ai_thinking: self.ai_thinking,
            hint_thinking: self.hint_thinking,
            suggestion: self.suggestion.clone(),
            promotion: self.promotion,
            move_confirmation_pending: self.pending_move.is_some(),
            screen: self.screen,
            settings: self.settings,
            animations,
            resolutions: self.resolutions.clone(),
            refreshes: self.refreshes.clone(),
            open_dropdown: self.open_dropdown,
            dropdown_scroll: self.dropdown_scroll,
            settings_page: self.settings_page,
            settings_scroll: self.settings_scroll,
            game_over_progress,
            mouse,
            mouse_pressed,
            mouse_down,
            mouse_released,
            dragging_from: self.dragging_from,
            menu_time: now.duration_since(self.started_at).as_secs_f32(),
            white_clock: self.white_clock,
            black_clock: self.black_clock,
            claimed_draw: self.claimed_draw,
        }
    }

    fn handle_scroll(&mut self, y: f32) {
        let delta = (y / 3.0).round() as i32;
        if delta == 0 {
            return;
        }
        if self.open_dropdown.is_some() {
            let next = self.dropdown_scroll as i32 - delta;
            self.dropdown_scroll = next.max(0) as usize;
        } else if self.screen == Screen::Settings {
            let next = self.settings_scroll as i32 - delta;
            self.settings_scroll = next.max(0) as usize;
        }
    }

    fn set_preference(&mut self, item: PreferenceItem, value: usize) {
        let board = &mut self.settings.board;
        match item {
            PreferenceItem::BoardStyle => {
                if let Some(style) = BoardStyle::ALL.get(value) {
                    self.settings.board_style = *style;
                }
            }
            PreferenceItem::PieceSet => {
                if let Some(style) = PieceSet::ALL.get(value) {
                    self.settings.piece_set = *style;
                }
            }
            PreferenceItem::ZenMode => {
                board.zen_mode = [ZenMode::No, ZenMode::Yes, ZenMode::GameOnly]
                    .get(value)
                    .copied()
                    .unwrap_or(board.zen_mode)
            }
            PreferenceItem::PieceNotation => {
                board.piece_notation = [PieceNotation::Symbols, PieceNotation::Letters]
                    .get(value)
                    .copied()
                    .unwrap_or(board.piece_notation)
            }
            PreferenceItem::Coordinates => board.coordinates = value != 0,
            PreferenceItem::MagnifyDraggedPiece => board.magnify_dragged_piece = value != 0,
            PreferenceItem::DragTarget => {
                board.drag_target = [DragTarget::Circle, DragTarget::Square, DragTarget::None]
                    .get(value)
                    .copied()
                    .unwrap_or(board.drag_target)
            }
            PreferenceItem::PieceAnimation => board.piece_animation = value != 0,
            PreferenceItem::ImmersiveMode => board.immersive_mode = value != 0,
            PreferenceItem::PieceDestinations => board.piece_destinations = value != 0,
            PreferenceItem::BoardHighlights => board.board_highlights = value != 0,
            PreferenceItem::ShowMoveList => board.show_move_list = value != 0,
            PreferenceItem::ClockPosition => {
                board.clock_position = [ClockPosition::Left, ClockPosition::Right]
                    .get(value)
                    .copied()
                    .unwrap_or(board.clock_position)
            }
            PreferenceItem::Premoves => board.premoves = value != 0,
            PreferenceItem::Takebacks => {
                board.takebacks = [
                    AccessPolicy::Never,
                    AccessPolicy::Casual,
                    AccessPolicy::Always,
                ]
                .get(value)
                .copied()
                .unwrap_or(board.takebacks)
            }
            PreferenceItem::AutoPromotion => {
                board.auto_promotion = [
                    AutoPromotion::Never,
                    AutoPromotion::Premove,
                    AutoPromotion::Always,
                ]
                .get(value)
                .copied()
                .unwrap_or(board.auto_promotion)
            }
            PreferenceItem::AutoThreefold => {
                board.auto_threefold = [
                    AutoThreefold::Always,
                    AutoThreefold::Never,
                    AutoThreefold::UnderThirtySeconds,
                ]
                .get(value)
                .copied()
                .unwrap_or(board.auto_threefold)
            }
            PreferenceItem::MoveConfirmation => board.move_confirmation = value != 0,
            PreferenceItem::ConfirmResignDraw => board.confirm_resign_draw = value != 0,
            PreferenceItem::CastlingMethod => {
                board.castling_method =
                    [CastlingMethod::KingOntoRook, CastlingMethod::KingTwoSquares]
                        .get(value)
                        .copied()
                        .unwrap_or(board.castling_method)
            }
            PreferenceItem::ChessClockEnabled => board.chess_clock_enabled = value != 0,
            PreferenceItem::GiveMoreTime => {
                board.give_more_time = [
                    AccessPolicy::Never,
                    AccessPolicy::Casual,
                    AccessPolicy::Always,
                ]
                .get(value)
                .copied()
                .unwrap_or(board.give_more_time)
            }
            PreferenceItem::ClockWarning => board.clock_warning = value != 0,
            PreferenceItem::ClockTenths => {
                board.clock_tenths = [
                    ClockTenths::Never,
                    ClockTenths::UnderTenSeconds,
                    ClockTenths::Always,
                ]
                .get(value)
                .copied()
                .unwrap_or(board.clock_tenths)
            }
        }
    }
}

fn position_key(position: &Chess) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}",
        position.board(),
        position.turn(),
        position.castles(),
        position.maybe_ep_square()
    )
}

#[cfg(target_os = "windows")]
fn play_clock_warning() {
    unsafe {
        winapi::um::winuser::MessageBeep(0x0000_0030);
    }
}

#[cfg(not(target_os = "windows"))]
fn play_clock_warning() {}

#[cfg(target_os = "windows")]
fn resize_native_window(window: &Window, client_width: u32, client_height: u32) {
    use winapi::shared::windef::{HWND, RECT};
    use winapi::um::winuser::{
        AdjustWindowRectEx, GWL_EXSTYLE, GWL_STYLE, GetWindowLongW, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOZORDER, SetWindowPos,
    };

    let hwnd = window.get_window_handle() as HWND;
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: client_width as i32,
            bottom: client_height as i32,
        };
        if AdjustWindowRectEx(&mut rect, style, 0, ex_style) != 0 {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn set_native_window_icons(window: &Window) {
    use winapi::shared::windef::{HICON, HWND};
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winapi::um::winuser::{
        GetSystemMetrics, ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW,
        MAKEINTRESOURCEW, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, SendMessageW, WM_SETICON,
    };

    let hwnd = window.get_window_handle() as HWND;
    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        let small = LoadImageW(
            module,
            MAKEINTRESOURCEW(1),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            LR_DEFAULTCOLOR,
        ) as HICON;
        let big = LoadImageW(
            module,
            MAKEINTRESOURCEW(1),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
            LR_DEFAULTCOLOR,
        ) as HICON;
        if !small.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, small as isize);
        }
        if !big.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, big as isize);
        }
    }
}

#[allow(deprecated)]
fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        winapi::um::winuser::SetProcessDPIAware();
    }

    let images = assets::PieceImages::load();
    let text = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &text);
    let mut app = App::new();

    let (win_w, win_h) = (
        app.settings.resolution.0 as usize,
        app.settings.resolution.1 as usize,
    );
    let options = WindowOptions {
        resize: true,
        ..Default::default()
    };
    let window_title = format!("{} · Rust", app.settings.language.text().title);
    let mut window = match Window::new(&window_title, win_w, win_h, options) {
        Ok(window) => window,
        Err(e) => {
            eprintln!("窗口创建失败: {e}");
            return;
        }
    };
    let (screen_w, screen_h) =
        display::desktop_resolution().unwrap_or((win_w as u32, win_h as u32));
    window.set_position(
        ((screen_w as isize - win_w as isize) / 2).max(0),
        ((screen_h as isize - win_h as isize) / 2).max(0),
    );
    set_native_window_icons(&window);

    let mut buffer = vec![0u32; win_w * win_h];
    let mut mouse_was_down = false;
    while window.is_open() && !window.is_key_down(Key::Escape) && !app.exit_requested {
        app.tick_clock();
        app.poll_ai();
        app.poll_hint();
        let (mut cur_w, mut cur_h) = window.get_size();
        cur_w = cur_w.max(1);
        cur_h = cur_h.max(1);
        if buffer.len() != cur_w * cur_h {
            buffer.resize(cur_w * cur_h, 0);
        }
        let mouse = window.get_mouse_pos(MouseMode::Discard);
        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let pressed = mouse_down && !mouse_was_down;
        let released = !mouse_down && mouse_was_down;
        mouse_was_down = mouse_down;
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            app.handle_scroll(scroll_y);
        }
        let view = app.view_state(mouse, pressed, mouse_down, released);
        let actions = renderer.render(&mut buffer, cur_w, cur_h, &view);
        for action in actions {
            app.handle_action(action);
        }

        if app.window_title_update_requested {
            let title = format!("{} · Rust", app.settings.language.text().title);
            window.set_title(&title);
            app.window_title_update_requested = false;
        }
        if let Some((width, height)) = app.window_resize_requested.take() {
            resize_native_window(&window, width, height);
        }

        let frame_rate = if app.screen == Screen::Menu {
            app.settings.fps.min(60)
        } else {
            app.settings.fps
        };
        window.limit_update_rate(Some(Duration::from_secs_f64(
            1.0 / frame_rate.max(1) as f64,
        )));
        window
            .update_with_buffer(&buffer, cur_w, cur_h)
            .expect("更新窗口失败");
    }
}

#[cfg(all(test, target_os = "windows"))]
mod windows_resource_tests {
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winapi::um::winuser::{IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW, MAKEINTRESOURCEW};

    #[test]
    fn embedded_window_icon_is_loadable() {
        unsafe {
            let module = GetModuleHandleW(std::ptr::null());
            let icon = LoadImageW(
                module,
                MAKEINTRESOURCEW(1),
                IMAGE_ICON,
                16,
                16,
                LR_DEFAULTCOLOR,
            );
            assert!(!icon.is_null());
        }
    }
}

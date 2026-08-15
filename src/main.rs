#![windows_subsystem = "windows"]

mod ai;
mod assets;
mod display;
mod font;
mod i18n;
mod render;
mod settings;
mod theme;

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use shakmaty::san::San;
use shakmaty::{Chess, Color, File, Move, Position, Role, Square};

use render::{
    DropdownKind, PieceAnimView, Renderer, Screen, Settings, UiAction, ViewState,
};

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
    selected: Option<Square>,
    last_move: Option<(Square, Square)>,
    ai_mode: Option<Color>,
    ai_rx: Option<Receiver<Move>>,
    ai_thinking: bool,
    hint_rx: Option<Receiver<Move>>,
    hint_thinking: bool,
    suggestion: Option<(Square, Square, String)>,
    promotion: Option<(Square, Square)>,
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
    game_over_at: Option<Instant>,
    started_at: Instant,
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
            selected: None,
            last_move: None,
            ai_mode: None,
            ai_rx: None,
            ai_thinking: false,
            hint_rx: None,
            hint_thinking: false,
            suggestion: None,
            promotion: None,
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
            game_over_at: None,
            started_at: Instant::now(),
        }
    }

    fn game_over(&self) -> bool {
        self.pos.outcome().is_some()
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
        if self.game_over()
            || self.ai_thinking
            || self.hint_thinking
            || self.promotion.is_some()
        {
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
            Move::Normal { role: _, from, to, .. } => {
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
            self.selected = None;
            self.promotion = None;
            self.animations.extend(anims);
            if self.pos.outcome().is_some() && self.game_over_at.is_none() {
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
            UiAction::Square(sq) => self.handle_board_click(sq),
            UiAction::Promote(role) => self.handle_promotion(role),
        }
    }

    fn start_game(&mut self) {
        self.ai_mode = self.settings.mode;
        self.new_game();
        self.screen = Screen::Game;
    }

    fn open_settings(&mut self) {
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
        self.screen = Screen::Menu;
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
            if moves.iter().any(|m| m.promotion().is_some()) {
                self.promotion = Some((sel, sq));
            } else {
                self.apply_move(moves[0].clone());
                self.maybe_spawn_ai();
            }
        } else if let Some(piece) = self.pos.board().piece_at(sq) {
            if piece.color == self.pos.turn() {
                self.selected = Some(sq);
            }
        }
    }

    fn handle_promotion(&mut self, role: Role) {
        if let Some((from, to)) = self.promotion {
            let mv = self
                .pos
                .legal_moves()
                .iter()
                .cloned()
                .find(|m| m.from() == Some(from) && m.to() == to && m.promotion() == Some(role));
            if let Some(mv) = mv {
                self.apply_move(mv);
            }
        }
        self.promotion = None;
        self.maybe_spawn_ai();
    }

    fn undo(&mut self) {
        self.ai_rx = None;
        self.ai_reply_at = None;
        self.ai_thinking = false;
        self.hint_rx = None;
        self.hint_thinking = false;
        self.suggestion = None;
        self.promotion = None;
        self.selected = None;
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
        self.selected = None;
        self.last_move = None;
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
        for mv in &self.history {
            pos = pos.play(mv).expect("棋谱回放失败");
        }
        self.pos = pos;
        self.last_move = self
            .history
            .last()
            .map(|m| (m.from().expect("standard move has origin"), m.to()));
    }

    fn view_state(&self, mouse: Option<(f32, f32)>, mouse_pressed: bool) -> ViewState {
        let now = Instant::now();
        let animations: Vec<PieceAnimView> = self
            .animations
            .iter()
            .filter(|a| !a.done(now))
            .map(|a| a.view(now))
            .collect();
        let game_over_progress = if self.pos.outcome().is_some() {
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
            screen: self.screen,
            settings: self.settings,
            animations,
            resolutions: self.resolutions.clone(),
            refreshes: self.refreshes.clone(),
            open_dropdown: self.open_dropdown,
            dropdown_scroll: self.dropdown_scroll,
            game_over_progress,
            mouse,
            mouse_pressed,
            menu_time: now.duration_since(self.started_at).as_secs_f32(),
        }
    }

    fn handle_scroll(&mut self, y: f32) {
        if self.open_dropdown.is_none() {
            return;
        }
        let delta = (y / 3.0).round() as i32;
        if delta == 0 {
            return;
        }
        let next = self.dropdown_scroll as i32 - delta;
        self.dropdown_scroll = next.max(0) as usize;
    }
}

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
        MAKEINTRESOURCEW, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, SendMessageW,
        WM_SETICON,
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
        mouse_was_down = mouse_down;
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            app.handle_scroll(scroll_y);
        }
        let view = app.view_state(mouse, pressed);
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

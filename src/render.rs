use shakmaty::{Chess, Color, File, Outcome, Piece, Position, Rank, Role, Square};

use crate::assets::{
    draw_arrow_down, draw_arrow_up, draw_ring, draw_scaled, draw_scaled_rotated,
    draw_scaled_tinted, fill_circle, fill_rect, fill_rect_alpha, PieceImages,
};
use crate::font::TextRenderer;
use crate::i18n::Language;
use crate::theme::{Palette, Theme};

pub struct ViewState {
    pub pos: Chess,
    pub selected: Option<Square>,
    pub last_move: Option<(Square, Square)>,
    pub legal_targets: Vec<Square>,
    pub history_sans: Vec<String>,
    pub ai_thinking: bool,
    pub hint_thinking: bool,
    pub suggestion: Option<(Square, Square, String)>,
    pub promotion: Option<(Square, Square)>,
    pub screen: Screen,
    pub settings: Settings,
    pub animations: Vec<PieceAnimView>,
    pub resolutions: Vec<(u32, u32)>,
    pub refreshes: Vec<u32>,
    pub open_dropdown: Option<DropdownKind>,
    pub dropdown_scroll: usize,
    pub game_over_progress: f32,
    pub mouse: Option<(f32, f32)>,
    pub mouse_pressed: bool,
    pub menu_time: f32,
}

#[derive(Clone, Copy)]
pub struct PieceAnimView {
    pub color: Color,
    pub role: Role,
    pub from: Square,
    pub to: Square,
    pub progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Menu,
    Settings,
    Game,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Option<Color>,
    pub ai_depth: u32,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub theme: Theme,
    pub language: Language,
    pub flip_for_black: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: None,
            ai_depth: 2,
            resolution: (1280, 720),
            fps: 60,
            theme: Theme::DarkPlus,
            language: Language::Chinese,
            flip_for_black: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropdownKind {
    Mode,
    Difficulty,
    Resolution,
    Fps,
    Theme,
    Language,
    BoardView,
}

#[derive(Clone, Copy, Debug)]
pub enum UiAction {
    StartGame,
    OpenSettings,
    BackToMenu,
    ExitGame,
    SetDifficulty(u32),
    SetResolution((u32, u32)),
    SetFps(u32),
    SetTheme(Theme),
    SetLanguage(Language),
    SetFlipForBlack(bool),
    ToggleDropdown(DropdownKind),
    NewGame,
    Undo,
    Hint,
    Mode(Option<Color>),
    Square(Square),
    Promote(Role),
}

pub struct Renderer<'a> {
    images: &'a PieceImages,
    font: &'a TextRenderer,
}

struct Layout {
    w: usize,
    h: usize,
    board_left: f32,
    board_top: f32,
    board_size: f32,
    sq: f32,
    panel_x: f32,
    panel_w: f32,
}

impl Layout {
    fn new(w: usize, h: usize) -> Self {
        let panel_w = 280.0;
        let margin = 24.0;
        let avail_w = (w as f32 - panel_w - margin * 2.0).max(320.0);
        let avail_h = (h as f32 - 80.0).max(320.0);
        let board_size = avail_w.min(avail_h).min(1000.0).max(320.0);
        let board_left = (w as f32 - panel_w - margin * 2.0 - board_size) / 2.0 + margin;
        let board_top = (h as f32 - board_size) / 2.0;
        let panel_x = w as f32 - panel_w - margin;
        Self {
            w,
            h,
            board_left,
            board_top,
            board_size,
            sq: board_size / 8.0,
            panel_x,
            panel_w,
        }
    }
}

impl<'a> Renderer<'a> {
    pub fn new(images: &'a PieceImages, font: &'a TextRenderer) -> Self {
        Self { images, font }
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        width: usize,
        height: usize,
        view: &ViewState,
    ) -> Vec<UiAction> {
        let layout = Layout::new(width, height);
        let pal = view.settings.theme.palette();
        let mut actions = Vec::new();
        fill_rect(buf, width, height, 0, 0, width as i32, height as i32, pal.bg);
        match view.screen {
            Screen::Menu => self.draw_menu(buf, &layout, &pal, view, &mut actions),
            Screen::Settings => self.draw_settings(buf, &layout, &pal, view, &mut actions),
            Screen::Game => {
                self.draw_board(buf, &layout, &pal, view);
                self.draw_panel(buf, &layout, &pal, view, &mut actions);
                if view.promotion.is_some() {
                    self.draw_promotion_dialog(buf, &layout, &pal, view, &mut actions);
                } else if let Some((mx, my)) = view.mouse {
                    if view.mouse_pressed {
                        if let Some(sq) = self.square_at(mx, my, &layout, board_flipped(view)) {
                            actions.push(UiAction::Square(sq));
                        }
                    }
                }
                if view.game_over_progress > 0.0 {
                    self.draw_game_over(buf, &layout, &pal, view, &mut actions);
                }
            }
        }
        actions
    }

    fn square_at(&self, mx: f32, my: f32, layout: &Layout, flipped: bool) -> Option<Square> {
        if mx >= layout.board_left
            && mx < layout.board_left + layout.board_size
            && my >= layout.board_top
            && my < layout.board_top + layout.board_size
        {
            let screen_file = ((mx - layout.board_left) / layout.sq).floor() as u32;
            let screen_rank = ((my - layout.board_top) / layout.sq).floor() as u32;
            return oriented_square(screen_file, screen_rank, flipped);
        }
        None
    }

    fn draw_menu(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let tr = view.settings.language.text();
        let window_w = layout.w as f32;
        let window_h = layout.h as f32;
        let logo_size = (window_h * 0.60).min(window_w * 0.40).clamp(250.0, 430.0);
        let logo_cx = window_w * 0.70;
        let logo_cy = window_h * 0.45;
        let gear_size = logo_size * 1.32;
        let horse_size = logo_size * 0.92;
        let gear_cy = logo_cy - logo_size * 0.08;
        let gear_x = logo_cx - gear_size / 2.0;
        let gear_y = gear_cy - gear_size / 2.0;
        let horse_x = logo_cx - horse_size / 2.0;
        let horse_y = logo_cy - horse_size / 2.0;
        draw_scaled_rotated(
            buf,
            layout.w,
            layout.h,
            self.images.menu_gear(),
            gear_x,
            gear_y,
            gear_size,
            view.menu_time * 0.18,
            2.0 / 3.0,
        );
        draw_scaled_tinted(
            buf,
            layout.w,
            layout.h,
            self.images.menu_knight_buffer(),
            horse_x,
            horse_y,
            horse_size,
            horse_size,
            pal.bg,
        );
        draw_scaled(
            buf,
            layout.w,
            layout.h,
            self.images.menu_knight(),
            horse_x,
            horse_y,
            horse_size,
            horse_size,
        );

        let title = tr.title;
        let title_size = 44.0;
        let title_stretch = 1.28;
        let title_tracking = 3.5;
        let tw = self
            .font
            .serif_text_width(title, title_size, title_stretch, title_tracking);
        self.font.draw_serif_text(
            buf,
            layout.w,
            layout.h,
            logo_cx - tw / 2.0,
            gear_y + gear_size + 16.0,
            title,
            pal.text,
            title_size,
            title_stretch,
            title_tracking,
        );
        let sub = tr.subtitle;
        let sw = self.font.text_width(sub, 18.0);
        self.font
            .draw_text(buf, layout.w, layout.h, logo_cx - sw / 2.0, gear_y + gear_size + 46.0, sub, pal.muted, 18.0);

        let btn_w = (window_w * 0.31).clamp(280.0, 390.0);
        let btn_h = 58.0;
        let x = (window_w * 0.06).max(36.0);
        let first_y = (window_h * 0.37).clamp(190.0, 310.0);
        self.button(
            buf,
            layout,
            pal,
            view,
            x,
            first_y,
            btn_w,
            btn_h,
            tr.start_game,
            pal.button,
            22.0,
            actions,
            UiAction::StartGame,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            x,
            first_y + 76.0,
            btn_w,
            btn_h,
            tr.settings,
            pal.button,
            22.0,
            actions,
            UiAction::OpenSettings,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            x,
            first_y + 152.0,
            btn_w,
            btn_h,
            tr.exit,
            pal.button,
            22.0,
            actions,
            UiAction::ExitGame,
        );

        let footer = tr.attribution;
        let fw = self.font.text_width(footer, 14.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            logo_cx - fw / 2.0,
            layout.h as f32 - 24.0,
            footer,
            pal.muted,
            14.0,
        );
    }

    fn draw_settings(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let tr = view.settings.language.text();
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 66.0, tr.settings, pal.text, 36.0);

        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 112.0, tr.mode, pal.muted, 17.0);
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 222.0, tr.difficulty, pal.muted, 17.0);
        let right_x = 520.0;
        self.font
            .draw_text(buf, layout.w, layout.h, right_x, 112.0, tr.resolution, pal.muted, 17.0);
        self.font
            .draw_text(buf, layout.w, layout.h, right_x, 222.0, tr.refresh_rate, pal.muted, 17.0);
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 332.0, tr.theme, pal.muted, 17.0);
        self.font
            .draw_text(buf, layout.w, layout.h, right_x, 332.0, tr.language, pal.muted, 17.0);
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 442.0, tr.board_view, pal.muted, 17.0);

        let dropdown_w = 420.0;
        let mut any_base_clicked = false;

        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            126.0,
            dropdown_w,
            DropdownKind::Mode,
            dropdown_current(view, DropdownKind::Mode),
        );
        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            236.0,
            dropdown_w,
            DropdownKind::Difficulty,
            dropdown_current(view, DropdownKind::Difficulty),
        );
        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            right_x,
            126.0,
            dropdown_w,
            DropdownKind::Resolution,
            dropdown_current(view, DropdownKind::Resolution),
        );
        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            right_x,
            236.0,
            dropdown_w,
            DropdownKind::Fps,
            dropdown_current(view, DropdownKind::Fps),
        );
        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            346.0,
            dropdown_w,
            DropdownKind::Theme,
            dropdown_current(view, DropdownKind::Theme),
        );
        any_base_clicked |= self.dropdown_base(
            buf, layout, pal, view, actions, right_x, 346.0, dropdown_w,
            DropdownKind::Language, dropdown_current(view, DropdownKind::Language),
        );
        any_base_clicked |= self.dropdown_base(
            buf, layout, pal, view, actions, 56.0, 456.0, dropdown_w,
            DropdownKind::BoardView, dropdown_current(view, DropdownKind::BoardView),
        );

        let back_w = 260.0;
        let back_x = (layout.w as f32 - back_w) / 2.0;
        let back_y = layout.h as f32 - 96.0;
        let back_clicked = view.mouse_pressed
            && view.mouse.map_or(false, |(mx, my)| {
                mx >= back_x && mx <= back_x + back_w && my >= back_y && my <= back_y + 50.0
            });
        self.button(
            buf,
            layout,
            pal,
            view,
            back_x,
            back_y,
            back_w,
            50.0,
            tr.back_to_menu,
            pal.button,
            20.0,
            actions,
            UiAction::BackToMenu,
        );

        let mut list_handled = false;
        if let Some(kind) = view.open_dropdown {
            let options = dropdown_options(view, kind);
            let (ox, oy) = dropdown_position(kind);
            list_handled = self.dropdown_list(
                buf,
                layout,
                pal,
                view,
                actions,
                ox,
                oy,
                dropdown_w,
                &options,
                view.dropdown_scroll,
            );
        }

        if !back_clicked
            && !any_base_clicked
            && !list_handled
            && view.open_dropdown.is_some()
            && view.mouse_pressed
        {
            actions.push(UiAction::ToggleDropdown(view.open_dropdown.unwrap()));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dropdown_base(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
        x: f32,
        y: f32,
        w: f32,
        kind: DropdownKind,
        current: String,
    ) -> bool {
        let h = 40.0;
        let hover = view
            .mouse
            .map_or(false, |(mx, my)| mx >= x && mx <= x + w && my >= y && my <= y + h);
        let color = if hover { pal.button_hover } else { pal.button };
        fill_rect(buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, color);
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            (y + h - 2.0) as i32,
            w as i32,
            2,
            pal.border,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + 12.0,
            y + h / 2.0 + 6.0,
            &current,
            pal.text,
            17.0,
        );
        draw_arrow_down(buf, layout.w, layout.h, x + w - 16.0, y + h / 2.0, 7.0, pal.accent);
        if hover && view.mouse_pressed {
            actions.push(UiAction::ToggleDropdown(kind));
            return true;
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn dropdown_list(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
        x: f32,
        y: f32,
        w: f32,
        options: &[(String, UiAction)],
        scroll: usize,
    ) -> bool {
        let h = 40.0;
        let option_h = 32.0;
        let max_visible = 9;
        let start = scroll.min(options.len().saturating_sub(1));
        let visible = (options.len() - start).min(max_visible);
        let list_h = visible as f32 * option_h;
        let mut list_y = y + h + 6.0;
        if list_y + list_h > layout.h as f32 - 8.0 {
            list_y = (y - list_h - 6.0).max(8.0);
        }
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            list_y as i32,
            w as i32,
            list_h as i32,
            pal.panel,
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            list_y as i32,
            w as i32,
            1,
            pal.border,
        );

        for i in 0..visible {
            let idx = start + i;
            let (label, action) = &options[idx];
            let oy = list_y + i as f32 * option_h;
            let row_hover = view.mouse.map_or(false, |(mx, my)| {
                mx >= x && mx <= x + w && my >= oy && my <= oy + option_h
            });
            fill_rect(
                buf,
                layout.w,
                layout.h,
                x as i32,
                oy as i32,
                w as i32,
                option_h as i32,
                if row_hover { pal.button_hover } else { pal.panel },
            );
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + 12.0,
                oy + option_h / 2.0 + 6.0,
                label,
                pal.text,
                16.0,
            );
            if row_hover && view.mouse_pressed {
                actions.push(*action);
                return true;
            }
        }

        if start > 0 {
            draw_arrow_up(buf, layout.w, layout.h, x + w / 2.0, list_y + 12.0, 8.0, pal.accent);
        }
        if start + visible < options.len() {
            draw_arrow_down(
                buf,
                layout.w,
                layout.h,
                x + w / 2.0,
                list_y + list_h - 12.0,
                8.0,
                pal.accent,
            );
        }
        false
    }

    #[allow(dead_code)]
    fn draw_settings_old(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 76.0, "游戏设置", pal.text, 40.0);

        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 140.0, "对战模式", pal.muted, 18.0);
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 330.0, "AI 难度", pal.muted, 18.0);
        let right_x = 520.0;
        self.font
            .draw_text(buf, layout.w, layout.h, right_x, 140.0, "窗口分辨率", pal.muted, 18.0);
        self.font
            .draw_text(buf, layout.w, layout.h, right_x, 330.0, "刷新率", pal.muted, 18.0);
        self.font
            .draw_text(buf, layout.w, layout.h, 56.0, 520.0, "主题", pal.muted, 18.0);

        let dropdown_w = 420.0;
        let mut click_handled = false;

        let mode_options: Vec<(String, UiAction)> = [
            ("双人对战", UiAction::Mode(None)),
            ("人机 · 执白", UiAction::Mode(Some(Color::White))),
            ("人机 · 执黑", UiAction::Mode(Some(Color::Black))),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect();
        click_handled |= self.dropdown_old(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            158.0,
            dropdown_w,
            DropdownKind::Mode,
            mode_name(view.settings.mode, view.settings.language).to_string(),
            &mode_options,
        );

        let difficulty_options: Vec<(String, UiAction)> = [
            ("简单", UiAction::SetDifficulty(1)),
            ("中等", UiAction::SetDifficulty(2)),
            ("困难", UiAction::SetDifficulty(3)),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect();
        click_handled |= self.dropdown_old(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            348.0,
            dropdown_w,
            DropdownKind::Difficulty,
            depth_name(view.settings.ai_depth, view.settings.language).to_string(),
            &difficulty_options,
        );

        let resolution_options: Vec<(String, UiAction)> = view
            .resolutions
            .iter()
            .map(|&(w, h)| (format!("{w} × {h}"), UiAction::SetResolution((w, h))))
            .collect();
        let current_res = format!(
            "{} × {}",
            view.settings.resolution.0, view.settings.resolution.1
        );
        click_handled |= self.dropdown_old(
            buf,
            layout,
            pal,
            view,
            actions,
            right_x,
            158.0,
            dropdown_w,
            DropdownKind::Resolution,
            current_res,
            &resolution_options,
        );

        let fps_options: Vec<(String, UiAction)> = view
            .refreshes
            .iter()
            .map(|&fps| (format!("{fps} Hz"), UiAction::SetFps(fps)))
            .collect();
        click_handled |= self.dropdown_old(
            buf,
            layout,
            pal,
            view,
            actions,
            right_x,
            348.0,
            dropdown_w,
            DropdownKind::Fps,
            format!("{} Hz", view.settings.fps),
            &fps_options,
        );

        let theme_options: Vec<(String, UiAction)> = Theme::ALL
            .iter()
            .map(|&theme| (theme.label().to_string(), UiAction::SetTheme(theme)))
            .collect();
        click_handled |= self.dropdown_old(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            538.0,
            dropdown_w,
            DropdownKind::Theme,
            view.settings.theme.label().to_string(),
            &theme_options,
        );

        if !click_handled && view.open_dropdown.is_some() && view.mouse_pressed {
            actions.push(UiAction::ToggleDropdown(view.open_dropdown.unwrap()));
        }

        let back_w = 260.0;
        self.button(
            buf,
            layout,
            pal,
            view,
            (layout.w as f32 - back_w) / 2.0,
            layout.h as f32 - 96.0,
            back_w,
            50.0,
            "返回主菜单",
            pal.button,
            20.0,
            actions,
            UiAction::BackToMenu,
        );
    }

    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    fn dropdown_old(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
        x: f32,
        y: f32,
        w: f32,
        kind: DropdownKind,
        current: String,
        options: &[(String, UiAction)],
    ) -> bool {
        let h = 40.0;
        let open = view.open_dropdown == Some(kind);
        let hover = view
            .mouse
            .map_or(false, |(mx, my)| mx >= x && mx <= x + w && my >= y && my <= y + h);
        let base_color = if hover { pal.button_hover } else { pal.button };
        fill_rect(buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, base_color);
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            (y + h - 2.0) as i32,
            w as i32,
            2,
            pal.border,
        );
        self.font
            .draw_text(buf, layout.w, layout.h, x + 12.0, y + h / 2.0 + 6.0, &current, pal.text, 17.0);
        self.font
            .draw_text(buf, layout.w, layout.h, x + w - 24.0, y + h / 2.0 + 6.0, "▾", pal.accent, 17.0);

        if hover && view.mouse_pressed {
            actions.push(UiAction::ToggleDropdown(kind));
            return true;
        }
        if !open {
            return false;
        }

        let option_h = 32.0;
        let visible = options.len().min(9);
        let list_h = visible as f32 * option_h;
        let mut list_y = y + h + 6.0;
        if list_y + list_h > layout.h as f32 - 8.0 {
            list_y = (y - list_h - 6.0).max(8.0);
        }
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            list_y as i32,
            w as i32,
            list_h as i32,
            pal.panel,
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            list_y as i32,
            w as i32,
            1,
            pal.border,
        );

        for (i, (label, action)) in options.iter().take(visible).enumerate() {
            let oy = list_y + i as f32 * option_h;
            let row_hover = view.mouse.map_or(false, |(mx, my)| {
                mx >= x && mx <= x + w && my >= oy && my <= oy + option_h
            });
            fill_rect(
                buf,
                layout.w,
                layout.h,
                x as i32,
                oy as i32,
                w as i32,
                option_h as i32,
                if row_hover { pal.button_hover } else { pal.panel },
            );
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + 12.0,
                oy + option_h / 2.0 + 6.0,
                label,
                pal.text,
                16.0,
            );
            if row_hover && view.mouse_pressed {
                actions.push(*action);
                return true;
            }
        }
        if options.len() > visible {
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + 12.0,
                list_y + list_h - option_h / 2.0 + 6.0,
                "…",
                pal.muted,
                16.0,
            );
        }
        false
    }

    fn draw_board(&self, buf: &mut [u32], layout: &Layout, pal: &Palette, view: &ViewState) {
        let flipped = board_flipped(view);
        fill_rect(
            buf,
            layout.w,
            layout.h,
            layout.board_left as i32 - 7,
            layout.board_top as i32 - 7,
            layout.board_size as i32 + 14,
            layout.board_size as i32 + 14,
            pal.border,
        );

        for file in 0..8u32 {
            for rank in 0..8u32 {
                let sq = Square::from_coords(File::new(file), Rank::new(rank));
                let (x, y) = square_rect(sq, layout, flipped);
                let color = if (file + rank) % 2 == 0 {
                    pal.light_square
                } else {
                    pal.dark_square
                };
                fill_rect(
                    buf,
                    layout.w,
                    layout.h,
                    x as i32,
                    y as i32,
                    layout.sq as i32,
                    layout.sq as i32,
                    color,
                );
            }
        }

        if let Some((from, to)) = view.last_move {
            self.highlight_square(buf, layout, pal.last_move, 95, from, flipped);
            self.highlight_square(buf, layout, pal.last_move, 95, to, flipped);
        }
        if let Some(sq) = view.selected {
            self.highlight_square(buf, layout, pal.selected, 120, sq, flipped);
        }
        if let Some((from, to, _)) = &view.suggestion {
            self.highlight_square(buf, layout, pal.accent, 105, *from, flipped);
            self.highlight_square(buf, layout, pal.accent, 150, *to, flipped);
        }

        for &sq in &view.legal_targets {
            let (x, y) = square_rect(sq, layout, flipped);
            let cx = x + layout.sq / 2.0;
            let cy = y + layout.sq / 2.0;
            if view.pos.board().piece_at(sq).is_some() {
                draw_ring(
                    buf,
                    layout.w,
                    layout.h,
                    cx,
                    cy,
                    layout.sq / 2.0 - 8.0,
                    pal.capture_ring,
                    230,
                );
            } else {
                fill_circle(
                    buf,
                    layout.w,
                    layout.h,
                    cx,
                    cy,
                    12.0,
                    pal.move_dot,
                    210,
                );
            }
        }

        if view.pos.is_check() {
            if let Some(ksq) = king_square(&view.pos, view.pos.turn()) {
                let (x, y) = square_rect(ksq, layout, flipped);
                draw_ring(
                    buf,
                    layout.w,
                    layout.h,
                    x + layout.sq / 2.0,
                    y + layout.sq / 2.0,
                    layout.sq / 2.0 - 5.0,
                    pal.check_ring,
                    255,
                );
            }
        }

        for i in 0..64u32 {
            let sq = Square::new(i);
            if let Some(piece) = view.pos.board().piece_at(sq) {
                if view.animations.iter().any(|a| a.to == sq) {
                    continue;
                }
                let tex = self.images.get(piece.color, piece.role);
                let (x, y) = square_rect(sq, layout, flipped);
                draw_scaled(
                    buf,
                    layout.w,
                    layout.h,
                    tex,
                    x + 4.0,
                    y + 4.0,
                    layout.sq - 8.0,
                    layout.sq - 8.0,
                );
            }
        }

        for anim in &view.animations {
            let tex = self.images.get(anim.color, anim.role);
            let (fx, fy) = square_rect(anim.from, layout, flipped);
            let (tx, ty) = square_rect(anim.to, layout, flipped);
            let x = fx + (tx - fx) * anim.progress;
            let y = fy + (ty - fy) * anim.progress;
            draw_scaled(
                buf,
                layout.w,
                layout.h,
                tex,
                x + 4.0,
                y + 4.0,
                layout.sq - 8.0,
                layout.sq - 8.0,
            );
        }

        for screen_file in 0..8u32 {
            let file = if flipped { 7 - screen_file } else { screen_file };
            let letter = char::from(b'a' + file as u8);
            let x = layout.board_left + screen_file as f32 * layout.sq + layout.sq - 20.0;
            let y = layout.board_top + layout.board_size + 8.0;
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x,
                y + 12.0,
                &letter.to_string(),
                pal.muted,
                14.0,
            );
        }
        for screen_rank in 0..8u32 {
            let rank = if flipped { screen_rank } else { 7 - screen_rank };
            let y = layout.board_top + screen_rank as f32 * layout.sq + 10.0;
            let x = layout.board_left - 20.0;
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x,
                y + 12.0,
                &(rank + 1).to_string(),
                pal.muted,
                14.0,
            );
        }
    }

    fn highlight_square(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        color: u32,
        alpha: u32,
        sq: Square,
        flipped: bool,
    ) {
        let (x, y) = square_rect(sq, layout, flipped);
        fill_rect_alpha(
            buf,
            layout.w,
            layout.h,
            x as i32,
            y as i32,
            layout.sq as i32,
            layout.sq as i32,
            color,
            alpha,
        );
    }

    fn draw_panel(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let tr = view.settings.language.text();
        fill_rect(
            buf,
            layout.w,
            layout.h,
            layout.panel_x as i32,
            0,
            layout.panel_w as i32,
            layout.h as i32,
            pal.panel,
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            layout.panel_x as i32 - 2,
            0,
            2,
            layout.h as i32,
            pal.border,
        );

        let x = layout.panel_x + 16.0;
        self.font
            .draw_text(buf, layout.w, layout.h, x, 34.0, tr.title, pal.text, 30.0);
        self.font
            .draw_text(buf, layout.w, layout.h, x, 58.0, tr.subtitle, pal.muted, 14.0);

        let btn_x = layout.panel_x + 14.0;
        let btn_w = layout.panel_w - 28.0;
        self.button(
            buf,
            layout,
            pal,
            view,
            btn_x,
            80.0,
            btn_w,
            38.0,
            tr.new_game,
            pal.button,
            18.0,
            actions,
            UiAction::NewGame,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            btn_x,
            126.0,
            btn_w,
            38.0,
            tr.undo,
            pal.button,
            18.0,
            actions,
            UiAction::Undo,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            btn_x,
            172.0,
            btn_w,
            38.0,
            if view.hint_thinking { tr.hint_thinking } else { tr.hint },
            if view.hint_thinking { pal.button_active } else { pal.button },
            18.0,
            actions,
            UiAction::Hint,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            btn_x,
            218.0,
            btn_w,
            38.0,
            tr.menu,
            pal.button,
            18.0,
            actions,
            UiAction::BackToMenu,
        );

        self.font
            .draw_text(buf, layout.w, layout.h, x, 282.0, tr.mode, pal.muted, 16.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x,
            306.0,
            mode_name(view.settings.mode, view.settings.language),
            pal.accent,
            18.0,
        );
        self.font
            .draw_text(buf, layout.w, layout.h, x, 340.0, tr.difficulty, pal.muted, 16.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x,
            364.0,
            depth_name(view.settings.ai_depth, view.settings.language),
            pal.accent,
            18.0,
        );

        let status = status_text(view);
        let status_color = if view.pos.is_check() && !view.ai_thinking {
            pal.accent
        } else {
            pal.text
        };
        self.font
            .draw_text(buf, layout.w, layout.h, x, 410.0, &status, status_color, 18.0);

        if let Some((_, _, san)) = &view.suggestion {
            let hint = format!("{}: {}", tr.suggested, san);
            self.font
                .draw_text(buf, layout.w, layout.h, x, 442.0, &hint, pal.accent, 17.0);
        }

        self.font
            .draw_text(buf, layout.w, layout.h, x, 474.0, tr.moves, pal.muted, 16.0);
        let start = view.history_sans.len().saturating_sub(11);
        for (i, san) in view.history_sans.iter().enumerate().skip(start) {
            let line = format!("{}. {}", i + 1, san);
            let y = 498.0 + (i - start) as f32 * 18.0;
            if y < layout.h as f32 - 12.0 {
                self.font
                    .draw_text(buf, layout.w, layout.h, x, y, &line, pal.history, 15.0);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn button(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        base_color: u32,
        font_size: f32,
        actions: &mut Vec<UiAction>,
        action: UiAction,
    ) {
        let hover = view
            .mouse
            .map_or(false, |(mx, my)| mx >= x && mx <= x + w && my >= y && my <= y + h);
        let color = if hover {
            if base_color == pal.button_active {
                pal.button_active_hover
            } else {
                pal.button_hover
            }
        } else {
            base_color
        };
        fill_rect(buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, color);
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            (y + h - 2.0) as i32,
            w as i32,
            2,
            pal.border,
        );
        let tw = self.font.text_width(label, font_size);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + (w - tw) / 2.0,
            y + h / 2.0 + 6.0,
            label,
            pal.text,
            font_size,
        );
        if hover && view.mouse_pressed && view.game_over_progress == 0.0 {
            actions.push(action);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn overlay_button(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        base_color: u32,
        font_size: f32,
        actions: &mut Vec<UiAction>,
        action: UiAction,
    ) {
        let hover = view
            .mouse
            .map_or(false, |(mx, my)| mx >= x && mx <= x + w && my >= y && my <= y + h);
        let color = if hover { pal.button_hover } else { base_color };
        fill_rect(buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, color);
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            (y + h - 2.0) as i32,
            w as i32,
            2,
            pal.border,
        );
        let tw = self.font.text_width(label, font_size);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + (w - tw) / 2.0,
            y + h / 2.0 + 6.0,
            label,
            pal.text,
            font_size,
        );
        if hover && view.mouse_pressed && view.game_over_progress >= 0.8 {
            actions.push(action);
        }
    }

    fn draw_game_over(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let tr = view.settings.language.text();
        let p = view.game_over_progress;
        fill_rect_alpha(
            buf,
            layout.w,
            layout.h,
            0,
            0,
            layout.w as i32,
            layout.h as i32,
            0x0000_00,
            (p * 170.0) as u32,
        );
        let box_w = 560.0;
        let box_h = 240.0;
        let bx = (layout.w as f32 - box_w) / 2.0;
        let by = (layout.h as f32 - box_h) / 2.0 - 20.0 + (1.0 - p) * 24.0;
        fill_rect_alpha(
            buf,
            layout.w,
            layout.h,
            bx as i32,
            by as i32,
            box_w as i32,
            box_h as i32,
            pal.panel,
            (p * 255.0) as u32,
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            bx as i32,
            (by + box_h - 3.0) as i32,
            box_w as i32,
            3,
            pal.button_active,
        );

        let title = status_text(view);
        let tw = self.font.text_width(&title, 34.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            bx + (box_w - tw) / 2.0,
            by + 64.0,
            &title,
            pal.text,
            34.0,
        );
        let sub = tr.game_over;
        let sw = self.font.text_width(sub, 20.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            bx + (box_w - sw) / 2.0,
            by + 106.0,
            sub,
            pal.muted,
            20.0,
        );

        let btn_w = 210.0;
        let btn_h = 48.0;
        let gap = 24.0;
        let total = btn_w * 2.0 + gap;
        let sx = bx + (box_w - total) / 2.0;
        self.overlay_button(
            buf,
            layout,
            pal,
            view,
            sx,
            by + 146.0,
            btn_w,
            btn_h,
            tr.play_again,
            pal.button_active,
            18.0,
            actions,
            UiAction::NewGame,
        );
        self.overlay_button(
            buf,
            layout,
            pal,
            view,
            sx + btn_w + gap,
            by + 146.0,
            btn_w,
            btn_h,
            tr.back_to_menu,
            pal.button,
            18.0,
            actions,
            UiAction::BackToMenu,
        );
    }

    fn draw_promotion_dialog(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let tr = view.settings.language.text();
        fill_rect_alpha(
            buf,
            layout.w,
            layout.h,
            0,
            0,
            layout.w as i32,
            layout.h as i32,
            0x0000_00,
            140,
        );

        let box_w = 560.0;
        let box_h = 230.0;
        let bx = (layout.w as f32 - box_w) / 2.0;
        let by = (layout.h as f32 - box_h) / 2.0;
        fill_rect(
            buf,
            layout.w,
            layout.h,
            bx as i32,
            by as i32,
            box_w as i32,
            box_h as i32,
            pal.panel,
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            bx as i32,
            (by + box_h - 3.0) as i32,
            box_w as i32,
            3,
            pal.button_active,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            bx + 30.0,
            by + 44.0,
            tr.promotion,
            pal.text,
            26.0,
        );

        let color = view.pos.turn();
        let roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
        let names = [tr.queen, tr.rook, tr.bishop, tr.knight];
        let bw = 100.0;
        let bh = 120.0;
        let gap = 16.0;
        let total = bw * 4.0 + gap * 3.0;
        let sx = bx + (box_w - total) / 2.0;

        for (i, role) in roles.iter().enumerate() {
            let x = sx + i as f32 * (bw + gap);
            let y = by + 68.0;
            let hover = view.mouse.map_or(false, |(mx, my)| {
                mx >= x && mx <= x + bw && my >= y && my <= y + bh
            });
            fill_rect(
                buf,
                layout.w,
                layout.h,
                x as i32,
                y as i32,
                bw as i32,
                bh as i32,
                if hover { pal.button_hover } else { pal.button },
            );
            let tex = self.images.get(color, *role);
            draw_scaled(buf, layout.w, layout.h, tex, x + 14.0, y + 8.0, bw - 28.0, 84.0);
            let tw = self.font.text_width(names[i], 18.0);
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + (bw - tw) / 2.0,
                y + bh - 12.0,
                names[i],
                pal.text,
                18.0,
            );
            if hover && view.mouse_pressed {
                actions.push(UiAction::Promote(*role));
            }
        }
    }
}

fn square_rect(sq: Square, layout: &Layout, flipped: bool) -> (f32, f32) {
    let file = u32::from(sq.file()) as f32;
    let rank = u32::from(sq.rank()) as f32;
    let screen_file = if flipped { 7.0 - file } else { file };
    let screen_rank = if flipped { rank } else { 7.0 - rank };
    (
        layout.board_left + screen_file * layout.sq,
        layout.board_top + screen_rank * layout.sq,
    )
}

fn board_flipped(view: &ViewState) -> bool {
    view.settings.flip_for_black && view.settings.mode == Some(Color::White)
}

fn oriented_square(screen_file: u32, screen_rank: u32, flipped: bool) -> Option<Square> {
    if screen_file >= 8 || screen_rank >= 8 {
        return None;
    }
    let file = if flipped { 7 - screen_file } else { screen_file };
    let rank = if flipped { screen_rank } else { 7 - screen_rank };
    Some(Square::from_coords(File::new(file), Rank::new(rank)))
}

fn king_square(pos: &Chess, color: Color) -> Option<Square> {
    (0..64u32)
        .map(Square::new)
        .find(|&sq| pos.board().piece_at(sq) == Some(Piece { color, role: Role::King }))
}

fn dropdown_options(view: &ViewState, kind: DropdownKind) -> Vec<(String, UiAction)> {
    let tr = view.settings.language.text();
    match kind {
        DropdownKind::Mode => [
            (tr.two_players, UiAction::Mode(None)),
            (tr.play_white, UiAction::Mode(Some(Color::Black))),
            (tr.play_black, UiAction::Mode(Some(Color::White))),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect(),
        DropdownKind::Difficulty => [
            (tr.beginner, UiAction::SetDifficulty(1)),
            (tr.easy, UiAction::SetDifficulty(2)),
            (tr.medium, UiAction::SetDifficulty(3)),
            (tr.hard, UiAction::SetDifficulty(4)),
            (tr.master, UiAction::SetDifficulty(5)),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect(),
        DropdownKind::Resolution => view
            .resolutions
            .iter()
            .map(|&(w, h)| (format!("{w} × {h}"), UiAction::SetResolution((w, h))))
            .collect(),
        DropdownKind::Fps => view
            .refreshes
            .iter()
            .map(|&fps| (format!("{fps} Hz"), UiAction::SetFps(fps)))
            .collect(),
        DropdownKind::Theme => Theme::ALL
            .iter()
            .map(|&theme| (theme.label().to_string(), UiAction::SetTheme(theme)))
            .collect(),
        DropdownKind::Language => Language::ALL
            .iter()
            .map(|&language| (language.native_name().to_string(), UiAction::SetLanguage(language)))
            .collect(),
        DropdownKind::BoardView => [
            (tr.flip_on, UiAction::SetFlipForBlack(true)),
            (tr.flip_off, UiAction::SetFlipForBlack(false)),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect(),
    }
}

fn dropdown_position(kind: DropdownKind) -> (f32, f32) {
    match kind {
        DropdownKind::Mode => (56.0, 126.0),
        DropdownKind::Difficulty => (56.0, 236.0),
        DropdownKind::Resolution => (520.0, 126.0),
        DropdownKind::Fps => (520.0, 236.0),
        DropdownKind::Theme => (56.0, 346.0),
        DropdownKind::Language => (520.0, 346.0),
        DropdownKind::BoardView => (56.0, 456.0),
    }
}

fn dropdown_current(view: &ViewState, kind: DropdownKind) -> String {
    match kind {
        DropdownKind::Mode => mode_name(view.settings.mode, view.settings.language).to_string(),
        DropdownKind::Difficulty => depth_name(view.settings.ai_depth, view.settings.language).to_string(),
        DropdownKind::Resolution => format!(
            "{} × {}",
            view.settings.resolution.0, view.settings.resolution.1
        ),
        DropdownKind::Fps => format!("{} Hz", view.settings.fps),
        DropdownKind::Theme => view.settings.theme.label().to_string(),
        DropdownKind::Language => view.settings.language.native_name().to_string(),
        DropdownKind::BoardView => if view.settings.flip_for_black {
            view.settings.language.text().flip_on
        } else {
            view.settings.language.text().flip_off
        }.to_string(),
    }
}

fn status_text(view: &ViewState) -> String {
    let tr = view.settings.language.text();
    if view.ai_thinking {
        return tr.ai_thinking.to_string();
    }
    if let Some(outcome) = view.pos.outcome() {
        return match outcome {
            Outcome::Decisive { winner } => {
                let name = if winner == Color::White { tr.white } else { tr.black };
                format!("{}{name}{}", tr.checkmate, tr.wins)
            }
            Outcome::Draw => tr.draw.to_string(),
        };
    }
    let turn = if view.pos.turn() == Color::White { tr.white } else { tr.black };
    let check = if view.pos.is_check() { tr.check } else { "" };
    format!("{turn}{}{check}", tr.to_move)
}

fn mode_name(mode: Option<Color>, language: Language) -> &'static str {
    let tr = language.text();
    match mode {
        None => tr.two_players,
        Some(Color::White) => tr.play_black,
        Some(Color::Black) => tr.play_white,
    }
}

fn depth_name(depth: u32, language: Language) -> &'static str {
    let tr = language.text();
    match depth {
        1 => tr.beginner,
        2 => tr.easy,
        3 => tr.medium,
        4 => tr.hard,
        _ => tr.master,
    }
}

#[cfg(test)]
mod tests {
    use super::{DropdownKind, Renderer, Screen, Settings, ViewState, oriented_square};
    use crate::{assets::PieceImages, font::TextRenderer, i18n::Language};
    use shakmaty::{Chess, Color, Square};

    #[test]
    fn board_orientation_maps_corner_squares() {
        assert_eq!(oriented_square(0, 0, false), Some(Square::A8));
        assert_eq!(oriented_square(7, 7, false), Some(Square::H1));
        assert_eq!(oriented_square(0, 0, true), Some(Square::H1));
        assert_eq!(oriented_square(7, 7, true), Some(Square::A8));
    }

    #[test]
    fn every_language_renders_all_screens() {
        let images = PieceImages::load();
        let text = TextRenderer::load();
        let renderer = Renderer::new(&images, &text);

        for language in Language::ALL {
            for screen in [Screen::Menu, Screen::Settings, Screen::Game] {
                let mut buffer = vec![0; 1280 * 720];
                let view = ViewState {
                    pos: Chess::default(),
                    selected: None,
                    last_move: None,
                    legal_targets: Vec::new(),
                    history_sans: Vec::new(),
                    ai_thinking: false,
                    hint_thinking: false,
                    suggestion: Some((Square::E2, Square::E4, "e4".to_string())),
                    promotion: None,
                    screen,
                    settings: Settings {
                        mode: Some(Color::White),
                        language,
                        ..Settings::default()
                    },
                    animations: Vec::new(),
                    resolutions: vec![(1280, 720)],
                    refreshes: vec![60],
                    open_dropdown: Some(DropdownKind::Language),
                    dropdown_scroll: 0,
                    game_over_progress: 0.0,
                    mouse: None,
                    mouse_pressed: false,
                    menu_time: 1.0,
                };
                let actions = renderer.render(&mut buffer, 1280, 720, &view);
                assert!(actions.is_empty());
                assert!(buffer.iter().any(|&pixel| pixel != buffer[0]));
            }
        }
    }
}

use shakmaty::{Chess, Color, File, Outcome, Piece, Position, Rank, Role, Square};

use crate::assets::{
    draw_arrow_down, draw_arrow_up, draw_ring, draw_scaled, fill_circle, fill_rect,
    fill_rect_alpha, PieceImages,
};
use crate::font::TextRenderer;
use crate::theme::{Palette, Theme};

pub struct ViewState {
    pub pos: Chess,
    pub selected: Option<Square>,
    pub last_move: Option<(Square, Square)>,
    pub legal_targets: Vec<Square>,
    pub history_sans: Vec<String>,
    pub ai_thinking: bool,
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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: None,
            ai_depth: 2,
            resolution: (1280, 720),
            fps: 60,
            theme: Theme::DarkPlus,
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
    ToggleDropdown(DropdownKind),
    NewGame,
    Undo,
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
                        if let Some(sq) = self.square_at(mx, my, &layout) {
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

    fn square_at(&self, mx: f32, my: f32, layout: &Layout) -> Option<Square> {
        if mx >= layout.board_left
            && mx < layout.board_left + layout.board_size
            && my >= layout.board_top
            && my < layout.board_top + layout.board_size
        {
            let file = ((mx - layout.board_left) / layout.sq).floor() as u32;
            let rank = 7 - ((my - layout.board_top) / layout.sq).floor() as u32;
            if file < 8 && rank < 8 {
                return Some(Square::from_coords(File::new(file), Rank::new(rank)));
            }
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
        let cx = layout.w as f32 / 2.0;
        let wk = self.images.get(Color::White, Role::King);
        let bk = self.images.get(Color::Black, Role::King);
        draw_scaled(buf, layout.w, layout.h, wk, cx - 260.0, 36.0, 130.0, 130.0);
        draw_scaled(buf, layout.w, layout.h, bk, cx + 130.0, 36.0, 130.0, 130.0);

        let title = "国际象棋";
        let tw = self.font.text_width(title, 64.0);
        self.font
            .draw_text(buf, layout.w, layout.h, cx - tw / 2.0, 210.0, title, pal.text, 64.0);
        let sub = "Rust · 本地窗口对弈";
        let sw = self.font.text_width(sub, 20.0);
        self.font
            .draw_text(buf, layout.w, layout.h, cx - sw / 2.0, 250.0, sub, pal.muted, 20.0);

        let btn_w = 320.0;
        let btn_h = 58.0;
        let x = cx - btn_w / 2.0;
        self.button(
            buf,
            layout,
            pal,
            view,
            x,
            330.0,
            btn_w,
            btn_h,
            "开始游戏",
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
            402.0,
            btn_w,
            btn_h,
            "游戏设置",
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
            474.0,
            btn_w,
            btn_h,
            "退出游戏",
            pal.button,
            22.0,
            actions,
            UiAction::ExitGame,
        );

        let footer = "素材来源：chess-viewer 开源项目 · CBurnett";
        let fw = self.font.text_width(footer, 14.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            cx - fw / 2.0,
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
        let mut any_base_clicked = false;

        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            56.0,
            158.0,
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
            348.0,
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
            158.0,
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
            348.0,
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
            538.0,
            dropdown_w,
            DropdownKind::Theme,
            dropdown_current(view, DropdownKind::Theme),
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
            "返回主菜单",
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
            mode_name(view.settings.mode).to_string(),
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
            depth_name(view.settings.ai_depth).to_string(),
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
                let x = layout.board_left + file as f32 * layout.sq;
                let y = layout.board_top + (7 - rank) as f32 * layout.sq;
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
            self.highlight_square(buf, layout, pal.last_move, 95, from);
            self.highlight_square(buf, layout, pal.last_move, 95, to);
        }
        if let Some(sq) = view.selected {
            self.highlight_square(buf, layout, pal.selected, 120, sq);
        }

        for &sq in &view.legal_targets {
            let (x, y) = square_rect(sq, layout);
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
                let (x, y) = square_rect(ksq, layout);
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
                let (x, y) = square_rect(sq, layout);
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
            let (fx, fy) = square_rect(anim.from, layout);
            let (tx, ty) = square_rect(anim.to, layout);
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

        for file in 0..8u32 {
            let letter = char::from(b'a' + file as u8);
            let x = layout.board_left + file as f32 * layout.sq + layout.sq - 20.0;
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
        for rank in 0..8u32 {
            let y = layout.board_top + (7 - rank) as f32 * layout.sq + 10.0;
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
    ) {
        let (x, y) = square_rect(sq, layout);
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
            .draw_text(buf, layout.w, layout.h, x, 34.0, "国际象棋", pal.text, 30.0);
        self.font
            .draw_text(buf, layout.w, layout.h, x, 58.0, "本地窗口 · Rust", pal.muted, 14.0);

        let btn_x = layout.panel_x + 14.0;
        let btn_w = layout.panel_w - 28.0;
        self.button(
            buf,
            layout,
            pal,
            view,
            btn_x,
            84.0,
            btn_w,
            42.0,
            "新对局",
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
            136.0,
            btn_w,
            42.0,
            "悔棋",
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
            188.0,
            btn_w,
            42.0,
            "主菜单",
            pal.button,
            18.0,
            actions,
            UiAction::BackToMenu,
        );

        self.font
            .draw_text(buf, layout.w, layout.h, x, 258.0, "对战模式", pal.muted, 16.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x,
            284.0,
            mode_name(view.settings.mode),
            pal.accent,
            18.0,
        );
        self.font
            .draw_text(buf, layout.w, layout.h, x, 320.0, "AI 难度", pal.muted, 16.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x,
            346.0,
            depth_name(view.settings.ai_depth),
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
            .draw_text(buf, layout.w, layout.h, x, 420.0, &status, status_color, 19.0);

        self.font
            .draw_text(buf, layout.w, layout.h, x, 466.0, "棋谱", pal.muted, 16.0);
        let start = view.history_sans.len().saturating_sub(11);
        for (i, san) in view.history_sans.iter().enumerate().skip(start) {
            let line = format!("{}. {}", i + 1, san);
            let y = 492.0 + (i - start) as f32 * 18.0;
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
        let sub = "本局结束";
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
            "再来一局",
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
            "返回主菜单",
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
            "选择升变棋子",
            pal.text,
            26.0,
        );

        let color = view.pos.turn();
        let roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
        let names = ["后", "车", "象", "马"];
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

fn square_rect(sq: Square, layout: &Layout) -> (f32, f32) {
    let file = u32::from(sq.file()) as f32;
    let rank = u32::from(sq.rank()) as f32;
    (
        layout.board_left + file * layout.sq,
        layout.board_top + (7.0 - rank) * layout.sq,
    )
}

fn king_square(pos: &Chess, color: Color) -> Option<Square> {
    (0..64u32)
        .map(Square::new)
        .find(|&sq| pos.board().piece_at(sq) == Some(Piece { color, role: Role::King }))
}

fn dropdown_options(view: &ViewState, kind: DropdownKind) -> Vec<(String, UiAction)> {
    match kind {
        DropdownKind::Mode => [
            ("双人对战", UiAction::Mode(None)),
            ("人机 · 执白", UiAction::Mode(Some(Color::White))),
            ("人机 · 执黑", UiAction::Mode(Some(Color::Black))),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect(),
        DropdownKind::Difficulty => [
            ("入门", UiAction::SetDifficulty(1)),
            ("简单", UiAction::SetDifficulty(2)),
            ("中等", UiAction::SetDifficulty(3)),
            ("困难", UiAction::SetDifficulty(4)),
            ("大师", UiAction::SetDifficulty(5)),
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
    }
}

fn dropdown_position(kind: DropdownKind) -> (f32, f32) {
    match kind {
        DropdownKind::Mode => (56.0, 158.0),
        DropdownKind::Difficulty => (56.0, 348.0),
        DropdownKind::Resolution => (520.0, 158.0),
        DropdownKind::Fps => (520.0, 348.0),
        DropdownKind::Theme => (56.0, 538.0),
    }
}

fn dropdown_current(view: &ViewState, kind: DropdownKind) -> String {
    match kind {
        DropdownKind::Mode => mode_name(view.settings.mode).to_string(),
        DropdownKind::Difficulty => depth_name(view.settings.ai_depth).to_string(),
        DropdownKind::Resolution => format!(
            "{} × {}",
            view.settings.resolution.0, view.settings.resolution.1
        ),
        DropdownKind::Fps => format!("{} Hz", view.settings.fps),
        DropdownKind::Theme => view.settings.theme.label().to_string(),
    }
}

fn status_text(view: &ViewState) -> String {
    if view.ai_thinking {
        return "电脑思考中…".to_string();
    }
    if let Some(outcome) = view.pos.outcome() {
        return match outcome {
            Outcome::Decisive { winner } => {
                let name = if winner == Color::White { "白方" } else { "黑方" };
                format!("将杀！{name}获胜")
            }
            Outcome::Draw => "和棋".to_string(),
        };
    }
    let turn = if view.pos.turn() == Color::White { "白方" } else { "黑方" };
    let check = if view.pos.is_check() { "（将军！）" } else { "" };
    format!("{turn}行棋{check}")
}

fn mode_name(mode: Option<Color>) -> &'static str {
    match mode {
        None => "双人对战",
        Some(Color::White) => "人机 · 执白",
        Some(Color::Black) => "人机 · 执黑",
    }
}

fn depth_name(depth: u32) -> &'static str {
    match depth {
        1 => "入门",
        2 => "简单",
        3 => "中等",
        4 => "困难",
        _ => "大师",
    }
}

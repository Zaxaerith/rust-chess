use shakmaty::{Chess, Color, File, Outcome, Piece, Position, Rank, Role, Square};

use crate::assets::{
    PieceImages, PieceSet, draw_arrow_down, draw_arrow_up, draw_ring, draw_scaled,
    draw_scaled_rotated, draw_scaled_tinted, fill_circle, fill_rect, fill_rect_alpha,
};
use crate::font::TextRenderer;
use crate::i18n::Language;
use crate::preferences::{
    AccessPolicy, AutoPromotion, AutoThreefold, BoardPreferences, CastlingMethod, ClockPosition,
    ClockTenths, DragTarget, PieceNotation, ZenMode,
};
use crate::theme::{BoardStyle, Palette, Theme};

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
    pub move_confirmation_pending: bool,
    pub screen: Screen,
    pub settings: Settings,
    pub animations: Vec<PieceAnimView>,
    pub resolutions: Vec<(u32, u32)>,
    pub refreshes: Vec<u32>,
    pub open_dropdown: Option<DropdownKind>,
    pub dropdown_scroll: usize,
    pub settings_page: SettingsPage,
    pub settings_scroll: usize,
    pub game_over_progress: f32,
    pub mouse: Option<(f32, f32)>,
    pub mouse_pressed: bool,
    pub mouse_down: bool,
    pub mouse_released: bool,
    pub dragging_from: Option<Square>,
    pub menu_time: f32,
    pub white_clock: f32,
    pub black_clock: f32,
    pub claimed_draw: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPage {
    Root,
    Board,
    Display,
    Behavior,
    Clock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceItem {
    BoardStyle,
    PieceSet,
    ZenMode,
    PieceNotation,
    Coordinates,
    MagnifyDraggedPiece,
    DragTarget,
    PieceAnimation,
    ImmersiveMode,
    PieceDestinations,
    BoardHighlights,
    ShowMoveList,
    ClockPosition,
    Premoves,
    Takebacks,
    AutoPromotion,
    AutoThreefold,
    MoveConfirmation,
    ConfirmResignDraw,
    CastlingMethod,
    ChessClockEnabled,
    GiveMoreTime,
    ClockWarning,
    ClockTenths,
}

const DISPLAY_PREFERENCES: [PreferenceItem; 13] = [
    PreferenceItem::BoardStyle,
    PreferenceItem::PieceSet,
    PreferenceItem::ZenMode,
    PreferenceItem::PieceNotation,
    PreferenceItem::Coordinates,
    PreferenceItem::MagnifyDraggedPiece,
    PreferenceItem::DragTarget,
    PreferenceItem::PieceAnimation,
    PreferenceItem::ImmersiveMode,
    PreferenceItem::PieceDestinations,
    PreferenceItem::BoardHighlights,
    PreferenceItem::ShowMoveList,
    PreferenceItem::ClockPosition,
];

const BEHAVIOR_PREFERENCES: [PreferenceItem; 7] = [
    PreferenceItem::Premoves,
    PreferenceItem::Takebacks,
    PreferenceItem::AutoPromotion,
    PreferenceItem::AutoThreefold,
    PreferenceItem::MoveConfirmation,
    PreferenceItem::ConfirmResignDraw,
    PreferenceItem::CastlingMethod,
];

const CLOCK_PREFERENCES: [PreferenceItem; 4] = [
    PreferenceItem::ChessClockEnabled,
    PreferenceItem::GiveMoreTime,
    PreferenceItem::ClockWarning,
    PreferenceItem::ClockTenths,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Option<Color>,
    pub ai_depth: u32,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub theme: Theme,
    pub board_style: BoardStyle,
    pub piece_set: PieceSet,
    pub language: Language,
    pub flip_for_black: bool,
    pub board: BoardPreferences,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: None,
            ai_depth: 2,
            resolution: (1280, 720),
            fps: 60,
            theme: Theme::DarkPlus,
            board_style: BoardStyle::Classic,
            piece_set: PieceSet::Cburnett,
            language: Language::Chinese,
            flip_for_black: true,
            board: BoardPreferences::default(),
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
    Preference(PreferenceItem),
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
    OpenSettingsPage(SettingsPage),
    SettingsBack,
    SetPreference(PreferenceItem, usize),
    NewGame,
    Undo,
    Hint,
    Mode(Option<Color>),
    PointerDown(Square),
    PointerUp(Option<Square>),
    ConfirmMove,
    CancelMove,
    AddClockTime(Color),
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
    fn new(w: usize, h: usize, focus_board: bool) -> Self {
        let panel_w = if focus_board { 0.0 } else { 280.0 };
        let margin = 24.0;
        let avail_w = (w as f32 - panel_w - margin * 2.0).max(320.0);
        let avail_h = (h as f32 - 80.0).max(320.0);
        let board_size = avail_w.min(avail_h).min(1000.0).max(320.0);
        let board_left = (w as f32 - panel_w - margin * 2.0 - board_size) / 2.0 + margin;
        let board_top = (h as f32 - board_size) / 2.0;
        let panel_x = if focus_board {
            w as f32
        } else {
            w as f32 - panel_w - margin
        };
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
        let focus_board = view.screen == Screen::Game
            && (view.settings.board.immersive_mode || view.settings.board.zen_mode != ZenMode::No);
        let layout = Layout::new(width, height, focus_board);
        let pal = view.settings.theme.palette();
        let mut actions = Vec::new();
        fill_rect(
            buf,
            width,
            height,
            0,
            0,
            width as i32,
            height as i32,
            pal.bg,
        );
        match view.screen {
            Screen::Menu => self.draw_menu(buf, &layout, &pal, view, &mut actions),
            Screen::Settings => self.draw_settings(buf, &layout, &pal, view, &mut actions),
            Screen::Game => {
                self.draw_board(buf, &layout, &pal, view);
                if !focus_board {
                    self.draw_panel(buf, &layout, &pal, view, &mut actions);
                    if view.settings.board.chess_clock_enabled {
                        self.draw_clocks(buf, &layout, &pal, view, &mut actions);
                    }
                } else {
                    self.button(
                        buf,
                        &layout,
                        &pal,
                        view,
                        layout.w as f32 - 148.0,
                        12.0,
                        132.0,
                        36.0,
                        view.settings.language.text().menu,
                        pal.button,
                        17.0,
                        &mut actions,
                        UiAction::BackToMenu,
                    );
                }
                if view.move_confirmation_pending {
                    self.draw_move_confirmation(buf, &layout, &pal, view, &mut actions);
                } else if view.promotion.is_some() {
                    self.draw_promotion_dialog(buf, &layout, &pal, view, &mut actions);
                } else if let Some((mx, my)) = view.mouse {
                    if view.mouse_pressed && actions.is_empty() {
                        if let Some(sq) = self.square_at(mx, my, &layout, board_flipped(view)) {
                            actions.push(UiAction::PointerDown(sq));
                        }
                    }
                    if view.mouse_released && actions.is_empty() {
                        actions.push(UiAction::PointerUp(self.square_at(
                            mx,
                            my,
                            &layout,
                            board_flipped(view),
                        )));
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
        let title_vertical_stretch = 1.22;
        let title_tracking = 3.5;
        let title_baseline = gear_y + gear_size + 18.0;
        let tw = self
            .font
            .serif_text_width(title, title_size, title_stretch, title_tracking);
        self.font.draw_serif_text(
            buf,
            layout.w,
            layout.h,
            logo_cx - tw / 2.0,
            title_baseline,
            title,
            pal.text,
            title_size,
            title_stretch,
            title_vertical_stretch,
            title_tracking,
        );
        let line_w = tw * 0.86;
        fill_rect(
            buf,
            layout.w,
            layout.h,
            (logo_cx - line_w / 2.0) as i32,
            (title_baseline + 9.0) as i32,
            line_w as i32,
            3,
            0x0505_05,
        );
        let sub = tr.subtitle;
        let sw = self.font.text_width(sub, 18.0);
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            logo_cx - sw / 2.0,
            title_baseline + 37.0,
            sub,
            pal.muted,
            18.0,
        );

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
        match view.settings_page {
            SettingsPage::Root => self.draw_settings_root(buf, layout, pal, view, actions),
            SettingsPage::Board => self.draw_board_settings_hub(buf, layout, pal, view, actions),
            SettingsPage::Display => self.draw_preference_page(
                buf,
                layout,
                pal,
                view,
                actions,
                SettingsPage::Display,
                &DISPLAY_PREFERENCES,
            ),
            SettingsPage::Behavior => self.draw_preference_page(
                buf,
                layout,
                pal,
                view,
                actions,
                SettingsPage::Behavior,
                &BEHAVIOR_PREFERENCES,
            ),
            SettingsPage::Clock => self.draw_preference_page(
                buf,
                layout,
                pal,
                view,
                actions,
                SettingsPage::Clock,
                &CLOCK_PREFERENCES,
            ),
        }
    }

    fn draw_settings_root(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let tr = view.settings.language.text();
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            56.0,
            66.0,
            tr.settings,
            pal.text,
            36.0,
        );

        let (left_x, right_x, dropdown_w, first_label_y, row_gap) = settings_geometry(layout);
        let label_y = |row: usize| first_label_y + row as f32 * row_gap;
        let base_y = |row: usize| label_y(row) + 14.0;

        for (x, y, label) in [
            (left_x, label_y(0), tr.mode),
            (right_x, label_y(0), tr.resolution),
            (left_x, label_y(1), tr.difficulty),
            (right_x, label_y(1), tr.refresh_rate),
            (left_x, label_y(2), tr.theme),
            (right_x, label_y(2), tr.language),
            (left_x, label_y(3), tr.board_view),
        ] {
            self.font
                .draw_text(buf, layout.w, layout.h, x, y, label, pal.muted, 17.0);
        }

        let mut any_base_clicked = false;

        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            left_x,
            base_y(0),
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
            left_x,
            base_y(1),
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
            base_y(0),
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
            base_y(1),
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
            left_x,
            base_y(2),
            dropdown_w,
            DropdownKind::Theme,
            dropdown_current(view, DropdownKind::Theme),
        );
        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            right_x,
            base_y(2),
            dropdown_w,
            DropdownKind::Language,
            dropdown_current(view, DropdownKind::Language),
        );
        any_base_clicked |= self.dropdown_base(
            buf,
            layout,
            pal,
            view,
            actions,
            left_x,
            base_y(3),
            dropdown_w,
            DropdownKind::BoardView,
            dropdown_current(view, DropdownKind::BoardView),
        );
        any_base_clicked |= self.settings_nav_row(
            buf,
            layout,
            pal,
            view,
            actions,
            right_x,
            base_y(3),
            dropdown_w,
            board_text(view.settings.language, BoardText::BoardSettings),
            UiAction::OpenSettingsPage(SettingsPage::Board),
        );

        let back_w = 260.0;
        let back_x = (layout.w as f32 - back_w) / 2.0;
        let back_y = settings_back_y(layout);
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
            UiAction::SettingsBack,
        );

        let mut list_handled = false;
        if let Some(kind) = view.open_dropdown {
            let options = dropdown_options(view, kind);
            let (ox, oy) = dropdown_position(layout, kind);
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

    fn draw_board_settings_hub(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let language = view.settings.language;
        self.settings_heading(
            buf,
            layout,
            pal,
            board_text(language, BoardText::BoardSettings),
            board_text(language, BoardText::ChooseCategory),
        );

        let width = (layout.w as f32 - 112.0).min(760.0);
        let x = (layout.w as f32 - width) / 2.0;
        for (index, (label, page)) in [
            (
                board_text(language, BoardText::Display),
                SettingsPage::Display,
            ),
            (
                board_text(language, BoardText::Behavior),
                SettingsPage::Behavior,
            ),
            (
                board_text(language, BoardText::ChessClock),
                SettingsPage::Clock,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            self.settings_nav_row(
                buf,
                layout,
                pal,
                view,
                actions,
                x,
                132.0 + index as f32 * 68.0,
                width,
                label,
                UiAction::OpenSettingsPage(page),
            );
        }

        self.settings_back_button(buf, layout, pal, view, actions);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_preference_page(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
        page: SettingsPage,
        items: &[PreferenceItem],
    ) {
        let language = view.settings.language;
        let title = match page {
            SettingsPage::Display => board_text(language, BoardText::Display),
            SettingsPage::Behavior => board_text(language, BoardText::Behavior),
            SettingsPage::Clock => board_text(language, BoardText::ChessClock),
            _ => board_text(language, BoardText::BoardSettings),
        };
        self.settings_heading(
            buf,
            layout,
            pal,
            title,
            board_text(language, BoardText::ClickToChange),
        );

        let row_h = 47.0;
        let top = 105.0;
        let bottom = 76.0;
        let visible = (((layout.h as f32 - top - bottom) / row_h).floor() as usize)
            .clamp(1, items.len().max(1));
        let start = view
            .settings_scroll
            .min(items.len().saturating_sub(visible));
        let end = (start + visible).min(items.len());
        let width = (layout.w as f32 - 72.0).min(980.0);
        let x = (layout.w as f32 - width) / 2.0;
        let mut base_handled = false;

        for (visible_index, item) in items[start..end].iter().enumerate() {
            let y = top + visible_index as f32 * row_h;
            let kind = DropdownKind::Preference(*item);
            let open = view.open_dropdown == Some(kind);
            let hover = view.mouse.is_some_and(|(mx, my)| {
                mx >= x && mx <= x + width && my >= y && my <= y + row_h - 3.0
            });
            fill_rect(
                buf,
                layout.w,
                layout.h,
                x as i32,
                y as i32,
                width as i32,
                (row_h - 3.0) as i32,
                if open {
                    pal.button_active
                } else if hover {
                    pal.button_hover
                } else {
                    pal.button
                },
            );
            fill_rect(
                buf,
                layout.w,
                layout.h,
                x as i32,
                (y + row_h - 5.0) as i32,
                width as i32,
                2,
                pal.border,
            );
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + 14.0,
                y + 29.0,
                preference_label(*item, language),
                pal.text,
                17.0,
            );
            let value = preference_value(*item, &view.settings, language);
            let value_width = self.font.text_width(&value, 16.0);
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                (x + width - value_width - 34.0).max(x + width * 0.52),
                y + 29.0,
                &value,
                pal.accent,
                16.0,
            );
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + width - 20.0,
                y + 29.0,
                if open { "^" } else { "v" },
                pal.muted,
                16.0,
            );
            if hover
                && view.mouse_pressed
                && (view.open_dropdown.is_none() || view.open_dropdown == Some(kind))
            {
                actions.push(UiAction::ToggleDropdown(kind));
                base_handled = true;
            }
        }

        let mut list_handled = false;
        if let Some(DropdownKind::Preference(item)) = view.open_dropdown {
            if let Some(visible_index) = items[start..end].iter().position(|entry| *entry == item) {
                let y = top + visible_index as f32 * row_h;
                let choice_width = (width * 0.48).clamp(240.0, 420.0);
                let choice_x = x + width - choice_width;
                let options = preference_options(item, &view.settings, language);
                list_handled = self.dropdown_list(
                    buf,
                    layout,
                    pal,
                    view,
                    actions,
                    choice_x,
                    y,
                    choice_width,
                    &options,
                    view.dropdown_scroll,
                );
            }
        }

        if !base_handled
            && !list_handled
            && matches!(view.open_dropdown, Some(DropdownKind::Preference(_)))
            && view.mouse_pressed
        {
            actions.push(UiAction::ToggleDropdown(view.open_dropdown.unwrap()));
        }

        if items.len() > visible {
            let counter = format!("{}–{} / {}", start + 1, end, items.len());
            let counter_w = self.font.text_width(&counter, 14.0);
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                layout.w as f32 - counter_w - 28.0,
                layout.h as f32 - 54.0,
                &counter,
                pal.muted,
                14.0,
            );
        }
        self.settings_back_button(buf, layout, pal, view, actions);
    }

    fn settings_heading(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        title: &str,
        subtitle: &str,
    ) {
        self.font
            .draw_text(buf, layout.w, layout.h, 42.0, 50.0, title, pal.text, 31.0);
        self.font.draw_text(
            buf, layout.w, layout.h, 43.0, 78.0, subtitle, pal.muted, 15.0,
        );
    }

    fn settings_back_button(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        self.button(
            buf,
            layout,
            pal,
            view,
            24.0,
            layout.h as f32 - 58.0,
            176.0,
            40.0,
            board_text(view.settings.language, BoardText::Back),
            pal.button,
            18.0,
            actions,
            UiAction::SettingsBack,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn settings_nav_row(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
        x: f32,
        y: f32,
        width: f32,
        label: &str,
        action: UiAction,
    ) -> bool {
        let height = 48.0;
        let hover = view
            .mouse
            .is_some_and(|(mx, my)| mx >= x && mx <= x + width && my >= y && my <= y + height);
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            y as i32,
            width as i32,
            height as i32,
            if hover { pal.button_hover } else { pal.button },
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            (y + height - 2.0) as i32,
            width as i32,
            2,
            pal.border,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + 14.0,
            y + 31.0,
            label,
            pal.text,
            18.0,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + width - 20.0,
            y + 31.0,
            ">",
            pal.accent,
            18.0,
        );
        if hover && view.mouse_pressed {
            actions.push(action);
            true
        } else {
            false
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
        let hover = view.mouse.map_or(false, |(mx, my)| {
            mx >= x && mx <= x + w && my >= y && my <= y + h
        });
        let color = if hover { pal.button_hover } else { pal.button };
        fill_rect(
            buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, color,
        );
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
        draw_arrow_down(
            buf,
            layout.w,
            layout.h,
            x + w - 16.0,
            y + h / 2.0,
            7.0,
            pal.accent,
        );
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
                if row_hover {
                    pal.button_hover
                } else {
                    pal.panel
                },
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
            draw_arrow_up(
                buf,
                layout.w,
                layout.h,
                x + w / 2.0,
                list_y + 12.0,
                8.0,
                pal.accent,
            );
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
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            56.0,
            76.0,
            "游戏设置",
            pal.text,
            40.0,
        );

        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            56.0,
            140.0,
            "对战模式",
            pal.muted,
            18.0,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            56.0,
            330.0,
            "AI 难度",
            pal.muted,
            18.0,
        );
        let right_x = 520.0;
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            right_x,
            140.0,
            "窗口分辨率",
            pal.muted,
            18.0,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            right_x,
            330.0,
            "刷新率",
            pal.muted,
            18.0,
        );
        self.font.draw_text(
            buf, layout.w, layout.h, 56.0, 520.0, "主题", pal.muted, 18.0,
        );

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
        let hover = view.mouse.map_or(false, |(mx, my)| {
            mx >= x && mx <= x + w && my >= y && my <= y + h
        });
        let base_color = if hover { pal.button_hover } else { pal.button };
        fill_rect(
            buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, base_color,
        );
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
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + w - 24.0,
            y + h / 2.0 + 6.0,
            "▾",
            pal.accent,
            17.0,
        );

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
                if row_hover {
                    pal.button_hover
                } else {
                    pal.panel
                },
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

    fn draw_clocks(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let width = 220.0;
        let x = match view.settings.board.clock_position {
            ClockPosition::Left => layout.panel_x + 14.0,
            ClockPosition::Right => layout.panel_x + layout.panel_w - width - 14.0,
        };
        let clock_top = if layout.h < 600 { 282.0 } else { 394.0 };
        for (color, seconds, y) in [
            (Color::Black, view.black_clock, clock_top),
            (Color::White, view.white_clock, clock_top + 40.0),
        ] {
            let active = view.pos.turn() == color;
            fill_rect_alpha(
                buf,
                layout.w,
                layout.h,
                x as i32,
                y as i32,
                width as i32,
                34,
                if active { pal.button_active } else { pal.panel },
                225,
            );
            let label = format!(
                "{} {}",
                if color == Color::White { "W" } else { "B" },
                format_clock(seconds, view.settings.board.clock_tenths)
            );
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x + 9.0,
                y + 23.0,
                &label,
                if seconds <= 30.0 {
                    pal.accent
                } else {
                    pal.text
                },
                16.0,
            );
            if view.settings.board.give_more_time != AccessPolicy::Never {
                let add_x = x + width - 42.0;
                let hover = view.mouse.is_some_and(|(mx, my)| {
                    mx >= add_x && mx <= add_x + 38.0 && my >= y + 3.0 && my <= y + 31.0
                });
                fill_rect(
                    buf,
                    layout.w,
                    layout.h,
                    add_x as i32,
                    (y + 3.0) as i32,
                    38,
                    28,
                    if hover { pal.button_hover } else { pal.button },
                );
                self.font.draw_text(
                    buf,
                    layout.w,
                    layout.h,
                    add_x + 5.0,
                    y + 22.0,
                    "+15",
                    pal.accent,
                    14.0,
                );
                if hover && view.mouse_pressed {
                    actions.push(UiAction::AddClockTime(color));
                }
            }
        }
    }

    fn draw_board(&self, buf: &mut [u32], layout: &Layout, pal: &Palette, view: &ViewState) {
        let flipped = board_flipped(view);
        let (light_square, dark_square) = view.settings.board_style.squares(pal);
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
                    light_square
                } else {
                    dark_square
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

        if view.settings.board.board_highlights {
            if let Some((from, to)) = view.last_move {
                self.highlight_square(buf, layout, pal.last_move, 95, from, flipped);
                self.highlight_square(buf, layout, pal.last_move, 95, to, flipped);
            }
        }
        if let Some(sq) = view.selected {
            self.highlight_square(buf, layout, pal.selected, 120, sq, flipped);
        }
        if let Some((from, to, _)) = &view.suggestion {
            self.highlight_square(buf, layout, pal.accent, 105, *from, flipped);
            self.highlight_square(buf, layout, pal.accent, 150, *to, flipped);
        }

        if view.settings.board.piece_destinations {
            for &sq in &view.legal_targets {
                let (x, y) = square_rect(sq, layout, flipped);
                let cx = x + layout.sq / 2.0;
                let cy = y + layout.sq / 2.0;
                match view.settings.board.drag_target {
                    DragTarget::None => {}
                    DragTarget::Square => fill_rect_alpha(
                        buf,
                        layout.w,
                        layout.h,
                        (x + 7.0) as i32,
                        (y + 7.0) as i32,
                        (layout.sq - 14.0) as i32,
                        (layout.sq - 14.0) as i32,
                        pal.move_dot,
                        125,
                    ),
                    DragTarget::Circle => {
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
                            fill_circle(buf, layout.w, layout.h, cx, cy, 12.0, pal.move_dot, 210);
                        }
                    }
                }
            }
        }

        if view.settings.board.board_highlights && view.pos.is_check() {
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
                if view.mouse_down && view.dragging_from == Some(sq) {
                    continue;
                }
                if view.animations.iter().any(|a| a.to == sq) {
                    continue;
                }
                let tex = self
                    .images
                    .get(view.settings.piece_set, piece.color, piece.role);
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
            let tex = self
                .images
                .get(view.settings.piece_set, anim.color, anim.role);
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

        if view.mouse_down {
            if let (Some(from), Some((mx, my))) = (view.dragging_from, view.mouse) {
                if let Some(piece) = view.pos.board().piece_at(from) {
                    let scale = if view.settings.board.magnify_dragged_piece {
                        1.28
                    } else {
                        1.0
                    };
                    let size = (layout.sq - 8.0) * scale;
                    let texture = self
                        .images
                        .get(view.settings.piece_set, piece.color, piece.role);
                    draw_scaled(
                        buf,
                        layout.w,
                        layout.h,
                        texture,
                        mx - size / 2.0,
                        my - size / 2.0,
                        size,
                        size,
                    );
                }
            }
        }

        if view.settings.board.coordinates {
            for screen_file in 0..8u32 {
                let file = if flipped {
                    7 - screen_file
                } else {
                    screen_file
                };
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
                let rank = if flipped {
                    screen_rank
                } else {
                    7 - screen_rank
                };
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
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x,
            58.0,
            tr.subtitle,
            pal.muted,
            14.0,
        );

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
            if view.hint_thinking {
                tr.hint_thinking
            } else {
                tr.hint
            },
            if view.hint_thinking {
                pal.button_active
            } else {
                pal.button
            },
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

        if !(view.settings.board.chess_clock_enabled && layout.h < 600) {
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
            self.font.draw_text(
                buf,
                layout.w,
                layout.h,
                x,
                340.0,
                tr.difficulty,
                pal.muted,
                16.0,
            );
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
        }

        let status = status_text(view);
        let status_color = if view.pos.is_check() && !view.ai_thinking {
            pal.accent
        } else {
            pal.text
        };
        let (status_y, hint_y, moves_y, history_y, history_count) =
            if view.settings.board.chess_clock_enabled && layout.h < 600 {
                (374.0, 402.0, 430.0, 454.0, 1)
            } else if view.settings.board.chess_clock_enabled {
                (494.0, 524.0, 554.0, 578.0, 8)
            } else {
                (410.0, 442.0, 474.0, 498.0, 11)
            };
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x,
            status_y,
            &status,
            status_color,
            18.0,
        );

        if let Some((_, _, san)) = &view.suggestion {
            let hint = format!("{}: {}", tr.suggested, san);
            self.font
                .draw_text(buf, layout.w, layout.h, x, hint_y, &hint, pal.accent, 17.0);
        }

        if view.settings.board.show_move_list {
            self.font.draw_text(
                buf, layout.w, layout.h, x, moves_y, tr.moves, pal.muted, 16.0,
            );
            let start = view.history_sans.len().saturating_sub(history_count);
            for (i, san) in view.history_sans.iter().enumerate().skip(start) {
                let notation = move_notation(san, view.settings.board.piece_notation);
                let line = format!("{}. {}", i + 1, notation);
                let y = history_y + (i - start) as f32 * 18.0;
                if y < layout.h as f32 - 12.0 {
                    self.font
                        .draw_text(buf, layout.w, layout.h, x, y, &line, pal.history, 15.0);
                }
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
        let hover = view.mouse.map_or(false, |(mx, my)| {
            mx >= x && mx <= x + w && my >= y && my <= y + h
        });
        let color = if hover {
            if base_color == pal.button_active {
                pal.button_active_hover
            } else {
                pal.button_hover
            }
        } else {
            base_color
        };
        fill_rect(
            buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, color,
        );
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
        let hover = view.mouse.map_or(false, |(mx, my)| {
            mx >= x && mx <= x + w && my >= y && my <= y + h
        });
        let color = if hover { pal.button_hover } else { base_color };
        fill_rect(
            buf, layout.w, layout.h, x as i32, y as i32, w as i32, h as i32, color,
        );
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

    fn draw_move_confirmation(
        &self,
        buf: &mut [u32],
        layout: &Layout,
        pal: &Palette,
        view: &ViewState,
        actions: &mut Vec<UiAction>,
    ) {
        let width = 360.0;
        let height = 146.0;
        let x = (layout.w as f32 - width) / 2.0;
        let y = (layout.h as f32 - height) / 2.0;
        fill_rect_alpha(
            buf,
            layout.w,
            layout.h,
            0,
            0,
            layout.w as i32,
            layout.h as i32,
            0x0000_00,
            125,
        );
        fill_rect(
            buf,
            layout.w,
            layout.h,
            x as i32,
            y as i32,
            width as i32,
            height as i32,
            pal.panel,
        );
        self.font.draw_text(
            buf,
            layout.w,
            layout.h,
            x + 24.0,
            y + 40.0,
            board_text(view.settings.language, BoardText::ConfirmMove),
            pal.text,
            23.0,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            x + 20.0,
            y + 76.0,
            150.0,
            44.0,
            board_text(view.settings.language, BoardText::Cancel),
            pal.button,
            18.0,
            actions,
            UiAction::CancelMove,
        );
        self.button(
            buf,
            layout,
            pal,
            view,
            x + 190.0,
            y + 76.0,
            150.0,
            44.0,
            board_text(view.settings.language, BoardText::Confirm),
            pal.button_active,
            18.0,
            actions,
            UiAction::ConfirmMove,
        );
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
            let tex = self.images.get(view.settings.piece_set, color, *role);
            draw_scaled(
                buf,
                layout.w,
                layout.h,
                tex,
                x + 14.0,
                y + 8.0,
                bw - 28.0,
                84.0,
            );
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
    let file = if flipped {
        7 - screen_file
    } else {
        screen_file
    };
    let rank = if flipped {
        screen_rank
    } else {
        7 - screen_rank
    };
    Some(Square::from_coords(File::new(file), Rank::new(rank)))
}

fn king_square(pos: &Chess, color: Color) -> Option<Square> {
    (0..64u32).map(Square::new).find(|&sq| {
        pos.board().piece_at(sq)
            == Some(Piece {
                color,
                role: Role::King,
            })
    })
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
            .map(|&language| {
                (
                    language.native_name().to_string(),
                    UiAction::SetLanguage(language),
                )
            })
            .collect(),
        DropdownKind::BoardView => [
            (tr.flip_on, UiAction::SetFlipForBlack(true)),
            (tr.flip_off, UiAction::SetFlipForBlack(false)),
        ]
        .iter()
        .map(|(label, action)| ((*label).to_string(), *action))
        .collect(),
        DropdownKind::Preference(item) => {
            preference_options(item, &view.settings, view.settings.language)
        }
    }
}

fn settings_geometry(layout: &Layout) -> (f32, f32, f32, f32, f32) {
    let left_x = 56.0_f32.min((layout.w as f32 * 0.07).max(24.0));
    let column_gap = (layout.w as f32 * 0.035).clamp(24.0, 44.0);
    let dropdown_w = ((layout.w as f32 - left_x * 2.0 - column_gap) / 2.0).clamp(220.0, 420.0);
    let right_x = left_x + dropdown_w + column_gap;
    let first_label_y = if layout.h < 600 { 82.0 } else { 102.0 };
    let available_gap = (layout.h as f32 - first_label_y - 130.0) / 4.0;
    let row_gap = available_gap.clamp(58.0, 82.0);
    (left_x, right_x, dropdown_w, first_label_y, row_gap)
}

#[derive(Clone, Copy)]
enum BoardText {
    BoardSettings,
    ChooseCategory,
    Display,
    Behavior,
    ChessClock,
    ClickToChange,
    Back,
    ConfirmMove,
    Confirm,
    Cancel,
}

fn localized(
    language: Language,
    simplified: &'static str,
    traditional: &'static str,
    english: &'static str,
) -> &'static str {
    match language {
        Language::Chinese => simplified,
        Language::TraditionalChinese => traditional,
        _ => english,
    }
}

fn board_text(language: Language, text: BoardText) -> &'static str {
    match text {
        BoardText::BoardSettings => localized(language, "棋盘设置", "棋盤設定", "Board settings"),
        BoardText::ChooseCategory => localized(
            language,
            "选择一个分类以调整棋盘与对局体验",
            "選擇一個分類以調整棋盤與對局體驗",
            "Choose a category to customize the board and game experience",
        ),
        BoardText::Display => localized(language, "界面设置", "介面設定", "Display"),
        BoardText::Behavior => localized(language, "对局行为", "對局行為", "Game behavior"),
        BoardText::ChessClock => localized(language, "棋钟", "棋鐘", "Chess clock"),
        BoardText::ClickToChange => localized(
            language,
            "点击选项切换设置；滚轮可浏览更多项目",
            "點擊選項切換設定；滾輪可瀏覽更多項目",
            "Click an item to change it; use the wheel to see more",
        ),
        BoardText::Back => localized(language, "返回上一级", "返回上一級", "Back"),
        BoardText::ConfirmMove => localized(
            language,
            "确认这一步着法？",
            "確認這一步著法？",
            "Confirm this move?",
        ),
        BoardText::Confirm => localized(language, "确认", "確認", "Confirm"),
        BoardText::Cancel => localized(language, "取消", "取消", "Cancel"),
    }
}

fn preference_label(item: PreferenceItem, language: Language) -> &'static str {
    if item == PreferenceItem::BoardStyle {
        return language.text().board_skin;
    }
    if item == PreferenceItem::PieceSet {
        return language.text().piece_skin;
    }
    let (zh, tw, en) = match item {
        PreferenceItem::BoardStyle | PreferenceItem::PieceSet => unreachable!(),
        PreferenceItem::ZenMode => ("禅意模式", "禪意模式", "Zen mode"),
        PreferenceItem::PieceNotation => ("记谱模式", "記譜模式", "Move notation"),
        PreferenceItem::Coordinates => ("棋盘坐标", "棋盤座標", "Board coordinates"),
        PreferenceItem::MagnifyDraggedPiece => {
            ("放大拖动的棋子", "放大拖動的棋子", "Magnify dragged piece")
        }
        PreferenceItem::DragTarget => (
            "拖动棋子落点标记",
            "拖動棋子落點標記",
            "Dragged piece target",
        ),
        PreferenceItem::PieceAnimation => ("棋子动画", "棋子動畫", "Piece animation"),
        PreferenceItem::ImmersiveMode => ("沉浸模式", "沉浸模式", "Immersive mode"),
        PreferenceItem::PieceDestinations => (
            "棋子落点（有效走法与预走棋）",
            "棋子落點（有效走法與預走棋）",
            "Piece destinations",
        ),
        PreferenceItem::BoardHighlights => (
            "棋盘高亮（最后一步与将军）",
            "棋盤高亮（最後一步與將軍）",
            "Board highlights",
        ),
        PreferenceItem::ShowMoveList => (
            "对局时显示可走着法",
            "對局時顯示可走著法",
            "Show move list while playing",
        ),
        PreferenceItem::ClockPosition => ("棋钟位置", "棋鐘位置", "Clock position"),
        PreferenceItem::Premoves => ("预走棋", "預走棋", "Premoves"),
        PreferenceItem::Takebacks => (
            "悔棋选项（需对手同意）",
            "悔棋選項（需對手同意）",
            "Takebacks (with opponent approval)",
        ),
        PreferenceItem::AutoPromotion => {
            ("自动升变后", "自動升變后", "Promote to queen automatically")
        }
        PreferenceItem::AutoThreefold => (
            "三次重复局面自动提和",
            "三次重複局面自動提和",
            "Claim threefold draw automatically",
        ),
        PreferenceItem::MoveConfirmation => ("着法确认", "著法確認", "Move confirmation"),
        PreferenceItem::ConfirmResignDraw => (
            "确认认输和提和请求",
            "確認認輸和提和請求",
            "Confirm resignation and draw offers",
        ),
        PreferenceItem::CastlingMethod => ("王车易位方式", "王車易位方式", "Castling method"),
        PreferenceItem::ChessClockEnabled => ("启用棋钟", "啟用棋鐘", "Enable chess clock"),
        PreferenceItem::GiveMoreTime => ("给对方更多时间", "給對方更多時間", "Give more time"),
        PreferenceItem::ClockWarning => (
            "时间不足时声音提醒",
            "時間不足時聲音提醒",
            "Sound when time gets critical",
        ),
        PreferenceItem::ClockTenths => ("显示十分之一秒", "顯示十分之一秒", "Tenths of seconds"),
    };
    localized(language, zh, tw, en)
}

fn preference_value(item: PreferenceItem, settings: &Settings, language: Language) -> String {
    let board = settings.board;
    let yes_no = |enabled| {
        localized(
            language,
            if enabled { "是" } else { "否" },
            if enabled { "是" } else { "否" },
            if enabled { "On" } else { "Off" },
        )
        .to_string()
    };
    let value = match item {
        PreferenceItem::BoardStyle => return settings.board_style.label().to_string(),
        PreferenceItem::PieceSet => return settings.piece_set.label().to_string(),
        PreferenceItem::ZenMode => match board.zen_mode {
            ZenMode::No => localized(language, "否", "否", "No"),
            ZenMode::Yes => localized(language, "是", "是", "Yes"),
            ZenMode::GameOnly => localized(language, "仅在对局中", "僅在對局中", "In game only"),
        },
        PreferenceItem::PieceNotation => match board.piece_notation {
            PieceNotation::Symbols => localized(language, "棋子符号", "棋子符號", "Piece symbols"),
            PieceNotation::Letters => localized(language, "字母", "字母", "Letters"),
        },
        PreferenceItem::Coordinates => return yes_no(board.coordinates),
        PreferenceItem::MagnifyDraggedPiece => return yes_no(board.magnify_dragged_piece),
        PreferenceItem::DragTarget => match board.drag_target {
            DragTarget::Circle => localized(language, "圆形", "圓形", "Circle"),
            DragTarget::Square => localized(language, "方形", "方形", "Square"),
            DragTarget::None => localized(language, "无", "無", "None"),
        },
        PreferenceItem::PieceAnimation => return yes_no(board.piece_animation),
        PreferenceItem::ImmersiveMode => return yes_no(board.immersive_mode),
        PreferenceItem::PieceDestinations => return yes_no(board.piece_destinations),
        PreferenceItem::BoardHighlights => return yes_no(board.board_highlights),
        PreferenceItem::ShowMoveList => return yes_no(board.show_move_list),
        PreferenceItem::ClockPosition => match board.clock_position {
            ClockPosition::Left => localized(language, "左侧", "左側", "Left"),
            ClockPosition::Right => localized(language, "右侧", "右側", "Right"),
        },
        PreferenceItem::Premoves => return yes_no(board.premoves),
        PreferenceItem::Takebacks => access_value(board.takebacks, language),
        PreferenceItem::AutoPromotion => match board.auto_promotion {
            AutoPromotion::Never => localized(language, "从不", "從不", "Never"),
            AutoPromotion::Premove => localized(language, "预走棋时", "預走棋時", "On premove"),
            AutoPromotion::Always => localized(language, "总是", "總是", "Always"),
        },
        PreferenceItem::AutoThreefold => match board.auto_threefold {
            AutoThreefold::Always => localized(language, "总是", "總是", "Always"),
            AutoThreefold::Never => localized(language, "从不", "從不", "Never"),
            AutoThreefold::UnderThirtySeconds => localized(
                language,
                "剩余时间小于30秒时",
                "剩餘時間小於30秒時",
                "When under 30 seconds",
            ),
        },
        PreferenceItem::MoveConfirmation => return yes_no(board.move_confirmation),
        PreferenceItem::ConfirmResignDraw => return yes_no(board.confirm_resign_draw),
        PreferenceItem::CastlingMethod => match board.castling_method {
            CastlingMethod::KingOntoRook => localized(
                language,
                "将王移到车上",
                "將王移到車上",
                "Move king onto rook",
            ),
            CastlingMethod::KingTwoSquares => localized(
                language,
                "将王移动两格",
                "將王移動兩格",
                "Move king two squares",
            ),
        },
        PreferenceItem::ChessClockEnabled => return yes_no(board.chess_clock_enabled),
        PreferenceItem::GiveMoreTime => access_value(board.give_more_time, language),
        PreferenceItem::ClockWarning => return yes_no(board.clock_warning),
        PreferenceItem::ClockTenths => match board.clock_tenths {
            ClockTenths::Never => localized(language, "从不", "從不", "Never"),
            ClockTenths::UnderTenSeconds => localized(
                language,
                "剩余时间小于10秒时",
                "剩餘時間小於10秒時",
                "When under 10 seconds",
            ),
            ClockTenths::Always => localized(language, "总是", "總是", "Always"),
        },
    };
    value.to_string()
}

fn preference_options(
    item: PreferenceItem,
    settings: &Settings,
    language: Language,
) -> Vec<(String, UiAction)> {
    let labels: Vec<String> = match item {
        PreferenceItem::BoardStyle => BoardStyle::ALL
            .iter()
            .map(|style| style.label().to_string())
            .collect(),
        PreferenceItem::PieceSet => PieceSet::ALL
            .iter()
            .map(|set| set.label().to_string())
            .collect(),
        PreferenceItem::ZenMode => [
            localized(language, "否", "否", "No"),
            localized(language, "是", "是", "Yes"),
            localized(language, "仅在对局中", "僅在對局中", "In game only"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::PieceNotation => [
            localized(language, "棋子符号", "棋子符號", "Piece symbols"),
            localized(language, "字母", "字母", "Letters"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::Coordinates
        | PreferenceItem::MagnifyDraggedPiece
        | PreferenceItem::PieceAnimation
        | PreferenceItem::ImmersiveMode
        | PreferenceItem::PieceDestinations
        | PreferenceItem::BoardHighlights
        | PreferenceItem::ShowMoveList
        | PreferenceItem::Premoves
        | PreferenceItem::MoveConfirmation
        | PreferenceItem::ConfirmResignDraw
        | PreferenceItem::ChessClockEnabled
        | PreferenceItem::ClockWarning => [
            localized(language, "否", "否", "Off"),
            localized(language, "是", "是", "On"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::DragTarget => [
            localized(language, "圆形", "圓形", "Circle"),
            localized(language, "方形", "方形", "Square"),
            localized(language, "无", "無", "None"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::ClockPosition => [
            localized(language, "左侧", "左側", "Left"),
            localized(language, "右侧", "右側", "Right"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::Takebacks | PreferenceItem::GiveMoreTime => [
            localized(language, "从不", "從不", "Never"),
            localized(
                language,
                "仅限休闲对局",
                "僅限休閒對局",
                "Casual games only",
            ),
            localized(language, "总是", "總是", "Always"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::AutoPromotion => [
            localized(language, "从不", "從不", "Never"),
            localized(language, "预走棋时", "預走棋時", "On premove"),
            localized(language, "总是", "總是", "Always"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::AutoThreefold => [
            localized(language, "总是", "總是", "Always"),
            localized(language, "从不", "從不", "Never"),
            localized(
                language,
                "剩余时间小于30秒时",
                "剩餘時間小於30秒時",
                "When under 30 seconds",
            ),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::CastlingMethod => [
            localized(
                language,
                "将王移到车上",
                "將王移到車上",
                "Move king onto rook",
            ),
            localized(
                language,
                "将王移动两格",
                "將王移動兩格",
                "Move king two squares",
            ),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        PreferenceItem::ClockTenths => [
            localized(language, "从不", "從不", "Never"),
            localized(
                language,
                "剩余时间小于10秒时",
                "剩餘時間小於10秒時",
                "When under 10 seconds",
            ),
            localized(language, "总是", "總是", "Always"),
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    };
    let selected = preference_selected_index(item, settings);
    labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            (
                if index == selected {
                    format!("[x] {label}")
                } else {
                    format!("[ ] {label}")
                },
                UiAction::SetPreference(item, index),
            )
        })
        .collect()
}

fn preference_selected_index(item: PreferenceItem, settings: &Settings) -> usize {
    let board = settings.board;
    match item {
        PreferenceItem::BoardStyle => BoardStyle::ALL
            .iter()
            .position(|style| *style == settings.board_style)
            .unwrap_or(0),
        PreferenceItem::PieceSet => PieceSet::ALL
            .iter()
            .position(|set| *set == settings.piece_set)
            .unwrap_or(0),
        PreferenceItem::ZenMode => match board.zen_mode {
            ZenMode::No => 0,
            ZenMode::Yes => 1,
            ZenMode::GameOnly => 2,
        },
        PreferenceItem::PieceNotation => match board.piece_notation {
            PieceNotation::Symbols => 0,
            PieceNotation::Letters => 1,
        },
        PreferenceItem::Coordinates => board.coordinates as usize,
        PreferenceItem::MagnifyDraggedPiece => board.magnify_dragged_piece as usize,
        PreferenceItem::DragTarget => match board.drag_target {
            DragTarget::Circle => 0,
            DragTarget::Square => 1,
            DragTarget::None => 2,
        },
        PreferenceItem::PieceAnimation => board.piece_animation as usize,
        PreferenceItem::ImmersiveMode => board.immersive_mode as usize,
        PreferenceItem::PieceDestinations => board.piece_destinations as usize,
        PreferenceItem::BoardHighlights => board.board_highlights as usize,
        PreferenceItem::ShowMoveList => board.show_move_list as usize,
        PreferenceItem::ClockPosition => match board.clock_position {
            ClockPosition::Left => 0,
            ClockPosition::Right => 1,
        },
        PreferenceItem::Premoves => board.premoves as usize,
        PreferenceItem::Takebacks => access_index(board.takebacks),
        PreferenceItem::AutoPromotion => match board.auto_promotion {
            AutoPromotion::Never => 0,
            AutoPromotion::Premove => 1,
            AutoPromotion::Always => 2,
        },
        PreferenceItem::AutoThreefold => match board.auto_threefold {
            AutoThreefold::Always => 0,
            AutoThreefold::Never => 1,
            AutoThreefold::UnderThirtySeconds => 2,
        },
        PreferenceItem::MoveConfirmation => board.move_confirmation as usize,
        PreferenceItem::ConfirmResignDraw => board.confirm_resign_draw as usize,
        PreferenceItem::CastlingMethod => match board.castling_method {
            CastlingMethod::KingOntoRook => 0,
            CastlingMethod::KingTwoSquares => 1,
        },
        PreferenceItem::ChessClockEnabled => board.chess_clock_enabled as usize,
        PreferenceItem::GiveMoreTime => access_index(board.give_more_time),
        PreferenceItem::ClockWarning => board.clock_warning as usize,
        PreferenceItem::ClockTenths => match board.clock_tenths {
            ClockTenths::Never => 0,
            ClockTenths::UnderTenSeconds => 1,
            ClockTenths::Always => 2,
        },
    }
}

fn access_index(policy: AccessPolicy) -> usize {
    match policy {
        AccessPolicy::Never => 0,
        AccessPolicy::Casual => 1,
        AccessPolicy::Always => 2,
    }
}

fn access_value(policy: AccessPolicy, language: Language) -> &'static str {
    match policy {
        AccessPolicy::Never => localized(language, "从不", "從不", "Never"),
        AccessPolicy::Casual => localized(
            language,
            "仅限休闲对局",
            "僅限休閒對局",
            "Casual games only",
        ),
        AccessPolicy::Always => localized(language, "总是", "總是", "Always"),
    }
}

fn move_notation(san: &str, notation: PieceNotation) -> String {
    if notation == PieceNotation::Letters {
        return san.to_string();
    }
    san.chars()
        .map(|ch| match ch {
            'K' => '♔',
            'Q' => '♕',
            'R' => '♖',
            'B' => '♗',
            'N' => '♘',
            _ => ch,
        })
        .collect()
}

fn format_clock(seconds: f32, tenths: ClockTenths) -> String {
    let show_tenths = match tenths {
        ClockTenths::Never => false,
        ClockTenths::UnderTenSeconds => seconds < 10.0,
        ClockTenths::Always => true,
    };
    let seconds = seconds.max(0.0);
    let minutes = (seconds / 60.0).floor() as u32;
    if show_tenths {
        format!("{minutes}:{:04.1}", seconds % 60.0)
    } else {
        format!("{minutes}:{:02}", (seconds % 60.0).floor() as u32)
    }
}

fn settings_back_y(layout: &Layout) -> f32 {
    let (_, _, _, first_label_y, row_gap) = settings_geometry(layout);
    let last_dropdown_y = first_label_y + row_gap * 4.0 + 14.0;
    (last_dropdown_y + 58.0).min(layout.h as f32 - 64.0)
}

fn dropdown_position(layout: &Layout, kind: DropdownKind) -> (f32, f32) {
    let (left_x, right_x, _, first_label_y, row_gap) = settings_geometry(layout);
    let base_y = |row: usize| first_label_y + row as f32 * row_gap + 14.0;
    match kind {
        DropdownKind::Mode => (left_x, base_y(0)),
        DropdownKind::Difficulty => (left_x, base_y(1)),
        DropdownKind::Resolution => (right_x, base_y(0)),
        DropdownKind::Fps => (right_x, base_y(1)),
        DropdownKind::Theme => (left_x, base_y(2)),
        DropdownKind::Language => (right_x, base_y(2)),
        DropdownKind::BoardView => (left_x, base_y(3)),
        DropdownKind::Preference(_) => (0.0, 0.0),
    }
}

fn dropdown_current(view: &ViewState, kind: DropdownKind) -> String {
    match kind {
        DropdownKind::Mode => mode_name(view.settings.mode, view.settings.language).to_string(),
        DropdownKind::Difficulty => {
            depth_name(view.settings.ai_depth, view.settings.language).to_string()
        }
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
        }
        .to_string(),
        DropdownKind::Preference(item) => {
            preference_value(item, &view.settings, view.settings.language)
        }
    }
}

fn status_text(view: &ViewState) -> String {
    let tr = view.settings.language.text();
    if view.claimed_draw {
        return tr.draw.to_string();
    }
    if view.ai_thinking {
        return tr.ai_thinking.to_string();
    }
    if let Some(outcome) = view.pos.outcome() {
        return match outcome {
            Outcome::Decisive { winner } => {
                let name = if winner == Color::White {
                    tr.white
                } else {
                    tr.black
                };
                format!("{}{name}{}", tr.checkmate, tr.wins)
            }
            Outcome::Draw => tr.draw.to_string(),
        };
    }
    let turn = if view.pos.turn() == Color::White {
        tr.white
    } else {
        tr.black
    };
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
    use super::{
        BEHAVIOR_PREFERENCES, CLOCK_PREFERENCES, DISPLAY_PREFERENCES, DropdownKind, Renderer,
        Screen, Settings, SettingsPage, ViewState, oriented_square, preference_options,
    };
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
                    move_confirmation_pending: false,
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
                    settings_page: SettingsPage::Root,
                    settings_scroll: 0,
                    game_over_progress: 0.0,
                    mouse: None,
                    mouse_pressed: false,
                    mouse_down: false,
                    mouse_released: false,
                    dragging_from: None,
                    menu_time: 1.0,
                    white_clock: 600.0,
                    black_clock: 600.0,
                    claimed_draw: false,
                };
                let actions = renderer.render(&mut buffer, 1280, 720, &view);
                assert!(actions.is_empty());
                assert!(buffer.iter().any(|&pixel| pixel != buffer[0]));
            }
        }
    }

    #[test]
    fn every_board_settings_page_renders() {
        let images = PieceImages::load();
        let text = TextRenderer::load();
        let renderer = Renderer::new(&images, &text);
        for settings_page in [
            SettingsPage::Root,
            SettingsPage::Board,
            SettingsPage::Display,
            SettingsPage::Behavior,
            SettingsPage::Clock,
        ] {
            let mut buffer = vec![0; 1280 * 720];
            let view = ViewState {
                pos: Chess::default(),
                selected: None,
                last_move: None,
                legal_targets: Vec::new(),
                history_sans: Vec::new(),
                ai_thinking: false,
                hint_thinking: false,
                suggestion: None,
                promotion: None,
                move_confirmation_pending: false,
                screen: Screen::Settings,
                settings: Settings::default(),
                animations: Vec::new(),
                resolutions: vec![(1280, 720)],
                refreshes: vec![60],
                open_dropdown: None,
                dropdown_scroll: 0,
                settings_page,
                settings_scroll: 0,
                game_over_progress: 0.0,
                mouse: None,
                mouse_pressed: false,
                mouse_down: false,
                mouse_released: false,
                dragging_from: None,
                menu_time: 0.0,
                white_clock: 600.0,
                black_clock: 600.0,
                claimed_draw: false,
            };
            assert!(renderer.render(&mut buffer, 1280, 720, &view).is_empty());
            assert!(buffer.iter().any(|&pixel| pixel != buffer[0]));
        }
    }

    #[test]
    fn every_board_preference_has_a_selectable_dropdown() {
        let settings = Settings::default();
        for item in DISPLAY_PREFERENCES
            .into_iter()
            .chain(BEHAVIOR_PREFERENCES)
            .chain(CLOCK_PREFERENCES)
        {
            let options = preference_options(item, &settings, Language::Chinese);
            assert!(
                options.len() >= 2,
                "{item:?} should have at least two choices"
            );
            assert_eq!(
                options
                    .iter()
                    .filter(|(label, _)| label.starts_with("[x]"))
                    .count(),
                1,
                "{item:?} should mark exactly one current choice"
            );
        }
    }
}

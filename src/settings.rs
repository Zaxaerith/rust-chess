use std::fs;
use std::path::PathBuf;

use shakmaty::Color;

use crate::assets::PieceSet;
use crate::i18n::Language;
use crate::preferences::{
    AccessPolicy, AutoPromotion, AutoThreefold, CastlingMethod, ClockPosition, ClockTenths,
    DragTarget, PieceNotation, ZenMode, parse_bool,
};
use crate::render::Settings;
use crate::theme::{BoardStyle, Theme};

pub fn settings_path() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join("settings.cfg")
}

pub fn save(settings: &Settings) {
    let text = format!(
        concat!(
            "mode={}\nai_depth={}\nresolution={}x{}\nfps={}\ntheme={}\n",
            "board_style={}\npiece_set={}\nlanguage={}\nflip_for_black={}\n",
            "zen_mode={}\npiece_notation={}\ncoordinates={}\nmagnify_dragged_piece={}\n",
            "drag_target={}\npiece_animation={}\nimmersive_mode={}\npiece_destinations={}\n",
            "board_highlights={}\nshow_move_list={}\nclock_position={}\npremoves={}\n",
            "takebacks={}\nauto_promotion={}\nauto_threefold={}\nmove_confirmation={}\n",
            "confirm_resign_draw={}\ncastling_method={}\nchess_clock_enabled={}\ngive_more_time={}\nclock_warning={}\n",
            "clock_tenths={}\n"
        ),
        mode_key(settings.mode),
        settings.ai_depth,
        settings.resolution.0,
        settings.resolution.1,
        settings.fps,
        settings.theme.key(),
        settings.board_style.key(),
        settings.piece_set.key(),
        settings.language.key(),
        settings.flip_for_black,
        settings.board.zen_mode.key(),
        settings.board.piece_notation.key(),
        settings.board.coordinates,
        settings.board.magnify_dragged_piece,
        settings.board.drag_target.key(),
        settings.board.piece_animation,
        settings.board.immersive_mode,
        settings.board.piece_destinations,
        settings.board.board_highlights,
        settings.board.show_move_list,
        settings.board.clock_position.key(),
        settings.board.premoves,
        settings.board.takebacks.key(),
        settings.board.auto_promotion.key(),
        settings.board.auto_threefold.key(),
        settings.board.move_confirmation,
        settings.board.confirm_resign_draw,
        settings.board.castling_method.key(),
        settings.board.chess_clock_enabled,
        settings.board.give_more_time.key(),
        settings.board.clock_warning,
        settings.board.clock_tenths.key(),
    );
    let _ = fs::write(settings_path(), text);
}

pub fn load() -> Option<Settings> {
    let text = fs::read_to_string(settings_path()).ok()?;
    let mut settings = Settings::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "mode" => settings.mode = mode_from_key(value),
            "ai_depth" => {
                if let Ok(depth) = value.parse::<u32>() {
                    settings.ai_depth = depth.clamp(1, 5);
                }
            }
            "resolution" => {
                if let Some((w, h)) = parse_resolution(value) {
                    settings.resolution = (w, h);
                }
            }
            "fps" => {
                if let Ok(fps) = value.parse::<u32>() {
                    settings.fps = fps.clamp(30, 480);
                }
            }
            "theme" => {
                if let Some(theme) = Theme::from_key(value) {
                    settings.theme = theme;
                }
            }
            "board_style" => {
                if let Some(style) = BoardStyle::from_key(value) {
                    settings.board_style = style;
                }
            }
            "piece_set" => {
                if let Some(style) = PieceSet::from_key(value) {
                    settings.piece_set = style;
                }
            }
            "language" => {
                if let Some(language) = Language::from_key(value) {
                    settings.language = language;
                }
            }
            "flip_for_black" => settings.flip_for_black = value == "true",
            "zen_mode" => {
                if let Some(v) = ZenMode::from_key(value) {
                    settings.board.zen_mode = v;
                }
            }
            "piece_notation" => {
                if let Some(v) = PieceNotation::from_key(value) {
                    settings.board.piece_notation = v;
                }
            }
            "coordinates" => {
                settings.board.coordinates = parse_bool(value, settings.board.coordinates)
            }
            "magnify_dragged_piece" => {
                settings.board.magnify_dragged_piece =
                    parse_bool(value, settings.board.magnify_dragged_piece)
            }
            "drag_target" => {
                if let Some(v) = DragTarget::from_key(value) {
                    settings.board.drag_target = v;
                }
            }
            "piece_animation" => {
                settings.board.piece_animation = parse_bool(value, settings.board.piece_animation)
            }
            "immersive_mode" => {
                settings.board.immersive_mode = parse_bool(value, settings.board.immersive_mode)
            }
            "piece_destinations" => {
                settings.board.piece_destinations =
                    parse_bool(value, settings.board.piece_destinations)
            }
            "board_highlights" => {
                settings.board.board_highlights = parse_bool(value, settings.board.board_highlights)
            }
            "show_move_list" => {
                settings.board.show_move_list = parse_bool(value, settings.board.show_move_list)
            }
            "clock_position" => {
                if let Some(v) = ClockPosition::from_key(value) {
                    settings.board.clock_position = v;
                }
            }
            "premoves" => settings.board.premoves = parse_bool(value, settings.board.premoves),
            "takebacks" => {
                if let Some(v) = AccessPolicy::from_key(value) {
                    settings.board.takebacks = v;
                }
            }
            "auto_promotion" => {
                if let Some(v) = AutoPromotion::from_key(value) {
                    settings.board.auto_promotion = v;
                }
            }
            "auto_threefold" => {
                if let Some(v) = AutoThreefold::from_key(value) {
                    settings.board.auto_threefold = v;
                }
            }
            "move_confirmation" => {
                settings.board.move_confirmation =
                    parse_bool(value, settings.board.move_confirmation)
            }
            "confirm_resign_draw" => {
                settings.board.confirm_resign_draw =
                    parse_bool(value, settings.board.confirm_resign_draw)
            }
            "castling_method" => {
                if let Some(v) = CastlingMethod::from_key(value) {
                    settings.board.castling_method = v;
                }
            }
            "chess_clock_enabled" => {
                settings.board.chess_clock_enabled =
                    parse_bool(value, settings.board.chess_clock_enabled)
            }
            "give_more_time" => {
                if let Some(v) = AccessPolicy::from_key(value) {
                    settings.board.give_more_time = v;
                }
            }
            "clock_warning" => {
                settings.board.clock_warning = parse_bool(value, settings.board.clock_warning)
            }
            "clock_tenths" => {
                if let Some(v) = ClockTenths::from_key(value) {
                    settings.board.clock_tenths = v;
                }
            }
            _ => {}
        }
    }
    Some(settings)
}

fn mode_key(mode: Option<Color>) -> &'static str {
    match mode {
        None => "human",
        Some(Color::White) => "ai_white",
        Some(Color::Black) => "ai_black",
    }
}

fn mode_from_key(key: &str) -> Option<Color> {
    match key {
        "human" => None,
        "ai_white" => Some(Color::White),
        "ai_black" => Some(Color::Black),
        _ => None,
    }
}

fn parse_resolution(value: &str) -> Option<(u32, u32)> {
    let (w, h) = value.split_once('x')?;
    let w = w.trim().parse::<u32>().ok()?;
    let h = h.trim().parse::<u32>().ok()?;
    if w >= 640 && h >= 480 {
        Some((w, h))
    } else {
        None
    }
}

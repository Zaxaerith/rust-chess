#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZenMode {
    No,
    Yes,
    GameOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceNotation {
    Symbols,
    Letters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragTarget {
    Circle,
    Square,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockPosition {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessPolicy {
    Never,
    Casual,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoPromotion {
    Never,
    Premove,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoThreefold {
    Always,
    Never,
    UnderThirtySeconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastlingMethod {
    KingOntoRook,
    KingTwoSquares,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockTenths {
    Never,
    UnderTenSeconds,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardPreferences {
    pub zen_mode: ZenMode,
    pub piece_notation: PieceNotation,
    pub coordinates: bool,
    pub magnify_dragged_piece: bool,
    pub drag_target: DragTarget,
    pub piece_animation: bool,
    pub immersive_mode: bool,
    pub piece_destinations: bool,
    pub board_highlights: bool,
    pub show_move_list: bool,
    pub clock_position: ClockPosition,
    pub premoves: bool,
    pub takebacks: AccessPolicy,
    pub auto_promotion: AutoPromotion,
    pub auto_threefold: AutoThreefold,
    pub move_confirmation: bool,
    pub confirm_resign_draw: bool,
    pub castling_method: CastlingMethod,
    pub chess_clock_enabled: bool,
    pub give_more_time: AccessPolicy,
    pub clock_warning: bool,
    pub clock_tenths: ClockTenths,
}

impl Default for BoardPreferences {
    fn default() -> Self {
        Self {
            zen_mode: ZenMode::No,
            piece_notation: PieceNotation::Letters,
            coordinates: true,
            magnify_dragged_piece: true,
            drag_target: DragTarget::Circle,
            piece_animation: true,
            immersive_mode: false,
            piece_destinations: true,
            board_highlights: true,
            show_move_list: true,
            clock_position: ClockPosition::Right,
            premoves: true,
            takebacks: AccessPolicy::Casual,
            auto_promotion: AutoPromotion::Premove,
            auto_threefold: AutoThreefold::Never,
            move_confirmation: false,
            confirm_resign_draw: true,
            castling_method: CastlingMethod::KingOntoRook,
            chess_clock_enabled: false,
            give_more_time: AccessPolicy::Casual,
            clock_warning: true,
            clock_tenths: ClockTenths::UnderTenSeconds,
        }
    }
}

macro_rules! key_enum {
    ($ty:ty, {$($variant:path => $key:literal),+ $(,)?}) => {
        impl $ty {
            pub fn key(self) -> &'static str {
                match self { $($variant => $key),+ }
            }

            pub fn from_key(key: &str) -> Option<Self> {
                match key { $($key => Some($variant)),+, _ => None }
            }
        }
    };
}

key_enum!(ZenMode, {
    ZenMode::No => "no",
    ZenMode::Yes => "yes",
    ZenMode::GameOnly => "game_only",
});
key_enum!(PieceNotation, {
    PieceNotation::Symbols => "symbols",
    PieceNotation::Letters => "letters",
});
key_enum!(DragTarget, {
    DragTarget::Circle => "circle",
    DragTarget::Square => "square",
    DragTarget::None => "none",
});
key_enum!(ClockPosition, {
    ClockPosition::Left => "left",
    ClockPosition::Right => "right",
});
key_enum!(AccessPolicy, {
    AccessPolicy::Never => "never",
    AccessPolicy::Casual => "casual",
    AccessPolicy::Always => "always",
});
key_enum!(AutoPromotion, {
    AutoPromotion::Never => "never",
    AutoPromotion::Premove => "premove",
    AutoPromotion::Always => "always",
});
key_enum!(AutoThreefold, {
    AutoThreefold::Always => "always",
    AutoThreefold::Never => "never",
    AutoThreefold::UnderThirtySeconds => "under_30",
});
key_enum!(CastlingMethod, {
    CastlingMethod::KingOntoRook => "king_onto_rook",
    CastlingMethod::KingTwoSquares => "king_two_squares",
});
key_enum!(ClockTenths, {
    ClockTenths::Never => "never",
    ClockTenths::UnderTenSeconds => "under_10",
    ClockTenths::Always => "always",
});

pub fn parse_bool(value: &str, fallback: bool) -> bool {
    match value {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preference_keys_round_trip() {
        for value in [ZenMode::No, ZenMode::Yes, ZenMode::GameOnly] {
            assert_eq!(ZenMode::from_key(value.key()), Some(value));
        }
        for value in [PieceNotation::Symbols, PieceNotation::Letters] {
            assert_eq!(PieceNotation::from_key(value.key()), Some(value));
        }
        for value in [DragTarget::Circle, DragTarget::Square, DragTarget::None] {
            assert_eq!(DragTarget::from_key(value.key()), Some(value));
        }
        for value in [
            AccessPolicy::Never,
            AccessPolicy::Casual,
            AccessPolicy::Always,
        ] {
            assert_eq!(AccessPolicy::from_key(value.key()), Some(value));
        }
        for value in [
            AutoPromotion::Never,
            AutoPromotion::Premove,
            AutoPromotion::Always,
        ] {
            assert_eq!(AutoPromotion::from_key(value.key()), Some(value));
        }
        for value in [
            AutoThreefold::Always,
            AutoThreefold::Never,
            AutoThreefold::UnderThirtySeconds,
        ] {
            assert_eq!(AutoThreefold::from_key(value.key()), Some(value));
        }
        for value in [CastlingMethod::KingOntoRook, CastlingMethod::KingTwoSquares] {
            assert_eq!(CastlingMethod::from_key(value.key()), Some(value));
        }
        for value in [
            ClockTenths::Never,
            ClockTenths::UnderTenSeconds,
            ClockTenths::Always,
        ] {
            assert_eq!(ClockTenths::from_key(value.key()), Some(value));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    DarkPlus,
    LightPlus,
    Monokai,
    SolarizedDark,
    Nord,
    Abyss,
    KimbieDark,
    QuietLight,
    TomorrowNightBlue,
    Red,
}

impl Theme {
    pub const ALL: [Theme; 10] = [
        Theme::DarkPlus,
        Theme::LightPlus,
        Theme::Monokai,
        Theme::SolarizedDark,
        Theme::Nord,
        Theme::Abyss,
        Theme::KimbieDark,
        Theme::QuietLight,
        Theme::TomorrowNightBlue,
        Theme::Red,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Theme::DarkPlus => "Dark+",
            Theme::LightPlus => "Light+",
            Theme::Monokai => "Monokai",
            Theme::SolarizedDark => "Solarized",
            Theme::Nord => "Nord",
            Theme::Abyss => "Abyss",
            Theme::KimbieDark => "Kimbie Dark",
            Theme::QuietLight => "Quiet Light",
            Theme::TomorrowNightBlue => "Tomorrow Night Blue",
            Theme::Red => "Red",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Theme::DarkPlus => "dark_plus",
            Theme::LightPlus => "light_plus",
            Theme::Monokai => "monokai",
            Theme::SolarizedDark => "solarized_dark",
            Theme::Nord => "nord",
            Theme::Abyss => "abyss",
            Theme::KimbieDark => "kimbie_dark",
            Theme::QuietLight => "quiet_light",
            Theme::TomorrowNightBlue => "tomorrow_night_blue",
            Theme::Red => "red",
        }
    }

    pub fn from_key(key: &str) -> Option<Theme> {
        match key {
            "dark_plus" => Some(Theme::DarkPlus),
            "light_plus" => Some(Theme::LightPlus),
            "monokai" => Some(Theme::Monokai),
            "solarized_dark" => Some(Theme::SolarizedDark),
            "nord" => Some(Theme::Nord),
            "abyss" => Some(Theme::Abyss),
            "kimbie_dark" => Some(Theme::KimbieDark),
            "quiet_light" => Some(Theme::QuietLight),
            "tomorrow_night_blue" => Some(Theme::TomorrowNightBlue),
            "red" => Some(Theme::Red),
            _ => None,
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::DarkPlus => Palette {
                bg: 0x001e_1e1e,
                panel: 0x0025_2526,
                border: 0x000f_0f0f,
                text: 0x00cc_cccc,
                muted: 0x0085_8585,
                button: 0x0033_3333,
                button_hover: 0x003a_3d41,
                button_active: 0x000e_639c,
                button_active_hover: 0x001b_7ec2,
                light_square: 0x00eb_ecd0,
                dark_square: 0x0073_9552,
                selected: 0x00ff_ff66,
                last_move: 0x00f7_f769,
                move_dot: 0x004a_4f55,
                capture_ring: 0x00cc_3333,
                check_ring: 0x00e2_3c3c,
                accent: 0x00ff_d27f,
                history: 0x00c6_cdd6,
            },
            Theme::LightPlus => Palette {
                bg: 0x00ff_ffff,
                panel: 0x00f3_f3f3,
                border: 0x00cc_cccc,
                text: 0x001f_1f1f,
                muted: 0x0076_7676,
                button: 0x00e8_e8e8,
                button_hover: 0x00dc_dcdc,
                button_active: 0x0000_7acc,
                button_active_hover: 0x001a_8fd4,
                light_square: 0x00f0_d9b5,
                dark_square: 0x00b5_8863,
                selected: 0x00ff_e066,
                last_move: 0x00ff_f2a8,
                move_dot: 0x0055_5555,
                capture_ring: 0x00d0_021b,
                check_ring: 0x00e2_3c3c,
                accent: 0x0000_7acc,
                history: 0x003a_3a3a,
            },
            Theme::Monokai => Palette {
                bg: 0x0027_2822,
                panel: 0x001e_1f1c,
                border: 0x0011_110e,
                text: 0x00f8_f8f2,
                muted: 0x0075_715e,
                button: 0x003e_3d32,
                button_hover: 0x0049_483e,
                button_active: 0x00ae_81ff,
                button_active_hover: 0x00c3_9bff,
                light_square: 0x00d0_c8a0,
                dark_square: 0x007c_8060,
                selected: 0x00ff_d866,
                last_move: 0x00ff_e680,
                move_dot: 0x006a_6a5a,
                capture_ring: 0x00f9_2672,
                check_ring: 0x00f9_2672,
                accent: 0x00a6_e22e,
                history: 0x00b8_b8a8,
            },
            Theme::SolarizedDark => Palette {
                bg: 0x0000_2b36,
                panel: 0x0007_3642,
                border: 0x0000_1a20,
                text: 0x0083_9496,
                muted: 0x0058_6e75,
                button: 0x0013_4652,
                button_hover: 0x001a_5561,
                button_active: 0x0026_8bd2,
                button_active_hover: 0x004d_a3e0,
                light_square: 0x00ee_e8d5,
                dark_square: 0x0093_a1a1,
                selected: 0x00ff_e58a,
                last_move: 0x00ff_e58a,
                move_dot: 0x0058_6e75,
                capture_ring: 0x00dc_322f,
                check_ring: 0x00dc_322f,
                accent: 0x002a_a198,
                history: 0x0093_a1a1,
            },
            Theme::Nord => Palette {
                bg: 0x002e_3440,
                panel: 0x003b_4252,
                border: 0x0023_2833,
                text: 0x00ec_eff4,
                muted: 0x008f_9bb0,
                button: 0x0043_4c5e,
                button_hover: 0x004c_566a,
                button_active: 0x005e_81ac,
                button_active_hover: 0x0081_a1c1,
                light_square: 0x00e5_e9f0,
                dark_square: 0x0081_a1c1,
                selected: 0x00ff_d87e,
                last_move: 0x00d8_dee9,
                move_dot: 0x004c_566a,
                capture_ring: 0x00bf_616a,
                check_ring: 0x00bf_616a,
                accent: 0x0088_c0d0,
                history: 0x00d8_dee9,
            },
            Theme::Abyss => Palette {
                bg: 0x0000_0c18,
                panel: 0x0006_0621,
                border: 0x0000_0000,
                text: 0x0091_a7d5,
                muted: 0x0059_6f99,
                button: 0x0015_2037,
                button_hover: 0x002b_3c5d,
                button_active: 0x0008_286b,
                button_active_hover: 0x000d_3b99,
                light_square: 0x00c7_d4e8,
                dark_square: 0x0040_5a84,
                selected: 0x00ff_d766,
                last_move: 0x0089_a8d8,
                move_dot: 0x0015_2037,
                capture_ring: 0x00b5_1f2e,
                check_ring: 0x00e0_4444,
                accent: 0x0066_88cc,
                history: 0x0091_a7d5,
            },
            Theme::KimbieDark => Palette {
                bg: 0x0022_1a0f,
                panel: 0x0036_2712,
                border: 0x0012_0d08,
                text: 0x00d3_af86,
                muted: 0x008b_7965,
                button: 0x004d_3b27,
                button_hover: 0x006e_583b,
                button_active: 0x007c_5021,
                button_active_hover: 0x009c_672d,
                light_square: 0x00d8_bd8f,
                dark_square: 0x008f_673d,
                selected: 0x00f4_c95d,
                last_move: 0x00d6_a648,
                move_dot: 0x005e_4831,
                capture_ring: 0x00dc_3954,
                check_ring: 0x00f0_6431,
                accent: 0x00f0_6431,
                history: 0x00c8_9b6e,
            },
            Theme::QuietLight => Palette {
                bg: 0x00f5_f5f5,
                panel: 0x00f2_f2f2,
                border: 0x00c9_d0d9,
                text: 0x0033_3333,
                muted: 0x0077_7777,
                button: 0x00e6_e6e6,
                button_hover: 0x00d3_dbcd,
                button_active: 0x0070_5697,
                button_active_hover: 0x0085_68ad,
                light_square: 0x00f0_e8d8,
                dark_square: 0x00a7_b893,
                selected: 0x00ff_d866,
                last_move: 0x00c4_d9b1,
                move_dot: 0x0068_737d,
                capture_ring: 0x00c4_384b,
                check_ring: 0x00d1_2f3f,
                accent: 0x0097_69dc,
                history: 0x0044_4444,
            },
            Theme::TomorrowNightBlue => Palette {
                bg: 0x0000_2451,
                panel: 0x0000_1c40,
                border: 0x0000_1126,
                text: 0x00ff_ffff,
                muted: 0x008e_acd1,
                button: 0x0000_3570,
                button_hover: 0x0000_4896,
                button_active: 0x000b_66b2,
                button_active_hover: 0x0018_80d0,
                light_square: 0x00d6_e6f7,
                dark_square: 0x004b_78a8,
                selected: 0x00ff_e580,
                last_move: 0x0080_baff,
                move_dot: 0x0000_3f8e,
                capture_ring: 0x00ff_7882,
                check_ring: 0x00ff_4f63,
                accent: 0x00bb_daff,
                history: 0x00bb_daff,
            },
            Theme::Red => Palette {
                bg: 0x0039_0000,
                panel: 0x0033_0000,
                border: 0x0018_0000,
                text: 0x00f8_f8f8,
                muted: 0x00c2_8f8f,
                button: 0x0066_3333,
                button_hover: 0x0088_3333,
                button_active: 0x00aa_0000,
                button_active_hover: 0x00c5_1a1a,
                light_square: 0x00ea_d6d1,
                dark_square: 0x009e_5555,
                selected: 0x00ff_d266,
                last_move: 0x00d9_8989,
                move_dot: 0x006b_1818,
                capture_ring: 0x00ff_8855,
                check_ring: 0x00ff_ffff,
                accent: 0x00ff_6666,
                history: 0x00f0_c1c1,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardStyle {
    Classic,
    Walnut,
    Ocean,
    Emerald,
    Marble,
    Graphite,
}

impl BoardStyle {
    pub const ALL: [BoardStyle; 6] = [
        BoardStyle::Classic,
        BoardStyle::Walnut,
        BoardStyle::Ocean,
        BoardStyle::Emerald,
        BoardStyle::Marble,
        BoardStyle::Graphite,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BoardStyle::Classic => "Classic",
            BoardStyle::Walnut => "Walnut",
            BoardStyle::Ocean => "Ocean",
            BoardStyle::Emerald => "Emerald",
            BoardStyle::Marble => "Marble",
            BoardStyle::Graphite => "Graphite",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            BoardStyle::Classic => "classic",
            BoardStyle::Walnut => "walnut",
            BoardStyle::Ocean => "ocean",
            BoardStyle::Emerald => "emerald",
            BoardStyle::Marble => "marble",
            BoardStyle::Graphite => "graphite",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "classic" => Some(BoardStyle::Classic),
            "walnut" => Some(BoardStyle::Walnut),
            "ocean" => Some(BoardStyle::Ocean),
            "emerald" => Some(BoardStyle::Emerald),
            "marble" => Some(BoardStyle::Marble),
            "graphite" => Some(BoardStyle::Graphite),
            _ => None,
        }
    }

    pub fn squares(self, palette: &Palette) -> (u32, u32) {
        match self {
            BoardStyle::Classic => (palette.light_square, palette.dark_square),
            BoardStyle::Walnut => (0x00e0_c09d, 0x009b_6a4a),
            BoardStyle::Ocean => (0x00ca_d9e8, 0x005d_83a6),
            BoardStyle::Emerald => (0x00e8_edcc, 0x0077_9556),
            BoardStyle::Marble => (0x00f1_f1e8, 0x00a9_a9a3),
            BoardStyle::Graphite => (0x00b9_c0c7, 0x0059_636d),
        }
    }
}

pub struct Palette {
    pub bg: u32,
    pub panel: u32,
    pub border: u32,
    pub text: u32,
    pub muted: u32,
    pub button: u32,
    pub button_hover: u32,
    pub button_active: u32,
    pub button_active_hover: u32,
    pub light_square: u32,
    pub dark_square: u32,
    pub selected: u32,
    pub last_move: u32,
    pub move_dot: u32,
    pub capture_ring: u32,
    pub check_ring: u32,
    pub accent: u32,
    pub history: u32,
}

#[cfg(test)]
mod tests {
    use super::{BoardStyle, Theme};

    #[test]
    fn theme_and_board_style_keys_round_trip() {
        for theme in Theme::ALL {
            assert_eq!(Theme::from_key(theme.key()), Some(theme));
            assert!(!theme.label().is_empty());
        }
        let palette = Theme::DarkPlus.palette();
        for style in BoardStyle::ALL {
            assert_eq!(BoardStyle::from_key(style.key()), Some(style));
            assert!(!style.label().is_empty());
            let (light, dark) = style.squares(&palette);
            assert_ne!(light, dark);
        }
    }
}

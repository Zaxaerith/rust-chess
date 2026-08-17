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
                bg: 0x1e1e_1e,
                panel: 0x2525_26,
                border: 0x0f0f_0f,
                text: 0xcccc_cc,
                muted: 0x8585_85,
                button: 0x3333_33,
                button_hover: 0x3a3d_41,
                button_active: 0x0e63_9c,
                button_active_hover: 0x1b7e_c2,
                light_square: 0xebec_d0,
                dark_square: 0x7395_52,
                selected: 0xffff_66,
                last_move: 0xf7f7_69,
                move_dot: 0x4a4f_55,
                capture_ring: 0xcc33_33,
                check_ring: 0xe23c_3c,
                accent: 0xffd2_7f,
                history: 0xc6cd_d6,
            },
            Theme::LightPlus => Palette {
                bg: 0xffff_ff,
                panel: 0xf3f3_f3,
                border: 0xcccc_cc,
                text: 0x1f1f_1f,
                muted: 0x7676_76,
                button: 0xe8e8_e8,
                button_hover: 0xdcdc_dc,
                button_active: 0x007a_cc,
                button_active_hover: 0x1a8f_d4,
                light_square: 0xf0d9_b5,
                dark_square: 0xb588_63,
                selected: 0xffe0_66,
                last_move: 0xfff2_a8,
                move_dot: 0x5555_55,
                capture_ring: 0xd002_1b,
                check_ring: 0xe23c_3c,
                accent: 0x007a_cc,
                history: 0x3a3a_3a,
            },
            Theme::Monokai => Palette {
                bg: 0x2728_22,
                panel: 0x1e1f_1c,
                border: 0x1111_0e,
                text: 0xf8f8_f2,
                muted: 0x7571_5e,
                button: 0x3e3d_32,
                button_hover: 0x4948_3e,
                button_active: 0xae81_ff,
                button_active_hover: 0xc39b_ff,
                light_square: 0xd0c8_a0,
                dark_square: 0x7c80_60,
                selected: 0xffd8_66,
                last_move: 0xffe6_80,
                move_dot: 0x6a6a_5a,
                capture_ring: 0xf926_72,
                check_ring: 0xf926_72,
                accent: 0xa6e2_2e,
                history: 0xb8b8_a8,
            },
            Theme::SolarizedDark => Palette {
                bg: 0x002b_36,
                panel: 0x0736_42,
                border: 0x001a_20,
                text: 0x8394_96,
                muted: 0x586e_75,
                button: 0x1346_52,
                button_hover: 0x1a55_61,
                button_active: 0x268b_d2,
                button_active_hover: 0x4da3_e0,
                light_square: 0xeee8_d5,
                dark_square: 0x93a1_a1,
                selected: 0xffe5_8a,
                last_move: 0xffe5_8a,
                move_dot: 0x586e_75,
                capture_ring: 0xdc32_2f,
                check_ring: 0xdc32_2f,
                accent: 0x2aa1_98,
                history: 0x93a1_a1,
            },
            Theme::Nord => Palette {
                bg: 0x2e34_40,
                panel: 0x3b42_52,
                border: 0x2328_33,
                text: 0xecef_f4,
                muted: 0x8f9b_b0,
                button: 0x434c_5e,
                button_hover: 0x4c56_6a,
                button_active: 0x5e81_ac,
                button_active_hover: 0x81a1_c1,
                light_square: 0xe5e9_f0,
                dark_square: 0x81a1_c1,
                selected: 0xffd8_7e,
                last_move: 0xd8de_e9,
                move_dot: 0x4c56_6a,
                capture_ring: 0xbf61_6a,
                check_ring: 0xbf61_6a,
                accent: 0x88c0_d0,
                history: 0xd8de_e9,
            },
            Theme::Abyss => Palette {
                bg: 0x000c_18,
                panel: 0x0606_21,
                border: 0x0000_00,
                text: 0x91a7_d5,
                muted: 0x596f_99,
                button: 0x1520_37,
                button_hover: 0x2b3c_5d,
                button_active: 0x0828_6b,
                button_active_hover: 0x0d3b_99,
                light_square: 0xc7d4_e8,
                dark_square: 0x405a_84,
                selected: 0xffd7_66,
                last_move: 0x89a8_d8,
                move_dot: 0x1520_37,
                capture_ring: 0xb51f_2e,
                check_ring: 0xe044_44,
                accent: 0x6688_cc,
                history: 0x91a7_d5,
            },
            Theme::KimbieDark => Palette {
                bg: 0x221a_0f,
                panel: 0x3627_12,
                border: 0x120d_08,
                text: 0xd3af_86,
                muted: 0x8b79_65,
                button: 0x4d3b_27,
                button_hover: 0x6e58_3b,
                button_active: 0x7c50_21,
                button_active_hover: 0x9c67_2d,
                light_square: 0xd8bd_8f,
                dark_square: 0x8f67_3d,
                selected: 0xf4c9_5d,
                last_move: 0xd6a6_48,
                move_dot: 0x5e48_31,
                capture_ring: 0xdc39_54,
                check_ring: 0xf064_31,
                accent: 0xf064_31,
                history: 0xc89b_6e,
            },
            Theme::QuietLight => Palette {
                bg: 0xf5f5_f5,
                panel: 0xf2f2_f2,
                border: 0xc9d0_d9,
                text: 0x3333_33,
                muted: 0x7777_77,
                button: 0xe6e6_e6,
                button_hover: 0xd3db_cd,
                button_active: 0x7056_97,
                button_active_hover: 0x8568_ad,
                light_square: 0xf0e8_d8,
                dark_square: 0xa7b8_93,
                selected: 0xffd8_66,
                last_move: 0xc4d9_b1,
                move_dot: 0x6873_7d,
                capture_ring: 0xc438_4b,
                check_ring: 0xd12f_3f,
                accent: 0x9769_dc,
                history: 0x4444_44,
            },
            Theme::TomorrowNightBlue => Palette {
                bg: 0x0024_51,
                panel: 0x001c_40,
                border: 0x0011_26,
                text: 0xffff_ff,
                muted: 0x8eac_d1,
                button: 0x0035_70,
                button_hover: 0x0048_96,
                button_active: 0x0b66_b2,
                button_active_hover: 0x1880_d0,
                light_square: 0xd6e6_f7,
                dark_square: 0x4b78_a8,
                selected: 0xffe5_80,
                last_move: 0x80ba_ff,
                move_dot: 0x003f_8e,
                capture_ring: 0xff78_82,
                check_ring: 0xff4f_63,
                accent: 0xbbda_ff,
                history: 0xbbda_ff,
            },
            Theme::Red => Palette {
                bg: 0x3900_00,
                panel: 0x3300_00,
                border: 0x1800_00,
                text: 0xf8f8_f8,
                muted: 0xc28f_8f,
                button: 0x6633_33,
                button_hover: 0x8833_33,
                button_active: 0xaa00_00,
                button_active_hover: 0xc51a_1a,
                light_square: 0xead6_d1,
                dark_square: 0x9e55_55,
                selected: 0xffd2_66,
                last_move: 0xd989_89,
                move_dot: 0x6b18_18,
                capture_ring: 0xff88_55,
                check_ring: 0xffff_ff,
                accent: 0xff66_66,
                history: 0xf0c1_c1,
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
            BoardStyle::Walnut => (0xe0c0_9d, 0x9b6a_4a),
            BoardStyle::Ocean => (0xcad9_e8, 0x5d83_a6),
            BoardStyle::Emerald => (0xe8ed_cc, 0x7795_56),
            BoardStyle::Marble => (0xf1f1_e8, 0xa9a9_a3),
            BoardStyle::Graphite => (0xb9c0_c7, 0x5963_6d),
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

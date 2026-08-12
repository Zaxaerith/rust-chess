#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    DarkPlus,
    LightPlus,
    Monokai,
    SolarizedDark,
    Nord,
}

impl Theme {
    pub const ALL: [Theme; 5] = [
        Theme::DarkPlus,
        Theme::LightPlus,
        Theme::Monokai,
        Theme::SolarizedDark,
        Theme::Nord,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Theme::DarkPlus => "Dark+",
            Theme::LightPlus => "Light+",
            Theme::Monokai => "Monokai",
            Theme::SolarizedDark => "Solarized",
            Theme::Nord => "Nord",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Theme::DarkPlus => "dark_plus",
            Theme::LightPlus => "light_plus",
            Theme::Monokai => "monokai",
            Theme::SolarizedDark => "solarized_dark",
            Theme::Nord => "nord",
        }
    }

    pub fn from_key(key: &str) -> Option<Theme> {
        match key {
            "dark_plus" => Some(Theme::DarkPlus),
            "light_plus" => Some(Theme::LightPlus),
            "monokai" => Some(Theme::Monokai),
            "solarized_dark" => Some(Theme::SolarizedDark),
            "nord" => Some(Theme::Nord),
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

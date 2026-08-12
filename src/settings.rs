use std::fs;
use std::path::PathBuf;

use shakmaty::Color;

use crate::render::Settings;
use crate::theme::Theme;

pub fn settings_path() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join("settings.cfg")
}

pub fn save(settings: &Settings) {
    let text = format!(
        "mode={}\nai_depth={}\nresolution={}x{}\nfps={}\ntheme={}\n",
        mode_key(settings.mode),
        settings.ai_depth,
        settings.resolution.0,
        settings.resolution.1,
        settings.fps,
        settings.theme.key(),
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

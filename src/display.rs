use winapi::um::wingdi::DEVMODEW;
use winapi::um::winuser::{EnumDisplaySettingsW, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh: u32,
}

pub fn display_modes() -> Vec<DisplayMode> {
    let mut modes = Vec::new();
    let mut dm: DEVMODEW = unsafe { std::mem::zeroed() };
    dm.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    let mut index = 0u32;
    loop {
        let ok = unsafe { EnumDisplaySettingsW(std::ptr::null::<u16>(), index, &mut dm) };
        if ok == 0 {
            break;
        }
        if dm.dmPelsWidth > 0 && dm.dmPelsHeight > 0 {
            modes.push(DisplayMode {
                width: dm.dmPelsWidth,
                height: dm.dmPelsHeight,
                refresh: dm.dmDisplayFrequency,
            });
        }
        index += 1;
    }
    modes
}

pub fn desktop_resolution() -> Option<(u32, u32)> {
    let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    if w > 0 && h > 0 {
        Some((w as u32, h as u32))
    } else {
        None
    }
}

pub fn unique_resolutions(modes: &[DisplayMode]) -> Vec<(u32, u32)> {
    let mut list: Vec<(u32, u32)> = modes
        .iter()
        .map(|m| (m.width, m.height))
        .collect();
    list.sort_unstable();
    list.dedup();
    if list.is_empty() {
        list = vec![
            (1280, 720),
            (1366, 768),
            (1440, 900),
            (1600, 900),
            (1680, 1050),
            (1920, 1080),
            (2560, 1440),
            (3440, 1440),
            (3840, 2160),
        ];
    }
    if let Some(desktop) = desktop_resolution() {
        if !list.contains(&desktop) {
            list.insert(0, desktop);
        }
    }
    list
}

pub fn unique_refreshes(modes: &[DisplayMode]) -> Vec<u32> {
    let mut list: Vec<u32> = modes.iter().map(|m| m.refresh).collect();
    list.sort_unstable();
    list.dedup();
    if list.is_empty() {
        list = vec![60, 75, 90, 100, 120, 144, 165, 240];
    }
    if !list.contains(&60) {
        list.insert(0, 60);
    }
    list
}

//! Small Windows adapters. No runtime files, external programs or GUI framework.
use std::ffi::c_void;
use std::path::{Path, PathBuf};

#[repr(C)]
struct OpenFileName {
    size: u32,
    owner: *mut c_void,
    instance: *mut c_void,
    filter: *const u16,
    custom: *mut u16,
    max_custom: u32,
    index: u32,
    file: *mut u16,
    max_file: u32,
    file_title: *mut u16,
    max_title: u32,
    initial: *const u16,
    title: *const u16,
    flags: u32,
    offset: u16,
    extension: u16,
    default_extension: *const u16,
    custom_data: isize,
    hook: *mut c_void,
    template: *const u16,
    reserved: *mut c_void,
    reserved_num: u32,
    flags_ex: u32,
}
#[link(name = "comdlg32")]
unsafe extern "system" {
    fn GetOpenFileNameW(p: *mut OpenFileName) -> i32;
    fn GetSaveFileNameW(p: *mut OpenFileName) -> i32;
    fn CommDlgExtendedError() -> u32;
}
#[link(name = "user32")]
unsafe extern "system" {
    fn OpenClipboard(hwnd: *mut c_void) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, ptr: *mut c_void) -> *mut c_void;
    fn GetClipboardData(format: u32) -> *mut c_void;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, size: usize) -> *mut c_void;
    fn GlobalLock(ptr: *mut c_void) -> *mut c_void;
    fn GlobalUnlock(ptr: *mut c_void) -> i32;
    fn GlobalFree(ptr: *mut c_void) -> *mut c_void;
    fn GlobalSize(ptr: *mut c_void) -> usize;
    fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    fn GetLocalTime(time: *mut [u16; 8]);
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn local_date() -> String {
    let mut t = [0u16; 8];
    unsafe {
        GetLocalTime(&mut t);
    }
    format!("{:04}.{:02}.{:02}", t[0], t[1], t[3])
}

pub fn choose_pgn(save: bool, owner: usize, default_name: &str) -> Result<Option<PathBuf>, String> {
    choose_file(save, owner, default_name, 0)
}
pub fn choose_engine(owner: usize) -> Result<Option<PathBuf>, String> {
    choose_file(false, owner, "", 1)
}
pub fn choose_csv(owner: usize) -> Result<Option<PathBuf>, String> {
    choose_file(false, owner, "", 2)
}
pub fn choose_json(save: bool, owner: usize, name: &str) -> Result<Option<PathBuf>, String> {
    choose_file(save, owner, name, 3)
}
fn choose_file(
    save: bool,
    owner: usize,
    default_name: &str,
    kind: u8,
) -> Result<Option<PathBuf>, String> {
    let mut file = vec![0u16; 32768];
    let name = wide(default_name);
    file[..name.len()].copy_from_slice(&name);
    let filter = wide(if kind == 1 {
        "UCI engine (*.exe)\0*.exe\0"
    } else if kind == 2 {
        "Lichess puzzles (*.csv)\0*.csv\0"
    } else if kind == 3 {
        "Repertoire JSON (*.json)\0*.json\0"
    } else {
        "Portable Game Notation (*.pgn)\0*.pgn\0All files (*.*)\0*.*\0"
    });
    let ext = wide(if kind == 1 {
        "exe"
    } else if kind == 2 {
        "csv"
    } else if kind == 3 {
        "json"
    } else {
        "pgn"
    });
    let mut dialog: OpenFileName = unsafe { std::mem::zeroed() };
    dialog.size = std::mem::size_of::<OpenFileName>() as u32;
    dialog.owner = owner as *mut c_void;
    dialog.filter = filter.as_ptr();
    dialog.file = file.as_mut_ptr();
    dialog.max_file = file.len() as u32;
    dialog.default_extension = ext.as_ptr();
    dialog.flags = 0x0008_0000 | 0x0000_0008 | if save { 0x0000_0002 } else { 0x0000_1000 };
    let result = unsafe {
        if save {
            GetSaveFileNameW(&mut dialog)
        } else {
            GetOpenFileNameW(&mut dialog)
        }
    };
    if result == 0 {
        let error = unsafe { CommDlgExtendedError() };
        return if error == 0 {
            Ok(None)
        } else {
            Err(format!("File dialog error: {error:#x}"))
        };
    }
    let len = file
        .iter()
        .position(|c| *c == 0)
        .ok_or("Invalid file dialog path")?;
    use std::os::windows::ffi::OsStringExt;
    Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
        &file[..len],
    ))))
}

pub fn replace_file(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    let from: Vec<_> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<_> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0x1 | 0x8) } == 0 {
        Err(format!("Replace file: {}", std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}

pub fn copy_text(text: &str, owner: usize) -> Result<(), String> {
    let value = wide(text);
    unsafe {
        if OpenClipboard(owner as *mut c_void) == 0 {
            return Err("Clipboard is busy".into());
        }
        let result = (|| {
            let ptr = GlobalAlloc(0x2, value.len() * 2);
            if ptr.is_null() {
                return Err("Clipboard allocation failed".into());
            }
            let data = GlobalLock(ptr) as *mut u16;
            if data.is_null() {
                GlobalFree(ptr);
                return Err("Clipboard lock failed".into());
            }
            std::ptr::copy_nonoverlapping(value.as_ptr(), data, value.len());
            GlobalUnlock(ptr);
            if EmptyClipboard() == 0 || SetClipboardData(13, ptr).is_null() {
                GlobalFree(ptr);
                return Err("Clipboard write failed".into());
            }
            Ok(())
        })();
        CloseClipboard();
        result
    }
}
pub fn paste_text(owner: usize) -> Result<String, String> {
    unsafe {
        if OpenClipboard(owner as *mut c_void) == 0 {
            return Err("Clipboard is busy".into());
        }
        let result = (|| {
            let handle = GetClipboardData(13);
            if handle.is_null() {
                return Err("Clipboard has no Unicode text".into());
            }
            let bytes = GlobalSize(handle);
            if bytes > 1024 * 1024 {
                return Err("Clipboard text too large".into());
            }
            let ptr = GlobalLock(handle) as *const u16;
            if ptr.is_null() {
                return Err("Clipboard lock failed".into());
            }
            let chars = std::slice::from_raw_parts(ptr, bytes / 2);
            let len = chars.iter().position(|c| *c == 0).unwrap_or(chars.len());
            let result =
                String::from_utf16(&chars[..len]).map_err(|e| format!("Clipboard Unicode: {e}"));
            GlobalUnlock(handle);
            result
        })();
        CloseClipboard();
        result
    }
}

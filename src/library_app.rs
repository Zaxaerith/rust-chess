//! Library screen state and a single bounded import worker; no engine ownership.
use crate::{
    App, Key, Screen,
    advanced_app::Intent,
    library::{self, ImportReport, Library, Sort},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enter,
    Details,
    Back,
    Import,
    Open,
    Select(usize),
    Favorite,
    Favorites,
    Result,
    Sort,
    Remove,
    ConfirmRemove,
    CancelRemove,
    Export,
    Save,
    Add,
    Check,
    Search,
    Clear,
    Up,
    Down,
}
#[derive(Default)]
pub struct LibraryState {
    pub store: Option<Library>,
    pub root: Option<PathBuf>,
    pub error: Option<String>,
    pub query: String,
    pub favorites: bool,
    pub result: usize,
    pub sort: Sort,
    pub visible: Vec<String>,
    pub selected: Option<String>,
    pub scroll: usize,
    pub focus: bool,
    pub remove: Option<String>,
    pub notice: Option<String>,
    task: Option<Receiver<(Library, ImportReport)>>,
    pub progress: Arc<AtomicUsize>,
}
pub const RESULTS: [&str; 5] = ["", "*", "1-0", "0-1", "1/2-1/2"];
impl LibraryState {
    pub fn busy(&self) -> bool {
        self.task.is_some()
    }
    pub fn refresh(&mut self) {
        self.visible = self
            .store
            .as_ref()
            .map(|s| {
                s.search(
                    &self.query,
                    self.favorites,
                    (self.result > 0).then_some(RESULTS[self.result]),
                    self.sort,
                )
            })
            .unwrap_or_default();
        if self
            .selected
            .as_ref()
            .is_none_or(|id| !self.visible.contains(id))
        {
            self.selected = self.visible.first().cloned();
        }
        self.scroll = self.scroll.min(self.visible.len().saturating_sub(1));
    }
}
impl App {
    pub(crate) fn library_root(&self) -> Result<PathBuf, String> {
        if let Some(path) = &self.library.root {
            return Ok(path.clone());
        }
        let base = if let Some(path) = &self.records.save_path {
            path.parent().ok_or("Missing data directory")?.to_path_buf()
        } else if let Some(p) = std::env::var_os("RUST_CHESS_DATA_DIR") {
            PathBuf::from(p)
        } else {
            crate::persistence::data_path()?
                .parent()
                .ok_or("Missing data directory")?
                .to_path_buf()
        };
        Ok(base.join("library-v1"))
    }
    pub(crate) fn ensure_library(&mut self) -> Result<(), String> {
        if self.library.store.is_none() {
            self.library.store = Some(Library::open(self.library_root()?)?);
            self.library.refresh();
        }
        Ok(())
    }
    pub(crate) fn enter_library(&mut self) -> Result<(), String> {
        if self.defer_replay_loss(Intent::LibraryPage) {
            return Ok(());
        }
        self.tick_clock();
        self.cancel_review();
        self.cancel_analysis();
        if self.records.replay.is_some() {
            self.leave_replay();
        }
        self.screen = Screen::Library;
        self.ensure_library()
    }
    pub(crate) fn open_library_id(&mut self, id: &str) -> Result<(), String> {
        if self.library.busy() {
            return Err("Library import is running".into());
        }
        self.ensure_library()?;
        let tree = self.library.store.as_ref().expect("library").load(id)?;
        if self.defer_replay_loss(Intent::LibraryGame(id.into())) {
            return Ok(());
        }
        self.open_annotated_replay(tree, true);
        self.records.library_id = Some(id.into());
        self.records.replay_return = Screen::Library;
        Ok(())
    }
    pub(crate) fn save_replay_library(&mut self, add: bool) -> Result<(), String> {
        if self.library.busy() {
            return Err("Library import is running".into());
        }
        if self.advanced.editor.is_some() {
            self.apply_annotation_edit()?;
        }
        self.ensure_library()?;
        let tree = &self
            .records
            .replay
            .as_ref()
            .ok_or("Open Replay first")?
            .tree;
        let store = self.library.store.as_mut().expect("library");
        let id = if add {
            match store.add(tree)? {
                Some(id) => id,
                None => {
                    let hash = library::digest(crate::pgn::export_annotated(tree)?.as_bytes());
                    store
                        .entries
                        .iter()
                        .find(|e| e.digest == hash)
                        .ok_or("Duplicate entry missing")?
                        .id
                        .clone()
                }
            }
        } else {
            let id = self
                .records
                .library_id
                .as_ref()
                .ok_or("Replay is not a library game")?
                .clone();
            store.save(&id, tree)?;
            id
        };
        self.records.library_id = Some(id.clone());
        self.library.selected = Some(id);
        self.library.refresh();
        self.records.replay.as_mut().expect("replay").dirty = false;
        self.library.notice = Some(crate::library_ui::words(self.settings.language)[18].into());
        self.update_opening_members();
        Ok(())
    }
    pub(crate) fn start_library_import(&mut self, path: &Path) -> Result<(), String> {
        if self.screen != Screen::Library || self.library.busy() {
            return Err("Open an idle Library screen to import".into());
        }
        self.ensure_library()?;
        let mut store = self.library.store.as_ref().expect("library").clone();
        let path = path.to_path_buf();
        let progress = self.library.progress.clone();
        progress.store(0, Ordering::Relaxed);
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("pgn-library-import".into())
            .spawn(move || {
                let report = match library::read_bounded(&path, crate::pgn::MAX_COLLECTION_BYTES) {
                    Ok(bytes) => store.import(&bytes, |n| progress.store(n, Ordering::Relaxed)),
                    Err(e) => ImportReport {
                        failed: 1,
                        error: Some(format!("Read failed: {e}")),
                        ..ImportReport::default()
                    },
                };
                let _ = tx.send((store, report));
            })
            .map_err(|e| format!("Start library import: {e}"))?;
        self.library.task = Some(rx);
        self.library.notice = None;
        self.library.error = None;
        Ok(())
    }
    pub(crate) fn poll_library(&mut self) {
        let Some(rx) = &self.library.task else {
            return;
        };
        match rx.try_recv() {
            Ok((store, report)) => {
                self.library.store = Some(store);
                self.library.task = None;
                self.library.refresh();
                let t = crate::library_ui::words(self.settings.language);
                self.library.notice = Some(format!(
                    "{} {} · {} {} · {} {}",
                    t[19], report.added, t[20], report.skipped, t[21], report.failed
                ));
                self.library.error = report.error;
                self.update_opening_members();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.library.task = None;
                self.library.store = None;
                self.library.error = Some(
                    "Library worker exited unexpectedly; reopen index/check for interrupted files"
                        .into(),
                );
            }
        }
    }
    pub(crate) fn handle_library_action(&mut self, action: Action) {
        let result = (|| {
            if self.library.busy() {
                return Err(
                    "Library import is running; wait for the bounded import to finish".into(),
                );
            }
            let id = self.library.selected.clone();
            match action {
                Action::Enter => self.enter_library()?,
                Action::Details => {
                    self.records.message = self
                        .library
                        .error
                        .clone()
                        .or_else(|| self.library.notice.clone())
                }
                Action::Back => {
                    self.screen = Screen::Menu;
                    self.library.focus = false;
                }
                Action::Search => self.library.focus = true,
                Action::Clear => {
                    self.library.query.clear();
                    self.library.scroll = 0;
                    self.library.refresh();
                }
                Action::Up => self.library.scroll = self.library.scroll.saturating_sub(1),
                Action::Down => {
                    self.library.scroll =
                        (self.library.scroll + 1).min(self.library.visible.len().saturating_sub(1))
                }
                Action::Select(i) => {
                    self.library.selected = self.library.visible.get(i).cloned();
                    self.library.focus = false;
                }
                Action::Favorites => {
                    self.library.favorites = !self.library.favorites;
                    self.library.scroll = 0;
                    self.library.refresh();
                }
                Action::Result => {
                    self.library.result = (self.library.result + 1) % RESULTS.len();
                    self.library.scroll = 0;
                    self.library.refresh();
                }
                Action::Sort => {
                    self.library.sort = self.library.sort.next();
                    self.library.refresh();
                }
                Action::Open => {
                    if let Some(id) = id {
                        self.open_library_id(&id)?;
                    }
                }
                Action::Save => self.save_replay_library(false)?,
                Action::Add => self.save_replay_library(true)?,
                Action::Import => {
                    if let Some(path) =
                        crate::native_files::choose_pgn(false, self.records.owner, "")?
                    {
                        self.start_library_import(&path)?;
                    }
                }
                Action::Favorite => {
                    if let Some(id) = id {
                        self.ensure_library()?;
                        self.library
                            .store
                            .as_mut()
                            .expect("library")
                            .favorite(&id)?;
                        self.library.refresh();
                    }
                }
                Action::Remove => {
                    self.library.remove = id;
                    self.library.focus = false;
                }
                Action::CancelRemove => self.library.remove = None,
                Action::ConfirmRemove => {
                    if let Some(id) = self.library.remove.take() {
                        self.ensure_library()?;
                        let result = self.library.store.as_mut().expect("library").remove(&id);
                        self.library.refresh();
                        result?;
                    }
                }
                Action::Export => {
                    if let Some(id) = id {
                        self.ensure_library()?;
                        let game = self.library.store.as_ref().expect("library").load(&id)?;
                        let text = crate::pgn::export_annotated(&game)?;
                        if let Some(path) = crate::native_files::choose_pgn(
                            true,
                            self.records.owner,
                            "library-game.pgn",
                        )? {
                            // Refuse exporting over any archive/index/cache/user-data file.
                            self.guard_library_export(&path)?;
                            crate::persistence::atomic_write(&path, text.as_bytes())?;
                        }
                    }
                }
                Action::Check => {
                    self.ensure_library()?;
                    let result = self.library.store.as_mut().expect("library").recover();
                    self.library.refresh();
                    self.library.notice = Some(result?);
                }
            }
            Ok::<_, String>(())
        })();
        self.library.error = result.err();
        self.update_opening_members();
        if self.screen == Screen::Replay
            && let Some(e) = &self.library.error
        {
            self.records.message = Some(e.clone());
        }
    }
    pub(crate) fn guard_library_export(&self, path: &Path) -> Result<(), String> {
        fn key(path: &Path) -> Result<String, String> {
            let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
            let mut clean = PathBuf::new();
            for c in absolute.components() {
                match c {
                    std::path::Component::ParentDir => {
                        clean.pop();
                    }
                    std::path::Component::CurDir => {}
                    _ => clean.push(c.as_os_str()),
                }
            }
            let mut existing = clean.clone();
            let mut tail = vec![];
            while !existing.exists() {
                tail.push(
                    existing
                        .file_name()
                        .ok_or("Invalid export path")?
                        .to_os_string(),
                );
                if !existing.pop() {
                    return Err("Invalid export path".into());
                }
            }
            let mut resolved = existing.canonicalize().map_err(|e| e.to_string())?;
            for name in tail.into_iter().rev() {
                resolved.push(name);
            }
            Ok(resolved.to_string_lossy().to_lowercase())
        }
        let library = self.library_root()?;
        let data = library.parent().ok_or("Missing data directory")?;
        let root = key(data)?;
        let target = key(path)?;
        if target == root
            || target.starts_with(&format!("{root}\\"))
            || target.starts_with(&format!("{root}/"))
        {
            return Err("Export a separate copy outside the application data directory; use Save to Library for managed PGNs".into());
        }
        Ok(())
    }
    pub(crate) fn library_keys(&mut self, keys: &[Key], chars: &[char], ctrl: bool) -> bool {
        if self.screen != Screen::Library {
            return false;
        }
        if self.library.busy() {
            return true;
        }
        if self.library.remove.is_some() {
            if keys.contains(&Key::Escape) {
                self.library.remove = None;
            }
            return true;
        }
        if keys.contains(&Key::Escape) {
            if self.library.focus {
                self.library.focus = false;
            } else {
                self.handle_library_action(Action::Back);
            }
            return true;
        }
        if self.library.focus {
            let previous = self.library.query.clone();
            if ctrl && keys.contains(&Key::A) {
                self.library.query.clear();
            }
            if keys.contains(&Key::Backspace) {
                self.library.query.pop();
            }
            if !ctrl {
                for c in chars {
                    if !c.is_control() && self.library.query.len() + c.len_utf8() <= 512 {
                        self.library.query.push(*c);
                    }
                }
            }
            if self.library.query != previous {
                self.library.scroll = 0;
                self.library.refresh();
            }
            if keys.contains(&Key::Enter) {
                self.library.focus = false;
            }
        } else if keys.contains(&Key::Enter) {
            self.handle_library_action(Action::Open);
        }
        true
    }
}

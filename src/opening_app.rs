//! Independent explorer navigation and one cancellable local statistics worker.
use crate::{
    App, Chess, Key, Move, Position, Screen,
    advanced_app::Intent,
    game::position_key,
    opening_book::{Book, Match},
    opening_stats::{self, Counts, Stats},
};
use std::{
    path::PathBuf,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Idle,
    Loading,
    Running,
    Cancelling,
    Ready,
    Cancelled,
    Failed,
    Stale,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enter,
    Back,
    Build,
    Cancel,
    Previous,
    Next,
    First,
    Choose(usize),
    Reference(usize),
    Game(usize),
    Moves,
    Search,
    Games,
    Focus,
    Clear,
    Up,
    Down,
    Info,
}
pub struct Choice {
    pub uci: String,
    pub san: String,
    pub counts: Option<Counts>,
}
struct Staged(PathBuf);
impl Drop for Staged {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.0)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("Opening staging file preserved: {e}");
        }
    }
}
struct Output {
    book: Option<Arc<Book>>,
    stats: Result<Option<Stats>, String>,
    cached: bool,
    staged: Option<Staged>,
    cache_error: Option<String>,
}
struct Task {
    rx: Receiver<Output>,
    cancel: Arc<AtomicBool>,
    join: JoinHandle<()>,
    key: String,
    root: PathBuf,
}
pub struct Opening {
    pub book: Option<Arc<Book>>,
    pub stats: Option<Stats>,
    pub expected: String,
    pub status: Status,
    pub error: Option<String>,
    pub cached: bool,
    pub positions: Vec<Chess>,
    pub moves: Vec<Move>,
    pub cursor: usize,
    pub path: String,
    pub named: Option<Match>,
    pub choices: Vec<Choice>,
    pub related: Vec<(String, usize)>,
    pub tab: usize,
    pub query: String,
    pub focus: bool,
    pub found: Vec<usize>,
    pub scroll: usize,
    pub progress: Arc<AtomicUsize>,
    pub total: usize,
    task: Option<Task>,
}
impl Default for Opening {
    fn default() -> Self {
        Self {
            book: None,
            stats: None,
            expected: String::new(),
            status: Status::Idle,
            error: None,
            cached: false,
            positions: vec![Chess::default()],
            moves: vec![],
            cursor: 0,
            path: String::new(),
            named: None,
            choices: vec![],
            related: vec![],
            tab: 0,
            query: String::new(),
            focus: false,
            found: vec![],
            scroll: 0,
            progress: Arc::new(AtomicUsize::new(0)),
            total: 0,
            task: None,
        }
    }
}
pub(crate) fn shared_book(cancel: &AtomicBool) -> Result<Arc<Book>, String> {
    static BOOK: OnceLock<Arc<Book>> = OnceLock::new();
    if let Some(b) = BOOK.get() {
        return Ok(b.clone());
    }
    let book = Arc::new(Book::load(cancel)?);
    let _ = BOOK.set(book.clone());
    Ok(book)
}
impl Opening {
    pub fn position(&self) -> &Chess {
        &self.positions[self.cursor]
    }
    pub fn current_stats(&self) -> Option<&Stats> {
        self.stats.as_ref().filter(|s| s.key == self.expected)
    }
    pub fn busy(&self) -> bool {
        self.task.is_some()
    }
    pub fn refresh(&mut self) {
        self.named = self
            .book
            .as_ref()
            .and_then(|b| b.identify(self.position(), &self.moves[..self.cursor]));
        let key = position_key(self.position());
        let mut choices = std::collections::BTreeMap::<String, Choice>::new();
        if let Some(node) = self.current_stats().and_then(|s| s.nodes.get(&key)) {
            for (uci, counts) in &node.moves {
                if let Ok(Some(san)) = crate::uci_protocol::legal_move(self.position(), uci) {
                    choices.insert(
                        uci.clone(),
                        Choice {
                            uci: uci.clone(),
                            san,
                            counts: Some(counts.clone()),
                        },
                    );
                }
            }
        }
        if let Some(next) = self.book.as_ref().and_then(|b| b.next.get(&key)) {
            for (uci, san) in next {
                choices.entry(uci.clone()).or_insert_with(|| Choice {
                    uci: uci.clone(),
                    san: san.clone(),
                    counts: None,
                });
            }
        }
        self.choices = choices.into_values().collect();
        self.choices.sort_by(|a, b| {
            b.counts
                .as_ref()
                .map(Counts::total)
                .unwrap_or(0)
                .cmp(&a.counts.as_ref().map(Counts::total).unwrap_or(0))
                .then(a.uci.cmp(&b.uci))
        });
        self.related = self
            .current_stats()
            .and_then(|s| s.nodes.get(&key))
            .map(|n| n.games.iter().map(|(id, s)| (id.clone(), s.ply)).collect())
            .unwrap_or_default();
        self.path.clear();
        for i in 0..self.cursor {
            self.path.push_str(&format!(
                "{}{} ",
                if self.positions[i].turn() == shakmaty::Color::White {
                    format!("{}. ", self.positions[i].fullmoves())
                } else {
                    String::new()
                },
                shakmaty::san::SanPlus::from_move(self.positions[i].clone(), &self.moves[i])
            ));
        }
        self.scroll = 0;
    }
    pub fn choose(&mut self, index: usize) -> Result<(), String> {
        if self.cursor >= 160 {
            return Err("Explorer navigation limit: 160 plies".into());
        }
        let uci = self
            .choices
            .get(index)
            .ok_or("Unknown continuation")?
            .uci
            .clone();
        let mv = uci
            .parse::<shakmaty::uci::UciMove>()
            .map_err(|e| e.to_string())?
            .to_move(self.position())
            .map_err(|e| e.to_string())?;
        let mut pos = self.position().clone();
        pos.play_unchecked(&mv);
        self.moves.truncate(self.cursor);
        self.positions.truncate(self.cursor + 1);
        self.moves.push(mv);
        self.positions.push(pos);
        self.cursor += 1;
        self.refresh();
        Ok(())
    }
    pub fn reference(&mut self, id: usize) -> Result<(), String> {
        let moves = self
            .book
            .as_ref()
            .and_then(|b| b.entries.get(id))
            .ok_or("Unknown opening reference")?
            .moves
            .clone();
        let mut positions = vec![Chess::default()];
        for m in &moves {
            positions.push(
                positions
                    .last()
                    .expect("initial")
                    .clone()
                    .play(m)
                    .map_err(|e| e.to_string())?,
            );
        }
        let expected = &self.book.as_ref().expect("book").entries[id].key;
        if position_key(positions.last().expect("initial")) != *expected {
            return Err("Reference position mismatch".into());
        }
        self.moves = moves;
        self.positions = positions;
        self.cursor = self.moves.len();
        self.refresh();
        Ok(())
    }
    pub fn cancel(&mut self) {
        if let Some(t) = &self.task {
            t.cancel.store(true, Ordering::Relaxed);
            self.status = Status::Cancelling;
        }
    }
    pub fn shutdown(&mut self, timeout: Duration) -> bool {
        self.cancel();
        let until = Instant::now() + timeout;
        while self.task.as_ref().is_some_and(|t| !t.join.is_finished()) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(2));
        }
        if self.task.as_ref().is_some_and(|t| t.join.is_finished())
            && let Some(t) = self.task.take()
        {
            self.status = Status::Cancelled;
            return t.join.join().is_ok();
        }
        self.task.is_none()
    }
}
impl Drop for Opening {
    fn drop(&mut self) {
        if let Some(t) = &self.task {
            t.cancel.store(true, Ordering::Relaxed);
        }
    }
}
impl App {
    pub(crate) fn update_opening_members(&mut self) {
        if let Some(lib) = &self.library.store {
            self.opening.expected = opening_stats::identity(lib);
            if self
                .opening
                .stats
                .as_ref()
                .is_some_and(|s| s.key != self.opening.expected)
            {
                if !self.opening.busy() {
                    self.opening.status = Status::Stale;
                }
                self.opening.refresh();
            }
        }
    }
    pub(crate) fn enter_opening(&mut self) -> Result<(), String> {
        if self.defer_replay_loss(Intent::OpeningPage) {
            return Ok(());
        }
        if self.library.busy() {
            return Err("Library import is running".into());
        }
        self.tick_clock();
        self.cancel_review();
        self.cancel_analysis();
        if self.records.replay.is_some() {
            self.leave_replay();
        }
        self.library.store = Some(crate::library::Library::open(self.library_root()?)?);
        self.library.refresh();
        self.update_opening_members();
        self.screen = Screen::Opening;
        if self.opening.book.is_none() && !self.opening.busy() {
            self.start_opening(false)?;
        }
        Ok(())
    }
    pub(crate) fn start_opening(&mut self, build: bool) -> Result<(), String> {
        if self.opening.busy() || self.library.busy() {
            return Err("A library/statistics task is already running".into());
        }
        let lib = crate::library::Library::open(self.library_root()?)?;
        let key = opening_stats::identity(&lib);
        self.library.store = Some(lib.clone());
        self.library.refresh();
        self.update_opening_members();
        let root = lib.root.clone();
        let path = root
            .parent()
            .ok_or("Missing data directory")?
            .join("opening-stats-v1.json");
        let cancel = Arc::new(AtomicBool::new(false));
        let c = cancel.clone();
        let progress = self.opening.progress.clone();
        progress.store(0, Ordering::Relaxed);
        self.opening.total = lib.entries.len();
        let (tx, rx) = mpsc::sync_channel(1);
        let join = std::thread::Builder::new()
            .name("opening-statistics".into())
            .spawn(move || {
                let book = shared_book(&c);
                let out = match book {
                    Err(e) => Output {
                        book: None,
                        stats: Err(e),
                        cached: false,
                        staged: None,
                        cache_error: None,
                    },
                    Ok(book) => {
                        let stats = if build {
                            opening_stats::build(&lib, &book, &c, &progress).map(Some)
                        } else {
                            opening_stats::load(&path, &lib, &book, &c)
                        };
                        let mut staged = None;
                        let mut cache_error = None;
                        if build
                            && !c.load(Ordering::Relaxed)
                            && let Ok(Some(data)) = &stats
                        {
                            let pending = path.with_file_name(format!(
                                ".opening-stats-{}-{}.pending",
                                std::process::id(),
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos()
                            ));
                            match opening_stats::save(&pending, data) {
                                Ok(()) => staged = Some(Staged(pending)),
                                Err(e) => cache_error = Some(e),
                            }
                        }
                        Output {
                            staged,
                            cache_error,
                            book: Some(book),
                            cached: !build,
                            stats,
                        }
                    }
                };
                let _ = tx.send(out);
            })
            .map_err(|e| e.to_string())?;
        self.opening.task = Some(Task {
            rx,
            cancel,
            join,
            key,
            root,
        });
        self.opening.status = if build {
            Status::Running
        } else {
            Status::Loading
        };
        self.opening.error = None;
        Ok(())
    }
    pub(crate) fn poll_opening(&mut self) {
        let Some(task) = &self.opening.task else {
            return;
        };
        // Wait for thread exit as well as delivery, without blocking the UI.
        if !task.join.is_finished() {
            return;
        }
        let task = self.opening.task.take().expect("task");
        let output = match task.rx.try_recv() {
            Ok(o) => o,
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                self.opening.status = Status::Failed;
                self.opening.error = Some("Opening worker exited without a result".into());
                let _ = task.join.join();
                return;
            }
        };
        let _ = task.join.join();
        if let Some(book) = output.book {
            self.opening.book = Some(book);
            self.opening.found = self
                .opening
                .book
                .as_ref()
                .expect("book")
                .search(&self.opening.query);
        }
        if task.cancel.load(Ordering::Relaxed) {
            self.opening.status = Status::Cancelled;
            self.opening.refresh();
            return;
        }
        let latest = crate::library::Library::open(task.root.clone());
        match latest {
            Ok(lib) => {
                let key = opening_stats::identity(&lib);
                self.library.store = Some(lib);
                self.library.refresh();
                self.update_opening_members();
                if key != task.key {
                    self.opening.status = Status::Stale;
                    self.opening.error =
                        Some("Library changed during statistics; rebuild the new snapshot".into());
                    self.opening.refresh();
                    return;
                }
            }
            Err(e) => {
                self.opening.status = Status::Failed;
                self.opening.error = Some(e);
                return;
            }
        }
        match output.stats {
            Err(e) => {
                self.opening.status = Status::Failed;
                self.opening.error = Some(e);
            }
            Ok(None) => self.opening.status = Status::Idle,
            Ok(Some(stats)) => {
                self.opening.error = output.cache_error;
                if let Some(staged) = output.staged {
                    let target = task
                        .root
                        .parent()
                        .expect("data directory")
                        .join("opening-stats-v1.json");
                    self.opening.error =
                        crate::native_files::replace_file(&staged.0, &target).err();
                }
                self.opening.cached = output.cached;
                self.opening.stats = Some(stats);
                self.opening.status = Status::Ready;
                self.opening
                    .progress
                    .store(self.opening.total, Ordering::Relaxed);
            }
        }
        self.opening.refresh();
    }
    pub(crate) fn opening_game(&mut self, id: &str, key: &str) -> Result<(), String> {
        if self.defer_replay_loss(Intent::OpeningGame(id.into(), key.into())) {
            return Ok(());
        }
        self.ensure_library()?;
        let tree = self.library.store.as_ref().expect("library").load(id)?;
        let ply = tree
            .mainline
            .positions
            .iter()
            .take(opening_stats::PREFIX + 1)
            .position(|p| position_key(p) == key)
            .ok_or("Game no longer reaches this explorer position")?;
        self.open_annotated_replay(tree, true);
        self.records.library_id = Some(id.into());
        let r = self.records.replay.as_mut().ok_or("No Replay")?;
        r.jump(ply);
        r.scroll = ply.saturating_sub(2);
        self.records.replay_return = Screen::Opening;
        Ok(())
    }
    pub(crate) fn opening_view(&self, v: &mut crate::render::ViewState) {
        if self.screen != Screen::Opening {
            return;
        }
        v.pos = self.opening.position().clone();
        v.history_sans.clear();
        v.selected = None;
        v.legal_targets.clear();
        v.last_move = self
            .opening
            .moves
            .get(self.opening.cursor.saturating_sub(1))
            .filter(|_| self.opening.cursor > 0)
            .and_then(crate::uci_protocol::move_arrow);
        v.ai_thinking = false;
        v.hint_thinking = false;
        v.suggestion = None;
        v.promotion = None;
        v.move_confirmation_pending = false;
        v.animations.clear();
        v.dragging_from = None;
        v.game_over_progress = 0.0;
        v.result = None;
        v.analysis = crate::analysis_board::Visual::default();
        v.settings.mode = None;
    }
    pub(crate) fn handle_opening_action(&mut self, action: Action) {
        let result = (|| {
            match action {
                Action::Enter => self.enter_opening()?,
                Action::Back => {
                    self.opening.cancel();
                    self.enter_library()?;
                }
                Action::Build => self.start_opening(true)?,
                Action::Cancel => self.opening.cancel(),
                Action::Previous => {
                    self.opening.cursor = self.opening.cursor.saturating_sub(1);
                    self.opening.refresh();
                }
                Action::Next => {
                    self.opening.cursor = (self.opening.cursor + 1).min(self.opening.moves.len());
                    self.opening.refresh();
                }
                Action::First => {
                    self.opening.cursor = 0;
                    self.opening.refresh();
                }
                Action::Choose(i) => self.opening.choose(i)?,
                Action::Reference(i) => {
                    self.opening.reference(i)?;
                    self.opening.tab = 0;
                    self.opening.focus = false;
                }
                Action::Game(i) => {
                    let (id, _) = self
                        .opening
                        .related
                        .get(i)
                        .ok_or("Unknown related game")?
                        .clone();
                    let key = position_key(self.opening.position());
                    self.opening_game(&id, &key)?;
                }
                Action::Moves | Action::Search | Action::Games => {
                    self.opening.tab = match action {
                        Action::Moves => 0,
                        Action::Search => 1,
                        _ => 2,
                    };
                    self.opening.focus = self.opening.tab == 1;
                    self.opening.scroll = 0;
                }
                Action::Focus => self.opening.focus = true,
                Action::Clear => {
                    self.opening.query.clear();
                    if let Some(b) = &self.opening.book {
                        self.opening.found = b.search("");
                    }
                    self.opening.scroll = 0;
                }
                Action::Up => self.opening.scroll = self.opening.scroll.saturating_sub(1),
                Action::Down => {
                    self.opening.scroll = self
                        .opening
                        .scroll
                        .saturating_add(1)
                        .min(self.opening_list_len().saturating_sub(1))
                }
                Action::Info => {
                    let mut text = self.opening.error.clone().unwrap_or_default();
                    text.push_str(&format!(
                        "\n{}\n{}",
                        crate::opening_ui::words(self.settings.language)[27],
                        self.opening.path
                    ));
                    if let Some(b) = &self.opening.book
                        && let Some(m) = &self.opening.named
                    {
                        for &i in b
                            .positions
                            .get(&position_key(self.opening.position()))
                            .into_iter()
                            .flatten()
                            .take(8)
                        {
                            let e = &b.entries[i];
                            text.push_str(&format!(
                                "\n{} {}{}",
                                e.eco,
                                e.name,
                                if i == m.primary { " *" } else { "" }
                            ));
                        }
                    }
                    self.records.message = Some(text);
                }
            }
            Ok::<_, String>(())
        })();
        if let Err(e) = result {
            if self.screen != Screen::Opening {
                self.records.message = Some(e.clone());
            }
            self.opening.error = Some(e);
        }
    }
    pub(crate) fn opening_list_len(&self) -> usize {
        match self.opening.tab {
            0 => self.opening.choices.len(),
            1 => self.opening.found.len(),
            _ => self.opening.related.len(),
        }
    }
    pub(crate) fn opening_keys(&mut self, keys: &[Key], chars: &[char], ctrl: bool) -> bool {
        if self.screen != Screen::Opening {
            return false;
        }
        if self.opening.focus {
            let old = self.opening.query.clone();
            if ctrl && keys.contains(&Key::A) {
                self.opening.query.clear();
            }
            if keys.contains(&Key::Backspace) {
                self.opening.query.pop();
            }
            if !ctrl {
                for c in chars {
                    if !c.is_control() && self.opening.query.len() + c.len_utf8() <= 512 {
                        self.opening.query.push(*c);
                    }
                }
            }
            if old != self.opening.query {
                if let Some(b) = &self.opening.book {
                    self.opening.found = b.search(&self.opening.query);
                }
                self.opening.scroll = 0;
            }
            if keys.contains(&Key::Escape) || keys.contains(&Key::Enter) {
                self.opening.focus = false;
            }
            return true;
        }
        for (k, a) in [
            (Key::Escape, Action::Back),
            (Key::Left, Action::Previous),
            (Key::Right, Action::Next),
            (Key::Home, Action::First),
        ] {
            if keys.contains(&k) {
                self.handle_opening_action(a);
            }
        }
        true
    }
}

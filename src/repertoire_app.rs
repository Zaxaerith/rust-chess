//! Independent editor/training integration. No live moves or engine ownership.
use crate::{
    App, Chess, Duration, Instant, Key, Move, Position, Role, Screen, Square,
    advanced_app::Intent,
    repertoire::{self, Collection, Repertoire, Route, Side},
    repertoire_store,
    repertoire_train::{self, Mode, Progress, Session, State},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Edit,
    Train,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enter,
    Back,
    NewWhite,
    NewBlack,
    SelectBook(usize),
    Rename,
    BookNote,
    RemoveBook,
    ConfirmRemove,
    CancelRemove,
    Edit,
    Train,
    SaveProgress,
    DiscardProgress,
    SelectRoute(usize),
    SaveRoute,
    Branch,
    RemoveRoute,
    First,
    Previous,
    Next,
    Mode,
    Hint,
    Reveal,
    Retry,
    NextItem,
    FromOpening,
    FromLibrary,
    AddPending,
    PrefixLess,
    PrefixMore,
    Export,
    Import,
    ImportNew,
    ImportMerge,
    CancelImport,
    ApplyText,
    CancelText,
    RouteName,
    RouteNote,
    DiscardEdit,
    KeepEdit,
    Up,
    Down,
    Details,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirm {
    Book,
    Route,
    Leave,
    Progress,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKind {
    NewWhite,
    NewBlack,
    Rename,
    BookNote,
    RouteName,
    RouteNote,
}
pub struct Input {
    pub kind: InputKind,
    pub text: String,
    pub cursor: usize,
    original: String,
}
pub struct RepState {
    pub content: Collection,
    pub progress: Progress,
    pub root: Option<PathBuf>,
    pub selected: Option<String>,
    pub route: Option<String>,
    pub page: Page,
    pub positions: Vec<Chess>,
    pub moves: Vec<Move>,
    pub cursor: usize,
    pub path: String,
    pub eco: String,
    pub selected_square: Option<Square>,
    pub dragging: Option<Square>,
    pub promotion: Option<(Square, Square)>,
    pub dirty: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub scroll: usize,
    pub input: Option<Input>,
    pub pending_backup: Option<Collection>,
    pub confirm: Option<Confirm>,
    pub session: Option<Session>,
    pub training_status: String,
    pub training_detail: String,
    pub stats: String,
    pub book_stats: std::collections::BTreeMap<String, String>,
    stat_details: std::collections::BTreeMap<String, String>,
    pub mode: usize,
    pub seed: u64,
    pub import_plies: usize,
    pub pending_route: Option<Vec<String>>,
    pub route_name: String,
    pub route_note: String,
    pub reference: Option<Arc<crate::opening_book::Book>>,
    pub progress_blocked: bool,
    pub return_screen: Screen,
    source: Vec<String>,
    pending_action: Option<Action>,
    pending_global: Option<Intent>,
    reference_task: Option<JoinHandle<Result<Arc<crate::opening_book::Book>, String>>>,
    cancel: Arc<AtomicBool>,
}
impl Default for RepState {
    fn default() -> Self {
        Self {
            content: Collection::default(),
            progress: Progress::default(),
            root: None,
            selected: None,
            route: None,
            page: Page::Home,
            positions: vec![Chess::default()],
            moves: vec![],
            cursor: 0,
            path: String::new(),
            eco: String::new(),
            selected_square: None,
            dragging: None,
            promotion: None,
            dirty: false,
            error: None,
            notice: None,
            scroll: 0,
            input: None,
            pending_backup: None,
            confirm: None,
            session: None,
            training_status: String::new(),
            training_detail: String::new(),
            stats: String::new(),
            book_stats: Default::default(),
            stat_details: Default::default(),
            mode: 0,
            seed: 0,
            import_plies: 40,
            pending_route: None,
            route_name: String::new(),
            route_note: String::new(),
            reference: None,
            progress_blocked: false,
            return_screen: Screen::Menu,
            source: vec![],
            pending_action: None,
            pending_global: None,
            reference_task: None,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}
impl RepState {
    pub fn book(&self) -> Option<&Repertoire> {
        self.selected.as_deref().and_then(|id| self.content.get(id))
    }
    pub fn position(&self) -> &Chess {
        if self.page == Page::Train
            && let Some(s) = &self.session
        {
            &s.pos
        } else {
            &self.positions[self.cursor]
        }
    }
    pub fn modal(&self) -> bool {
        self.input.is_some() || self.confirm.is_some() || self.pending_backup.is_some()
    }
    pub fn unsaved(&self) -> bool {
        self.dirty || self.input.as_ref().is_some_and(|i| i.text != i.original)
    }
    pub fn shutdown(&mut self, timeout: Duration) -> bool {
        self.cancel.store(true, Ordering::Relaxed);
        let Some(join) = self.reference_task.take() else {
            return true;
        };
        let until = Instant::now() + timeout;
        while !join.is_finished() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !join.is_finished() {
            self.reference_task = Some(join);
            return false;
        }
        let _ = join.join();
        true
    }
    fn clear_input(&mut self) {
        self.selected_square = None;
        self.dragging = None;
        self.promotion = None;
    }
}
impl Drop for RepState {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
pub fn utc_day(seconds: u64) -> u64 {
    seconds / 86400
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
impl App {
    pub(crate) fn repertoire_root(&self) -> Result<PathBuf, String> {
        if let Some(p) = &self.repertoire.root {
            return if p.is_absolute() {
                Ok(p.clone())
            } else {
                std::env::current_dir()
                    .map(|d| d.join(p))
                    .map_err(|e| e.to_string())
            };
        }
        Ok(self
            .library_root()?
            .parent()
            .ok_or("Missing repertoire data directory")?
            .to_path_buf())
    }
    pub(crate) fn defer_repertoire_loss(&mut self, intent: Intent) -> bool {
        if self.screen != Screen::Repertoire {
            return false;
        }
        let pending = self
            .repertoire
            .session
            .as_ref()
            .is_some_and(Session::has_pending);
        if !self.repertoire.unsaved() && !pending {
            return false;
        }
        if self.repertoire.pending_global.is_none() {
            self.repertoire.pending_global = Some(intent);
            self.repertoire.confirm = Some(if pending {
                Confirm::Progress
            } else {
                Confirm::Leave
            });
        }
        true
    }
    fn rep_guard(&mut self, a: Action) -> bool {
        if self.repertoire.unsaved() {
            self.repertoire.pending_action = Some(a);
            self.repertoire.confirm = Some(Confirm::Leave);
            true
        } else {
            false
        }
    }
    pub(crate) fn enter_repertoire(&mut self) -> Result<(), String> {
        if self.defer_replay_loss(Intent::Repertoire(Action::Enter)) {
            return Ok(());
        }
        let root = self.repertoire_root()?;
        let content = repertoire_store::load(&root.join("repertoires-v1.json"))?;
        let previous = self.screen;
        self.tick_clock();
        self.repertoire.root = Some(root.clone());
        self.repertoire.content = content;
        match repertoire_train::load_progress(&root.join("repertoire-progress-v1.json")) {
            Ok(p) => {
                self.repertoire.progress = p;
                self.repertoire.progress_blocked = false;
            }
            Err(e) => {
                self.repertoire.error = Some(e);
                self.repertoire.progress_blocked = true;
            }
        }
        if previous != Screen::Repertoire {
            self.repertoire.return_screen = previous;
        }
        self.screen = Screen::Repertoire;
        self.repertoire.page = Page::Home;
        self.repertoire.scroll = 0;
        if self.repertoire.book().is_none() {
            self.repertoire.selected = self
                .repertoire
                .content
                .repertoires
                .first()
                .map(|b| b.id.clone());
        }
        // Reuse the pinned reference index, lazily off the rendering thread.
        if self.repertoire.reference.is_none() {
            self.repertoire.reference = self.opening.book.clone();
        }
        if self.repertoire.reference.is_none() && self.repertoire.reference_task.is_none() {
            let flag = self.repertoire.cancel.clone();
            flag.store(false, Ordering::Relaxed);
            self.repertoire.reference_task = Some(std::thread::spawn(move || {
                crate::opening_app::shared_book(&flag)
            }));
        }
        self.refresh_repertoire_stats();
        self.refresh_repertoire();
        Ok(())
    }
    pub(crate) fn repertoire_external(
        &mut self,
        moves: Vec<String>,
        source: Screen,
    ) -> Result<(), String> {
        if moves.is_empty() || moves.len() > repertoire::MAX_PLIES {
            return Err("Choose a standard route of 1..80 plies".into());
        }
        repertoire::validate_route(&Route {
            id: "preview".into(),
            name: String::new(),
            note: String::new(),
            moves: moves.clone(),
        })?;
        self.enter_repertoire()?;
        if self.screen != Screen::Repertoire {
            return Ok(());
        }
        self.repertoire.return_screen = source;
        self.repertoire.import_plies = moves.len();
        self.repertoire.source = moves.clone();
        self.repertoire.pending_route = Some(moves);
        self.refresh_repertoire();
        Ok(())
    }
    pub(crate) fn repertoire_library_prefix(&mut self) -> Result<(), String> {
        self.ensure_library()?;
        if self.library.busy() {
            return Err("Library import is running".into());
        }
        let id = self
            .library
            .selected
            .as_deref()
            .ok_or("Select a Library game")?;
        let game = self
            .library
            .store
            .as_ref()
            .ok_or("Library unavailable")?
            .load(id)?
            .mainline;
        if crate::fen::write(&game.positions[0]) != crate::fen::write(&Chess::default()) {
            return Err(
                "Repertoires require the standard initial position; FEN game rejected".into(),
            );
        }
        if game.moves.is_empty() {
            return Err("Library game contains no moves".into());
        }
        let source: Vec<_> = game
            .record
            .moves
            .into_iter()
            .take(repertoire::MAX_PLIES)
            .collect();
        let prefix = 40.min(source.len());
        self.repertoire_external(source[..prefix].to_vec(), Screen::Library)?;
        if self.screen == Screen::Repertoire {
            self.repertoire.source = source;
            self.repertoire.import_plies = prefix;
        }
        Ok(())
    }
    fn commit_repertoire(&mut self, next: Collection) -> Result<(), String> {
        repertoire_store::save(&self.repertoire_root()?.join("repertoires-v1.json"), &next)?;
        self.repertoire.content = next;
        self.repertoire.notice = Some("✓".into());
        if !self.repertoire.progress_blocked {
            self.repertoire.error = None;
        }
        self.refresh_repertoire_stats();
        self.refresh_repertoire();
        Ok(())
    }
    fn repertoire_stat_text(&self, s: &repertoire_train::Stats) -> String {
        let t = crate::repertoire_ui::words(self.settings.language);
        format!(
            "{} {} · {} {} · {} {} · {} {} · {} {:.0}% (n={}) · ✓{} · {} {} · {} {}",
            t[66],
            s.learned,
            t[67],
            s.new,
            t[68],
            s.due,
            t[69],
            s.completed,
            t[70],
            if s.recent_total > 0 {
                100.0 * s.recent_correct as f64 / s.recent_total as f64
            } else {
                0.0
            },
            s.recent_total,
            s.correct,
            t[71],
            s.errors,
            t[72],
            s.hints
        )
    }
    fn refresh_repertoire_stats(&mut self) {
        let mut rows = std::collections::BTreeMap::new();
        let mut details = std::collections::BTreeMap::new();
        let t = crate::repertoire_ui::words(self.settings.language);
        for b in &self.repertoire.content.repertoires {
            match self
                .repertoire
                .progress
                .stats(b, repertoire_train::today_utc())
            {
                Ok(s) => {
                    rows.insert(
                        b.id.clone(),
                        format!("{} {} · +{} · ✓{}", t[68], s.due, s.new, s.learned),
                    );
                    details.insert(b.id.clone(), self.repertoire_stat_text(&s));
                }
                Err(e) => self.repertoire.error = Some(e),
            }
        }
        self.repertoire.book_stats = rows;
        self.repertoire.stat_details = details;
    }
    pub(crate) fn refresh_repertoire(&mut self) {
        let t = crate::repertoire_ui::words(self.settings.language);
        let r = &mut self.repertoire;
        r.eco = if let Some(book) = &r.reference
            && let Some(m) = book.identify(
                r.position(),
                if r.page == Page::Edit {
                    &r.moves[..r.cursor]
                } else {
                    &[]
                },
            ) {
            let e = &book.entries[m.primary];
            format!("{} {} [{}]", e.eco, e.name, m.candidates)
        } else {
            t[45].into()
        };
        let mut path = String::new();
        if r.page == Page::Train {
            if let Some(s) = &r.session {
                path = s.path_sans();
            }
        } else {
            for i in 0..r.cursor {
                if r.positions[i].turn() == shakmaty::Color::White {
                    path.push_str(&format!("{}. ", r.positions[i].fullmoves()));
                }
                path.push_str(
                    &shakmaty::san::SanPlus::from_move(r.positions[i].clone(), &r.moves[i])
                        .to_string(),
                );
                path.push(' ');
            }
        }
        r.path = path;
        r.stats = r
            .selected
            .as_ref()
            .and_then(|id| r.stat_details.get(id))
            .cloned()
            .unwrap_or_default();
        if let Some(s) = &r.session {
            let state = t[match s.state {
                State::Waiting => 57,
                State::Responding => 58,
                State::Completed if s.remaining > 0 => 56,
                State::Completed => 59,
                State::Empty => 60,
            }];
            let feedback = t[match s.feedback {
                repertoire_train::Feedback::Ready => 57,
                repertoire_train::Feedback::Correct => 61,
                repertoire_train::Feedback::Outside => 62,
                repertoire_train::Feedback::Illegal => 63,
                repertoire_train::Feedback::Hint => 64,
                repertoire_train::Feedback::Revealed => 65,
                repertoire_train::Feedback::RouteEnd => 56,
                repertoire_train::Feedback::NoCards => 60,
            }];
            r.training_status =
                format!("{} · {}/{}", state, s.completed, s.completed + s.remaining);
            let card = s.current_key().and_then(|k| r.progress.entries.get(&k));
            r.training_detail = format!(
                "{} · L{} · ◷{}d · #{} · ?{} ×{}",
                feedback,
                card.map(|c| c.level).unwrap_or(0),
                card.map(|c| c.due_day.saturating_sub(repertoire_train::today_utc()))
                    .unwrap_or(0),
                s.allowed_uci().len(),
                s.hints,
                s.errors
            );
            // Prepared choices are answers: only an explicit hint/reveal may expose them.
            if s.feedback == repertoire_train::Feedback::Revealed {
                r.training_detail
                    .push_str(&format!(" · {}", s.allowed_sans().join(" / ")));
            }
        } else {
            r.training_status.clear();
            r.training_detail.clear();
        }
    }
    fn load_repertoire_route(&mut self, index: usize) -> Result<(), String> {
        let route = self
            .repertoire
            .book()
            .and_then(|b| b.routes.get(index))
            .ok_or("Route missing")?
            .clone();
        let v = repertoire::validate_route(&route)?;
        let r = &mut self.repertoire;
        r.route = Some(route.id);
        r.route_name = route.name;
        r.route_note = route.note;
        r.moves = v.moves;
        r.positions = v.positions;
        r.cursor = r.moves.len();
        r.dirty = false;
        r.page = Page::Edit;
        r.clear_input();
        self.refresh_repertoire();
        Ok(())
    }
    fn save_repertoire_route(&mut self) -> Result<(), String> {
        if self.repertoire.input.is_some() {
            self.apply_rep_text()?;
            if !self.repertoire.dirty {
                return Ok(());
            }
        }
        if !self.repertoire.dirty && self.repertoire.confirm == Some(Confirm::Leave) {
            return self.finish_repertoire_leave();
        }
        let r = &self.repertoire;
        let id = r.selected.clone().ok_or("Select a repertoire")?;
        let moves: Vec<_> = r.moves[..r.cursor]
            .iter()
            .map(|m| shakmaty::uci::UciMove::from_standard(m).to_string())
            .collect();
        let mut next = r.content.clone();
        let changed = if let Some(route) = &r.route {
            next.update_route(
                &id,
                route,
                moves.clone(),
                &r.route_name,
                &r.route_note,
                now(),
            )?
        } else {
            next.add_route(&id, moves.clone(), &r.route_name, &r.route_note, now())?
        };
        if !changed && r.route.is_some() {
            return Err("Identical route already exists; draft preserved".into());
        }
        let saved = next
            .get(&id)
            .and_then(|b| b.routes.iter().find(|line| line.moves == moves))
            .ok_or("Saved route missing")?
            .clone();
        self.commit_repertoire(next)?;
        self.repertoire.route = Some(saved.id);
        self.repertoire.route_name = saved.name;
        self.repertoire.route_note = saved.note;
        self.repertoire.dirty = false;
        self.repertoire.notice = Some(if changed { "✓" } else { "=" }.into());
        self.finish_repertoire_leave()?;
        Ok(())
    }
    fn finish_repertoire_leave(&mut self) -> Result<(), String> {
        if !matches!(
            self.repertoire.confirm,
            Some(Confirm::Leave | Confirm::Progress)
        ) {
            return Ok(());
        }
        self.repertoire.confirm = None;
        let global = self.repertoire.pending_global.take();
        let action = self.repertoire.pending_action.take();
        if let Some(intent) = global {
            self.finish_intent(intent)?;
        } else if let Some(a) = action {
            self.handle_repertoire_action(a);
        }
        Ok(())
    }
    fn rep_input(&mut self, kind: InputKind, text: String) {
        let cursor = text.len();
        self.repertoire.input = Some(Input {
            kind,
            text: text.clone(),
            cursor,
            original: text,
        });
    }
    fn apply_rep_text(&mut self) -> Result<(), String> {
        let input = self
            .repertoire
            .input
            .as_ref()
            .ok_or("No repertoire text input")?;
        let text = input.text.clone();
        let kind = input.kind;
        let mut next = self.repertoire.content.clone();
        let id = self.repertoire.selected.clone();
        match kind {
            InputKind::NewWhite | InputKind::NewBlack => {
                let id = next.create(
                    &text,
                    if kind == InputKind::NewWhite {
                        Side::White
                    } else {
                        Side::Black
                    },
                    now(),
                )?;
                self.commit_repertoire(next)?;
                self.repertoire.selected = Some(id);
            }
            InputKind::Rename | InputKind::BookNote => {
                let id = id.ok_or("Select repertoire")?;
                let b = next.get(&id).ok_or("Repertoire missing")?;
                let (name, note) = if kind == InputKind::Rename {
                    (text, b.note.clone())
                } else {
                    (b.name.clone(), text)
                };
                next.rename(&id, &name, &note, now())?;
                self.commit_repertoire(next)?;
            }
            InputKind::RouteName | InputKind::RouteNote => {
                let limit = if kind == InputKind::RouteName {
                    128
                } else {
                    4096
                };
                if text.len() > limit
                    || text.chars().any(|c| {
                        c.is_control() && (kind == InputKind::RouteName || (c != '\n' && c != '\t'))
                    })
                {
                    return Err("Invalid route text length/control character".into());
                }
                if kind == InputKind::RouteName {
                    self.repertoire.route_name = text;
                } else {
                    self.repertoire.route_note = text;
                }
                self.repertoire.dirty = true;
            }
        }
        self.repertoire.input = None;
        self.refresh_repertoire();
        if !self.repertoire.dirty {
            self.finish_repertoire_leave()?;
        }
        Ok(())
    }
    pub(crate) fn settle_repertoire(&mut self) -> Result<(), String> {
        if self
            .repertoire
            .session
            .as_ref()
            .is_none_or(|s| !s.has_pending())
        {
            return Ok(());
        }
        let mut session = self.repertoire.session.clone().expect("pending session");
        let mut progress = self.repertoire.progress.clone();
        if session.settle(&mut progress, repertoire_train::today_utc()) {
            let path = self.repertoire_root()?.join("repertoire-progress-v1.json");
            if let Err(e) = repertoire_train::save_progress(&path, &progress) {
                self.repertoire.progress_blocked = true;
                return Err(e);
            }
            self.repertoire.progress = progress;
            self.repertoire.session = Some(session);
            self.repertoire.progress_blocked = false;
            self.repertoire.error = None;
            if let (Some(id), Some(session)) =
                (self.repertoire.selected.clone(), &self.repertoire.session)
            {
                let stats = session.stats(&self.repertoire.progress, repertoire_train::today_utc());
                let text = self.repertoire_stat_text(&stats);
                let t = crate::repertoire_ui::words(self.settings.language);
                self.repertoire.book_stats.insert(
                    id.clone(),
                    format!(
                        "{} {} · +{} · ✓{}",
                        t[68], stats.due, stats.new, stats.learned
                    ),
                );
                self.repertoire.stat_details.insert(id, text);
            }
            self.refresh_repertoire();
        }
        Ok(())
    }
    pub(crate) fn poll_repertoire(&mut self) {
        if self
            .repertoire
            .reference_task
            .as_ref()
            .is_some_and(|t| t.is_finished())
        {
            match self.repertoire.reference_task.take().unwrap().join() {
                Ok(Ok(b)) => self.repertoire.reference = Some(b),
                Ok(Err(e)) => self.repertoire.error = Some(e),
                Err(_) => self.repertoire.error = Some("ECO index task failed".into()),
            };
            self.refresh_repertoire();
        }
        if self.screen == Screen::Repertoire
            && self.repertoire.page == Page::Train
            && !self.repertoire.modal()
        {
            if let Some(s) = &mut self.repertoire.session
                && s.state == State::Responding
            {
                s.respond();
                self.refresh_repertoire();
            }
            if !self.repertoire.progress_blocked
                && let Err(e) = self.settle_repertoire()
            {
                self.repertoire.error = Some(e);
                self.repertoire.progress_blocked = true;
            }
        }
    }
    fn safe_repertoire_backup(&self, path: &Path) -> Result<(), String> {
        let root = self.repertoire_root()?;
        let parent = path.parent().ok_or("Backup needs a directory")?;
        let root = root.canonicalize().unwrap_or(root);
        let parent = parent
            .canonicalize()
            .map_err(|e| format!("Backup directory: {e}"))?;
        if parent.starts_with(&root) {
            return Err("Choose a backup outside managed application data".into());
        }
        Ok(())
    }
    pub(crate) fn handle_repertoire_action(&mut self, a: Action) {
        if matches!(a, Action::Enter | Action::FromOpening | Action::FromLibrary)
            && self.defer_replay_loss(Intent::Repertoire(a))
        {
            return;
        }
        if self.screen == Screen::Repertoire
            && matches!(
                a,
                Action::Back
                    | Action::SelectBook(_)
                    | Action::SelectRoute(_)
                    | Action::Edit
                    | Action::Train
                    | Action::Import
                    | Action::FromOpening
                    | Action::FromLibrary
                    | Action::Enter
                    | Action::NewWhite
                    | Action::NewBlack
            )
            && self.rep_guard(a)
        {
            return;
        }
        let result = (|| -> Result<(), String> {
            if matches!(
                a,
                Action::Back
                    | Action::SelectBook(_)
                    | Action::Edit
                    | Action::Train
                    | Action::SelectRoute(_)
                    | Action::Enter
                    | Action::Import
                    | Action::Mode
            ) && self
                .repertoire
                .session
                .as_ref()
                .is_some_and(Session::has_pending)
                && let Err(e) = self.settle_repertoire()
            {
                self.repertoire.pending_action = Some(a);
                self.repertoire.confirm = Some(Confirm::Progress);
                return Err(e);
            }
            match a {
                Action::Enter => self.enter_repertoire()?,
                Action::FromOpening => {
                    let moves = self.opening.moves[..self.opening.cursor]
                        .iter()
                        .map(|m| shakmaty::uci::UciMove::from_standard(m).to_string())
                        .collect();
                    self.repertoire_external(moves, Screen::Opening)?;
                }
                Action::FromLibrary => self.repertoire_library_prefix()?,
                _ if self.screen != Screen::Repertoire => return Ok(()),
                Action::Back => {
                    self.settle_repertoire()?;
                    self.repertoire.clear_input();
                    if self.repertoire.page == Page::Home {
                        self.screen = self.repertoire.return_screen;
                        self.clock_last_tick = Instant::now();
                    } else {
                        self.repertoire.page = Page::Home;
                        self.repertoire.session = None;
                    }
                }
                Action::SelectBook(i) => {
                    self.repertoire.page = Page::Home;
                    self.repertoire.session = None;
                    let id = self
                        .repertoire
                        .content
                        .repertoires
                        .get(i)
                        .ok_or("Repertoire missing")?
                        .id
                        .clone();
                    self.repertoire.selected = Some(id);
                    self.repertoire.route = None;
                    self.repertoire.scroll = 0;
                }
                Action::NewWhite => self.rep_input(InputKind::NewWhite, String::new()),
                Action::NewBlack => self.rep_input(InputKind::NewBlack, String::new()),
                Action::Rename | Action::BookNote => {
                    let b = self.repertoire.book().ok_or("Select repertoire")?;
                    let (kind, text) = if a == Action::Rename {
                        (InputKind::Rename, b.name.clone())
                    } else {
                        (InputKind::BookNote, b.note.clone())
                    };
                    self.rep_input(kind, text);
                }
                Action::RouteName => {
                    self.rep_input(InputKind::RouteName, self.repertoire.route_name.clone())
                }
                Action::RouteNote => {
                    self.rep_input(InputKind::RouteNote, self.repertoire.route_note.clone())
                }
                Action::SaveProgress => {
                    self.settle_repertoire()?;
                    self.finish_repertoire_leave()?;
                }
                Action::DiscardProgress => {
                    self.repertoire.session = None;
                    self.finish_repertoire_leave()?;
                }
                Action::ApplyText => self.apply_rep_text()?,
                Action::CancelText => self.repertoire.input = None,
                Action::RemoveBook => {
                    self.repertoire.book().ok_or("Select repertoire")?;
                    self.repertoire.confirm = Some(Confirm::Book);
                }
                Action::RemoveRoute => {
                    self.repertoire
                        .route
                        .as_ref()
                        .ok_or("Select a saved route")?;
                    self.repertoire.confirm = Some(Confirm::Route);
                }
                Action::CancelRemove => self.repertoire.confirm = None,
                Action::ConfirmRemove => {
                    let mut next = self.repertoire.content.clone();
                    let id = self
                        .repertoire
                        .selected
                        .clone()
                        .ok_or("Select repertoire")?;
                    match self.repertoire.confirm {
                        Some(Confirm::Book) => next.remove_book(&id)?,
                        Some(Confirm::Route) => next.remove_route(
                            &id,
                            self.repertoire.route.as_deref().ok_or("Route missing")?,
                            now(),
                        )?,
                        _ => return Err("No removal confirmation".into()),
                    };
                    self.commit_repertoire(next)?;
                    self.repertoire.confirm = None;
                    self.repertoire.route = None;
                    self.repertoire.dirty = false;
                    self.repertoire.moves.clear();
                    self.repertoire.positions = vec![Chess::default()];
                    self.repertoire.cursor = 0;
                    if self.repertoire.book().is_none() {
                        self.repertoire.selected = self
                            .repertoire
                            .content
                            .repertoires
                            .first()
                            .map(|b| b.id.clone());
                    }
                }
                Action::Edit => {
                    self.repertoire.book().ok_or("Select repertoire")?;
                    if self.repertoire.book().is_some_and(|b| !b.routes.is_empty()) {
                        self.load_repertoire_route(0)?;
                    } else {
                        let r = &mut self.repertoire;
                        r.page = Page::Edit;
                        r.route = None;
                        r.moves.clear();
                        r.positions = vec![Chess::default()];
                        r.cursor = 0;
                        r.route_name.clear();
                        r.route_note.clear();
                        r.dirty = false;
                    }
                }
                Action::SelectRoute(i) => self.load_repertoire_route(i)?,
                Action::SaveRoute => self.save_repertoire_route()?,
                Action::Branch => {
                    let r = &mut self.repertoire;
                    r.route = None;
                    r.moves.truncate(r.cursor);
                    r.positions.truncate(r.cursor + 1);
                    r.dirty = !r.moves.is_empty();
                    r.clear_input();
                }
                Action::First if self.repertoire.page != Page::Edit => {}
                Action::Previous if self.repertoire.page != Page::Edit => {}
                Action::Next if self.repertoire.page != Page::Edit => {}
                Action::First => {
                    self.repertoire.cursor = 0;
                    self.repertoire.clear_input();
                }
                Action::Previous => {
                    self.repertoire.cursor = self.repertoire.cursor.saturating_sub(1);
                    self.repertoire.clear_input();
                }
                Action::Next => {
                    self.repertoire.cursor =
                        (self.repertoire.cursor + 1).min(self.repertoire.moves.len());
                    self.repertoire.clear_input();
                }
                Action::Mode => {
                    let mode = (self.repertoire.mode + 1) % 4;
                    if self.repertoire.page == Page::Train {
                        if self.repertoire.progress_blocked {
                            return Err("Training progress is damaged/unwritable; restore it before changing sessions".into());
                        }
                        let book = self.repertoire.book().ok_or("Select repertoire")?;
                        let session = Session::new(
                            book,
                            [Mode::Due, Mode::New, Mode::All, Mode::Errors][mode],
                            &self.repertoire.progress,
                            repertoire_train::today_utc(),
                            self.repertoire.seed,
                        )?;
                        self.repertoire.session = Some(session);
                        self.repertoire.seed = self.repertoire.seed.wrapping_add(1);
                        self.repertoire.clear_input();
                    }
                    self.repertoire.mode = mode;
                }
                Action::Train => {
                    if self.repertoire.progress_blocked {
                        return Err("Training progress is damaged/unwritable; restore it before starting another session".into());
                    }
                    let book = self.repertoire.book().ok_or("Select repertoire")?;
                    let session = Session::new(
                        book,
                        [Mode::Due, Mode::New, Mode::All, Mode::Errors][self.repertoire.mode],
                        &self.repertoire.progress,
                        utc_day(now()),
                        self.repertoire.seed,
                    )?;
                    self.repertoire.seed = self.repertoire.seed.wrapping_add(1);
                    self.repertoire.session = Some(session);
                    self.repertoire.page = Page::Train;
                    self.repertoire.clear_input();
                }
                Action::Hint => {
                    if let Some(s) = &mut self.repertoire.session {
                        s.hint();
                    }
                    self.settle_repertoire()?;
                }
                Action::Reveal => {
                    if let Some(s) = &mut self.repertoire.session {
                        self.records.message = Some(s.reveal());
                    }
                    self.settle_repertoire()?;
                }
                Action::Retry => {
                    if let Some(s) = &mut self.repertoire.session {
                        s.retry();
                    }
                    self.repertoire.clear_input();
                }
                Action::NextItem => {
                    self.settle_repertoire()?;
                    if let Some(s) = &mut self.repertoire.session {
                        s.next();
                    }
                    self.repertoire.clear_input();
                }
                Action::AddPending => {
                    let id = self
                        .repertoire
                        .selected
                        .clone()
                        .ok_or("Choose target repertoire")?;
                    let moves = self
                        .repertoire
                        .pending_route
                        .clone()
                        .ok_or("No pending route")?;
                    let mut next = self.repertoire.content.clone();
                    let added = next.add_route(&id, moves, "", "", now())?;
                    if added {
                        self.commit_repertoire(next)?;
                    }
                    self.repertoire.notice = Some(if added { "✓" } else { "=" }.into());
                    self.repertoire.pending_route = None;
                    self.repertoire.source.clear();
                }
                Action::PrefixLess | Action::PrefixMore => {
                    if !self.repertoire.source.is_empty() {
                        let n = if a == Action::PrefixLess {
                            self.repertoire.import_plies.saturating_sub(1).max(1)
                        } else {
                            (self.repertoire.import_plies + 1).min(self.repertoire.source.len())
                        };
                        self.repertoire.import_plies = n;
                        self.repertoire.pending_route = Some(self.repertoire.source[..n].to_vec());
                    }
                }
                Action::Export => {
                    if let Some(path) = crate::native_files::choose_json(
                        true,
                        self.records.owner,
                        "repertoires-backup.json",
                    )? {
                        self.safe_repertoire_backup(&path)?;
                        repertoire_store::export(&path, &self.repertoire.content)?;
                        self.repertoire.notice = Some("✓".into());
                    }
                }
                Action::Import => {
                    if let Some(path) =
                        crate::native_files::choose_json(false, self.records.owner, "")?
                    {
                        self.repertoire.pending_backup = Some(repertoire_store::load(&path)?);
                    }
                }
                Action::CancelImport => self.repertoire.pending_backup = None,
                Action::ImportNew | Action::ImportMerge => {
                    let incoming = self
                        .repertoire
                        .pending_backup
                        .as_ref()
                        .ok_or("No backup selected")?;
                    let next = if a == Action::ImportNew {
                        repertoire_store::import_new(&self.repertoire.content, incoming, now())?
                    } else {
                        repertoire_store::merge(
                            &self.repertoire.content,
                            incoming,
                            self.repertoire
                                .selected
                                .as_deref()
                                .ok_or("Choose merge target")?,
                            now(),
                        )?
                    };
                    self.commit_repertoire(next)?;
                    self.repertoire.pending_backup = None;
                    if self.repertoire.book().is_none() {
                        self.repertoire.selected = self
                            .repertoire
                            .content
                            .repertoires
                            .first()
                            .map(|b| b.id.clone());
                    }
                }
                Action::DiscardEdit => {
                    self.repertoire.dirty = false;
                    self.repertoire.input = None;
                    self.repertoire.clear_input();
                    self.finish_repertoire_leave()?;
                }
                Action::KeepEdit => {
                    self.repertoire.confirm = None;
                    self.repertoire.pending_action = None;
                    self.repertoire.pending_global = None;
                }
                Action::Up => self.repertoire.scroll = self.repertoire.scroll.saturating_sub(1),
                Action::Down => self.repertoire.scroll = (self.repertoire.scroll + 1).min(999),
                Action::Details => {
                    self.records.message = Some(format!(
                        "{}\n{}\n{}\n{}\n{}",
                        self.repertoire.error.as_deref().unwrap_or(""),
                        self.repertoire.notice.as_deref().unwrap_or(""),
                        self.repertoire.stats,
                        self.repertoire.route_note,
                        self.repertoire
                            .book()
                            .map(|b| b.note.as_str())
                            .unwrap_or("")
                    ));
                }
            }
            self.refresh_repertoire();
            Ok(())
        })();
        if let Err(e) = result {
            if self.screen != Screen::Repertoire {
                self.records.message = Some(e.clone());
            }
            self.repertoire.error = Some(e);
        }
    }
    pub(crate) fn repertoire_move(&mut self, mv: Move) {
        let result = (|| -> Result<(), String> {
            if self.repertoire.modal() {
                return Ok(());
            }
            match self.repertoire.page {
                Page::Home => {}
                Page::Edit => {
                    let r = &mut self.repertoire;
                    if r.cursor >= repertoire::MAX_PLIES {
                        return Err("Route exceeds 80 plies".into());
                    }
                    let mut pos = r.position().clone();
                    if !pos.is_legal(&mv) {
                        return Err("Illegal move".into());
                    }
                    pos.play_unchecked(&mv);
                    r.moves.truncate(r.cursor);
                    r.positions.truncate(r.cursor + 1);
                    r.moves.push(mv);
                    r.positions.push(pos);
                    r.cursor += 1;
                    r.dirty = true;
                }
                Page::Train => {
                    if let Some(s) = &mut self.repertoire.session {
                        if !s.attempt(mv)? {
                            self.repertoire.notice = Some("∉".into());
                        } else {
                            self.repertoire.notice = Some("✓".into());
                        }
                        self.settle_repertoire()?;
                    }
                }
            }
            Ok(())
        })();
        self.repertoire.clear_input();
        if let Err(e) = result {
            self.repertoire.error = Some(e);
        }
        self.refresh_repertoire();
    }
    fn repertoire_to(&mut self, from: Square, to: Square) {
        let moves: Vec<_> = self
            .repertoire
            .position()
            .legal_moves()
            .into_iter()
            .filter(|m| m.from() == Some(from) && crate::puzzle_app::target(m) == to)
            .collect();
        if moves.iter().any(|m| m.promotion().is_some()) {
            self.repertoire.promotion = Some((from, to));
        } else if let Some(m) = moves.first() {
            self.repertoire_move(m.clone());
        } else {
            self.repertoire.notice =
                Some(crate::repertoire_ui::words(self.settings.language)[63].into());
            if self.repertoire.page == Page::Train
                && let Some(session) = &mut self.repertoire.session
            {
                session.feedback = repertoire_train::Feedback::Illegal;
            }
            self.repertoire.clear_input();
            self.refresh_repertoire();
        }
    }
    pub(crate) fn repertoire_down(&mut self, sq: Square) {
        if self.repertoire.modal()
            || self.repertoire.promotion.is_some()
            || self.repertoire.page == Page::Home
        {
            return;
        }
        if self.repertoire.page == Page::Train
            && self
                .repertoire
                .session
                .as_ref()
                .is_none_or(|s| s.state != State::Waiting)
        {
            return;
        }
        let pos = self.repertoire.position();
        if let Some(from) = self.repertoire.selected_square
            && from != sq
            && pos
                .board()
                .piece_at(sq)
                .is_none_or(|p| p.color != pos.turn())
        {
            self.repertoire_to(from, sq);
            return;
        }
        if pos
            .board()
            .piece_at(sq)
            .is_some_and(|p| p.color == pos.turn())
        {
            self.repertoire.selected_square = Some(sq);
            self.repertoire.dragging = Some(sq);
        }
    }
    pub(crate) fn repertoire_up(&mut self, sq: Option<Square>) {
        if let Some(from) = self.repertoire.dragging.take()
            && let Some(to) = sq
            && from != to
        {
            self.repertoire_to(from, to);
        }
    }
    pub(crate) fn repertoire_promote(&mut self, role: Role) {
        if let Some((from, to)) = self.repertoire.promotion
            && let Some(m) = self.repertoire.position().legal_moves().iter().find(|m| {
                m.from() == Some(from)
                    && crate::puzzle_app::target(m) == to
                    && m.promotion() == Some(role)
            })
        {
            self.repertoire_move(m.clone());
        }
    }
    pub(crate) fn repertoire_view(&self, v: &mut crate::render::ViewState) {
        if self.screen != Screen::Repertoire {
            return;
        }
        let r = &self.repertoire;
        if r.page == Page::Home {
            v.screen = Screen::Library;
        }
        v.pos = r.position().clone();
        v.analysis = Default::default();
        v.animations.clear();
        v.history_sans.clear();
        v.selected = r.selected_square.or_else(|| {
            r.session
                .as_ref()
                .filter(|_| r.page == Page::Train)
                .and_then(|s| s.hint_square)
        });
        v.dragging_from = r.dragging;
        v.legal_targets = r
            .selected_square
            .map(|from| {
                v.pos
                    .legal_moves()
                    .iter()
                    .filter(|m| m.from() == Some(from))
                    .map(crate::puzzle_app::target)
                    .collect()
            })
            .unwrap_or_default();
        v.last_move = if r.page == Page::Edit {
            r.moves
                .get(r.cursor.wrapping_sub(1))
                .and_then(crate::uci_protocol::move_arrow)
        } else {
            r.session
                .as_ref()
                .and_then(|s| s.path.last())
                .and_then(|u| match u.parse::<shakmaty::uci::UciMove>().ok()? {
                    shakmaty::uci::UciMove::Normal { from, to, .. } => Some((from, to)),
                    _ => None,
                })
        };
        v.promotion = r.promotion;
        v.settings.mode = r.book().map(|b| !b.side.color());
        v.settings.flip_for_black = true;
        v.settings.board.zen_mode = crate::preferences::ZenMode::No;
        v.settings.board.immersive_mode = false;
        v.move_confirmation_pending = false;
        v.result = None;
        v.game_over_progress = 0.0;
        v.ai_thinking = false;
        v.hint_thinking = false;
        v.suggestion = None;
        v.search_failed = false;
    }
    pub(crate) fn repertoire_keys(&mut self, keys: &[Key], chars: &[char], ctrl: bool) -> bool {
        if self.screen != Screen::Repertoire {
            return false;
        }
        if self.repertoire.input.is_some() {
            if keys.contains(&Key::Escape) {
                self.handle_repertoire_action(Action::CancelText);
                return true;
            }
            if keys.contains(&Key::Enter) {
                self.handle_repertoire_action(Action::ApplyText);
                return true;
            }
            let pasted = if ctrl && keys.contains(&Key::V) {
                crate::native_files::paste_text(self.records.owner).ok()
            } else {
                None
            };
            if let Some(i) = &mut self.repertoire.input {
                if ctrl && keys.contains(&Key::A) {
                    i.text.clear();
                    i.cursor = 0;
                }
                if keys.contains(&Key::Left) && i.cursor > 0 {
                    i.cursor = i.text[..i.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(n, _)| n)
                        .unwrap_or(0);
                }
                if keys.contains(&Key::Right) && i.cursor < i.text.len() {
                    i.cursor += i.text[i.cursor..].chars().next().unwrap().len_utf8();
                }
                if keys.contains(&Key::Home) {
                    i.cursor = 0;
                }
                if keys.contains(&Key::End) {
                    i.cursor = i.text.len();
                }
                if keys.contains(&Key::Backspace) && i.cursor > 0 {
                    let start = i.text[..i.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(n, _)| n)
                        .unwrap_or(0);
                    i.text.replace_range(start..i.cursor, "");
                    i.cursor = start;
                }
                if keys.contains(&Key::Delete) && i.cursor < i.text.len() {
                    let end = i.cursor + i.text[i.cursor..].chars().next().unwrap().len_utf8();
                    i.text.replace_range(i.cursor..end, "");
                }
                let insert = pasted.unwrap_or_else(|| {
                    if !ctrl {
                        chars.iter().filter(|c| !c.is_control()).collect()
                    } else {
                        String::new()
                    }
                });
                if i.text.len() + insert.len() <= 4096 {
                    i.text.insert_str(i.cursor, &insert);
                    i.cursor += insert.len();
                }
            }
            return true;
        }
        if self.repertoire.modal() {
            if keys.contains(&Key::Escape) {
                self.handle_repertoire_action(if self.repertoire.pending_backup.is_some() {
                    Action::CancelImport
                } else if self.repertoire.confirm == Some(Confirm::Leave) {
                    Action::KeepEdit
                } else {
                    Action::CancelRemove
                });
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
                self.handle_repertoire_action(a);
            }
        }
        true
    }
}

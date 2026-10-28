//! Replay-only metadata editing, close/replacement guards and frame-driven playback.
use crate::{
    App, Instant, Key, Screen,
    annotated_game::{self, AnnotatedGame},
    records_ui,
    render::UiAction,
};
use std::path::Path;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Before,
    After,
    Nags,
}
pub struct Editor {
    pub node: usize,
    pub kind: EditKind,
    pub text: String,
    original: String,
    pub cursor: usize,
    pub selected: bool,
}
pub enum Intent {
    Back,
    LibraryPage,
    OpeningPage,
    PuzzlePage,
    Repertoire(crate::repertoire_app::Action),
    OpeningGame(String, String),
    LibraryGame(String),
    Quit,
    Record(records_ui::Action),
    Ui(UiAction),
    Replace(Box<AnnotatedGame>, bool),
    Fen(String),
    Restore(Box<crate::persistence::Snapshot>),
}
pub struct AdvancedState {
    pub editor: Option<Editor>,
    pub pending: Option<Intent>,
    pub error: Option<String>,
    pub choices: bool,
    pub choice_scroll: usize,
    pub note_scroll: usize,
    pub last_tick: Instant,
}
impl Default for AdvancedState {
    fn default() -> Self {
        Self {
            editor: None,
            pending: None,
            error: None,
            choices: false,
            choice_scroll: 0,
            note_scroll: 0,
            last_tick: Instant::now(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Play,
    Speed,
    Choices,
    Choose(usize),
    Node(usize),
    Parent,
    Mainline,
    Comment,
    Nags,
    Apply,
    CancelEdit,
    Clear,
    ToggleBefore,
    Paste,
    CancelExit,
    Discard,
    ExportExit,
    NoteUp,
    NoteDown,
}
impl App {
    pub(crate) fn advanced_modal(&self) -> bool {
        self.advanced.editor.is_some() || self.advanced.pending.is_some()
    }
    pub(crate) fn needs_replay_save(&self) -> bool {
        self.records.replay.as_ref().is_some_and(|r| r.dirty)
            || self
                .advanced
                .editor
                .as_ref()
                .is_some_and(|e| e.text != e.original)
    }
    pub(crate) fn defer_replay_loss(&mut self, intent: Intent) -> bool {
        if !self.needs_replay_save() {
            return false;
        }
        if let Some(r) = &mut self.records.replay {
            r.pause();
        }
        if self.advanced.pending.is_none() {
            self.advanced.pending = Some(intent);
        }
        true
    }
    pub(crate) fn finish_intent(&mut self, intent: Intent) -> Result<(), String> {
        match intent {
            Intent::Back => self.leave_replay(),
            Intent::LibraryPage => self.enter_library()?,
            Intent::OpeningPage => self.enter_opening()?,
            Intent::PuzzlePage => self.enter_puzzles()?,
            Intent::Repertoire(a) => self.handle_repertoire_action(a),
            Intent::OpeningGame(id, key) => self.opening_game(&id, &key)?,
            Intent::LibraryGame(id) => self.open_library_id(&id)?,
            Intent::Quit => self.exit_requested = true,
            Intent::Record(a) => self.handle_record_action(a),
            Intent::Ui(a) => self.handle_action(a),
            Intent::Replace(tree, external) => self.open_annotated_replay(*tree, external),
            Intent::Fen(text) => self.import_fen(&text)?,
            Intent::Restore(snapshot) => self.restore_snapshot(*snapshot)?,
        }
        Ok(())
    }
    pub(crate) fn open_annotated_replay(&mut self, tree: AnnotatedGame, external: bool) {
        if self.needs_replay_save() {
            self.defer_replay_loss(Intent::Replace(Box::new(tree), external));
            return;
        }
        self.cancel_review();
        self.review.panel = false;
        if external {
            self.analysis_context(true);
        } else {
            self.cancel_analysis();
        }
        self.tick_clock();
        if self.screen != Screen::Replay {
            self.records.replay_return = self.screen;
        }
        self.records.library_id = None;
        self.records.replay = Some(crate::replay::Replay::from_annotated(tree, external));
        if let Some(r) = &mut self.records.replay {
            r.scroll = r.path_cursor().saturating_sub(2);
        }
        self.screen = Screen::Replay;
        self.advanced = AdvancedState::default();
    }
    pub(crate) fn import_annotated_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let tree = crate::pgn::import_annotated(bytes)?;
        self.open_annotated_replay(tree, true);
        Ok(())
    }
    pub(crate) fn save_annotated_to(&mut self, path: &Path) -> Result<(), String> {
        let r = self.records.replay.as_ref().ok_or("No Replay to export")?;
        let text = crate::pgn::export_annotated(&r.tree)?;
        crate::persistence::atomic_write(path, text.as_bytes())?;
        if let Some(r) = &mut self.records.replay {
            r.dirty = false;
        }
        Ok(())
    }
    pub(crate) fn export_annotated_dialog(&mut self) -> Result<bool, String> {
        if self.advanced.editor.is_some() {
            self.apply_annotation_edit()?;
        }
        let r = self.records.replay.as_ref().ok_or("No Replay to export")?;
        // Serialize before opening the dialog, so malformed metadata cannot lose edits.
        crate::pgn::export_annotated(&r.tree)?;
        let name = format!("rust-chess-{}-annotated.pgn", r.game.record.created_unix);
        let picked = crate::native_files::choose_pgn(true, self.records.owner, &name);
        self.advanced.last_tick = Instant::now();
        self.clock_last_tick = Instant::now();
        let Some(path) = picked? else {
            return Ok(false);
        };
        self.guard_library_export(&path)?;
        self.save_annotated_to(&path)?;
        Ok(true)
    }
    pub(crate) fn apply_annotation_edit(&mut self) -> Result<(), String> {
        let e = self
            .advanced
            .editor
            .as_ref()
            .ok_or("No annotation editor")?;
        let r = self.records.replay.as_mut().ok_or("No Replay")?;
        let node = &r.tree.nodes[e.node];
        if e.kind == EditKind::Nags {
            let nags = parse_nags(&e.text)?;
            let total = r.tree.nodes.iter().map(|n| n.nags.len()).sum::<usize>() - node.nags.len()
                + nags.len();
            if total > annotated_game::MAX_TOTAL_NAGS {
                return Err("PGN total NAG limit".into());
            }
            if node.nags != nags {
                r.tree.nodes[e.node].nags = nags;
                r.dirty = true;
            }
        } else {
            let old = if e.kind == EditKind::Before {
                &node.before
            } else {
                &node.after
            };
            if e.text == old.join("\n") {
                self.advanced.editor = None;
                return Ok(());
            }
            annotated_game::validate_comment(&e.text)?;
            let total = r
                .tree
                .nodes
                .iter()
                .map(|n| {
                    n.before
                        .iter()
                        .chain(&n.after)
                        .map(String::len)
                        .sum::<usize>()
                })
                .sum::<usize>()
                - old.iter().map(String::len).sum::<usize>()
                + e.text.len();
            if total > annotated_game::MAX_TOTAL_COMMENT_BYTES {
                return Err("PGN total comments exceed 1 MiB".into());
            }
            let replacement = if e.text.is_empty() {
                vec![]
            } else {
                vec![e.text.clone()]
            };
            let count = r
                .tree
                .nodes
                .iter()
                .map(|n| n.before.len() + n.after.len())
                .sum::<usize>()
                - old.len()
                + replacement.len();
            if count > annotated_game::MAX_COMMENTS {
                return Err("PGN comment count limit".into());
            }
            if *old != replacement {
                let node = &mut r.tree.nodes[e.node];
                if e.kind == EditKind::Before {
                    node.before = replacement;
                } else {
                    node.after = replacement;
                }
                r.dirty = true;
            }
        }
        self.advanced.editor = None;
        Ok(())
    }
    pub(crate) fn begin_annotation_edit(&mut self, kind: EditKind) -> Result<(), String> {
        let r = self.records.replay.as_mut().ok_or("No Replay")?;
        r.pause();
        let node = &r.tree.nodes[r.node];
        let kind = if r.node == 0 && kind != EditKind::Nags {
            EditKind::Before
        } else {
            kind
        };
        if kind == EditKind::Nags && r.node == 0 {
            return Err("NAG requires a move node".into());
        }
        let text = match kind {
            EditKind::Before => node.before.join("\n"),
            EditKind::After => node.after.join("\n"),
            EditKind::Nags => node
                .nags
                .iter()
                .map(|n| format!("${n}"))
                .collect::<Vec<_>>()
                .join(" "),
        };
        self.advanced.editor = Some(Editor {
            node: r.node,
            kind,
            cursor: text.len(),
            original: text.clone(),
            text,
            selected: false,
        });
        self.advanced.error = None;
        Ok(())
    }
    pub(crate) fn insert_annotation_text(&mut self, text: &str) -> Result<(), String> {
        let e = self.advanced.editor.as_mut().ok_or("No editor")?;
        let mut value = if e.selected {
            String::new()
        } else {
            e.text.clone()
        };
        let cursor = if e.selected { 0 } else { e.cursor };
        value.insert_str(cursor, text);
        if e.kind != EditKind::Nags {
            annotated_game::validate_comment(&value)?;
        } else if value.len() > 1024
            || value
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t'))
        {
            return Err("NAG input exceeds limit or contains controls".into());
        }
        e.text = value;
        e.cursor = cursor + text.len();
        e.selected = false;
        Ok(())
    }
    pub(crate) fn advanced_keys(&mut self, keys: &[Key], chars: &[char], ctrl: bool) -> bool {
        if self.advanced.pending.is_some() {
            if keys.contains(&Key::Escape) {
                self.handle_advanced_action(Action::CancelExit);
            }
            return true;
        }
        if self.advanced.editor.is_none() {
            return false;
        }
        if keys.contains(&Key::Escape) {
            self.handle_advanced_action(Action::CancelEdit);
            return true;
        }
        if keys.contains(&Key::Enter) {
            self.handle_advanced_action(Action::Apply);
            return true;
        }
        if ctrl && keys.contains(&Key::V) {
            self.handle_advanced_action(Action::Paste);
            return true;
        }
        let e = self.advanced.editor.as_mut().expect("editor");
        if ctrl && keys.contains(&Key::A) {
            e.selected = true;
            return true;
        }
        if keys.contains(&Key::Home) {
            e.cursor = 0;
            e.selected = false;
        }
        if keys.contains(&Key::End) {
            e.cursor = e.text.len();
            e.selected = false;
        }
        if keys.contains(&Key::Left) {
            e.cursor = e.text[..e.cursor]
                .char_indices()
                .last()
                .map(|p| p.0)
                .unwrap_or(0);
            e.selected = false;
        }
        if keys.contains(&Key::Right) {
            e.cursor += e.text[e.cursor..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
            e.selected = false;
        }
        if keys.contains(&Key::Backspace) || keys.contains(&Key::Delete) {
            if e.selected {
                e.text.clear();
                e.cursor = 0;
                e.selected = false;
            } else if keys.contains(&Key::Backspace) && e.cursor > 0 {
                let previous = e.text[..e.cursor]
                    .char_indices()
                    .last()
                    .expect("previous char")
                    .0;
                e.text.drain(previous..e.cursor);
                e.cursor = previous;
            } else if keys.contains(&Key::Delete) && e.cursor < e.text.len() {
                e.text.remove(e.cursor);
            }
        }
        if !ctrl {
            let text: String = chars.iter().filter(|c| !matches!(c, '\r' | '\n')).collect();
            if !text.is_empty() {
                self.advanced.error = self.insert_annotation_text(&text).err();
            }
        }
        true
    }
    pub(crate) fn handle_advanced_action(&mut self, action: Action) {
        let result = (|| {
            match action {
                Action::Apply => self.apply_annotation_edit()?,
                Action::CancelEdit => self.advanced.editor = None,
                Action::Comment => self.begin_annotation_edit(EditKind::After)?,
                Action::Nags => self.begin_annotation_edit(EditKind::Nags)?,
                Action::Paste => {
                    let text = crate::native_files::paste_text(self.records.owner)?;
                    self.insert_annotation_text(&text)?;
                }
                Action::Clear => {
                    let e = self.advanced.editor.as_ref().ok_or("No editor")?;
                    let r = self.records.replay.as_mut().ok_or("No Replay")?;
                    let n = &mut r.tree.nodes[e.node];
                    if e.kind == EditKind::Nags {
                        if !n.nags.is_empty() {
                            n.nags.clear();
                            r.dirty = true;
                        }
                    } else if !n.before.is_empty() || !n.after.is_empty() {
                        n.before.clear();
                        n.after.clear();
                        r.dirty = true;
                    }
                    self.advanced.editor = None;
                }
                Action::ToggleBefore => {
                    let r = self.records.replay.as_ref().ok_or("No Replay")?;
                    let n = &r.tree.nodes[r.node];
                    if r.node != 0
                        && n.parent
                            .is_some_and(|p| r.tree.nodes[p].children.first() == Some(&r.node))
                    {
                        return Err(
                            "Before-comments can only be edited at game/variation starts".into(),
                        );
                    }
                    let kind = if self
                        .advanced
                        .editor
                        .as_ref()
                        .is_some_and(|e| e.kind == EditKind::Before)
                    {
                        EditKind::After
                    } else {
                        EditKind::Before
                    };
                    self.apply_annotation_edit()?;
                    self.begin_annotation_edit(kind)?;
                }
                Action::CancelExit => {
                    self.advanced.pending = None;
                }
                Action::Discard => {
                    self.advanced.editor = None;
                    if let Some(r) = &mut self.records.replay {
                        r.dirty = false;
                    }
                    if let Some(intent) = self.advanced.pending.take() {
                        self.finish_intent(intent)?;
                    }
                }
                Action::ExportExit => {
                    let exported = self.export_annotated_dialog()?;
                    self.finish_export_exit(exported)?;
                }
                Action::NoteUp => {
                    self.advanced.note_scroll = self.advanced.note_scroll.saturating_sub(1)
                }
                Action::NoteDown => {
                    self.advanced.note_scroll = (self.advanced.note_scroll + 1).min(
                        (annotated_game::MAX_TOTAL_COMMENT_BYTES + annotated_game::MAX_COMMENTS)
                            / 16,
                    )
                }
                a => {
                    let r = self.records.replay.as_mut().ok_or("No Replay")?;
                    match a {
                        Action::Play => r.toggle_play(),
                        Action::Speed => r.cycle_speed(),
                        Action::Choices => {
                            r.pause();
                            self.advanced.choices = !self.advanced.choices;
                            self.advanced.choice_scroll = 0;
                        }
                        Action::Choose(i) => {
                            r.enter_choice(i)?;
                            self.advanced.choices = false;
                        }
                        Action::Node(id) => {
                            if let Some(index) = r.selected_path.iter().position(|n| *n == id) {
                                r.jump(index);
                            } else {
                                r.select_node(id)?;
                            }
                        }
                        Action::Parent => r.parent_variation(),
                        Action::Mainline => r.return_mainline(),
                        _ => unreachable!(),
                    }
                    self.advanced.note_scroll = 0;
                    if let Some(r) = &mut self.records.replay {
                        r.scroll = r.path_cursor().saturating_sub(2);
                    }
                    self.advanced.last_tick = Instant::now();
                    self.cancel_analysis();
                }
            }
            Ok::<_, String>(())
        })();
        self.advanced.error = result.err();
    }
    pub(crate) fn tick_replay_at(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.advanced.last_tick);
        self.advanced.last_tick = now;
        if self.screen != Screen::Replay
            || self.advanced_modal()
            || self.records.message.is_some()
            || self.records.fen_input.is_some()
        {
            return;
        }
        if self
            .records
            .replay
            .as_mut()
            .is_some_and(|r| r.tick(elapsed))
        {
            self.advanced.note_scroll = 0;
            if let Some(r) = &mut self.records.replay {
                r.scroll = r.path_cursor().saturating_sub(2);
            }
            self.cancel_analysis();
        }
    }
    pub(crate) fn finish_export_exit(&mut self, exported: bool) -> Result<(), String> {
        if !exported {
            return Ok(());
        }
        if self.needs_replay_save() {
            return Err("Annotations are still unsaved".into());
        }
        if let Some(intent) = self.advanced.pending.take() {
            self.finish_intent(intent)?;
        }
        Ok(())
    }
}
pub fn parse_nags(text: &str) -> Result<Vec<u8>, String> {
    let values = text
        .split_whitespace()
        .map(|s| match s {
            "!" => Ok(1),
            "?" => Ok(2),
            "!!" => Ok(3),
            "??" => Ok(4),
            "!?" => Ok(5),
            "?!" => Ok(6),
            _ => {
                let n = s.strip_prefix('$').unwrap_or(s);
                if n.is_empty() || !n.bytes().all(|v| v.is_ascii_digit()) {
                    Err("NAG must be 0..255".to_owned())
                } else {
                    n.parse::<u8>().map_err(|_| "NAG must be 0..255".to_owned())
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() > annotated_game::MAX_NAGS_PER_NODE {
        return Err("At most 64 NAGs per node".into());
    }
    Ok(values)
}
pub fn nag_label(n: u8) -> String {
    match n {
        1 => "!".into(),
        2 => "?".into(),
        3 => "!!".into(),
        4 => "??".into(),
        5 => "!?".into(),
        6 => "?!".into(),
        _ => format!("${n}"),
    }
}

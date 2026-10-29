//! Independent study line, guarded app integration and optional explicit recovery.
use crate::{
    App, Chess, Color, Key, Move, Position, Role, Screen, Square,
    advanced_app::Intent,
    fen,
    game_record::GameRecord,
    position_editor::{self, Draft},
};
use serde::{Deserialize, Serialize};
use shakmaty::{EnPassantMode, Setup, san::SanPlus};
use std::{io::Read, path::Path};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Edit,
    Analyze,
}
#[derive(Clone)]
pub struct Study {
    pub positions: Vec<Chess>,
    pub moves: Vec<Move>,
    pub sans: Vec<String>,
    pub cursor: usize,
    keys: Vec<String>,
    results: Vec<Option<crate::game::GameResult>>,
}
impl Study {
    pub fn new(pos: Chess) -> Self {
        let key = crate::game::position_key(&pos);
        let result = crate::game::GameResult::from_position(&pos, 1);
        Self {
            positions: vec![pos],
            moves: vec![],
            sans: vec![],
            cursor: 0,
            keys: vec![key],
            results: vec![result],
        }
    }
    pub fn position(&self) -> &Chess {
        &self.positions[self.cursor]
    }
    pub fn play(&mut self, mv: Move) -> Result<(), String> {
        let pos = self.position();
        if !pos.is_legal(&mv) {
            return Err("Illegal move".into());
        }
        if self.cursor >= crate::game_record::MAX_PLIES {
            return Err("Study line limit: 4096 plies".into());
        }
        if (pos.turn() == Color::Black && pos.fullmoves().get() == u32::MAX)
            || (!mv.is_zeroing() && pos.halfmoves() == u32::MAX)
        {
            return Err("Move counter overflow: edit the initial counters".into());
        }
        if self.result().is_some() {
            return Err("Study position has ended".into());
        }
        let san = SanPlus::from_move(pos.clone(), &mv).to_string();
        let next = pos.clone().play(&mv).map_err(|e| e.to_string())?;
        self.moves.truncate(self.cursor);
        self.sans.truncate(self.cursor);
        self.positions.truncate(self.cursor + 1);
        self.keys.truncate(self.cursor + 1);
        self.results.truncate(self.cursor + 1);
        let key = crate::game::position_key(&next);
        let repetitions = 1 + self.keys.iter().filter(|k| **k == key).count();
        self.results
            .push(crate::game::GameResult::from_position(&next, repetitions));
        self.keys.push(key);
        self.moves.push(mv);
        self.sans.push(san);
        self.positions.push(next);
        self.cursor += 1;
        Ok(())
    }
    pub fn result(&self) -> Option<crate::game::GameResult> {
        self.results[self.cursor]
    }
    pub fn record(&self) -> GameRecord {
        // Export the whole retained line, even when viewing an earlier cursor.
        let mut meta = GameRecord::new(&self.positions[0], None);
        meta.tags
            .insert("Event".into(), "RUST CHESS Position Study".into());
        meta.tags.insert("White".into(), "Study".into());
        meta.tags.insert("Black".into(), "Study".into());
        GameRecord::capture(
            &self.positions[0],
            &self.moves,
            meta,
            self.results[self.moves.len()],
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Fen,
    Ep,
    Halfmove,
    Fullmove,
}
pub struct Input {
    pub field: Field,
    pub text: String,
    pub cursor: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enter,
    FromCurrent,
    Back,
    Empty,
    Starting,
    Flip,
    Tool(usize),
    ClearSquare,
    UndoEdit,
    RedoEdit,
    Turn,
    Castle(usize),
    Input(Field),
    Apply,
    Paste,
    CancelInput,
    Validate,
    Analyze,
    Edit,
    First,
    Previous,
    Next,
    Last,
    CopyFen,
    Export,
    Save,
    Parameters,
    Engine,
    Discard,
    Keep,
}
pub struct Workspace {
    pub mode: Mode,
    pub draft: Draft,
    pub study: Option<Study>,
    pub flipped: bool,
    pub tool: usize,
    pub selected: Option<Square>,
    pub dragging: Option<Square>,
    pub promotion: Option<(Square, Square)>,
    pub dirty: bool,
    pub error: Option<String>,
    pub notice: Option<usize>,
    pub input: Option<Input>,
    pub parameters: bool,
    pub pending: Option<Intent>,
    pub return_screen: Screen,
    pub initialized: bool,
    pub revision: u64,
}
impl Default for Workspace {
    fn default() -> Self {
        Self {
            mode: Mode::Edit,
            draft: Draft::default(),
            study: None,
            flipped: false,
            tool: 13,
            selected: None,
            dragging: None,
            promotion: None,
            dirty: false,
            error: None,
            notice: None,
            input: None,
            parameters: false,
            pending: None,
            return_screen: Screen::Menu,
            initialized: false,
            revision: 0,
        }
    }
}
impl Workspace {
    pub fn modal(&self) -> bool {
        self.pending.is_some() || self.input.is_some()
    }
    pub fn unsaved(&self) -> bool {
        self.dirty || self.input.is_some()
    }
    pub fn chess(&self) -> Option<&Chess> {
        (self.mode == Mode::Analyze)
            .then_some(self.study.as_ref())
            .flatten()
            .map(Study::position)
    }
    pub fn text(&self) -> String {
        self.chess()
            .map(fen::write)
            .unwrap_or_else(|| self.draft.text())
    }
    pub fn clear_input(&mut self) {
        self.selected = None;
        self.dragging = None;
        self.promotion = None;
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    schema_version: u32,
    draft: String,
    root: Option<String>,
    moves: Vec<String>,
    cursor: usize,
    flipped: bool,
}
fn decode(bytes: &[u8]) -> Result<(Draft, Option<Study>, bool), String> {
    if bytes.len() > 1024 * 1024 {
        return Err("Workspace exceeds 1 MiB".into());
    }
    let data: Saved = serde_json::from_slice(bytes).map_err(|e| format!("Workspace: {e}"))?;
    if data.schema_version != 1 {
        return Err("Unsupported workspace schema".into());
    }
    let mut draft = Draft::default();
    draft.setup = position_editor::parse_draft(&data.draft)?;
    let study = if let Some(root) = data.root {
        let root = position_editor::validate(&position_editor::parse_draft(&root)?)?;
        if data.moves.len() > crate::game_record::MAX_PLIES || data.cursor > data.moves.len() {
            return Err("Workspace line limit/cursor".into());
        }
        let mut study = Study::new(root);
        for uci in data.moves {
            let mv = uci
                .parse::<shakmaty::uci::UciMove>()
                .map_err(|e| e.to_string())?
                .to_move(study.position())
                .map_err(|e| e.to_string())?;
            study.play(mv)?;
        }
        study.cursor = data.cursor;
        Some(study)
    } else {
        if !data.moves.is_empty() || data.cursor != 0 {
            return Err("Workspace line without root".into());
        }
        None
    };
    Ok((draft, study, data.flipped))
}
fn read(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
        Ok(m) if !m.is_file() || m.file_type().is_symlink() || m.len() > 1024 * 1024 => {
            return Err("Unsafe/oversized workspace file".into());
        }
        _ => {}
    }
    let mut bytes = vec![];
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Err("Workspace exceeds 1 MiB".into());
    }
    Ok(Some(bytes))
}
impl App {
    pub(crate) fn workspace_path(&self) -> Result<std::path::PathBuf, String> {
        self.records
            .save_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.join("position-workspace-v1.json"))
            .or_else(|| {
                std::env::var_os("RUST_CHESS_DATA_DIR")
                    .map(|p| std::path::PathBuf::from(p).join("position-workspace-v1.json"))
            })
            .or_else(|| {
                crate::persistence::data_path()
                    .ok()
                    .map(|p| p.with_file_name("position-workspace-v1.json"))
            })
            .ok_or_else(|| "Workspace data directory unavailable".into())
    }
    pub(crate) fn save_workspace(&mut self) -> Result<(), String> {
        let path = self.workspace_path()?;
        if let Some(old) = read(&path)? {
            decode(&old)?;
        }
        let s = &self.workspace;
        if s.input.is_some() {
            return Err("Apply or cancel the text input before saving".into());
        }
        let data = Saved {
            schema_version: 1,
            draft: s.draft.text(),
            root: s.study.as_ref().map(|s| fen::write(&s.positions[0])),
            moves: s
                .study
                .as_ref()
                .map(|s| {
                    s.moves
                        .iter()
                        .map(|m| shakmaty::uci::UciMove::from_standard(m).to_string())
                        .collect()
                })
                .unwrap_or_default(),
            cursor: s.study.as_ref().map(|s| s.cursor).unwrap_or(0),
            flipped: s.flipped,
        };
        let bytes = serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?;
        decode(&bytes)?;
        crate::persistence::atomic_write(&path, &bytes)?;
        self.workspace.dirty = false;
        self.workspace.notice = Some(32);
        Ok(())
    }
    pub(crate) fn defer_workspace_loss(&mut self, intent: Intent) -> bool {
        if self.screen != Screen::PositionWorkspace || !self.workspace.unsaved() {
            return false;
        }
        if self.workspace.pending.is_none() {
            self.workspace.pending = Some(intent);
        }
        true
    }
    pub(crate) fn invalidate_workspace(&mut self) {
        self.cancel_analysis();
        self.workspace.revision = self.workspace.revision.wrapping_add(1);
        self.workspace.clear_input();
        self.workspace.error = None;
        self.workspace.notice = None;
    }
    pub(crate) fn enter_workspace(&mut self, source: Option<Chess>) -> Result<(), String> {
        // Validate first, then ask about any currently edited Replay/Workspace.
        if let Some(pos) = &source {
            position_editor::validate(&pos.clone().into_setup(EnPassantMode::Always))?;
        }
        let a = if source.is_some() {
            Action::FromCurrent
        } else {
            Action::Enter
        };
        if self.defer_replay_loss(Intent::Workspace(a)) {
            return Ok(());
        }
        let previous = self.screen;
        self.cancel_review();
        self.review.panel = false;
        self.cancel_analysis();
        self.analysis.panel = false;
        self.analysis.context = self.analysis.context.wrapping_add(1);
        if let Some(r) = &mut self.records.replay {
            r.pause();
        }
        if previous != Screen::PositionWorkspace {
            self.workspace.return_screen = previous;
        }
        self.screen = Screen::PositionWorkspace;
        self.clock_last_tick = crate::Instant::now();
        if let Some(pos) = source {
            self.workspace
                .draft
                .change(pos.clone().into_setup(EnPassantMode::Always));
            self.workspace.study = Some(Study::new(pos));
            self.workspace.mode = Mode::Analyze;
            self.workspace.dirty = true;
        } else if !self.workspace.initialized {
            if let Some(bytes) = read(&self.workspace_path()?)? {
                let (draft, study, flipped) = decode(&bytes)?;
                self.workspace.draft = draft;
                self.workspace.study = study;
                self.workspace.flipped = flipped;
            }
            self.workspace.mode = Mode::Edit;
        }
        self.workspace.initialized = true;
        self.invalidate_workspace();
        Ok(())
    }
    fn finish_workspace_pending(&mut self) -> Result<(), String> {
        if let Some(intent) = self.workspace.pending.take() {
            self.finish_intent(intent)?;
        }
        Ok(())
    }
    pub(crate) fn workspace_source(&self) -> Result<Chess, String> {
        match self.screen {
            Screen::Game => Ok(self.pos.clone()),
            Screen::Replay => self
                .records
                .replay
                .as_ref()
                .map(|r| r.position().clone())
                .ok_or_else(|| "No Replay position".into()),
            Screen::Opening => Ok(self.opening.position().clone()),
            _ => Err("No source position".into()),
        }
    }
    pub(crate) fn handle_workspace_action(&mut self, a: Action) {
        use Action::*;
        if self.workspace.pending.is_some() && !matches!(a, Save | Discard | Keep) {
            return;
        }
        if self.workspace.pending.is_none()
            && self.workspace.input.is_some()
            && !matches!(a, Apply | Paste | CancelInput | Back)
        {
            return;
        }
        if self.workspace.mode != Mode::Edit
            && matches!(
                a,
                Empty | Starting | UndoEdit | RedoEdit | ClearSquare | Turn | Castle(_)
            )
        {
            return;
        }
        if matches!(a, Back | Empty | Starting | Enter | FromCurrent)
            && self.defer_workspace_loss(Intent::Workspace(a))
        {
            return;
        }
        let result = (|| {
            match a {
                Enter => self.enter_workspace(None)?,
                FromCurrent => {
                    let p = self.workspace_source()?;
                    self.enter_workspace(Some(p))?;
                }
                Back => {
                    self.invalidate_workspace();
                    self.analysis.panel = false;
                    self.screen = self.workspace.return_screen;
                    self.clock_last_tick = crate::Instant::now();
                }
                Empty | Starting => {
                    self.invalidate_workspace();
                    self.workspace.mode = Mode::Edit;
                    self.workspace.draft.change(if a == Empty {
                        Setup::empty()
                    } else {
                        Setup::default()
                    });
                    self.workspace.dirty = true;
                }
                Flip => {
                    self.workspace.flipped = !self.workspace.flipped;
                    self.workspace.dirty = true;
                }
                Tool(i) => self.workspace.tool = i.min(13),
                Parameters => self.workspace.parameters = !self.workspace.parameters,
                UndoEdit | RedoEdit => {
                    self.invalidate_workspace();
                    if a == UndoEdit {
                        self.workspace.draft.undo();
                    } else {
                        self.workspace.draft.redo();
                    }
                    self.workspace.dirty = true;
                }
                ClearSquare => {
                    if let Some(sq) = self.workspace.selected {
                        self.invalidate_workspace();
                        self.workspace.draft.place(sq, None);
                        self.workspace.dirty = true;
                    }
                }
                Turn | Castle(_) => {
                    self.invalidate_workspace();
                    let mut s = self.workspace.draft.setup.clone();
                    if a == Turn {
                        s.turn = !s.turn;
                    } else if let Castle(i) = a
                        && let Some(sq) = position_editor::RIGHTS.get(i)
                    {
                        s.castling_rights.toggle(*sq);
                    }
                    self.workspace.draft.change(s);
                    self.workspace.dirty = true;
                }
                Input(field) => {
                    self.cancel_analysis();
                    let s = &self.workspace;
                    let text = match field {
                        Field::Fen => s.text(),
                        Field::Ep => s
                            .draft
                            .setup
                            .ep_square
                            .map(|x| x.to_string())
                            .unwrap_or_else(|| "-".into()),
                        Field::Halfmove => s.draft.setup.halfmoves.to_string(),
                        Field::Fullmove => s.draft.setup.fullmoves.to_string(),
                    };
                    self.workspace.input = Some(crate::position_workspace::Input {
                        cursor: text.len(),
                        field,
                        text,
                    });
                }
                Apply => {
                    let i = self.workspace.input.as_ref().ok_or("No workspace input")?;
                    let text = i.text.trim();
                    let mut setup = self.workspace.draft.setup.clone();
                    match i.field {
                        Field::Fen => setup = position_editor::parse_draft(text)?,
                        Field::Ep => {
                            setup.ep_square = if text == "-" {
                                None
                            } else {
                                Some(text.parse::<Square>().map_err(|e| e.to_string())?)
                            }
                        }
                        Field::Halfmove | Field::Fullmove => {
                            if text.is_empty() || !text.bytes().all(|c| c.is_ascii_digit()) {
                                return Err("Unsigned decimal counter required".into());
                            }
                            let n = text.parse::<u32>().map_err(|e| e.to_string())?;
                            if i.field == Field::Halfmove {
                                setup.halfmoves = n;
                            } else {
                                setup.fullmoves = std::num::NonZeroU32::new(n)
                                    .ok_or("Fullmove number must be at least 1")?;
                            }
                        }
                    }
                    self.invalidate_workspace();
                    self.workspace.draft.change(setup);
                    self.workspace.mode = Mode::Edit;
                    self.workspace.input = None;
                    self.workspace.dirty = true;
                }
                Paste => {
                    let t = crate::native_files::paste_text(self.records.owner)?;
                    if t.len() > 256 || t.chars().any(|c| c.is_control() && !c.is_whitespace()) {
                        return Err("Workspace input limit/characters".into());
                    }
                    if let Some(i) = &mut self.workspace.input {
                        i.text = t;
                        i.cursor = i.text.len();
                    }
                }
                CancelInput => {
                    self.workspace.input = None;
                    self.workspace.error = None;
                }
                Validate | Analyze => {
                    let pos = self.workspace.draft.validate()?;
                    if a == Analyze {
                        // An edited root replaces the old line only after full validation.
                        let different = self
                            .workspace
                            .study
                            .as_ref()
                            .is_none_or(|s| fen::write(&s.positions[0]) != fen::write(&pos));
                        self.invalidate_workspace();
                        if different {
                            self.analysis.context = self.analysis.context.wrapping_add(1);
                            self.workspace.study = Some(Study::new(pos));
                            self.workspace.dirty = true;
                        }
                        self.workspace.mode = Mode::Analyze;
                    }
                    self.workspace.error = None;
                    self.workspace.notice = Some(30);
                }
                Edit => {
                    self.invalidate_workspace();
                    self.workspace.mode = Mode::Edit;
                    self.analysis.panel = false;
                }
                First | Previous | Next | Last => {
                    self.invalidate_workspace();
                    if let Some(s) = &mut self.workspace.study {
                        s.cursor = match a {
                            First => 0,
                            Previous => s.cursor.saturating_sub(1),
                            Next => (s.cursor + 1).min(s.moves.len()),
                            _ => s.moves.len(),
                        };
                    }
                }
                Engine => {
                    if self.workspace.chess().is_none() {
                        return Err("Validate the draft before UCI analysis".into());
                    }
                    self.analysis.panel = !self.analysis.panel;
                    if !self.analysis.panel {
                        self.cancel_analysis();
                    }
                }
                CopyFen => {
                    let pos = if let Some(p) = self.workspace.chess() {
                        p.clone()
                    } else {
                        self.workspace.draft.validate()?
                    };
                    crate::native_files::copy_text(&fen::write(&pos), self.records.owner)?;
                    self.workspace.notice = Some(32);
                }
                Export => {
                    let s = self
                        .workspace
                        .study
                        .as_ref()
                        .ok_or("Validate and enter Analyze before exporting PGN")?;
                    let pgn = crate::pgn::export(&s.record())?;
                    if let Some(path) = crate::native_files::choose_pgn(
                        true,
                        self.records.owner,
                        "position-study.pgn",
                    )? {
                        self.guard_library_export(&path)?;
                        crate::persistence::atomic_write(&path, pgn.as_bytes())?;
                        self.workspace.notice = Some(32);
                    }
                }
                Save => {
                    self.save_workspace()?;
                    self.workspace.input = None;
                    self.finish_workspace_pending()?;
                }
                Discard => {
                    self.workspace.dirty = false;
                    self.workspace.input = None;
                    self.finish_workspace_pending()?;
                }
                Keep => self.workspace.pending = None,
            }
            Ok::<_, String>(())
        })();
        if let Err(e) = result {
            self.workspace.error = Some(e);
        } else {
            self.workspace.error = None;
        }
    }
    pub(crate) fn workspace_play(&mut self, mv: Move) {
        if self.workspace.mode != Mode::Analyze || self.workspace.modal() {
            return;
        }
        let result = self
            .workspace
            .study
            .as_mut()
            .ok_or_else(|| "No study".to_owned())
            .and_then(|s| s.play(mv));
        match result {
            Ok(()) => {
                self.invalidate_workspace();
                self.workspace.dirty = true;
            }
            Err(e) => self.workspace.error = Some(e),
        }
    }
    fn workspace_to(&mut self, from: Square, to: Square) {
        let Some(p) = self.workspace.chess() else {
            return;
        };
        let moves: Vec<_> = p
            .legal_moves()
            .iter()
            .filter(|m| m.from() == Some(from) && crate::puzzle_app::target(m) == to)
            .cloned()
            .collect();
        if moves.iter().any(|m| m.promotion().is_some()) {
            self.workspace.promotion = Some((from, to));
        } else if let Some(m) = moves.first() {
            self.workspace_play(m.clone());
        } else {
            self.workspace.error = Some("Illegal move".into());
        }
    }
    pub(crate) fn workspace_down(&mut self, sq: Square) {
        if self.workspace.modal() || self.workspace.promotion.is_some() {
            return;
        }
        if self.workspace.mode == Mode::Edit {
            self.workspace.selected = Some(sq);
            self.workspace.dragging = Some(sq);
        } else if let Some(p) = self.workspace.chess() {
            if p.board()
                .piece_at(sq)
                .is_some_and(|piece| piece.color == p.turn())
            {
                self.workspace.selected = if self.workspace.selected == Some(sq) {
                    None
                } else {
                    Some(sq)
                };
                // A second click cancels selection on release, but may still
                // become a drag of that same piece when released elsewhere.
                self.workspace.dragging = Some(sq);
                self.workspace.error = None;
            } else if let Some(from) = self.workspace.selected {
                self.workspace_to(from, sq);
            }
        }
    }
    pub(crate) fn workspace_up(&mut self, to: Option<Square>) {
        if self.workspace.modal() {
            self.workspace.dragging = None;
            return;
        }
        let Some(from) = self.workspace.dragging.take() else {
            return;
        };
        let Some(to) = to else {
            return;
        };
        if self.workspace.mode == Mode::Edit {
            let old = self.workspace.draft.setup.clone();
            if from != to {
                self.workspace.draft.relocate(from, to);
            } else if self.workspace.tool < 13 {
                let tool = self.workspace.tool;
                let piece = if tool == 12 {
                    None
                } else {
                    Some(shakmaty::Piece {
                        color: if tool < 6 { Color::White } else { Color::Black },
                        role: [
                            Role::King,
                            Role::Queen,
                            Role::Rook,
                            Role::Bishop,
                            Role::Knight,
                            Role::Pawn,
                        ][tool % 6],
                    })
                };
                self.workspace.draft.place(to, piece);
            }
            if old != self.workspace.draft.setup {
                self.invalidate_workspace();
                self.workspace.selected = Some(to);
                self.workspace.dirty = true;
            }
        } else if from != to {
            self.workspace_to(from, to);
        }
    }
    pub(crate) fn workspace_promote(&mut self, role: Role) {
        if let Some((from, to)) = self.workspace.promotion
            && let Some(m) = self.workspace.chess().and_then(|p| {
                p.legal_moves()
                    .iter()
                    .find(|m| {
                        m.from() == Some(from)
                            && crate::puzzle_app::target(m) == to
                            && m.promotion() == Some(role)
                    })
                    .cloned()
            })
        {
            self.workspace_play(m);
        }
    }
    pub(crate) fn workspace_view(&self, v: &mut crate::render::ViewState) {
        if self.screen != Screen::PositionWorkspace {
            return;
        }
        let s = &self.workspace;
        v.pos = s.chess().cloned().unwrap_or_default();
        v.board_override = (s.mode == Mode::Edit).then(|| s.draft.setup.board.clone());
        v.selected = s.selected;
        v.dragging_from = s.dragging;
        v.promotion = s.promotion;
        v.animations.clear();
        v.history_sans.clear();
        v.analysis = if s.mode == Mode::Analyze {
            self.analysis_visual()
        } else {
            Default::default()
        };
        v.legal_targets = s
            .chess()
            .and_then(|p| {
                s.selected.or(s.dragging).map(|sq| {
                    p.legal_moves()
                        .iter()
                        .filter(|m| m.from() == Some(sq))
                        .map(crate::puzzle_app::target)
                        .collect()
                })
            })
            .unwrap_or_default();
        v.last_move = s
            .study
            .as_ref()
            .filter(|_| s.mode == Mode::Analyze)
            .and_then(|s| s.moves.get(s.cursor.wrapping_sub(1)))
            .and_then(crate::uci_protocol::move_arrow);
        v.settings.mode = if s.flipped { Some(Color::White) } else { None };
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
    pub(crate) fn workspace_keys(&mut self, keys: &[Key], chars: &[char], ctrl: bool) -> bool {
        if self.screen != Screen::PositionWorkspace {
            return false;
        }
        if self.workspace.pending.is_some() {
            if keys.contains(&Key::Escape) {
                self.handle_workspace_action(Action::Keep);
            }
            return true;
        }
        if self.workspace.input.is_some() {
            if keys.contains(&Key::Escape) {
                self.handle_workspace_action(Action::CancelInput);
                return true;
            }
            if keys.contains(&Key::Enter) {
                self.handle_workspace_action(Action::Apply);
                return true;
            }
            if ctrl && keys.contains(&Key::V) {
                self.handle_workspace_action(Action::Paste);
                return true;
            }
            let i = self.workspace.input.as_mut().unwrap();
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
                let n = i.text[..i.cursor].char_indices().next_back().unwrap().0;
                i.text.replace_range(n..i.cursor, "");
                i.cursor = n;
            }
            if keys.contains(&Key::Delete) && i.cursor < i.text.len() {
                let n = i.cursor + i.text[i.cursor..].chars().next().unwrap().len_utf8();
                i.text.replace_range(i.cursor..n, "");
            }
            if !ctrl {
                let t: String = chars.iter().filter(|c| !c.is_control()).collect();
                if i.text.len() + t.len() <= 256 {
                    i.text.insert_str(i.cursor, &t);
                    i.cursor += t.len();
                }
            }
            return true;
        }
        for (key, a) in [
            (Key::Escape, Action::Back),
            (Key::Left, Action::Previous),
            (Key::Right, Action::Next),
            (Key::Home, Action::First),
            (Key::End, Action::Last),
        ] {
            if keys.contains(&key) {
                self.handle_workspace_action(a);
            }
        }
        true
    }
}

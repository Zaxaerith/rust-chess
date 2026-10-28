//! Prefix-safe offline repertoire practice and deterministic UTC-day Leitner cards.
//! A position key shares memory progress, never the live route continuation graph.
use crate::{
    game::position_key,
    repertoire::{Repertoire, Side, ValidatedRoute, validate_route},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shakmaty::{Chess, Move, Position, Square, san::SanPlus};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::Read,
    path::Path,
    sync::Arc,
};

pub const INTERVALS: [u64; 5] = [1, 3, 7, 14, 30];
const MAX_PROGRESS_BYTES: usize = 16 * 1024 * 1024;
const MAX_CARDS: usize = 100_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Due,
    New,
    All,
    Errors,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Waiting,
    Responding,
    Completed,
    Empty,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feedback {
    Ready,
    Correct,
    Outside,
    Illegal,
    Hint,
    Revealed,
    RouteEnd,
    NoCards,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultKind {
    Clean,
    Assisted,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    pub level: u8,
    pub last_day: u64,
    pub due_day: u64,
    pub correct: u64,
    pub errors: u64,
    pub hints: u64,
    pub last_result: Option<ResultKind>,
    pub last_completed_day: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub schema_version: u32,
    pub entries: BTreeMap<String, Card>,
    #[serde(default)]
    pub recent: Vec<Recall>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recall {
    pub key: String,
    pub clean: bool,
    pub day: u64,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: BTreeMap::new(),
            recent: vec![],
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub learned: usize,
    pub new: usize,
    pub due: usize,
    pub completed: usize,
    pub correct: u64,
    pub errors: u64,
    pub hints: u64,
    pub recent_correct: u64,
    pub recent_total: u64,
}
#[cfg(test)]
#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub key: String,
    pub position: String,
    pub allowed: Vec<String>,
    pub prefix: Vec<String>,
}
#[derive(Clone)]
struct Context {
    key: String,
    position: String,
    allowed: Vec<String>,
    route: usize,
    ply: usize,
}
struct Graph {
    routes: Vec<ValidatedRoute>,
    contexts: BTreeMap<String, Context>,
    nodes: BTreeMap<String, Context>,
}
fn node_key(book: &str, position: &str, allowed: &[String]) -> String {
    // Delimit components unambiguously and retain the full SHA-256 identity.
    let bytes = serde_json::to_vec(&(1u32, book, position, allowed)).expect("string serialization");
    format!("{:x}", Sha256::digest(bytes))
}
impl Graph {
    fn new(book: &Repertoire) -> Result<Self, String> {
        if book.id.is_empty() || book.routes.len() > 1000 {
            return Err("Repertoire graph limit/identity".into());
        }
        let routes: Vec<_> = book
            .routes
            .iter()
            .map(validate_route)
            .collect::<Result<_, _>>()?;
        let mut contexts = BTreeMap::<String, Context>::new();
        for (route, line) in routes.iter().enumerate() {
            let mut prefix = String::new();
            for (ply, uci) in line.route.moves.iter().enumerate() {
                if line.positions[ply].turn() == book.side.color() {
                    let c = contexts.entry(prefix.clone()).or_insert_with(|| Context {
                        key: String::new(),
                        position: position_key(&line.positions[ply]),
                        allowed: vec![],
                        route,
                        ply,
                    });
                    if !c.allowed.contains(uci) {
                        c.allowed.push(uci.clone());
                    }
                }
                if !prefix.is_empty() {
                    prefix.push(' ');
                }
                prefix.push_str(uci);
            }
        }
        let mut nodes = BTreeMap::new();
        for c in contexts.values_mut() {
            c.allowed.sort();
            c.key = node_key(&book.id, &c.position, &c.allowed);
            nodes.entry(c.key.clone()).or_insert_with(|| c.clone());
        }
        Ok(Self {
            routes,
            contexts,
            nodes,
        })
    }
    fn prefix(&self, c: &Context) -> Vec<String> {
        self.routes[c.route].route.moves[..c.ply].to_vec()
    }
}
#[cfg(test)]
pub fn cards(book: &Repertoire) -> Result<BTreeMap<String, NodeInfo>, String> {
    let graph = Graph::new(book)?;
    Ok(graph
        .nodes
        .iter()
        .map(|(key, c)| {
            (
                key.clone(),
                NodeInfo {
                    key: key.clone(),
                    position: c.position.clone(),
                    allowed: c.allowed.clone(),
                    prefix: graph.prefix(c),
                },
            )
        })
        .collect())
}
impl Progress {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.entries.len() > MAX_CARDS
            || self.recent.len() > 256
            || self.recent.iter().any(|r| {
                r.key.len() != 64
                    || !r
                        .key
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            })
            || self.entries.iter().any(|(key, c)| {
                key.len() != 64
                    || !key
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                    || !(1..=5).contains(&c.level)
                    || c.due_day < c.last_day
                    || c.last_result.is_none()
                    || c.last_completed_day.is_some_and(|day| day > c.last_day)
            })
        {
            return Err("Unsupported/invalid repertoire progress (preserved)".into());
        }
        Ok(())
    }
    pub fn stats(&self, book: &Repertoire, today: u64) -> Result<Stats, String> {
        let graph = Graph::new(book)?;
        Ok(stats_for_nodes(&graph.nodes, self, today))
    }
}
fn stats_for_nodes(nodes: &BTreeMap<String, Context>, progress: &Progress, today: u64) -> Stats {
    let mut stats = Stats::default();
    for key in nodes.keys() {
        if let Some(card) = progress.entries.get(key) {
            stats.learned += 1;
            stats.due += usize::from(card.due_day <= today);
            stats.completed += usize::from(card.last_completed_day == Some(today));
            stats.correct = stats.correct.saturating_add(card.correct);
            stats.errors = stats.errors.saturating_add(card.errors);
            stats.hints = stats.hints.saturating_add(card.hints);
        } else {
            stats.new += 1;
        }
    }
    for recall in progress
        .recent
        .iter()
        .rev()
        .filter(|r| nodes.contains_key(&r.key))
        .take(100)
    {
        stats.recent_total += 1;
        stats.recent_correct += u64::from(recall.clean);
    }
    stats
}
pub fn load_progress(path: &Path) -> Result<Progress, String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Progress::default()),
        Err(e) => return Err(format!("Read repertoire progress {}: {e}", path.display())),
    };
    let mut bytes = vec![];
    file.take(MAX_PROGRESS_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_PROGRESS_BYTES {
        return Err("Repertoire progress exceeds 16 MiB (preserved)".into());
    }
    let p: Progress = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Repertoire progress damaged (preserved): {e}"))?;
    p.validate()?;
    Ok(p)
}
pub fn save_progress(path: &Path, progress: &Progress) -> Result<(), String> {
    let _ = load_progress(path)?;
    progress.validate()?;
    let bytes = serde_json::to_vec(progress).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_PROGRESS_BYTES {
        return Err("Repertoire progress exceeds 16 MiB".into());
    }
    crate::persistence::atomic_write(path, &bytes)
}
pub fn today_utc() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400
}

#[derive(Clone)]
struct Queued {
    context: Context,
    remedial: bool,
}
#[derive(Clone)]
struct Event {
    key: String,
    correct: u64,
    errors: u64,
    hints: u64,
    clean: bool,
}
#[derive(Clone)]
pub struct Session {
    pub pos: Chess,
    pub side: Side,
    pub state: State,
    pub hint_square: Option<Square>,
    pub feedback: Feedback,
    pub completed: usize,
    pub remaining: usize,
    pub skipped: usize,
    pub hints: u32,
    pub errors: u32,
    pub path: Vec<String>,
    path_text: String,
    allowed_cache: Vec<(String, String)>,
    graph: Arc<Graph>,
    targets: BTreeSet<String>,
    done: BTreeSet<String>,
    skipped_keys: BTreeSet<String>,
    graded: BTreeSet<String>,
    queue: VecDeque<Queued>,
    retried: BTreeSet<String>,
    current: Option<Context>,
    leg: Option<Context>,
    assisted: BTreeSet<String>,
    active_remedial: bool,
    current_errors: u64,
    current_hints: u64,
    pending: BTreeMap<String, Event>,
    pending_recalls: VecDeque<(String, bool)>,
    seed: u64,
    hint_level: u8,
}
impl Session {
    pub fn new(
        book: &Repertoire,
        mode: Mode,
        progress: &Progress,
        today: u64,
        seed: u64,
    ) -> Result<Self, String> {
        progress.validate()?;
        let graph = Arc::new(Graph::new(book)?);
        let targets: BTreeSet<_> = graph
            .nodes
            .iter()
            .filter(|(key, _)| match (mode, progress.entries.get(*key)) {
                (Mode::All, _) => true,
                (Mode::New, None) => true,
                (Mode::Due, Some(c)) => c.due_day <= today,
                (Mode::Errors, Some(c)) => c.last_result == Some(ResultKind::Assisted),
                _ => false,
            })
            .map(|(key, _)| key.clone())
            .collect();
        let mut queue: Vec<_> = targets
            .iter()
            .filter_map(|key| graph.nodes.get(key))
            .map(|c| Queued {
                context: c.clone(),
                remedial: false,
            })
            .collect();
        queue.sort_by(|a, b| {
            a.context.ply.cmp(&b.context.ply).then_with(|| {
                graph.routes[a.context.route].route.moves[..a.context.ply]
                    .cmp(&graph.routes[b.context.route].route.moves[..b.context.ply])
            })
        });
        let mut session = Self {
            pos: Chess::default(),
            side: book.side,
            state: State::Empty,
            hint_square: None,
            feedback: Feedback::NoCards,
            completed: 0,
            remaining: targets.len(),
            skipped: 0,
            hints: 0,
            errors: 0,
            path: vec![],
            path_text: String::new(),
            allowed_cache: vec![],
            graph,
            targets,
            done: BTreeSet::new(),
            skipped_keys: BTreeSet::new(),
            graded: BTreeSet::new(),
            queue: queue.into(),
            retried: BTreeSet::new(),
            current: None,
            leg: None,
            assisted: BTreeSet::new(),
            active_remedial: false,
            current_errors: 0,
            current_hints: 0,
            pending: BTreeMap::new(),
            pending_recalls: VecDeque::new(),
            seed,
            hint_level: 0,
        };
        session.next();
        Ok(session)
    }
    fn context(&self) -> Option<&Context> {
        self.graph.contexts.get(&self.path.join(" "))
    }
    pub fn stats(&self, progress: &Progress, today: u64) -> Stats {
        stats_for_nodes(&self.graph.nodes, progress, today)
    }
    pub fn current_key(&self) -> Option<String> {
        self.context().map(|c| c.key.clone())
    }
    fn choices(&self) -> BTreeMap<String, Move> {
        let mut out = BTreeMap::new();
        let ply = self.path.len();
        for line in &self.graph.routes {
            if line.route.moves.len() > ply && line.route.moves[..ply] == self.path {
                out.entry(line.route.moves[ply].clone())
                    .or_insert_with(|| line.moves[ply].clone());
            }
        }
        out
    }
    pub fn allowed_uci(&self) -> Vec<String> {
        self.allowed_cache
            .iter()
            .map(|(uci, _)| uci.clone())
            .collect()
    }
    pub fn allowed_sans(&self) -> Vec<String> {
        self.allowed_cache
            .iter()
            .map(|(_, san)| san.clone())
            .collect()
    }
    pub fn path_sans(&self) -> String {
        self.path_text.clone()
    }
    fn refresh_view(&mut self) {
        self.allowed_cache = if self.state == State::Waiting {
            self.choices()
                .into_iter()
                .map(|(uci, mv)| (uci, SanPlus::from_move(self.pos.clone(), &mv).to_string()))
                .collect()
        } else {
            vec![]
        };
    }
    fn refresh_counts(&mut self) {
        self.completed = self.done.len();
        self.skipped = self.skipped_keys.len();
        self.remaining = self
            .targets
            .difference(&self.done)
            .filter(|key| !self.skipped_keys.contains(*key))
            .count()
            + self.queue.iter().filter(|q| q.remedial).count()
            + usize::from(self.active_remedial);
    }
    fn advance(&mut self) {
        self.hint_square = None;
        self.hint_level = 0;
        self.current_errors = 0;
        self.current_hints = 0;
        if self.choices().is_empty() {
            self.state = State::Completed;
            self.feedback = Feedback::RouteEnd;
            self.allowed_cache.clear();
            self.refresh_counts();
            return;
        }
        if self.pos.turn() == self.side.color()
            && let Some(c) = self.context().cloned()
            && self.targets.contains(&c.key)
            && !self.done.contains(&c.key)
            && !self.skipped_keys.contains(&c.key)
        {
            self.current = Some(c);
            self.state = State::Waiting;
            self.refresh_counts();
            self.refresh_view();
            return;
        }
        self.state = State::Responding;
        self.allowed_cache.clear();
        self.refresh_counts();
    }
    fn play(&mut self, uci: String, mv: Move) {
        if !self.path_text.is_empty() {
            self.path_text.push(' ');
        }
        if self.pos.turn() == shakmaty::Color::White {
            self.path_text
                .push_str(&format!("{}. ", self.pos.fullmoves()));
        }
        self.path_text
            .push_str(&SanPlus::from_move(self.pos.clone(), &mv).to_string());
        self.pos.play_unchecked(&mv);
        self.path.push(uci);
        self.advance();
    }
    fn next_seed(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seed
    }
    pub fn respond(&mut self) {
        if self.state != State::Responding {
            return;
        }
        let choices = self.choices();
        if choices.is_empty() {
            self.state = State::Completed;
            self.feedback = Feedback::RouteEnd;
            return;
        }
        let index = (self.next_seed() % choices.len() as u64) as usize;
        let (uci, mv) = choices.into_iter().nth(index).expect("bounded chosen move");
        self.play(uci, mv);
    }
    fn enqueue_retry(&mut self) {
        if let Some(c) = self.context().cloned()
            && self.retried.insert(c.key.clone())
        {
            self.queue.push_back(Queued {
                context: c,
                remedial: true,
            });
            self.refresh_counts();
        }
    }
    pub fn attempt(&mut self, mv: Move) -> Result<bool, String> {
        if self.state != State::Waiting {
            return Err("Not waiting for a repertoire move".into());
        }
        if !self.pos.is_legal(&mv) {
            self.feedback = Feedback::Illegal;
            return Err("Illegal move".into());
        }
        let context = self
            .context()
            .cloned()
            .ok_or("Missing repertoire context")?;
        let choices = self.choices();
        let uci = shakmaty::uci::UciMove::from_standard(&mv).to_string();
        if !choices.contains_key(&uci) {
            self.current_errors = self.current_errors.saturating_add(1);
            self.errors = self.errors.saturating_add(1);
            self.feedback = Feedback::Outside;
            self.assisted.insert(context.key.clone());
            self.record(Event {
                key: context.key,
                correct: 0,
                errors: 1,
                hints: 0,
                clean: false,
            });
            self.enqueue_retry();
            return Ok(false);
        }
        self.record(Event {
            key: context.key.clone(),
            correct: 1,
            errors: 0,
            hints: 0,
            clean: !self.assisted.contains(&context.key),
        });
        self.active_remedial = false;
        self.skipped_keys.remove(&context.key);
        self.done.insert(context.key);
        self.feedback = Feedback::Correct;
        self.play(uci, mv);
        Ok(true)
    }
    pub fn hint(&mut self) {
        if self.state != State::Waiting {
            return;
        }
        let Some(c) = self.context().cloned() else {
            return;
        };
        let Some(mv) = self.choices().into_values().next() else {
            return;
        };
        self.hint_level = self.hint_level.saturating_add(1);
        self.current_hints = self.current_hints.saturating_add(1);
        self.hints = self.hints.saturating_add(1);
        self.hint_square = if self.hint_level == 1 {
            mv.from()
        } else {
            crate::uci_protocol::move_arrow(&mv).map(|(_, to)| to)
        };
        self.feedback = Feedback::Hint;
        self.assisted.insert(c.key.clone());
        self.record(Event {
            key: c.key,
            correct: 0,
            errors: 0,
            hints: 1,
            clean: false,
        });
        self.enqueue_retry();
    }
    pub fn reveal(&mut self) -> String {
        let ply = self.path.len();
        let line = self
            .graph
            .routes
            .iter()
            .find(|line| line.route.moves.len() > ply && line.route.moves[..ply] == self.path)
            .or_else(|| {
                self.graph
                    .routes
                    .iter()
                    .find(|line| line.route.moves == self.path)
            });
        let Some(line) = line else {
            return String::new();
        };
        let text = line.sans.join(" ");
        let displayed: BTreeMap<_, _> = (ply..line.moves.len())
            .filter(|&i| line.positions[i].turn() == self.side.color())
            .filter_map(|i| self.graph.contexts.get(&line.route.moves[..i].join(" ")))
            .map(|context| (context.key.clone(), context.clone()))
            .collect();
        let current = if self.state == State::Waiting {
            self.current_key()
        } else {
            None
        };
        if current.is_some() {
            self.hint();
        } else if !displayed.is_empty() {
            self.hints = self.hints.saturating_add(1);
        }
        for (key, context) in displayed {
            self.assisted.insert(key.clone());
            if current.as_ref() != Some(&key) {
                self.record(Event {
                    key: key.clone(),
                    correct: 0,
                    errors: 0,
                    hints: 1,
                    clean: false,
                });
                if self.targets.contains(&key) && self.retried.insert(key) {
                    self.queue.push_back(Queued {
                        context,
                        remedial: true,
                    });
                }
            }
        }
        self.feedback = Feedback::Revealed;
        self.refresh_counts();
        format!("{}\n{}", self.allowed_sans().join(" / "), text)
    }
    fn load_context(&mut self, c: Context, remedial: bool) {
        self.active_remedial = remedial && self.done.contains(&c.key);
        self.skipped_keys.remove(&c.key);
        self.leg = Some(c.clone());
        self.pos = self.graph.routes[c.route].positions[c.ply].clone();
        self.path = self.graph.prefix(&c);
        let line = &self.graph.routes[c.route];
        self.path_text = line.sans[..c.ply]
            .iter()
            .enumerate()
            .map(|(i, san)| {
                format!(
                    "{}{}",
                    if line.positions[i].turn() == shakmaty::Color::White {
                        format!("{}. ", line.positions[i].fullmoves())
                    } else {
                        String::new()
                    },
                    san
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        self.current = Some(c);
        self.current_errors = 0;
        self.current_hints = 0;
        self.hint_level = 0;
        self.hint_square = None;
        self.feedback = Feedback::Ready;
        self.state = State::Waiting;
        self.refresh_counts();
        self.refresh_view();
    }
    pub fn retry(&mut self) {
        if let Some(c) = self.leg.clone() {
            let remedial = self.done.contains(&c.key);
            self.load_context(c, remedial);
        }
    }
    pub fn next(&mut self) {
        if self.state == State::Waiting
            && let Some(context) = &self.current
            && !self.done.contains(&context.key)
        {
            self.skipped_keys.insert(context.key.clone());
        }
        while let Some(item) = self.queue.pop_front() {
            if !item.remedial
                && (self.done.contains(&item.context.key)
                    || self.skipped_keys.contains(&item.context.key))
            {
                continue;
            }
            self.load_context(item.context, item.remedial);
            return;
        }
        self.active_remedial = false;
        self.state = if self.targets.is_empty() {
            State::Empty
        } else {
            State::Completed
        };
        self.allowed_cache.clear();
        self.feedback = if self.targets.is_empty() {
            Feedback::NoCards
        } else {
            Feedback::RouteEnd
        };
        self.refresh_counts();
    }
    fn record(&mut self, event: Event) {
        // Record real submission order even when a write failure delays settling many cards.
        for _ in 0..event.errors.min(256) {
            self.pending_recalls.push_back((event.key.clone(), false));
        }
        for _ in 0..event.correct.min(256) {
            self.pending_recalls
                .push_back((event.key.clone(), event.clean));
        }
        while self.pending_recalls.len() > 256 {
            self.pending_recalls.pop_front();
        }
        self.pending
            .entry(event.key.clone())
            .and_modify(|e| {
                e.correct = e.correct.saturating_add(event.correct);
                e.errors = e.errors.saturating_add(event.errors);
                e.hints = e.hints.saturating_add(event.hints);
                e.clean &= event.clean;
            })
            .or_insert(event);
    }
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
    /// Caller can clone Session + Progress, save the candidate, and commit both only on success.
    pub fn settle(&mut self, progress: &mut Progress, today: u64) -> bool {
        if self.pending.is_empty() {
            return false;
        }
        for (key, clean) in self.pending_recalls.drain(..) {
            progress.recent.push(Recall {
                key,
                clean,
                day: today,
            });
        }
        if progress.recent.len() > 256 {
            let discard = progress.recent.len() - 256;
            progress.recent.drain(..discard);
        }
        for (_, e) in std::mem::take(&mut self.pending) {
            let c = progress.entries.entry(e.key.clone()).or_default();
            let anchor = today.max(c.last_day);
            let penalty = e.errors > 0 || e.hints > 0 || !e.clean;
            if penalty {
                c.level = 1;
            } else if !self.graded.contains(&e.key) && (c.level == 0 || c.last_day < today) {
                c.level = if c.level == 0 {
                    1
                } else {
                    (c.level + 1).min(5)
                };
            }
            if c.level == 0 {
                c.level = 1;
            }
            c.last_day = anchor;
            c.due_day = anchor.saturating_add(INTERVALS[usize::from(c.level - 1)]);
            c.correct = c.correct.saturating_add(e.correct);
            c.errors = c.errors.saturating_add(e.errors);
            c.hints = c.hints.saturating_add(e.hints);
            c.last_result = Some(if penalty {
                ResultKind::Assisted
            } else {
                ResultKind::Clean
            });
            if e.correct > 0 {
                c.last_completed_day = Some(anchor);
            }
            self.graded.insert(e.key);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repertoire::Route;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn book(side: Side, lines: &[&str]) -> Repertoire {
        Repertoire {
            id: "test-book".into(),
            name: "Opening".into(),
            side,
            created_unix: 1,
            modified_unix: 1,
            note: String::new(),
            routes: lines
                .iter()
                .enumerate()
                .map(|(i, s)| Route {
                    id: format!("r{i}"),
                    name: String::new(),
                    note: String::new(),
                    moves: s.split_whitespace().map(str::to_owned).collect(),
                })
                .collect(),
        }
    }
    fn play(s: &mut Session, text: &str) -> bool {
        let mv = text
            .parse::<shakmaty::uci::UciMove>()
            .unwrap()
            .to_move(&s.pos)
            .unwrap();
        s.attempt(mv).unwrap()
    }
    fn settle(s: &mut Session, p: &mut Progress, day: u64) {
        assert!(s.has_pending());
        assert!(s.settle(p, day));
        p.validate().unwrap();
        assert!(!s.has_pending());
    }
    fn complete_leg(s: &mut Session, p: &mut Progress, day: u64) {
        for _ in 0..100 {
            match s.state {
                State::Waiting => {
                    let text = s.allowed_uci().into_iter().next().unwrap();
                    assert!(play(s, &text));
                    if s.has_pending() {
                        settle(s, p, day);
                    }
                }
                State::Responding => s.respond(),
                State::Completed | State::Empty => return,
            }
        }
        panic!("bounded saved repertoire did not terminate");
    }
    #[test]
    fn shared_prefix_multiple_preparations_black_and_seeded_opponents() {
        let b = book(
            Side::White,
            &[
                "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5",
                "e2e4 e7e5 f1c4 g8f6 d2d3 b8c6",
                "d2d4 d7d5 c2c4 e7e6",
            ],
        );
        let p = Progress::default();
        let mut s = Session::new(&b, Mode::All, &p, 10, 22).unwrap();
        assert_eq!(s.pos.turn(), shakmaty::Color::White);
        assert_eq!(s.allowed_uci(), vec!["d2d4", "e2e4"]);
        assert!(play(&mut s, "e2e4"));
        assert_eq!(s.state, State::Responding);
        s.respond();
        assert_eq!(s.allowed_uci(), vec!["f1c4", "g1f3"]);
        assert_eq!(s.allowed_sans(), vec!["Bc4", "Nf3"]);
        assert!(play(&mut s, "f1c4"));
        assert_eq!(s.state, State::Responding);
        s.respond();
        assert_eq!(s.allowed_uci(), vec!["d2d3"]);
        assert!(s.path_sans().contains("Bc4"));
        let b = book(Side::Black, &["e2e4 e7e5 g1f3 b8c6", "e2e4 c7c5 g1f3 d7d6"]);
        let mut black = Session::new(&b, Mode::All, &p, 10, 5).unwrap();
        assert_eq!(black.side, Side::Black);
        assert_eq!(black.path, vec!["e2e4"]);
        assert_eq!(black.pos.turn(), shakmaty::Color::Black);
        assert_eq!(black.allowed_uci(), vec!["c7c5", "e7e5"]);
        assert!(play(&mut black, "c7c5"));
        black.respond();
        assert_eq!(black.allowed_uci(), vec!["d7d6"]);
        let b = book(
            Side::White,
            &["e2e4 c7c5 g1f3", "e2e4 e7e5 g1f3", "e2e4 c7c6 g1f3"],
        );
        let choices: Vec<_> = (0..20)
            .map(|seed| {
                let mut s = Session::new(&b, Mode::All, &p, 10, seed).unwrap();
                play(&mut s, "e2e4");
                s.respond();
                s.path[1].clone()
            })
            .collect();
        assert!(
            choices
                .iter()
                .all(|m| ["c7c5", "c7c6", "e7e5"].contains(&m.as_str()))
        );
        assert!(choices.iter().collect::<BTreeSet<_>>().len() > 1);
        let mut a = Session::new(&b, Mode::All, &p, 10, 7).unwrap();
        let mut c = a.clone();
        play(&mut a, "e2e4");
        play(&mut c, "e2e4");
        a.respond();
        c.respond();
        assert_eq!(a.path, c.path);
    }
    #[test]
    fn transposition_card_equivalence_conflicts_and_no_unsaved_hybrids() {
        let equivalent = book(
            Side::White,
            &["g1f3 d7d5 g2g3 g8f6 f1g2", "g2g3 d7d5 g1f3 g8f6 f1g2"],
        );
        let nodes = cards(&equivalent).unwrap();
        assert_eq!(nodes.len(), 4);
        for (key, node) in &nodes {
            assert_eq!(key, &node.key);
            assert!(!node.position.is_empty());
            assert!(!node.allowed.is_empty());
            assert!(node.prefix.len() <= 4);
        }
        let mut conflict = equivalent.clone();
        conflict.routes[1].moves[4] = "f1h3".into();
        assert_eq!(cards(&conflict).unwrap().len(), 5);
        let black = book(Side::Black, &["g1f3 d7d5 g2g3 g8f6", "g2g3 d7d5 g1f3 e7e6"]);
        let graph = Graph::new(&black).unwrap();
        let a = &graph.contexts["g1f3 d7d5 g2g3"];
        let b = &graph.contexts["g2g3 d7d5 g1f3"];
        assert_eq!(a.position, b.position);
        assert_ne!(a.key, b.key);
        let mut session = Session::new(&black, Mode::All, &Progress::default(), 10, 0).unwrap();
        assert_eq!(session.path, vec!["g1f3"]);
        assert!(play(&mut session, "d7d5"));
        session.respond();
        assert_eq!(session.path, vec!["g1f3", "d7d5", "g2g3"]);
        assert_eq!(session.allowed_uci(), vec!["g8f6"]);
        let before = crate::fen::write(&session.pos);
        assert!(!play(&mut session, "e7e6"));
        assert_eq!(crate::fen::write(&session.pos), before);
        assert_eq!(session.errors, 1);
    }
    #[test]
    fn illegal_moves_do_not_count_and_repeated_positions_do_not_loop_or_double_grade() {
        let b = book(
            Side::White,
            &["g1f3 g8f6 f3g1 f6g8 g1f3 g8f6 f3g1 f6g8 e2e4"],
        );
        let mut progress = Progress::default();
        let mut s = Session::new(&b, Mode::All, &progress, 100, 1).unwrap();
        let before = crate::fen::write(&s.pos);
        let illegal = Move::Normal {
            role: shakmaty::Role::Pawn,
            from: Square::A1,
            capture: None,
            to: Square::A8,
            promotion: None,
        };
        assert!(s.attempt(illegal).is_err());
        assert_eq!(s.errors, 0);
        assert!(!s.has_pending());
        assert_eq!(crate::fen::write(&s.pos), before);
        complete_leg(&mut s, &mut progress, 100);
        assert_eq!(s.state, State::Completed);
        assert_eq!(progress.entries.len(), 3);
        assert!(progress.entries.values().all(|c| c.correct == 1));
        assert_eq!(s.remaining, 0);
    }
    #[test]
    fn leitner_all_levels_utc_due_same_day_and_clock_rollback_are_bounded() {
        let b = book(Side::White, &["e2e4 e7e5"]);
        let key = cards(&b).unwrap().keys().next().unwrap().clone();
        let mut p = Progress::default();
        for (day, level, interval) in [
            (10, 1, 1),
            (11, 2, 3),
            (14, 3, 7),
            (21, 4, 14),
            (35, 5, 30),
            (65, 5, 30),
        ] {
            let mut s = Session::new(&b, Mode::All, &p, day, 0).unwrap();
            assert!(play(&mut s, "e2e4"));
            settle(&mut s, &mut p, day);
            assert_eq!(p.entries[&key].level, level);
            assert_eq!(p.entries[&key].due_day, day + interval);
            s.retry();
            assert!(play(&mut s, "e2e4"));
            settle(&mut s, &mut p, day);
            assert_eq!(p.entries[&key].level, level);
            let mut another = Session::new(&b, Mode::All, &p, day, 99).unwrap();
            assert!(play(&mut another, "e2e4"));
            settle(&mut another, &mut p, day);
            assert_eq!(p.entries[&key].level, level);
        }
        let previous = p.entries[&key].due_day;
        let mut back = Session::new(&b, Mode::All, &p, 2, 1).unwrap();
        assert!(play(&mut back, "e2e4"));
        settle(&mut back, &mut p, 2);
        assert_eq!(p.entries[&key].last_day, 65);
        assert_eq!(p.entries[&key].due_day, previous);
        assert_eq!(
            Session::new(&b, Mode::Due, &p, 94, 0).unwrap().state,
            State::Empty
        );
        assert_eq!(
            Session::new(&b, Mode::Due, &p, 95, 0).unwrap().state,
            State::Waiting
        );
    }
    #[test]
    fn hint_and_error_survive_retry_reveal_and_bounded_remedial_queue() {
        let b = book(Side::White, &["e2e4 e7e5"]);
        let mut p = Progress::default();
        let mut s = Session::new(&b, Mode::All, &p, 10, 0).unwrap();
        let key = s.current_key().unwrap();
        s.hint();
        assert_eq!(s.hint_square, Some(Square::E2));
        s.hint();
        assert_eq!(s.hint_square, Some(Square::E4));
        assert_eq!(s.hints, 2);
        settle(&mut s, &mut p, 10);
        assert_eq!(p.entries[&key].hints, 2);
        assert_eq!(p.entries[&key].last_result, Some(ResultKind::Assisted));
        s.retry();
        assert!(play(&mut s, "e2e4"));
        settle(&mut s, &mut p, 10);
        assert_eq!(p.entries[&key].last_result, Some(ResultKind::Assisted));
        assert_eq!(p.entries[&key].level, 1);
        s.respond();
        s.next();
        assert_eq!(s.state, State::Waiting);
        assert_eq!(s.remaining, 1);
        assert!(!play(&mut s, "d2d4"));
        assert!(s.reveal().contains("e4"));
        settle(&mut s, &mut p, 10);
        assert_eq!(p.entries[&key].errors, 1);
        assert!(play(&mut s, "e2e4"));
        settle(&mut s, &mut p, 10);
        s.respond();
        s.next();
        assert_eq!(s.state, State::Completed);
        assert_eq!(s.remaining, 0);
        assert_eq!(s.completed, 1);
    }
    #[test]
    fn mode_selection_empty_today_done_new_targets_and_local_identity_changes() {
        let b = book(Side::White, &["e2e4 e7e5 g1f3", "d2d4 d7d5 c2c4"]);
        let mut p = Progress::default();
        let mut s = Session::new(&b, Mode::All, &p, 10, 0).unwrap();
        let root = s.current_key().unwrap();
        assert!(play(&mut s, "e2e4"));
        settle(&mut s, &mut p, 10);
        assert_eq!(
            Session::new(&b, Mode::Due, &p, 10, 0).unwrap().state,
            State::Empty
        );
        let new = Session::new(&b, Mode::New, &p, 10, 0).unwrap();
        assert_eq!(new.path.len(), 2);
        assert_ne!(new.current_key(), Some(root.clone()));
        let mut old = b.clone();
        old.name = "Renamed 中文".into();
        old.routes[0].note = "Comment only".into();
        assert_eq!(
            cards(&old).unwrap().keys().collect::<Vec<_>>(),
            cards(&b).unwrap().keys().collect::<Vec<_>>()
        );
        old.routes[0].moves[2] = "f1c4".into();
        let before = cards(&b).unwrap();
        let after = cards(&old).unwrap();
        assert!(after.contains_key(&root));
        assert_eq!(before.keys().filter(|k| after.contains_key(*k)).count(), 2);
        assert_eq!(p.entries.len(), 1);
        let stats = p.stats(&old, 10).unwrap();
        assert_eq!((stats.learned, stats.new, stats.completed), (1, 2, 1));
        let mut empty = b.clone();
        empty.routes.clear();
        let empty = Session::new(&empty, Mode::All, &p, 10, 0).unwrap();
        assert_eq!(empty.state, State::Empty);
        assert_eq!(empty.remaining, 0);
    }
    fn temp() -> std::path::PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::current_dir()
            .unwrap()
            .join("target/v028-qa/train")
            .join(format!(
                "{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    #[test]
    fn progress_roundtrip_damaged_primary_failure_and_clone_transaction_preserve_pending() {
        let b = book(Side::White, &["e2e4 e7e5"]);
        let mut p = Progress::default();
        let mut s = Session::new(&b, Mode::All, &p, 20, 0).unwrap();
        s.hint();
        let root = temp();
        let file = root.join("progress.json");
        std::fs::write(&file, b"damaged-data").unwrap();
        let mut trial = s.clone();
        let mut candidate = p.clone();
        assert!(trial.settle(&mut candidate, 20));
        assert!(save_progress(&file, &candidate).is_err());
        assert!(s.has_pending());
        assert!(p.entries.is_empty());
        assert_eq!(std::fs::read(&file).unwrap(), b"damaged-data");
        let good = root.join("good.json");
        save_progress(&good, &candidate).unwrap();
        s = trial;
        p = candidate;
        assert!(!s.has_pending());
        let loaded = load_progress(&good).unwrap();
        assert_eq!(loaded.entries.len(), p.entries.len());
        assert_eq!(loaded.entries.values().next().unwrap().hints, 1);
        let newer = root.join("future.json");
        std::fs::write(&newer, b"{\"schema_version\":2,\"entries\":{}}").unwrap();
        assert!(load_progress(&newer).is_err());
        assert!(save_progress(&newer, &p).is_err());
        assert_eq!(
            std::fs::read(&newer).unwrap(),
            b"{\"schema_version\":2,\"entries\":{}}"
        );
        let blocked = root.join("directory.json");
        std::fs::create_dir(&blocked).unwrap();
        assert!(save_progress(&blocked, &p).is_err());
    }
    #[test]
    fn recent_recall_window_filters_books_bounds_pending_and_next_skips_honestly() {
        let b = book(Side::White, &["e2e4 e7e5"]);
        let mut other = b.clone();
        other.id = "other-book".into();
        let mut p = Progress::default();
        let mut a = Session::new(&b, Mode::All, &p, 10, 0).unwrap();
        for _ in 0..400 {
            assert!(!play(&mut a, "d2d4"));
        }
        assert!(play(&mut a, "e2e4"));
        settle(&mut a, &mut p, 10);
        assert_eq!(p.recent.len(), 256);
        let stats = a.stats(&p, 10);
        assert_eq!((stats.recent_correct, stats.recent_total), (0, 100));
        let mut other_session = Session::new(&other, Mode::All, &p, 10, 0).unwrap();
        for _ in 0..250 {
            assert!(play(&mut other_session, "e2e4"));
            other_session.retry();
        }
        settle(&mut other_session, &mut p, 10);
        assert_eq!(p.recent.len(), 256);
        assert_eq!(a.stats(&p, 10).recent_total, 6);
        assert_eq!(
            (
                other_session.stats(&p, 10).recent_correct,
                other_session.stats(&p, 10).recent_total
            ),
            (100, 100)
        );
        let mut skipped = Session::new(&b, Mode::All, &Progress::default(), 10, 0).unwrap();
        skipped.next();
        assert_eq!(skipped.state, State::Completed);
        assert_eq!(
            (skipped.completed, skipped.remaining, skipped.skipped),
            (0, 0, 1)
        );
        assert!(!skipped.has_pending());
        skipped.retry();
        assert_eq!(skipped.state, State::Waiting);
        assert_eq!((skipped.remaining, skipped.skipped), (1, 0));
        let mut failed = Session::new(&b, Mode::All, &Progress::default(), 10, 0).unwrap();
        assert!(!play(&mut failed, "d2d4"));
        failed.next();
        assert_eq!(failed.state, State::Waiting);
        assert_eq!(failed.remaining, 1);
        assert!(play(&mut failed, "e2e4"));
        failed.respond();
        failed.next();
        assert_eq!((failed.completed, failed.remaining), (1, 0));
        let mut value = serde_json::to_value(&p).unwrap();
        value.as_object_mut().unwrap().remove("recent");
        let old: Progress = serde_json::from_value(value).unwrap();
        assert!(old.recent.is_empty());
        old.validate().unwrap();
    }
    #[test]
    fn full_line_reveal_assists_only_displayed_future_nodes() {
        let b = book(Side::White, &["e2e4 e7e5 g1f3", "d2d4 d7d5 c2c4"]);
        let mut p = Progress::default();
        let mut s = Session::new(&b, Mode::All, &p, 10, 0).unwrap();
        let graph = Graph::new(&b).unwrap();
        let future = graph.contexts["e2e4 e7e5"].key.clone();
        let other = graph.contexts["d2d4 d7d5"].key.clone();
        assert!(s.reveal().contains("Nf3"));
        settle(&mut s, &mut p, 10);
        assert_eq!(p.entries[&future].last_result, Some(ResultKind::Assisted));
        assert!(!p.entries.contains_key(&other));
        assert!(play(&mut s, "e2e4"));
        s.respond();
        assert!(play(&mut s, "g1f3"));
        settle(&mut s, &mut p, 10);
        assert_eq!(p.entries[&future].last_result, Some(ResultKind::Assisted));
        assert_eq!(p.entries[&future].level, 1);
        assert_eq!(p.entries[&future].correct, 1);

        // Revealing while the scripted opponent is about to reply also discloses
        // the future player answer, and must prevent a later clean classification.
        let mut during_reply = Session::new(&b, Mode::All, &Progress::default(), 10, 0).unwrap();
        assert!(play(&mut during_reply, "e2e4"));
        assert_eq!(during_reply.state, State::Responding);
        assert!(during_reply.reveal().contains("Nf3"));
        let mut reply_progress = Progress::default();
        settle(&mut during_reply, &mut reply_progress, 10);
        during_reply.respond();
        assert!(play(&mut during_reply, "g1f3"));
        settle(&mut during_reply, &mut reply_progress, 10);
        assert_eq!(
            reply_progress.entries[&future].last_result,
            Some(ResultKind::Assisted)
        );

        // A completed short alternative must not reveal an unrelated longer
        // line through the context's representative route.
        let short = book(Side::White, &["e2e4 e7e5 g1f3", "d2d4"]);
        let mut finished = Session::new(&short, Mode::All, &Progress::default(), 10, 0).unwrap();
        assert!(play(&mut finished, "d2d4"));
        assert_eq!(finished.state, State::Completed);
        let answer = finished.reveal();
        assert!(answer.contains("d4"));
        assert!(!answer.contains("e4") && !answer.contains("Nf3"));
        let untouched = Graph::new(&short).unwrap().contexts["e2e4 e7e5"]
            .key
            .clone();
        let mut short_progress = Progress::default();
        settle(&mut finished, &mut short_progress, 10);
        assert!(!short_progress.entries.contains_key(&untouched));
    }
}

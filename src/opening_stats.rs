//! Mainline, first-arrival-per-game statistics, separate from engine review.
use crate::{
    game::position_key,
    library::Library,
    opening_book::{Book, SOURCE},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
pub const PREFIX: usize = 40;
pub const MAX_CACHE: usize = 32 * 1024 * 1024;
pub const ALGORITHM: u32 = 1;
pub const NORMALIZATION: u32 = 1;
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub white: u32,
    pub draws: u32,
    pub black: u32,
    pub unfinished: u32,
    pub nonstandard: u32,
}
impl Counts {
    pub fn total(&self) -> u32 {
        self.white + self.draws + self.black + self.unfinished
    }
    pub fn decided(&self) -> u32 {
        self.white + self.draws + self.black
    }
    pub fn percentages(&self) -> Option<[f64; 3]> {
        let n = self.decided();
        (n > 0).then(|| {
            [
                self.white as f64 * 100.0 / n as f64,
                self.draws as f64 * 100.0 / n as f64,
                self.black as f64 * 100.0 / n as f64,
            ]
        })
    }
    fn add(&mut self, result: &str, nonstandard: bool) {
        match result {
            "1-0" => self.white += 1,
            "0-1" => self.black += 1,
            "1/2-1/2" => self.draws += 1,
            _ => self.unfinished += 1,
        };
        self.nonstandard += u32::from(nonstandard);
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub ply: usize,
    pub result: String,
    pub next: Option<String>,
    pub nonstandard: bool,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub counts: Counts,
    pub moves: BTreeMap<String, Counts>,
    pub games: BTreeMap<String, Sample>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub key: String,
    pub members: Vec<(String, String)>,
    pub nodes: BTreeMap<String, Node>,
    pub classified: BTreeMap<String, Option<usize>>,
}
pub fn members(lib: &Library) -> Vec<(String, String)> {
    let mut v: Vec<_> = lib
        .entries
        .iter()
        .map(|e| (e.id.clone(), e.digest.clone()))
        .collect();
    v.sort();
    v
}
pub fn identity(lib: &Library) -> String {
    crate::library::digest(
        &serde_json::to_vec(&(ALGORITHM, SOURCE, NORMALIZATION, PREFIX, members(lib)))
            .expect("bounded library metadata"),
    )
}
pub fn build(
    lib: &Library,
    book: &Book,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Stats, String> {
    let mut stats = Stats {
        key: identity(lib),
        members: members(lib),
        nodes: BTreeMap::new(),
        classified: BTreeMap::new(),
    };
    for (index, (id, _)) in stats.members.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let game = lib
            .load(id)
            .map_err(|e| format!("Library game {id}: {e}"))?
            .mainline;
        let nonstandard =
            crate::fen::write(&game.positions[0]) != crate::fen::write(&shakmaty::Chess::default());
        stats
            .classified
            .insert(id.clone(), book.classify(&game).map(|m| m.primary));
        let mut seen = BTreeSet::new();
        for (ply, pos) in game.positions.iter().take(PREFIX + 1).enumerate() {
            let key = position_key(pos);
            if !seen.insert(key.clone()) {
                continue;
            }
            let node = stats.nodes.entry(key).or_default();
            node.counts.add(&game.record.result, nonstandard);
            let next = (ply < PREFIX)
                .then(|| game.record.moves.get(ply).cloned())
                .flatten();
            if let Some(m) = &next {
                node.moves
                    .entry(m.clone())
                    .or_default()
                    .add(&game.record.result, nonstandard);
            }
            node.games.insert(
                id.clone(),
                Sample {
                    ply,
                    result: game.record.result.clone(),
                    next,
                    nonstandard,
                },
            );
        }
        progress.store(index + 1, Ordering::Relaxed);
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    Ok(stats)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: u32,
    key: String,
    sha256: String,
    payload: String,
}
impl Stats {
    fn validate(&self, lib: &Library, book: &Book, cancel: &AtomicBool) -> Result<(), String> {
        if self.key != identity(lib)
            || self.members != members(lib)
            || self.classified.len() != self.members.len()
            || self.nodes.len() > lib.entries.len() * (PREFIX + 1)
        {
            return Err("Opening cache identity/limit mismatch".into());
        }
        for (id, _) in &self.members {
            if self
                .classified
                .get(id)
                .is_none_or(|v| v.is_some_and(|i| i >= book.entries.len()))
            {
                return Err("Opening cache classification mismatch".into());
            }
        }
        let mut total = 0;
        for (key, node) in &self.nodes {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            let pos = crate::fen::parse(&format!("{key} 0 1"))?;
            if position_key(&pos) != *key
                || node.games.is_empty()
                || node.games.len() > lib.entries.len()
            {
                return Err("Invalid opening cache position".into());
            }
            let mut counts = Counts::default();
            let mut moves: BTreeMap<String, Counts> = BTreeMap::new();
            for (id, s) in &node.games {
                if !self.classified.contains_key(id)
                    || s.ply > PREFIX
                    || !["1-0", "0-1", "1/2-1/2", "*"].contains(&s.result.as_str())
                {
                    return Err("Invalid opening cache sample".into());
                }
                counts.add(&s.result, s.nonstandard);
                if let Some(m) = &s.next {
                    if s.ply == PREFIX {
                        return Err("Opening cache exceeds prefix".into());
                    }
                    if crate::uci_protocol::legal_move(&pos, m)?.is_none() {
                        return Err("Opening cache has a null continuation".into());
                    }
                    moves
                        .entry(m.clone())
                        .or_default()
                        .add(&s.result, s.nonstandard);
                }
            }
            if counts != node.counts || moves != node.moves {
                return Err("Opening cache counts mismatch".into());
            }
            total += node.games.len();
        }
        if total > lib.entries.len() * (PREFIX + 1) {
            return Err("Opening cache total sample limit".into());
        }
        Ok(())
    }
}
pub fn load(
    path: &Path,
    lib: &Library,
    book: &Book,
    cancel: &AtomicBool,
) -> Result<Option<Stats>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let e: Envelope = serde_json::from_slice(&crate::library::read_bounded(path, MAX_CACHE)?)
        .map_err(|e| format!("Opening cache: {e}"))?;
    if e.schema != 1 {
        return Err("Unknown opening cache schema; preserved".into());
    }
    if e.sha256 != crate::library::digest(e.payload.as_bytes()) {
        return Err("Opening cache checksum mismatch; preserved".into());
    }
    if e.key != identity(lib) {
        return Ok(None);
    }
    let stats: Stats =
        serde_json::from_str(&e.payload).map_err(|e| format!("Opening cache payload: {e}"))?;
    stats.validate(lib, book, cancel)?;
    Ok(Some(stats))
}
pub fn save(path: &Path, stats: &Stats) -> Result<(), String> {
    let payload = serde_json::to_string(stats).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&Envelope {
        schema: 1,
        key: stats.key.clone(),
        sha256: crate::library::digest(payload.as_bytes()),
        payload,
    })
    .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_CACHE {
        return Err("Opening cache exceeds 32 MiB; statistics remain in memory".into());
    }
    crate::persistence::atomic_write(path, &bytes)
}

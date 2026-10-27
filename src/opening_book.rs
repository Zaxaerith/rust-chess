//! Pinned CC0 reference names. Constructed lazily off the UI thread.
use crate::{fen, game::position_key, game_record::ValidatedRecord};
use shakmaty::{Chess, Move, uci::UciMove};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
pub const SOURCE: &str = "a6189a30dc273ccb21fc2536a9a2fefd5592a67a";
pub const TSV: [&str; 5] = [
    include_str!("../data/chess-openings/a.tsv"),
    include_str!("../data/chess-openings/b.tsv"),
    include_str!("../data/chess-openings/c.tsv"),
    include_str!("../data/chess-openings/d.tsv"),
    include_str!("../data/chess-openings/e.tsv"),
];
pub struct Entry {
    pub eco: String,
    pub name: String,
    pub moves: Vec<Move>,
    pub ucis: Vec<String>,
    pub key: String,
}
pub struct Book {
    pub entries: Vec<Entry>,
    pub positions: BTreeMap<String, Vec<usize>>,
    pub next: BTreeMap<String, BTreeMap<String, String>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    pub primary: usize,
    pub candidates: usize,
    pub exact: bool,
}
impl Book {
    pub fn load(cancel: &AtomicBool) -> Result<Self, String> {
        let mut book = Self {
            entries: vec![],
            positions: BTreeMap::new(),
            next: BTreeMap::new(),
        };
        for (file, text) in TSV.iter().enumerate() {
            if text.len() > 512 * 1024 || text.lines().next() != Some("eco\tname\tpgn") {
                return Err("Invalid embedded ECO TSV".into());
            }
            for (row, line) in text.lines().skip(1).enumerate() {
                if cancel.load(Ordering::Relaxed) {
                    return Err("Cancelled".into());
                }
                if book.entries.len() >= 5000 || line.len() > 8192 {
                    return Err("ECO data limit".into());
                }
                let fields: Vec<_> = line.split('\t').collect();
                if fields.len() != 3
                    || fields[0].len() != 3
                    || !matches!(fields[0].as_bytes()[0], b'A'..=b'E')
                    || !fields[0].as_bytes()[1..].iter().all(u8::is_ascii_digit)
                    || fields[1].is_empty()
                {
                    return Err(format!("ECO TSV {file} row {}", row + 2));
                }
                // Reuse the strict, locked PGN parser and shakmaty SAN validation.
                let mut parsed =
                    crate::pgn::import_annotated(format!("{} *", fields[2]).as_bytes());
                // Reference TSV omits results, including two mating lines. Let
                // the existing board-result validator determine their outcome.
                if parsed
                    .as_ref()
                    .err()
                    .is_some_and(|e| e == "Result contradicts final board outcome")
                {
                    for result in ["1-0", "0-1", "1/2-1/2"] {
                        let candidate = crate::pgn::import_annotated(
                            format!("{} {result}", fields[2]).as_bytes(),
                        );
                        if candidate.is_ok() {
                            parsed = candidate;
                            break;
                        }
                    }
                }
                let g = parsed
                    .map_err(|e| format!("ECO {file}:{}: {e}", row + 2))?
                    .mainline;
                if g.moves.len() > 100 {
                    return Err("ECO reference path exceeds 100 plies".into());
                }
                for (i, m) in g.moves.iter().enumerate() {
                    book.next
                        .entry(position_key(&g.positions[i]))
                        .or_default()
                        .insert(UciMove::from_standard(m).to_string(), g.sans[i].clone());
                }
                let key = position_key(g.positions.last().expect("initial"));
                book.positions
                    .entry(key.clone())
                    .or_default()
                    .push(book.entries.len());
                book.entries.push(Entry {
                    eco: fields[0].into(),
                    name: fields[1].into(),
                    moves: g.moves,
                    ucis: g.record.moves,
                    key,
                });
            }
        }
        Ok(book)
    }
    /// Stable a.tsv..e.tsv / original row order, with exact history preferred.
    pub fn identify(&self, pos: &Chess, history: &[Move]) -> Option<Match> {
        let ids = self.positions.get(&position_key(pos))?;
        let ucis: Vec<_> = history
            .iter()
            .map(|m| UciMove::from_standard(m).to_string())
            .collect();
        let exact = ids.iter().copied().find(|i| self.entries[*i].ucis == ucis);
        Some(Match {
            primary: exact.unwrap_or(ids[0]),
            candidates: ids.len(),
            exact: exact.is_some(),
        })
    }
    pub fn classify(&self, g: &ValidatedRecord) -> Option<Match> {
        if fen::write(&g.positions[0]) != fen::write(&Chess::default()) {
            return None;
        }
        g.positions
            .iter()
            .enumerate()
            .filter_map(|(i, p)| self.identify(p, &g.moves[..i]))
            .next_back()
    }
    pub fn search(&self, q: &str) -> Vec<usize> {
        let q = q.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.eco.to_lowercase().contains(&q) || e.name.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }
}

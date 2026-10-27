//! Bounded official-schema CSV decoding and shakmaty-only puzzle validation.
use serde::{Deserialize, Serialize};
use shakmaty::{Chess, Color, Move, Position, san::SanPlus, uci::UciMove};
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PUZZLES: usize = 20_000;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    pub moves: Vec<String>,
    pub rating: u16,
    pub deviation: u16,
    pub popularity: i16,
    pub themes: Vec<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub row: Option<u64>,
}
#[derive(Clone)]
pub struct Validated {
    pub data: Puzzle,
    pub positions: Vec<Chess>,
    pub moves: Vec<Move>,
    pub sans: Vec<String>,
    pub solver: Color,
}
impl Puzzle {
    pub fn validate(self) -> Result<Validated, String> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err("Invalid PuzzleId".into());
        }
        if !(2..=32).contains(&self.moves.len())
            || !self.moves.len().is_multiple_of(2)
            || self.rating > 4000
            || self.deviation > 1000
            || !(-100..=100).contains(&self.popularity)
            || self.themes.len() > 32
            || self.themes.iter().any(|t| {
                t.is_empty() || t.len() > 64 || !t.bytes().all(|b| b.is_ascii_alphanumeric())
            })
            || self
                .url
                .as_ref()
                .is_some_and(|s| s.len() > 512 || s.chars().any(char::is_control))
        {
            return Err("Puzzle fields/answer limits".into());
        }
        let mut pos = crate::fen::parse(&self.fen)?;
        let mut positions = vec![pos.clone()];
        let mut moves = vec![];
        let mut sans = vec![];
        for (i, s) in self.moves.iter().enumerate() {
            if s.len() > 5 {
                return Err(format!("UCI length at ply {}", i + 1));
            }
            let mv = s
                .parse::<UciMove>()
                .map_err(|e| format!("UCI ply {}: {e}", i + 1))?
                .to_move(&pos)
                .map_err(|e| format!("Illegal answer ply {}: {e}", i + 1))?;
            sans.push(SanPlus::from_move(pos.clone(), &mv).to_string());
            pos.play_unchecked(&mv);
            moves.push(mv);
            positions.push(pos.clone());
        }
        let solver = positions[1].turn();
        if self.themes.iter().any(|s| s == "mateIn1") && (moves.len() != 2 || !pos.is_checkmate()) {
            return Err("mateIn1 answer must end in checkmate".into());
        }
        Ok(Validated {
            data: self,
            positions,
            moves,
            sans,
            solver,
        })
    }
}
pub fn validate_set(
    items: Vec<Puzzle>,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Vec<Arc<Validated>>, String> {
    if items.len() > MAX_PUZZLES {
        return Err("Puzzle count exceeds 20000".into());
    }
    let mut ids = BTreeSet::new();
    let mut out = vec![];
    for (i, p) in items.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        if !ids.insert(p.id.clone()) {
            return Err(format!("Duplicate PuzzleId {} at row {}", p.id, i + 1));
        }
        let id = p.id.clone();
        out.push(Arc::new(
            p.validate()
                .map_err(|e| format!("Puzzle {id}, row {}: {e}", i + 1))?,
        ));
        progress.store(i + 1, Ordering::Relaxed);
    }
    Ok(out)
}
pub fn builtin() -> Result<Vec<Arc<Validated>>, String> {
    static BANK: std::sync::OnceLock<Result<Vec<Arc<Validated>>, String>> =
        std::sync::OnceLock::new();
    BANK.get_or_init(|| {
        let data: Vec<Puzzle> =
            serde_json::from_str(include_str!("../data/chess-puzzles/puzzles.json"))
                .map_err(|e| e.to_string())?;
        validate_set(data, &AtomicBool::new(false), &AtomicUsize::new(0))
    })
    .clone()
}
// RFC 4180 fields, including quoted commas/newlines and doubled quotes. Strict suffixes.
fn csv_rows(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows = vec![];
    let mut row = vec![];
    let mut field = String::new();
    let mut quoted = false;
    let mut closed = false;
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' if field.is_empty() && !closed => quoted = true,
                ',' => {
                    row.push(std::mem::take(&mut field));
                    closed = false;
                }
                '\r' | '\n' => {
                    if c == '\r' && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut field));
                    if row.iter().any(|s| !s.is_empty()) {
                        rows.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                    closed = false;
                }
                _ if closed || c == '"' => return Err("Invalid CSV quote/suffix".into()),
                _ => field.push(c),
            }
        }
        if field.len() > 4096 || row.len() > 32 || rows.len() > MAX_PUZZLES + 1 {
            return Err("CSV resource limit".into());
        }
    }
    if quoted {
        return Err("Unclosed CSV quote".into());
    }
    if !field.is_empty() || !row.is_empty() || closed {
        row.push(field);
        rows.push(row);
    }
    Ok(rows)
}
pub fn csv(bytes: &[u8]) -> Result<Vec<Puzzle>, String> {
    if bytes.len() > MAX_BYTES {
        return Err("CSV exceeds 16 MiB".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| format!("CSV UTF-8: {e}"))?;
    let rows = csv_rows(text)?;
    let header = rows.first().ok_or("Empty CSV")?;
    let mut names = BTreeSet::new();
    if header.iter().any(|s| !names.insert(s)) {
        return Err("Duplicate CSV header".into());
    }
    let get = |row: &Vec<String>, key: &str| -> Result<String, String> {
        let i = header
            .iter()
            .position(|s| s == key)
            .ok_or_else(|| format!("Missing CSV column {key}"))?;
        row.get(i)
            .cloned()
            .ok_or_else(|| format!("Missing CSV field {key}"))
    };
    for key in [
        "PuzzleId",
        "FEN",
        "Moves",
        "Rating",
        "RatingDeviation",
        "Popularity",
        "NbPlays",
        "Themes",
    ] {
        if !header.iter().any(|s| s == key) {
            return Err(format!("Missing CSV column {key}"));
        }
    }
    let mut out = vec![];
    for (i, row) in rows.iter().enumerate().skip(1) {
        if row.len() != header.len() {
            return Err(format!("CSV row {} field count", i + 1));
        }
        let parse = || -> Result<Puzzle, String> {
            let number = |key| -> Result<u32, String> {
                get(row, key)?.parse().map_err(|_| format!("Invalid {key}"))
            };
            let rating = u16::try_from(number("Rating")?).map_err(|_| "Rating overflow")?;
            let deviation =
                u16::try_from(number("RatingDeviation")?).map_err(|_| "Deviation overflow")?;
            let _ = number("NbPlays")?;
            Ok(Puzzle {
                id: get(row, "PuzzleId")?,
                fen: get(row, "FEN")?,
                moves: get(row, "Moves")?
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect(),
                rating,
                deviation,
                popularity: get(row, "Popularity")?
                    .parse()
                    .map_err(|_| "Invalid Popularity")?,
                themes: get(row, "Themes")?
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect(),
                url: header
                    .iter()
                    .position(|s| s == "GameUrl")
                    .map(|j| row[j].clone())
                    .filter(|s| !s.is_empty()),
                row: None,
            })
        };
        out.push(parse().map_err(|e| format!("CSV row {}: {e}", i + 1))?);
    }
    if out.is_empty() || out.len() > MAX_PUZZLES {
        return Err("CSV must contain 1..20000 puzzles".into());
    }
    Ok(out)
}

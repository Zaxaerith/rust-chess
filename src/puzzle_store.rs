//! Separate user packs and progress; damaged primaries are never overwritten.
use crate::{
    puzzle_data::{self, Puzzle, Validated},
    puzzle_session::Outcome,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub attempts: u32,
    pub first: u32,
    pub assisted: u32,
    pub revealed: u32,
    pub skipped: u32,
    pub hints: u32,
    pub errors: u32,
    pub seconds: u64,
    pub last_unix: u64,
    pub last: Option<Outcome>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub schema_version: u32,
    pub entries: BTreeMap<String, Entry>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: BTreeMap::new(),
        }
    }
}
pub fn read(path: &Path, max: usize) -> Result<Option<Vec<u8>>, String> {
    use std::io::Read;
    let f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let mut bytes = vec![];
    f.take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > max {
        return Err("Puzzle storage size limit".into());
    }
    Ok(Some(bytes))
}
pub fn progress(path: &Path) -> Result<Progress, String> {
    let Some(bytes) = read(path, puzzle_data::MAX_BYTES)? else {
        return Ok(Progress::default());
    };
    let p: Progress =
        serde_json::from_slice(&bytes).map_err(|e| format!("Progress damaged (preserved): {e}"))?;
    if p.schema_version != 1
        || p.entries.len() > 50_000
        || p.entries.iter().any(|(id, e)| {
            id.is_empty()
                || id.len() > 64
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                || u64::from(e.attempts)
                    != u64::from(e.first)
                        + u64::from(e.assisted)
                        + u64::from(e.revealed)
                        + u64::from(e.skipped)
        })
    {
        return Err("Unsupported/invalid progress (preserved)".into());
    }
    Ok(p)
}
pub fn save_progress(path: &Path, p: &Progress) -> Result<(), String> {
    let _ = progress(path)?;
    let bytes = serde_json::to_vec(p).map_err(|e| e.to_string())?;
    if bytes.len() > puzzle_data::MAX_BYTES || p.entries.len() > 50_000 {
        return Err("Progress limit".into());
    }
    crate::persistence::atomic_write(path, &bytes)
}
pub fn pack_path(root: &Path) -> PathBuf {
    root.join("puzzles-v1/user.json")
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pack {
    schema_version: u32,
    puzzles: Vec<Puzzle>,
}
pub fn load_pack(
    root: &Path,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Vec<Arc<Validated>>, String> {
    let Some(b) = read(&pack_path(root), 32 * 1024 * 1024)? else {
        return Ok(vec![]);
    };
    let p: Pack =
        serde_json::from_slice(&b).map_err(|e| format!("User pack damaged (preserved): {e}"))?;
    if p.schema_version != 1 {
        return Err("Unsupported user pack".into());
    }
    let validated = puzzle_data::validate_set(p.puzzles, cancel, progress)?;
    let builtin = puzzle_data::builtin()?;
    let ids: std::collections::BTreeSet<_> = builtin.iter().map(|p| p.data.id.as_str()).collect();
    if validated.iter().any(|p| ids.contains(p.data.id.as_str())) {
        return Err("User pack duplicates an embedded PuzzleId (preserved)".into());
    }
    Ok(validated)
}
pub struct Imported {
    pub bank: Vec<Arc<Validated>>,
    pub added: usize,
    pub duplicates: usize,
}
pub fn import(
    root: &Path,
    source: &Path,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Imported, String> {
    let b = read(source, puzzle_data::MAX_BYTES)?.ok_or("CSV source missing")?;
    let items = puzzle_data::csv(&b)?;
    let mut existing = load_pack(root, cancel, progress)?;
    let mut bank = puzzle_data::builtin()?;
    let mut ids: std::collections::BTreeSet<_> = bank
        .iter()
        .chain(existing.iter())
        .map(|p| p.data.id.clone())
        .collect();
    let mut added = 0;
    let mut duplicates = 0;
    // Validate even duplicate input rows; deduplicate only after validation.
    for (i, p) in items.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let id = p.id.clone();
        let v = p
            .validate()
            .map_err(|e| format!("CSV puzzle {id}, record {}: {e}; 0 archived", i + 1))?;
        if ids.insert(id) {
            existing.push(Arc::new(v));
            added += 1;
        } else {
            duplicates += 1;
        }
        progress.store(i + 1, Ordering::Relaxed);
    }
    if existing.len() > puzzle_data::MAX_PUZZLES {
        return Err("User pack total exceeds 20000".into());
    }
    let pack = Pack {
        schema_version: 1,
        puzzles: existing.iter().map(|p| p.data.clone()).collect(),
    };
    let bytes = serde_json::to_vec(&pack).map_err(|e| e.to_string())?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("User pack exceeds 32 MiB".into());
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    crate::persistence::atomic_write(&pack_path(root), &bytes)?;
    bank.extend(existing);
    Ok(Imported {
        bank,
        added,
        duplicates,
    })
}

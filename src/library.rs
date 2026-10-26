//! Small offline PGN archive. The index never contains board/replay objects.
use crate::{annotated_game::AnnotatedGame, persistence, pgn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_ENTRIES: usize = 2000;
pub const MAX_INDEX_BYTES: usize = 4 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub digest: String,
    pub white: String,
    pub black: String,
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub result: String,
    pub plies: usize,
    pub favorite: bool,
    pub imported: u64,
    pub modified: u64,
}
impl Entry {
    pub fn file(&self) -> String {
        format!("{}-{}.pgn", self.id, self.digest)
    }
    fn from_game(id: String, digest: String, game: &AnnotatedGame) -> Self {
        let tag = |k: &str| {
            game.mainline
                .record
                .tags
                .get(k)
                .cloned()
                .unwrap_or_default()
        };
        Self {
            id,
            digest,
            white: tag("White"),
            black: tag("Black"),
            event: tag("Event"),
            site: tag("Site"),
            date: tag("Date"),
            round: tag("Round"),
            result: game.mainline.record.result.clone(),
            plies: game.mainline.moves.len(),
            favorite: false,
            imported: now(),
            modified: now(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    schema: u32,
    entries: Vec<Entry>,
}
#[derive(Clone)]
pub struct Library {
    pub root: PathBuf,
    pub entries: Vec<Entry>,
    revision: Option<String>,
}
#[derive(Default, Debug)]
pub struct ImportReport {
    pub added: usize,
    pub skipped: usize,
    pub failed: usize,
    pub error: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Recent,
    Date,
    Players,
    Favorite,
}
impl Sort {
    pub fn next(self) -> Self {
        match self {
            Self::Recent => Self::Date,
            Self::Date => Self::Players,
            Self::Players => Self::Favorite,
            Self::Favorite => Self::Recent,
        }
    }
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn regular(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !m.is_file() || m.file_type().is_symlink() {
        return Err(format!("Not a regular managed file: {}", path.display()));
    }
    Ok(())
}
pub fn read_bounded(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    regular(path)?;
    let mut out = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("Open {}: {e}", path.display()))?
        .take(max as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|e| format!("Read {}: {e}", path.display()))?;
    if out.len() > max {
        return Err(format!("File exceeds {max} bytes"));
    }
    Ok(out)
}
impl Library {
    pub fn open(root: PathBuf) -> Result<Self, String> {
        // Loading is read-only. A missing index never causes an automatic scan.
        let path = root.join("index.json");
        let entries = match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => vec![],
            Err(e) => return Err(format!("Library index: {e}")),
            Ok(_) => {
                let index: Index = serde_json::from_slice(&read_bounded(&path, MAX_INDEX_BYTES)?)
                    .map_err(|e| {
                    format!("Library index invalid; preserve it for recovery: {e}")
                })?;
                if index.schema != 1 {
                    return Err("Unsupported library index schema; file preserved".into());
                }
                index.entries
            }
        };
        let revision = if path.exists() {
            Some(digest(&read_bounded(&path, MAX_INDEX_BYTES)?))
        } else {
            None
        };
        let lib = Self {
            root,
            entries,
            revision,
        };
        lib.directories()?;
        lib.validate_entries()?;
        Ok(lib)
    }
    fn validate_entries(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        let mut hashes = HashSet::new();
        if self.entries.len() > MAX_ENTRIES {
            return Err("Library exceeds 2000 games".into());
        }
        for e in &self.entries {
            if !hex(&e.id)
                || !hex(&e.digest)
                || !ids.insert(&e.id)
                || !hashes.insert(&e.digest)
                || e.plies > 4096
                || !["*", "1-0", "0-1", "1/2-1/2"].contains(&e.result.as_str())
                || [&e.white, &e.black, &e.event, &e.site, &e.date, &e.round]
                    .iter()
                    .any(|s| s.len() > 4096 || s.chars().any(char::is_control))
            {
                return Err("Invalid/duplicate library index entry; index preserved".into());
            }
        }
        Ok(())
    }
    fn directories(&self) -> Result<(), String> {
        for p in [&self.root, self.root.join("games").as_path()] {
            if let Ok(m) = fs::symlink_metadata(p)
                && (!m.is_dir() || m.file_type().is_symlink())
            {
                return Err("Library directories must be ordinary directories".into());
            }
        }
        Ok(())
    }
    fn commit(&mut self, entries: Vec<Entry>) -> Result<(), String> {
        self.directories()?;
        let next = Self {
            root: self.root.clone(),
            entries,
            revision: self.revision.clone(),
        };
        next.validate_entries()?;
        let bytes = serde_json::to_vec_pretty(&Index {
            schema: 1,
            entries: next.entries.clone(),
        })
        .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_INDEX_BYTES {
            return Err("Library index exceeds 4 MiB".into());
        }
        let path = self.root.join("index.json");
        let actual = match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
            Ok(_) => Some(digest(&read_bounded(&path, MAX_INDEX_BYTES)?)),
        };
        if actual != self.revision {
            return Err(
                "Library index changed outside this instance; reopen Library before writing".into(),
            );
        }
        persistence::atomic_write(&path, &bytes)?;
        self.revision = Some(digest(&bytes));
        self.entries = next.entries;
        Ok(())
    }
    pub fn entry(&self, id: &str) -> Result<&Entry, String> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| "Library game no longer exists".into())
    }
    pub fn load(&self, id: &str) -> Result<AnnotatedGame, String> {
        self.directories()?;
        let e = self.entry(id)?;
        let bytes = read_bounded(
            &self.root.join("games").join(e.file()),
            crate::game_record::MAX_FILE_BYTES,
        )?;
        if digest(&bytes) != e.digest {
            return Err("Library PGN hash mismatch; file preserved".into());
        }
        pgn::import_annotated(&bytes)
    }
    pub fn add(&mut self, game: &AnnotatedGame) -> Result<Option<String>, String> {
        let text = pgn::export_annotated(game)?;
        let hash = digest(text.as_bytes());
        if self.entries.iter().any(|e| e.digest == hash) {
            return Ok(None);
        }
        if self.entries.len() >= MAX_ENTRIES {
            return Err("Library exceeds 2000 games".into());
        }
        let mut id = hash.clone();
        let mut n = 0;
        while self.entries.iter().any(|e| e.id == id) {
            n += 1;
            id = digest(format!("{hash}:{n}").as_bytes());
        }
        let mut entry = Entry::from_game(id.clone(), hash, game);
        entry.imported = entry.imported.max(
            self.entries
                .iter()
                .map(|e| e.imported)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
        entry.modified = entry.imported;
        self.directories()?;
        let path = self.root.join("games").join(entry.file());
        // Never overwrite even an unindexed copy from an interrupted transaction.
        if path.exists() {
            if read_bounded(&path, crate::game_record::MAX_FILE_BYTES)? != text.as_bytes() {
                return Err("Unindexed PGN conflict; run Library check".into());
            }
        } else {
            persistence::atomic_write(&path, text.as_bytes())?;
        }
        let mut next = self.entries.clone();
        next.push(entry);
        self.commit(next)?;
        Ok(Some(id))
    }
    pub fn save(&mut self, id: &str, game: &AnnotatedGame) -> Result<(), String> {
        let old = self.entry(id)?.clone();
        // Verify the old file before replacing its index pointer.
        self.load(id)?;
        let text = pgn::export_annotated(game)?;
        let hash = digest(text.as_bytes());
        if self.entries.iter().any(|e| e.id != id && e.digest == hash) {
            return Err(
                "Another library game already has this exact content; neither copy was replaced"
                    .into(),
            );
        }
        if hash == old.digest {
            return Ok(());
        }
        let mut entry = Entry::from_game(id.into(), hash, game);
        entry.favorite = old.favorite;
        entry.imported = old.imported;
        entry.modified = entry.modified.max(old.modified.saturating_add(1));
        let path = self.root.join("games").join(entry.file());
        if path.exists() {
            if read_bounded(&path, crate::game_record::MAX_FILE_BYTES)? != text.as_bytes() {
                return Err("Library revision conflict".into());
            }
        } else {
            persistence::atomic_write(&path, text.as_bytes())?;
        }
        let next = self
            .entries
            .iter()
            .map(|e| if e.id == id { entry.clone() } else { e.clone() })
            .collect();
        self.commit(next)?;
        fs::remove_file(self.root.join("games").join(old.file())).map_err(|e|format!("New PGN saved, but old managed revision could not be removed: {e}. Run Library check."))?;
        Ok(())
    }
    pub fn favorite(&mut self, id: &str) -> Result<(), String> {
        self.entry(id)?;
        let mut next = self.entries.clone();
        for e in &mut next {
            if e.id == id {
                e.favorite = !e.favorite;
            }
        }
        self.commit(next)
    }
    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        self.directories()?;
        let entry = self.entry(id)?.clone();
        let path = self.root.join("games").join(entry.file());
        // Missing files may be explicitly removed from the index. Other errors
        // are checked before committing; no source PGN/cache/autosave is touched.
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
            Ok(_) => regular(&path)?,
        }
        self.commit(
            self.entries
                .iter()
                .filter(|e| e.id != id)
                .cloned()
                .collect(),
        )?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!(
                "Index entry removed, but managed PGN removal failed: {e}. Run Library check."
            )),
        }
    }
    pub fn import(&mut self, bytes: &[u8], mut progress: impl FnMut(usize)) -> ImportReport {
        let mut report = ImportReport::default();
        let result = pgn::import_collection(bytes, |i, game| {
            match self.add(&game) {
                Ok(Some(_)) => report.added += 1,
                Ok(None) => report.skipped += 1,
                Err(e) => return Err(format!("Archive write failed: {e}")),
            };
            progress(i);
            Ok(())
        });
        if let Err(e) = result {
            report.failed = 1;
            report.error = Some(format!(
                "{}: {}; stopped, later games were not processed",
                if e.game == 0 {
                    "Input file".into()
                } else {
                    format!("Game {}", e.game)
                },
                e.reason
            ));
        }
        report
    }
    pub fn search(
        &self,
        query: &str,
        favorites: bool,
        result: Option<&str>,
        sort: Sort,
    ) -> Vec<String> {
        let q = query.to_lowercase();
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|e| {
                (!favorites || e.favorite)
                    && result.is_none_or(|r| r == e.result)
                    && [&e.white, &e.black, &e.event, &e.date, &e.result]
                        .iter()
                        .any(|s| s.to_lowercase().contains(&q))
            })
            .collect();
        entries.sort_by(|a, b| {
            match sort {
                Sort::Recent => b.imported.cmp(&a.imported),
                Sort::Date => b.date.cmp(&a.date),
                Sort::Players => a
                    .white
                    .to_lowercase()
                    .cmp(&b.white.to_lowercase())
                    .then(a.black.to_lowercase().cmp(&b.black.to_lowercase())),
                Sort::Favorite => b
                    .favorite
                    .cmp(&a.favorite)
                    .then(b.imported.cmp(&a.imported)),
            }
            .then(a.id.cmp(&b.id))
        });
        entries.into_iter().map(|e| e.id.clone()).collect()
    }
    /// Explicit bounded inspection/recovery; never run during startup/rendering.
    /// Index corruption is preserved, not replaced. Unindexed conflicting old
    /// revisions are reported for human recovery, never silently deleted.
    pub fn recover(&mut self) -> Result<String, String> {
        self.directories()?;
        let mut missing = 0;
        for e in &self.entries {
            if self.load(&e.id).is_err() {
                missing += 1;
            }
        }
        let dir = self.root.join("games");
        if !dir.exists() {
            return Ok(format!("Check: {missing} missing/invalid, 0 recovered"));
        }
        let mut recovered = 0;
        let mut conflicts = 0;
        for (count, item) in fs::read_dir(&dir).map_err(|e| e.to_string())?.enumerate() {
            if count >= MAX_ENTRIES * 2 {
                return Err("Library check scan limit (4000 files); no files deleted".into());
            }
            let path = item.map_err(|e| e.to_string())?.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some((id, hash)) = name.strip_suffix(".pgn").and_then(|s| s.split_once('-')) else {
                continue;
            };
            if !hex(id) || !hex(hash) || self.entries.iter().any(|e| e.file() == name) {
                continue;
            }
            if self.entries.iter().any(|e| e.id == id || e.digest == hash) {
                conflicts += 1;
                continue;
            }
            let bytes = read_bounded(&path, crate::game_record::MAX_FILE_BYTES)?;
            if digest(&bytes) != hash {
                conflicts += 1;
                continue;
            }
            let game = pgn::import_annotated(&bytes)?;
            let entry = Entry::from_game(id.into(), hash.into(), &game);
            let mut next = self.entries.clone();
            next.push(entry);
            self.commit(next)?;
            recovered += 1;
        }
        Ok(format!(
            "Check: {missing} missing/invalid, {recovered} recovered, {conflicts} conflicting revisions preserved"
        ))
    }
}

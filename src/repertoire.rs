//! Personal repertoire content only. Chess rules and SAN remain in shakmaty.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shakmaty::{Chess, Color, Move, Position, san::SanPlus, uci::UciMove};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_REPERTOIRES: usize = 24;
pub const MAX_ROUTES: usize = 1000;
pub const MAX_PLIES: usize = 80;
pub const MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    White,
    Black,
}
impl Side {
    pub fn color(self) -> Color {
        match self {
            Self::White => Color::White,
            Self::Black => Color::Black,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub id: String,
    pub name: String,
    pub note: String,
    pub moves: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repertoire {
    pub id: String,
    pub name: String,
    pub side: Side,
    pub created_unix: u64,
    pub modified_unix: u64,
    pub note: String,
    pub routes: Vec<Route>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    pub schema_version: u32,
    pub repertoires: Vec<Repertoire>,
}
impl Default for Collection {
    fn default() -> Self {
        Self {
            schema_version: 1,
            repertoires: Vec::new(),
        }
    }
}
#[derive(Clone)]
pub struct ValidatedRoute {
    pub route: Route,
    pub positions: Vec<Chess>,
    pub moves: Vec<Move>,
    pub sans: Vec<String>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn valid_name(s: &str, required: bool) -> bool {
    (!required || !s.trim().is_empty()) && s.len() <= 128 && !s.chars().any(char::is_control)
}
fn valid_note(s: &str) -> bool {
    s.len() <= 4096 && !s.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
}
/// IDs are immutable after creation, independent of names, paths and annotations.
pub(crate) fn new_id(prefix: &str) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seed = format!(
        "{prefix}/{nanos}/{}/{n}",
        std::process::id(),
        n = NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let digest = format!("{:x}", Sha256::digest(seed.as_bytes()));
    format!("{prefix}-{}", &digest[..32])
}

pub fn validate_route(route: &Route) -> Result<ValidatedRoute, String> {
    if !valid_id(&route.id) || !valid_name(&route.name, false) || !valid_note(&route.note) {
        return Err("Invalid route ID/name/note (128-byte name / 4096-byte note limit)".into());
    }
    if route.moves.is_empty() || route.moves.len() > MAX_PLIES {
        return Err("A route needs 1..80 plies from the standard initial position".into());
    }
    let mut pos = Chess::default();
    let mut positions = vec![pos.clone()];
    let mut moves = Vec::with_capacity(route.moves.len());
    let mut sans = Vec::with_capacity(route.moves.len());
    for (i, text) in route.moves.iter().enumerate() {
        if !(4..=5).contains(&text.len()) {
            return Err(format!("Invalid UCI at ply {}", i + 1));
        }
        let mv = text
            .parse::<UciMove>()
            .map_err(|e| format!("UCI ply {}: {e}", i + 1))?
            .to_move(&pos)
            .map_err(|e| format!("Illegal route ply {} ({text}): {e}", i + 1))?;
        if UciMove::from_standard(&mv).to_string() != *text {
            return Err(format!("Non-canonical standard UCI at ply {}", i + 1));
        }
        sans.push(SanPlus::from_move(pos.clone(), &mv).to_string());
        pos.play_unchecked(&mv);
        moves.push(mv);
        positions.push(pos.clone());
    }
    Ok(ValidatedRoute {
        route: route.clone(),
        positions,
        moves,
        sans,
    })
}
impl Collection {
    pub fn get(&self, id: &str) -> Option<&Repertoire> {
        self.repertoires.iter().find(|b| b.id == id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Repertoire> {
        self.repertoires.iter_mut().find(|b| b.id == id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Unsupported repertoire schema; file preserved".into());
        }
        if self.repertoires.len() > MAX_REPERTOIRES
            || self
                .repertoires
                .iter()
                .map(|b| b.routes.len())
                .sum::<usize>()
                > MAX_ROUTES
        {
            return Err("Repertoire limit: 24 books / 1000 total routes".into());
        }
        let mut books = BTreeSet::new();
        let mut routes = BTreeSet::new();
        for book in &self.repertoires {
            if !valid_id(&book.id)
                || !books.insert(&book.id)
                || !valid_name(&book.name, true)
                || !valid_note(&book.note)
                || book.modified_unix < book.created_unix
            {
                return Err("Invalid/duplicate repertoire identity or metadata".into());
            }
            let mut paths = BTreeSet::new();
            for route in &book.routes {
                if !routes.insert(&route.id) || !paths.insert(&route.moves) {
                    return Err("Duplicate route ID or identical path in repertoire".into());
                }
                validate_route(route)?;
            }
        }
        Ok(())
    }
    pub fn create(&mut self, name: &str, side: Side, now: u64) -> Result<String, String> {
        if !valid_name(name, true) {
            return Err("Repertoire name requires 1..128 bytes without control characters".into());
        }
        let mut next = self.clone();
        let id = loop {
            let id = new_id("b");
            if next.get(&id).is_none() {
                break id;
            }
        };
        next.repertoires.push(Repertoire {
            id: id.clone(),
            name: name.trim().into(),
            side,
            created_unix: now,
            modified_unix: now,
            note: String::new(),
            routes: vec![],
        });
        next.validate()?;
        *self = next;
        Ok(id)
    }
    pub fn add_route(
        &mut self,
        book: &str,
        moves: Vec<String>,
        name: &str,
        note: &str,
        now: u64,
    ) -> Result<bool, String> {
        let mut next = self.clone();
        let existing = next.get(book).ok_or("Repertoire missing")?;
        // Validate before deduplicating, including metadata supplied by the caller.
        let mut route = Route {
            id: new_id("r"),
            name: name.into(),
            note: note.into(),
            moves,
        };
        validate_route(&route)?;
        if existing.routes.iter().any(|r| r.moves == route.moves) {
            return Ok(false);
        }
        while next
            .repertoires
            .iter()
            .flat_map(|b| &b.routes)
            .any(|r| r.id == route.id)
        {
            route.id = new_id("r");
        }
        let target = next.get_mut(book).ok_or("Repertoire missing")?;
        target.modified_unix = now.max(target.modified_unix);
        target.routes.push(route);
        next.validate()?;
        *self = next;
        Ok(true)
    }
    pub fn update_route(
        &mut self,
        book: &str,
        route: &str,
        moves: Vec<String>,
        name: &str,
        note: &str,
        now: u64,
    ) -> Result<bool, String> {
        let mut next = self.clone();
        let target = next.get_mut(book).ok_or("Repertoire missing")?;
        let index = target
            .routes
            .iter()
            .position(|r| r.id == route)
            .ok_or("Route missing")?;
        let updated = Route {
            id: route.into(),
            name: name.into(),
            note: note.into(),
            moves,
        };
        validate_route(&updated)?;
        if target
            .routes
            .iter()
            .enumerate()
            .any(|(i, r)| i != index && r.moves == updated.moves)
        {
            return Ok(false);
        }
        target.routes[index] = updated;
        target.modified_unix = now.max(target.modified_unix);
        next.validate()?;
        *self = next;
        Ok(true)
    }
    pub fn remove_route(&mut self, book: &str, route: &str, now: u64) -> Result<(), String> {
        let mut next = self.clone();
        let target = next.get_mut(book).ok_or("Repertoire missing")?;
        let index = target
            .routes
            .iter()
            .position(|r| r.id == route)
            .ok_or("Route missing")?;
        target.routes.remove(index);
        target.modified_unix = now.max(target.modified_unix);
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn remove_book(&mut self, id: &str) -> Result<(), String> {
        let mut next = self.clone();
        let index = next
            .repertoires
            .iter()
            .position(|b| b.id == id)
            .ok_or("Repertoire missing")?;
        next.repertoires.remove(index);
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn rename(&mut self, book: &str, name: &str, note: &str, now: u64) -> Result<(), String> {
        let mut next = self.clone();
        let target = next.get_mut(book).ok_or("Repertoire missing")?;
        target.name = name.trim().into();
        target.note = note.into();
        target.modified_unix = now.max(target.modified_unix);
        next.validate()?;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_owned).collect()
    }
    #[test]
    fn content_white_black_shared_prefix_edit_remove_and_transactional_rejection() {
        let mut c = Collection::default();
        let w = c.create("白方 Italian", Side::White, 100).unwrap();
        let b = c.create("Sicilian", Side::Black, 101).unwrap();
        assert_eq!(c.get(&b).unwrap().side.color(), Color::Black);
        c.add_route(
            &w,
            path("e2e4 e7e5 g1f3 b8c6 f1c4 f8c5"),
            "Italian",
            "共同前缀\n准备",
            102,
        )
        .unwrap();
        c.add_route(
            &w,
            path("e2e4 e7e5 g1f3 b8c6 f1c4 g8f6"),
            "Two knights",
            "",
            103,
        )
        .unwrap();
        assert!(
            !c.add_route(
                &w,
                path("e2e4 e7e5 g1f3 b8c6 f1c4 f8c5"),
                "ignored duplicate",
                "",
                104
            )
            .unwrap()
        );
        let before = c.clone();
        assert!(c.add_route(&w, path("e2e5"), "", "", 105).is_err());
        assert_eq!(c, before);
        let id = c.get(&w).unwrap().routes[0].id.clone();
        c.update_route(&w, &id, path("e2e4 e7e5 g1f3"), "shorter", "note", 106)
            .unwrap();
        assert_eq!(c.get(&w).unwrap().routes[0].id, id);
        c.remove_route(&w, &id, 107).unwrap();
        assert_eq!(c.get(&w).unwrap().routes.len(), 1);
        assert_eq!(
            c.get(&w).unwrap().routes[0].moves,
            before.get(&w).unwrap().routes[1].moves
        );
        c.rename(&w, "New name", "book note", 108).unwrap();
        assert_eq!(c.get(&w).unwrap().id, w);
        c.remove_book(&b).unwrap();
        assert_eq!(c.repertoires.len(), 1);
    }
    #[test]
    fn content_limits_fields_unknown_schema_and_standard_chess_are_strict() {
        let mut c = Collection::default();
        for i in 0..MAX_REPERTOIRES {
            c.create(&format!("Book {i}"), Side::White, 0).unwrap();
        }
        assert!(c.create("overflow", Side::Black, 0).is_err());
        let mut bad = c.clone();
        bad.schema_version = 2;
        assert!(bad.validate().is_err());
        let id = c.repertoires[0].id.clone();
        let before = c.clone();
        for (moves, name, note) in [
            (vec![], "", ""),
            (path("e2e4"), "bad\nname", ""),
            (path("e2e4"), "", "bad\0note"),
            (vec!["e2e4".into(); 81], "", ""),
        ] {
            assert!(c.add_route(&id, moves, name, note, 1).is_err());
        }
        assert_eq!(c, before);
        let r = Route {
            id: "test".into(),
            name: String::new(),
            note: String::new(),
            moves: path("e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 e1g1"),
        };
        let v = validate_route(&r).unwrap();
        assert_eq!(v.positions.len(), 8);
        assert_eq!(v.sans.last().unwrap(), "O-O");
        let mut boundary = r.clone();
        boundary.moves = path(&"g1f3 g8f6 f3g1 f6g8 ".repeat(20));
        assert_eq!(validate_route(&boundary).unwrap().moves.len(), MAX_PLIES);
        boundary.moves.push("g1f3".into());
        assert!(validate_route(&boundary).is_err());
        let mut many = Collection::default();
        let id = many.create("Total route cap", Side::White, 0).unwrap();
        many.get_mut(&id).unwrap().routes = vec![r; MAX_ROUTES + 1];
        assert!(many.validate().is_err());
    }
}

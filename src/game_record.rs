use crate::{fen, game::GameResult};
use serde::{Deserialize, Serialize};
use shakmaty::{Chess, Color, Move, Position, san::SanPlus, uci::UciMove};
use std::collections::BTreeMap;

pub const MAX_PLIES: usize = 4096;
pub const MAX_FILE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GameRecord {
    pub initial_fen: String,
    pub moves: Vec<String>,
    pub tags: BTreeMap<String, String>,
    pub result: String,
    pub created_unix: u64,
    pub omitted_annotations: bool,
}

#[derive(Clone)]
pub struct ValidatedRecord {
    pub record: GameRecord,
    pub positions: Vec<Chess>,
    pub moves: Vec<Move>,
    pub sans: Vec<String>,
}

pub fn result_token(result: Option<GameResult>) -> &'static str {
    match result {
        Some(GameResult::Checkmate { winner }) => {
            if winner == Color::White {
                "1-0"
            } else {
                "0-1"
            }
        }
        Some(GameResult::Timeout { loser, draw: false }) => {
            if loser == Color::Black {
                "1-0"
            } else {
                "0-1"
            }
        }
        None | Some(GameResult::TimeoutUnresolved { .. }) => "*",
        _ => "1/2-1/2",
    }
}

impl GameRecord {
    pub fn new(initial: &Chess, ai: Option<Color>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Date is obtained from the local Windows calendar, never a hardcoded release date.
        let human = std::env::var("USERNAME")
            .ok()
            .filter(|name| {
                !name.is_empty() && name.len() <= 128 && !name.chars().any(char::is_control)
            })
            .unwrap_or_else(|| "Human".into());
        let mut tags = BTreeMap::new();
        for (key, value) in [
            ("Event", "RUST CHESS Local Game"),
            ("Site", "Local"),
            ("Round", "?"),
            (
                "White",
                if ai == Some(Color::White) {
                    "Pleco AI"
                } else {
                    human.as_str()
                },
            ),
            (
                "Black",
                if ai == Some(Color::Black) {
                    "Pleco AI"
                } else {
                    human.as_str()
                },
            ),
        ] {
            tags.insert(key.into(), value.into());
        }
        tags.insert("Date".into(), crate::native_files::local_date());
        Self {
            initial_fen: fen::write(initial),
            moves: vec![],
            tags,
            result: "*".into(),
            created_unix: now,
            omitted_annotations: false,
        }
    }

    pub fn capture(
        initial: &Chess,
        history: &[Move],
        mut meta: Self,
        result: Option<GameResult>,
    ) -> Self {
        meta.initial_fen = fen::write(initial);
        meta.moves = history
            .iter()
            .map(|m| UciMove::from_standard(m).to_string())
            .collect();
        meta.result = result_token(result).into();
        if let Some(result) = result {
            meta.tags
                .insert("Termination".into(), format!("{result:?}"));
        } else {
            meta.tags.remove("Termination");
        }
        meta
    }

    pub fn validate(&self) -> Result<ValidatedRecord, String> {
        if self.moves.len() > MAX_PLIES
            || self.tags.len() > 128
            || self.tags.iter().any(|(k, v)| {
                k.len() > 128
                    || v.len() > 4096
                    || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    || v.chars().any(char::is_control)
            })
        {
            return Err("Record exceeds limits or has invalid tags".into());
        }
        if !["*", "1-0", "0-1", "1/2-1/2"].contains(&self.result.as_str()) {
            return Err("Invalid result token".into());
        }
        let mut pos = fen::parse(&self.initial_fen)?;
        let mut positions = vec![pos.clone()];
        let mut moves = vec![];
        let mut sans = vec![];
        for (i, uci) in self.moves.iter().enumerate() {
            let mv = uci
                .parse::<UciMove>()
                .map_err(|e| format!("Ply {}: {e}", i + 1))?
                .to_move(&pos)
                .map_err(|e| format!("Illegal move at ply {} ({uci}): {e}", i + 1))?;
            sans.push(SanPlus::from_move(pos.clone(), &mv).to_string());
            pos = pos.play(&mv).map_err(|e| format!("Ply {}: {e}", i + 1))?;
            moves.push(mv);
            positions.push(pos.clone());
        }
        if let Some(outcome) = pos.outcome() {
            let actual = match outcome {
                shakmaty::Outcome::Decisive {
                    winner: Color::White,
                } => "1-0",
                shakmaty::Outcome::Decisive {
                    winner: Color::Black,
                } => "0-1",
                shakmaty::Outcome::Draw => "1/2-1/2",
            };
            if self.result != actual {
                return Err("Result contradicts final board outcome".into());
            }
        }
        Ok(ValidatedRecord {
            record: self.clone(),
            positions,
            moves,
            sans,
        })
    }
}

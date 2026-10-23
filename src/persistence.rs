use crate::{
    fen,
    game::{ClockState, GameResult},
    game_record::{GameRecord, MAX_FILE_BYTES, ValidatedRecord, result_token},
    preferences::BoardPreferences,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shakmaty::{Color, Position};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: u32,
    pub record: GameRecord,
    pub current_fen: String,
    pub clocks: ClockState,
    pub clock_history: Vec<ClockState>,
    #[serde(with = "optional_color_serde")]
    pub ai_color: Option<Color>,
    pub ai_depth: u32,
    pub board_preferences: BoardPreferences,
    pub result: Option<GameResult>,
    pub saved_unix: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: u32,
    sha256: String,
    payload: String,
}

impl Snapshot {
    pub fn validate(&self) -> Result<ValidatedRecord, String> {
        if self.schema_version != 1 {
            return Err("Unsupported save schema".into());
        }
        let v = self.record.validate()?;
        let current = v.positions.last().expect("initial position");
        if fen::write(current) != self.current_fen
            || self.clock_history.len() != self.record.moves.len() + 1
            || !(1..=5).contains(&self.ai_depth)
        {
            return Err("Save position/history/configuration mismatch".into());
        }
        if self.record.result != result_token(self.result) {
            return Err("Save result mismatch".into());
        }
        let repetitions = v
            .positions
            .iter()
            .filter(|p| crate::game::position_key(p) == crate::game::position_key(current))
            .count();
        let board_result = GameResult::from_position(current, repetitions);
        match self.result {
            Some(GameResult::Timeout { loser, .. } | GameResult::TimeoutUnresolved { loser }) => {
                if loser != current.turn()
                    || !self.clocks.remaining(loser).is_zero()
                    || board_result.is_some()
                {
                    return Err("Invalid saved timeout".into());
                }
            }
            Some(GameResult::ThreefoldRepetition) => {
                if repetitions < 3 || board_result.is_some() {
                    return Err("Invalid saved repetition".into());
                }
            }
            other => {
                if other != board_result {
                    return Err("Saved terminal reason contradicts board".into());
                }
            }
        }
        Ok(v)
    }
}

pub fn data_path() -> Result<PathBuf, String> {
    let base = std::env::var_os("LOCALAPPDATA")
        .ok_or("LOCALAPPDATA is unavailable; cannot select writable user storage")?;
    Ok(PathBuf::from(base)
        .join("RUST CHESS")
        .join("autosave-v1.json"))
}

pub fn read_limited(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = vec![];
    File::open(path)
        .map_err(|e| format!("Open {}: {e}", path.display()))?
        .take(MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read {}: {e}", path.display()))?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("File exceeds 4 MiB".into());
    }
    Ok(bytes)
}

pub fn decode(bytes: &[u8]) -> Result<Snapshot, String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("Save exceeds 4 MiB".into());
    }
    let envelope: Envelope =
        serde_json::from_slice(bytes).map_err(|e| format!("Save envelope: {e}"))?;
    if envelope.schema_version != 1 {
        return Err("Unsupported envelope schema".into());
    }
    let digest = format!("{:x}", Sha256::digest(envelope.payload.as_bytes()));
    if digest != envelope.sha256 {
        return Err("Save checksum mismatch".into());
    }
    let snapshot: Snapshot =
        serde_json::from_str(&envelope.payload).map_err(|e| format!("Save payload: {e}"))?;
    snapshot.validate()?;
    Ok(snapshot)
}

pub fn load(path: &Path) -> Result<Option<Snapshot>, String> {
    match fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Save metadata: {e}")),
        Ok(_) => decode(&read_limited(path)?).map(Some),
    }
}

pub fn save(path: &Path, snapshot: &Snapshot) -> Result<(), String> {
    snapshot.validate()?;
    // Preserve a corrupt or unknown-format primary rather than overwrite it.
    if let Some(previous) = load(path)? {
        let _ = previous;
        atomic_write(&path.with_extension("json.bak"), &read_limited(path)?)?;
    }
    let payload =
        serde_json::to_string(snapshot).map_err(|e| format!("Save serialization: {e}"))?;
    let envelope = Envelope {
        schema_version: 1,
        sha256: format!("{:x}", Sha256::digest(payload.as_bytes())),
        payload,
    };
    let bytes =
        serde_json::to_vec_pretty(&envelope).map_err(|e| format!("Save serialization: {e}"))?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("Save exceeds 4 MiB".into());
    }
    atomic_write(path, &bytes)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("File requires parent directory")?;
    fs::create_dir_all(parent).map_err(|e| format!("Create {}: {e}", parent.display()))?;
    let tmp = parent.join(format!(
        ".rust-chess-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .map_err(|e| format!("Create temporary file: {e}"))?;
    let result = (|| {
        file.write_all(bytes)
            .map_err(|e| format!("Write temporary file: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Flush temporary file: {e}"))?;
        drop(file);
        crate::native_files::replace_file(&tmp, path)
    })();
    // On failure leave the temporary file intact for recovery; never remove a user's file.
    result
}

pub mod color_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use shakmaty::Color;
    pub fn serialize<S: Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(if *color == Color::White {
            "white"
        } else {
            "black"
        })
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
        match String::deserialize(d)?.as_str() {
            "white" => Ok(Color::White),
            "black" => Ok(Color::Black),
            _ => Err(serde::de::Error::custom("Invalid color")),
        }
    }
}
mod optional_color_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use shakmaty::Color;
    pub fn serialize<S: Serializer>(color: &Option<Color>, s: S) -> Result<S::Ok, S::Error> {
        match color {
            None => s.serialize_none(),
            Some(Color::White) => s.serialize_some("white"),
            Some(Color::Black) => s.serialize_some("black"),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Color>, D::Error> {
        match Option::<String>::deserialize(d)?.as_deref() {
            None => Ok(None),
            Some("white") => Ok(Some(Color::White)),
            Some("black") => Ok(Some(Color::Black)),
            _ => Err(serde::de::Error::custom("Invalid AI color")),
        }
    }
}

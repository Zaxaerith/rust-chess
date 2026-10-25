//! Small content-addressed cache, separate from autosave and PGN. No eviction.
use crate::{
    game_record::ValidatedRecord,
    game_review::{Job, Options, Point, State},
    persistence, review_scoring,
    uci::{Config, Report},
    uci_protocol,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub const MAX_BYTES: usize = 1024 * 1024;
pub const MAX_FILES: usize = 32;
pub fn game_key(game: &ValidatedRecord) -> String {
    digest(
        &serde_json::to_vec(&(
            crate::fen::write(&game.positions[0]),
            &game.record.moves,
            &game.record.result,
        ))
        .expect("record identity"),
    )
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn key(game: &str, config: &Config, engine: &Report, options: Options) -> String {
    let mut config = config.clone();
    config.multipv = 1;
    config.movetime = options.movetime;
    digest(
        &serde_json::to_vec(&(
            1,
            review_scoring::ALGORITHM,
            game,
            config,
            &engine.engine_key,
            &engine.name,
            &engine.author,
            options,
        ))
        .expect("cache identity"),
    )
}
pub fn path(base: &Path, key: &str) -> PathBuf {
    base.join("review-cache-v1")
        .join(format!("{key}.review.json"))
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Data {
    schema: u32,
    algorithm: u32,
    key: String,
    game: String,
    engine: String,
    options: Options,
    state: State,
    points: Vec<Option<Point>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    sha256: String,
    payload: String,
}
fn decode(bytes: &[u8], job: &Job) -> Result<Data, String> {
    if bytes.len() > MAX_BYTES {
        return Err("Review cache exceeds 1 MiB".into());
    }
    let e: Envelope =
        serde_json::from_slice(bytes).map_err(|e| format!("Review cache envelope: {e}"))?;
    if digest(e.payload.as_bytes()) != e.sha256 {
        return Err("Review cache checksum mismatch".into());
    }
    let data: Data =
        serde_json::from_str(&e.payload).map_err(|e| format!("Review cache data: {e}"))?;
    if data.schema != 1
        || data.algorithm != review_scoring::ALGORITHM
        || data.key != job.cache_key
        || data.game != job.game_key
        || data.engine != job.engine
        || data.options != job.options
        || data.points.len() != job.points.len()
    {
        return Err("Review cache identity/schema mismatch".into());
    }
    for (i, point) in data.points.iter().enumerate() {
        let Some(p) = point else {
            continue;
        };
        if i > job.limit || p.fen != crate::fen::write(&job.game.positions[i]) {
            return Err("Review cache position mismatch".into());
        }
        if p.terminal.is_some() {
            if job.terminal(i).as_ref() != Some(p) {
                return Err("Review cache terminal mismatch".into());
            }
        } else {
            if p.score.is_none() {
                return Err("Review cache missing final score".into());
            }
            if job.terminal(i).is_some() {
                return Err("Review cache expected a terminal result".into());
            }
            let best = p.best.as_ref().ok_or("Review cache missing bestmove")?;
            if uci_protocol::legal_move(&job.game.positions[i], best)? != p.best_san {
                return Err("Review cache SAN mismatch".into());
            }
        }
    }
    if data.state == State::Completed && data.points[..=job.limit].iter().any(Option::is_none) {
        return Err("Review cache incomplete completion marker".into());
    }
    Ok(data)
}
pub fn load(path: &Path, job: &mut Job) -> Result<bool, String> {
    match fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
        Ok(m) if m.len() > MAX_BYTES as u64 => return Err("Review cache exceeds 1 MiB".into()),
        _ => {}
    }
    let data = decode(&persistence::read_limited(path)?, job)?;
    job.points = data.points;
    job.refresh();
    Ok(true)
}
pub fn save(path: &Path, job: &Job) -> Result<(), String> {
    // Corrupt cache may still have recovery value. Never silently replace it.
    if path.exists() {
        decode(&persistence::read_limited(path)?, job)?;
    } else if let Some(parent) = path.parent()
        && parent.exists()
    {
        let count = fs::read_dir(parent)
            .map_err(|e| e.to_string())?
            .try_fold(0usize, |n, e| {
                e.map(|v| {
                    n + usize::from(v.file_name().to_string_lossy().ends_with(".review.json"))
                })
            })
            .map_err(|e| e.to_string())?;
        if count >= MAX_FILES {
            return Err(
                "Review cache full (32 files); move old review cache files manually".into(),
            );
        }
    }
    let payload = serde_json::to_string(&Data {
        schema: 1,
        algorithm: review_scoring::ALGORITHM,
        key: job.cache_key.clone(),
        game: job.game_key.clone(),
        engine: job.engine.clone(),
        options: job.options,
        state: job.state,
        points: job.points.clone(),
    })
    .map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&Envelope {
        sha256: digest(payload.as_bytes()),
        payload,
    })
    .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("Review cache exceeds 1 MiB".into());
    }
    persistence::atomic_write(path, &bytes)
}

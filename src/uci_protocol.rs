//! Bounded, tolerant parsing of UCI output. Rules remain in shakmaty.
use shakmaty::{Chess, Color, Position, san::SanPlus, uci::UciMove};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Score {
    Cp(i32),
    Mate(i32),
    /// White-view mate zero needs an explicit mated side (integer -0 loses it).
    Mated(Color),
}
pub const MAX_PV_PLIES: usize = 128;
pub const MAX_MULTIPV: usize = 3;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub rank: usize,
    pub depth: Option<u32>,
    pub score: Option<(Score, Bound)>,
    pub pv: Option<Vec<String>>,
    pub nodes: Option<u64>,
    pub time: Option<u64>,
}
impl Default for Info {
    fn default() -> Self {
        Self {
            rank: 1,
            depth: None,
            score: None,
            pv: None,
            nodes: None,
            time: None,
        }
    }
}
/// Partial messages may merge only within an explicitly identified depth/rank.
/// A new or unspecified depth cannot inherit an older evaluation/PV pair.
pub fn update_line(slot: &mut Option<Info>, incoming: Info) {
    if let Some(old) = slot.as_mut() {
        if let (Some(a), Some(b)) = (old.depth, incoming.depth) {
            if b < a {
                return;
            }
            if a == b {
                if incoming.score.is_some() {
                    old.score = incoming.score;
                }
                if incoming.pv.is_some() {
                    old.pv = incoming.pv;
                }
                if incoming.nodes.is_some() {
                    old.nodes = incoming.nodes;
                }
                if incoming.time.is_some() {
                    old.time = incoming.time;
                }
                return;
            }
        } else if incoming.depth.is_none() && incoming.score.is_none() && incoming.pv.is_none() {
            if incoming.nodes.is_some() {
                old.nodes = incoming.nodes;
            }
            if incoming.time.is_some() {
                old.time = incoming.time;
            }
            return;
        }
    }
    *slot = Some(incoming);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Variation {
    pub san: String,
    pub arrow: Option<(shakmaty::Square, shakmaty::Square)>,
    pub error: Option<String>,
}
/// Replay on a temporary position. Keep the valid prefix and the exact failure.
pub fn variation(initial: &Chess, pv: &[String]) -> Variation {
    let mut pos = initial.clone();
    let mut result = Variation::default();
    for (i, text) in pv.iter().enumerate().take(MAX_PV_PLIES) {
        let mv = match text
            .parse::<UciMove>()
            .ok()
            .and_then(|m| m.to_move(&pos).ok())
        {
            Some(mv) => mv,
            None => {
                result.error = Some(format!("PV ply {}: Illegal UCI move: {text}", i + 1));
                result.arrow = None;
                break;
            }
        };
        if i == 0 {
            result.arrow = move_arrow(&mv);
        }
        if !result.san.is_empty() {
            result.san.push(' ');
        }
        if pos.turn() == Color::White {
            result.san.push_str(&format!("{}. ", pos.fullmoves()));
        } else if i == 0 {
            result.san.push_str(&format!("{}... ", pos.fullmoves()));
        }
        result
            .san
            .push_str(&SanPlus::from_move(pos.clone(), &mv).to_string());
        pos.play_unchecked(&mv);
    }
    if pv.len() > MAX_PV_PLIES {
        result.error = Some("PV exceeds 128 plies".into());
        result.arrow = None;
    }
    result
}
pub fn move_arrow(mv: &shakmaty::Move) -> Option<(shakmaty::Square, shakmaty::Square)> {
    use shakmaty::{File, Move, Square};
    match mv {
        Move::Castle { king, rook } => Some((
            *king,
            Square::from_coords(
                if rook.file() > king.file() {
                    File::G
                } else {
                    File::C
                },
                king.rank(),
            ),
        )),
        _ => mv.from().map(|from| (from, mv.to())),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineOption {
    pub name: String,
    pub kind: String,
    pub min: Option<i64>,
    pub max: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Name(String),
    Author(String),
    Option(EngineOption),
    UciOk,
    ReadyOk,
    Info(Info),
    Best(String),
    Other,
}
pub fn parse(line: &str) -> Result<Message, String> {
    if line.len() > 16384 {
        return Err("UCI output line exceeds 16 KiB".into());
    }
    let t: Vec<_> = line.split_whitespace().collect();
    Ok(match t.as_slice() {
        ["uciok"] => Message::UciOk,
        ["readyok"] => Message::ReadyOk,
        ["id", "name", rest @ ..] => Message::Name(rest.join(" ").chars().take(160).collect()),
        ["id", "author", rest @ ..] => Message::Author(rest.join(" ").chars().take(160).collect()),
        ["option", "name", rest @ ..] => {
            if let Some(i) = rest.iter().position(|v| *v == "type") {
                let number = |key| {
                    rest.windows(2)
                        .find(|w| w[0] == key)
                        .and_then(|w| w[1].parse().ok())
                };
                Message::Option(EngineOption {
                    name: rest[..i].join(" "),
                    kind: rest.get(i + 1).unwrap_or(&"").to_string(),
                    min: number("min"),
                    max: number("max"),
                })
            } else {
                Message::Other
            }
        }
        ["bestmove", mv, ..] => Message::Best((*mv).into()),
        ["bestmove"] => return Err("Missing UCI bestmove value".into()),
        ["info", rest @ ..] => {
            let mut info = Info::default();
            let mut i = 0;
            while i < rest.len() {
                match rest[i] {
                    "string" => break,
                    "pv" => {
                        let start = i + 1;
                        i = start;
                        // Standard PV consumes the remaining moves; tolerate recognized
                        // info fields after it as well. Malformed moves reach validation.
                        while i < rest.len()
                            && !matches!(
                                rest[i],
                                "depth"
                                    | "seldepth"
                                    | "multipv"
                                    | "score"
                                    | "nodes"
                                    | "time"
                                    | "nps"
                                    | "hashfull"
                                    | "tbhits"
                                    | "currmove"
                                    | "currmovenumber"
                                    | "string"
                                    | "wdl"
                                    | "cpuload"
                            )
                        {
                            i += 1;
                        }
                        if i == start {
                            return Err("Empty UCI PV".into());
                        }
                        if i - start > MAX_PV_PLIES {
                            return Err("PV exceeds 128 plies".into());
                        }
                        info.pv = Some(rest[start..i].iter().map(|s| (*s).to_owned()).collect());
                        continue;
                    }
                    "depth" => {
                        info.depth = rest.get(i + 1).and_then(|v| v.parse().ok());
                        i += 1;
                    }
                    "multipv" => {
                        info.rank = rest
                            .get(i + 1)
                            .and_then(|v| v.parse().ok())
                            .filter(|n| (1..=MAX_MULTIPV).contains(n))
                            .ok_or("Invalid MultiPV rank (expected 1..3)")?;
                        i += 1;
                    }
                    "nodes" => {
                        info.nodes = rest.get(i + 1).and_then(|v| v.parse().ok());
                        i += 1;
                    }
                    "time" => {
                        info.time = rest.get(i + 1).and_then(|v| v.parse().ok());
                        i += 1;
                    }
                    "score" => {
                        let n: i32 = rest
                            .get(i + 2)
                            .ok_or("Missing UCI score")?
                            .parse()
                            .map_err(|_| "Invalid UCI score")?;
                        if n == i32::MIN {
                            return Err("UCI score out of range".into());
                        }
                        let score = match rest.get(i + 1) {
                            Some(&"cp") => Score::Cp(n),
                            Some(&"mate") => Score::Mate(n),
                            _ => return Err("Unknown UCI score type".into()),
                        };
                        let bound = match rest.get(i + 3) {
                            Some(&"lowerbound") => Bound::Lower,
                            Some(&"upperbound") => Bound::Upper,
                            _ => Bound::Exact,
                        };
                        info.score = Some((score, bound));
                        i += 2;
                    }
                    _ => {}
                }
                i += 1;
            }
            Message::Info(info)
        }
        _ => Message::Other,
    })
}
pub fn white_score(score: (Score, Bound), turn: Color) -> (Score, Bound) {
    if score.0 == Score::Mate(0) {
        return (
            Score::Mated(turn),
            if turn == Color::Black {
                match score.1 {
                    Bound::Lower => Bound::Upper,
                    Bound::Upper => Bound::Lower,
                    Bound::Exact => Bound::Exact,
                }
            } else {
                score.1
            },
        );
    }
    if turn == Color::White {
        return score;
    }
    let value = match score.0 {
        Score::Cp(v) => Score::Cp(-v),
        Score::Mate(v) => Score::Mate(-v),
        Score::Mated(side) => Score::Mated(side),
    };
    (
        value,
        match score.1 {
            Bound::Exact => Bound::Exact,
            Bound::Lower => Bound::Upper,
            Bound::Upper => Bound::Lower,
        },
    )
}
pub fn legal_move(pos: &Chess, text: &str) -> Result<Option<String>, String> {
    if text == "0000" || text == "(none)" {
        return if pos.legal_moves().is_empty() {
            Ok(None)
        } else {
            Err("Engine returned no move in a playable position".into())
        };
    }
    let mv = text
        .parse::<UciMove>()
        .map_err(|_| format!("Invalid UCI move: {text}"))?
        .to_move(pos)
        .map_err(|_| format!("Illegal UCI move: {text}"))?;
    Ok(Some(SanPlus::from_move(pos.clone(), &mv).to_string()))
}
pub fn position(initial: &Chess, moves: &[shakmaty::Move]) -> String {
    let initial_fen = crate::fen::write(initial);
    let mut value = if initial_fen == crate::fen::write(&Chess::default()) {
        "position startpos".into()
    } else {
        format!("position fen {initial_fen}")
    };
    if !moves.is_empty() {
        value.push_str(" moves");
        for mv in moves {
            value.push(' ');
            value.push_str(&UciMove::from_move(mv, shakmaty::CastlingMode::Standard).to_string());
        }
    }
    value
}

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
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Info {
    pub depth: Option<u32>,
    pub score: Option<(Score, Bound)>,
    pub pv: Option<String>,
    pub nodes: Option<u64>,
    pub time: Option<u64>,
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
            let mut multipv = 1u32;
            while i < rest.len() {
                match rest[i] {
                    "string" => break,
                    "pv" => {
                        info.pv = rest.get(i + 1).map(|v| (*v).into());
                        break;
                    }
                    "depth" => {
                        info.depth = rest.get(i + 1).and_then(|v| v.parse().ok());
                        i += 1;
                    }
                    "multipv" => {
                        multipv = rest.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(0);
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
            if multipv != 1 {
                Message::Other
            } else {
                Message::Info(info)
            }
        }
        _ => Message::Other,
    })
}
pub fn white_score(score: (Score, Bound), turn: Color) -> (Score, Bound) {
    if turn == Color::White {
        return score;
    }
    let value = match score.0 {
        Score::Cp(v) => Score::Cp(-v),
        Score::Mate(v) => Score::Mate(-v),
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

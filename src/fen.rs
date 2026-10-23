use shakmaty::{CastlingMode, Chess, EnPassantMode, fen::Fen};

pub fn parse(text: &str) -> Result<Chess, String> {
    let text = text.trim();
    if text.len() > 256 || text.split_whitespace().count() != 6 {
        return Err("FEN requires exactly six fields (maximum 256 bytes)".into());
    }
    let fields: Vec<_> = text.split_whitespace().collect();
    if fields[4].parse::<u32>().is_err()
        || fields[5].parse::<u32>().is_err()
        || fields[5].parse::<u32>().ok().is_none_or(|n| n == 0)
    {
        return Err("Invalid FEN move counters".into());
    }
    text.parse::<Fen>()
        .map_err(|e| format!("Invalid FEN: {e}"))?
        .into_position(CastlingMode::Standard)
        .map_err(|e| format!("Illegal standard chess position: {e}"))
}

pub fn write(pos: &Chess) -> String {
    Fen::from_position(pos.clone(), EnPassantMode::Always).to_string()
}

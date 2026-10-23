use crate::{
    fen,
    game_record::{GameRecord, MAX_FILE_BYTES, MAX_PLIES, ValidatedRecord},
};
use pgn_reader::{BufferedReader, Nag, RawComment, RawHeader, SanPlus, Skip, Visitor};
use shakmaty::{Chess, Color, Outcome, Position};

pub fn export(record: &GameRecord) -> Result<String, String> {
    let validated = record.validate()?;
    let mut tags = record.tags.clone();
    tags.insert("Result".into(), record.result.clone());
    for (k, v) in [
        ("Event", "?"),
        ("Site", "?"),
        ("Date", "????.??.??"),
        ("Round", "?"),
        ("White", "?"),
        ("Black", "?"),
    ] {
        tags.entry(k.into()).or_insert_with(|| v.into());
    }
    if record.initial_fen != fen::write(&Chess::default()) {
        tags.insert("SetUp".into(), "1".into());
        tags.insert("FEN".into(), record.initial_fen.clone());
    } else {
        tags.remove("SetUp");
        tags.remove("FEN");
    }
    let escape = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let mut out = String::new();
    for key in ["Event", "Site", "Date", "Round", "White", "Black", "Result"] {
        out.push_str(&format!(
            "[{key} \"{}\"]\n",
            escape(&tags.remove(key).expect("required tag"))
        ));
    }
    for (k, v) in tags {
        out.push_str(&format!("[{k} \"{}\"]\n", escape(&v)));
    }
    out.push('\n');
    for (i, san) in validated.sans.iter().enumerate() {
        let pos = &validated.positions[i];
        if pos.turn() == Color::White {
            out.push_str(&format!("{}. ", pos.fullmoves()));
        } else if i == 0 {
            out.push_str(&format!("{}... ", pos.fullmoves()));
        }
        out.push_str(san);
        out.push(' ');
        if i % 12 == 11 {
            out.push('\n');
        }
    }
    out.push_str(&record.result);
    out.push('\n');
    Ok(out)
}

struct Importer {
    record: GameRecord,
    pos: Chess,
    error: Option<String>,
    outcome_seen: bool,
}
impl Visitor for Importer {
    type Result = Result<GameRecord, String>;
    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        let pair = std::str::from_utf8(key)
            .and_then(|k| value.decode_utf8().map(|v| (k.to_owned(), v.into_owned())));
        match pair {
            Ok((k, v)) => {
                if self.record.tags.insert(k, v).is_some() {
                    self.error = Some("Duplicate PGN tag".into());
                }
            }
            Err(_) => self.error = Some("PGN is not UTF-8".into()),
        }
    }
    fn end_headers(&mut self) -> Skip {
        if self
            .record
            .tags
            .get("SetUp")
            .is_some_and(|v| v != "0" && v != "1")
        {
            self.error = Some("Invalid SetUp tag".into());
        }
        if self.record.tags.get("SetUp").is_some_and(|v| v == "1")
            && !self.record.tags.contains_key("FEN")
        {
            self.error = Some("SetUp=1 requires FEN".into());
        }
        if self
            .record
            .tags
            .get("Variant")
            .is_some_and(|v| !["Standard", "Chess", "From Position"].contains(&v.as_str()))
        {
            self.error = Some("Only standard chess PGN is supported".into());
        }
        if self.record.tags.get("SetUp").is_some_and(|v| v == "0")
            && self.record.tags.contains_key("FEN")
        {
            self.error = Some("SetUp=0 contradicts FEN tag".into());
        }
        if let Some(fen) = self.record.tags.get("FEN") {
            match fen::parse(fen) {
                Ok(pos) => {
                    self.pos = pos;
                    self.record.initial_fen = fen::write(&self.pos);
                }
                Err(e) => self.error = Some(e),
            }
        }
        Skip(false)
    }
    fn san(&mut self, san: SanPlus) {
        if self.error.is_some() {
            return;
        }
        if self.record.moves.len() >= MAX_PLIES {
            self.error = Some("PGN exceeds 4096 plies".into());
            return;
        }
        match san.san.to_move(&self.pos) {
            Ok(mv) => {
                let canonical = SanPlus::from_move(self.pos.clone(), &mv);
                if san.suffix.is_some() && canonical.suffix != san.suffix {
                    self.error = Some(format!(
                        "Incorrect check/mate suffix at ply {}",
                        self.record.moves.len() + 1
                    ));
                    return;
                }
                self.record
                    .moves
                    .push(shakmaty::uci::UciMove::from_standard(&mv).to_string());
                self.pos.play_unchecked(&mv);
            }
            Err(e) => {
                self.error = Some(format!(
                    "Illegal SAN at ply {} ({san}): {e}",
                    self.record.moves.len() + 1
                ))
            }
        }
    }
    fn begin_variation(&mut self) -> Skip {
        self.record.omitted_annotations = true;
        Skip(true)
    }
    fn comment(&mut self, _: RawComment<'_>) {
        self.record.omitted_annotations = true;
    }
    fn nag(&mut self, _: Nag) {
        self.record.omitted_annotations = true;
    }
    fn outcome(&mut self, result: Option<Outcome>) {
        self.outcome_seen = true;
        self.record.result = match result {
            None => "*",
            Some(Outcome::Draw) => "1/2-1/2",
            Some(Outcome::Decisive {
                winner: Color::White,
            }) => "1-0",
            _ => "0-1",
        }
        .into();
    }
    fn end_game(&mut self) -> Self::Result {
        if let Some(e) = &self.error {
            return Err(e.clone());
        }
        if !self.outcome_seen {
            return Err("Missing PGN movetext result".into());
        }
        if self
            .record
            .tags
            .get("Result")
            .is_some_and(|v| v != &self.record.result)
        {
            return Err("Header and movetext results differ".into());
        }
        Ok(self.record.clone())
    }
}

// pgn-reader is deliberately tolerant. Reject malformed/unknown tokens before
// visiting so corrupted text cannot silently turn into a truncated mainline.
fn preflight(text: &str) -> Result<(), String> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut depth = 0;
    let mut mainline_plies = 0;
    while i < b.len() {
        match b[i] {
            c if c.is_ascii_whitespace() => i += 1,
            b';' | b'%' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'{' => {
                i += 1;
                while i < b.len() && b[i] != b'}' {
                    i += 1;
                }
                if i == b.len() {
                    return Err("Unclosed PGN comment".into());
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                if depth > 64 {
                    return Err("PGN variation nesting limit".into());
                }
                i += 1;
            }
            b')' => {
                if depth == 0 {
                    return Err("Unmatched variation delimiter".into());
                }
                depth -= 1;
                i += 1;
            }
            b'[' => {
                i += 1;
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                if i == start || i - start > 128 {
                    return Err("Invalid PGN tag name".into());
                }
                while i < b.len() && b[i].is_ascii_whitespace() {
                    i += 1;
                }
                if b.get(i) != Some(&b'"') {
                    return Err("Invalid PGN tag value".into());
                }
                i += 1;
                let value_start = i;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i >= b.len() || i - value_start > 4096 {
                    return Err("Unclosed or oversized PGN tag".into());
                }
                i += 1;
                while i < b.len() && b[i].is_ascii_whitespace() {
                    i += 1;
                }
                if b.get(i) != Some(&b']') {
                    return Err("Missing PGN tag delimiter".into());
                }
                i += 1;
            }
            b'$' => {
                i += 1;
                let start = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                if start == i {
                    return Err("Invalid NAG".into());
                }
            }
            _ => {
                let start = i;
                while i < b.len() && !b[i].is_ascii_whitespace() && !b"{}()[];$".contains(&b[i]) {
                    i += 1;
                }
                if start == i {
                    return Err("Invalid PGN delimiter".into());
                }
                let token = &text[start..i];
                if ["*", "1-0", "0-1", "1/2-1/2"].contains(&token) {
                    continue;
                }
                if token.as_bytes()[0].is_ascii_digit()
                    && !token
                        .trim_start_matches(|c: char| c.is_ascii_digit())
                        .starts_with('.')
                {
                    return Err(format!(
                        "Invalid move number near ply {}",
                        mainline_plies + 1
                    ));
                }
                let token = token
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('.')
                    .trim_end_matches(['!', '?']);
                if !token.is_empty() && SanPlus::from_ascii(token.as_bytes()).is_err() {
                    return Err(format!(
                        "Invalid PGN token near ply {}: {token}",
                        mainline_plies + 1
                    ));
                }
                if !token.is_empty() && depth == 0 {
                    mainline_plies += 1;
                    if mainline_plies > MAX_PLIES {
                        return Err("PGN exceeds 4096 plies".into());
                    }
                }
            }
        }
    }
    if depth != 0 {
        return Err("Unclosed PGN variation".into());
    }
    Ok(())
}

pub fn import(bytes: &[u8]) -> Result<ValidatedRecord, String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("PGN exceeds 4 MiB".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|e| format!("PGN UTF-8: {e}"))?
        .trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Err("Empty PGN".into());
    }
    preflight(text)?;
    let mut importer = Importer {
        record: GameRecord::new(&Chess::default(), None),
        pos: Chess::default(),
        error: None,
        outcome_seen: false,
    };
    importer.record.tags.clear();
    let mut reader = BufferedReader::new_cursor(text.as_bytes());
    let record = reader
        .read_game(&mut importer)
        .map_err(|e| format!("PGN read: {e}"))?
        .ok_or("No PGN game")??;
    if reader
        .read_game(&mut importer)
        .map_err(|e| format!("PGN read: {e}"))?
        .is_some()
    {
        return Err("Multiple PGN games are not supported".into());
    }
    record.validate()
}

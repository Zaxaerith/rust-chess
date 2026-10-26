use crate::{
    fen,
    game_record::{GameRecord, MAX_FILE_BYTES, MAX_PLIES},
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

struct Headers {
    record: GameRecord,
    pos: Chess,
    error: Option<String>,
}
impl Visitor for Headers {
    type Result = ();
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
    fn end_game(&mut self) {}
}

use crate::annotated_game::{self as tree, AnnotatedGame, Node};
struct Frame {
    resume: usize,
    moved: bool,
    before: Vec<String>,
    ended: bool,
}
struct TreeImporter {
    headers: Headers,
    nodes: Vec<Node>,
    current: usize,
    frames: Vec<Frame>,
    error: Option<String>,
    outcome: bool,
    comment_bytes: usize,
    comment_count: usize,
    nags: usize,
}
impl TreeImporter {
    fn fail(&mut self, e: impl Into<String>) {
        if self.error.is_none() {
            self.error = Some(e.into());
        }
    }
}
impl Visitor for TreeImporter {
    type Result = Result<AnnotatedGame, String>;
    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        self.headers.header(key, value);
    }
    fn end_headers(&mut self) -> Skip {
        let _ = self.headers.end_headers();
        self.nodes = vec![Node::root(self.headers.pos.clone())];
        if let Some(e) = self.headers.error.clone() {
            self.fail(e);
        }
        Skip(false)
    }
    fn san(&mut self, san: SanPlus) {
        if self.error.is_some() {
            return;
        }
        if self.outcome || self.frames.last().is_some_and(|f| f.ended) {
            self.fail("SAN after PGN result");
            return;
        }
        let parent = self.current;
        let before = &self.nodes[parent].position;
        if self.nodes.len() > tree::MAX_NODES || self.nodes[parent].ply >= MAX_PLIES {
            self.fail("PGN node/path limit (8192 nodes / 4096 plies)");
            return;
        }
        let mv = match san.san.to_move(before) {
            Ok(m) => m,
            Err(e) => {
                self.fail(format!(
                    "Illegal SAN in variation depth {} at ply {} ({san}): {e}",
                    self.frames.len(),
                    self.nodes[parent].ply + 1
                ));
                return;
            }
        };
        let canonical = SanPlus::from_move(before.clone(), &mv);
        if san.suffix.is_some() && san.suffix != canonical.suffix {
            self.fail("Incorrect PGN check/mate suffix");
            return;
        }
        let mut position = before.clone();
        position.play_unchecked(&mv);
        let prefix = self
            .frames
            .last_mut()
            .map(|f| {
                f.moved = true;
                std::mem::take(&mut f.before)
            })
            .unwrap_or_default();
        let id = self.nodes.len();
        let uci = shakmaty::uci::UciMove::from_standard(&mv).to_string();
        self.nodes.push(Node {
            id,
            parent: Some(parent),
            children: vec![],
            mv: Some(mv),
            uci: Some(uci.clone()),
            san: canonical.to_string(),
            position,
            ply: self.nodes[parent].ply + 1,
            before: prefix,
            after: vec![],
            nags: vec![],
            branch_result: None,
        });
        self.nodes[parent].children.push(id);
        self.current = id;
        if self.frames.is_empty() {
            self.headers.record.moves.push(uci);
        }
    }
    fn begin_variation(&mut self) -> Skip {
        if self.error.is_some() {
            return Skip(false);
        }
        if self.frames.len() >= tree::MAX_DEPTH
            || self.current == 0
            || self.outcome
            || self.frames.last().is_some_and(|f| !f.moved || f.ended)
        {
            self.fail("Invalid or too deeply nested PGN variation");
            return Skip(false);
        }
        let resume = self.current;
        self.current = self.nodes[resume].parent.expect("move parent");
        self.frames.push(Frame {
            resume,
            moved: false,
            before: vec![],
            ended: false,
        });
        Skip(false)
    }
    fn end_variation(&mut self) {
        if self.error.is_some() {
            return;
        }
        match self.frames.pop() {
            Some(f) if f.moved => self.current = f.resume,
            _ => self.fail("Empty/unmatched PGN variation"),
        }
    }
    fn comment(&mut self, c: RawComment<'_>) {
        if self.error.is_some() {
            return;
        }
        let value = match std::str::from_utf8(c.as_bytes()) {
            Ok(s) => s.to_owned(),
            Err(_) => {
                self.fail("PGN comment is not UTF-8");
                return;
            }
        };
        if let Err(e) = tree::validate_comment(&value) {
            self.fail(e);
            return;
        }
        self.comment_bytes += value.len();
        self.comment_count += 1;
        if self.comment_count > tree::MAX_COMMENTS
            || self.comment_bytes > tree::MAX_TOTAL_COMMENT_BYTES
        {
            self.fail("PGN total comments exceed 1 MiB");
            return;
        }
        if let Some(f) = self.frames.last_mut()
            && !f.moved
        {
            f.before.push(value);
        } else if self.current == 0 {
            self.nodes[0].before.push(value);
        } else {
            self.nodes[self.current].after.push(value);
        }
    }
    fn nag(&mut self, n: Nag) {
        if self.error.is_some() {
            return;
        }
        if self.current == 0
            || self.frames.last().is_some_and(|f| !f.moved || f.ended)
            || self.outcome
        {
            self.fail("NAG without a current move");
            return;
        }
        self.nags += 1;
        if self.nodes[self.current].nags.len() >= tree::MAX_NAGS_PER_NODE
            || self.nags > tree::MAX_TOTAL_NAGS
        {
            self.fail("PGN NAG limit");
            return;
        }
        self.nodes[self.current].nags.push(n.0);
    }
    fn outcome(&mut self, result: Option<Outcome>) {
        if self.error.is_some() {
            return;
        }
        let token = match result {
            None => "*",
            Some(Outcome::Draw) => "1/2-1/2",
            Some(Outcome::Decisive {
                winner: Color::White,
            }) => "1-0",
            _ => "0-1",
        };
        if let Some(f) = self.frames.last_mut() {
            if !f.moved || f.ended {
                self.fail("Invalid variation result");
                return;
            }
            f.ended = true;
            self.nodes[self.current].branch_result = Some(token.into());
        } else {
            if self.outcome {
                self.fail("Multiple PGN games/results are not supported");
                return;
            }
            self.outcome = true;
            self.headers.record.result = token.into();
        }
    }
    fn end_game(&mut self) -> Self::Result {
        if let Some(e) = &self.error {
            return Err(e.clone());
        }
        if !self.outcome || !self.frames.is_empty() {
            return Err("Missing result or unclosed PGN variation".into());
        }
        let r = &mut self.headers.record;
        if r.tags.get("Result").is_some_and(|v| v != &r.result) {
            return Err("Header and movetext results differ".into());
        }
        for (k, v) in [
            ("Event", "?"),
            ("Site", "?"),
            ("Date", "????.??.??"),
            ("Round", "?"),
            ("White", "?"),
            ("Black", "?"),
        ] {
            r.tags.entry(k.into()).or_insert_with(|| v.into());
        }
        r.tags.insert("Result".into(), r.result.clone());
        if r.initial_fen == fen::write(&Chess::default()) {
            r.tags.remove("SetUp");
            r.tags.remove("FEN");
        } else {
            r.tags.insert("SetUp".into(), "1".into());
            r.tags.insert("FEN".into(), r.initial_fen.clone());
        }
        r.omitted_annotations = false;
        let game = AnnotatedGame {
            mainline: r.validate()?,
            nodes: std::mem::take(&mut self.nodes),
        };
        game.validate()?;
        Ok(game)
    }
}

// Reader 0.26 skips semicolon comments and unknown tokens. Validate and
// normalize the lexical envelope first; SAN legality still belongs to shakmaty.
fn prepare(text: &str) -> Result<String, String> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut depth = 0;
    let mut moves = 0;
    let mut total = 0;
    let mut comments = 0;
    let mut comment_count = 0;
    let mut in_moves = false;
    let mut ended = false;
    let mut out = String::new();
    let mut headers = String::new();
    while i < b.len() {
        match b[i] {
            c if c.is_ascii_whitespace() => {
                out.push(' ');
                i += 1;
            }
            b'%' => return Err("PGN percent escape extensions are unsupported".into()),
            b'{' | b';' => {
                let brace = b[i] == b'{';
                i += 1;
                let start = i;
                while i < b.len() && if brace { b[i] != b'}' } else { b[i] != b'\n' } {
                    i += 1;
                }
                if brace && i == b.len() {
                    return Err("Unclosed PGN comment".into());
                }
                let value = &text[start..i];
                tree::validate_comment(value)?;
                comments += value.len();
                comment_count += 1;
                if comment_count > tree::MAX_COMMENTS || comments > tree::MAX_TOTAL_COMMENT_BYTES {
                    return Err("PGN total comments exceed 1 MiB".into());
                }
                out.push('{');
                out.push_str(value);
                out.push_str("} ");
                if brace {
                    i += 1;
                }
            }
            b'(' => {
                if ended {
                    return Err("Variation after result".into());
                }
                depth += 1;
                if depth > tree::MAX_DEPTH {
                    return Err("PGN variation nesting limit".into());
                }
                out.push_str(" ( ");
                i += 1;
            }
            b')' => {
                if depth == 0 {
                    return Err("Unmatched variation delimiter".into());
                }
                depth -= 1;
                out.push_str(" ) ");
                i += 1;
            }
            b'[' => {
                if in_moves || ended || depth > 0 {
                    return Err("Multiple PGN games or misplaced tags are not supported".into());
                }
                let start = i;
                i += 1;
                let key = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                if i == key || i - key > 128 {
                    return Err("Invalid PGN tag name".into());
                }
                while i < b.len() && b[i].is_ascii_whitespace() {
                    i += 1;
                }
                if b.get(i) != Some(&b'"') {
                    return Err("Invalid PGN tag value".into());
                }
                i += 1;
                let val = i;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                        if !matches!(b.get(i), Some(b'\\' | b'"')) {
                            return Err("Unsupported PGN tag escape".into());
                        }
                    }
                    i += 1;
                }
                if i >= b.len() || i - val > 4096 {
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
                headers.push_str(&text[start..i]);
                headers.push('\n');
            }
            b'$' => {
                if ended {
                    return Err("NAG after result".into());
                }
                let start = i;
                i += 1;
                let n = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                if i == n || text[n..i].parse::<u8>().is_err() {
                    return Err("Invalid NAG (expected 0..255)".into());
                }
                if i < b.len() && !b[i].is_ascii_whitespace() && !b"{}();$!?".contains(&b[i]) {
                    return Err("Invalid NAG delimiter".into());
                }
                out.push_str(&text[start..i]);
                out.push(' ');
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
                if ended {
                    return Err(
                        "Multiple PGN games or tokens after result are not supported".into(),
                    );
                }
                if ["*", "1-0", "0-1", "1/2-1/2"].contains(&token) {
                    if depth == 0 {
                        ended = true;
                    }
                    out.push_str(token);
                    out.push(' ');
                    continue;
                }
                let mut san = token;
                if san.as_bytes()[0].is_ascii_digit() && !san.starts_with("0-0") {
                    let rest = san.trim_start_matches(|c: char| c.is_ascii_digit());
                    if !rest.starts_with('.') {
                        return Err("Invalid PGN move number".into());
                    }
                    san = rest.trim_start_matches('.');
                }
                let core = san.trim_end_matches(['!', '?']);
                let glyph = &san[core.len()..];
                if !glyph.is_empty() && !matches!(glyph, "!" | "?" | "!!" | "??" | "!?" | "?!") {
                    return Err("Unsupported PGN annotation glyph".into());
                }
                if !core.is_empty() {
                    SanPlus::from_ascii(core.as_bytes())
                        .map_err(|_| format!("Invalid PGN token: {core}"))?;
                    in_moves = true;
                    total += 1;
                    if depth == 0 {
                        moves += 1;
                    }
                    if moves > MAX_PLIES || total > tree::MAX_NODES {
                        return Err("PGN mainline/node limit".into());
                    }
                } else if glyph.is_empty() && !san.is_empty() {
                    return Err("Invalid PGN token".into());
                }
                out.push_str(token);
                out.push(' ');
            }
        }
    }
    if depth != 0 {
        return Err("Unclosed PGN variation".into());
    }
    headers.push('\n');
    headers.push_str(&out);
    Ok(headers)
}
pub fn import_annotated(bytes: &[u8]) -> Result<AnnotatedGame, String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("PGN exceeds 4 MiB".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|e| format!("PGN UTF-8: {e}"))?
        .trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Err("Empty PGN".into());
    }
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t'))
    {
        return Err("PGN contains unsafe control characters".into());
    }
    let prepared = prepare(text)?;
    let headers = Headers {
        record: GameRecord::new(&Chess::default(), None),
        pos: Chess::default(),
        error: None,
    };
    let mut visitor = TreeImporter {
        headers,
        nodes: vec![],
        current: 0,
        frames: vec![],
        error: None,
        outcome: false,
        comment_bytes: 0,
        comment_count: 0,
        nags: 0,
    };
    visitor.headers.record.tags.clear();
    let mut reader = BufferedReader::new_cursor(prepared.as_bytes());
    let game = reader
        .read_game(&mut visitor)
        .map_err(|e| format!("PGN read: {e}"))?
        .ok_or("No PGN game")??;
    if reader
        .read_game(&mut visitor)
        .map_err(|e| format!("PGN read: {e}"))?
        .is_some()
    {
        return Err("Multiple PGN games are not supported".into());
    }
    Ok(game)
}
/// Legacy linear projection for the retained state/record regression suite.
/// Application imports always use import_annotated; linear export stays public.
#[cfg(test)]
pub fn import(bytes: &[u8]) -> Result<crate::game_record::ValidatedRecord, String> {
    let tree = import_annotated(bytes)?;
    let annotated = tree.has_annotations();
    let mut main = tree.mainline;
    main.record.omitted_annotations = annotated;
    Ok(main)
}
pub fn export_annotated(game: &AnnotatedGame) -> Result<String, String> {
    game.validate()?;
    let linear = export(&game.mainline.record)?;
    let header = linear.split_once("\n\n").ok_or("PGN header export")?.0;
    let mut out = format!("{header}\n\n");
    fn comments(out: &mut String, values: &[String]) {
        for s in values {
            out.push('{');
            out.push_str(s);
            out.push_str("} ");
        }
    }
    fn line(out: &mut String, game: &AnnotatedGame, mut id: usize, mut first: bool) {
        loop {
            let n = &game.nodes[id];
            let parent = n.parent.expect("move node");
            let before = &game.nodes[parent].position;
            comments(out, &n.before);
            if before.turn() == Color::White {
                out.push_str(&format!("{}. ", before.fullmoves()));
            } else if first {
                out.push_str(&format!("{}... ", before.fullmoves()));
            }
            out.push_str(&n.san);
            out.push(' ');
            for nag in &n.nags {
                out.push_str(&format!("${nag} "));
            }
            comments(out, &n.after);
            if game.nodes[parent].children.first() == Some(&id) {
                for &sibling in game.nodes[parent].children.iter().skip(1) {
                    out.push_str("( ");
                    line(out, game, sibling, true);
                    out.push_str(") ");
                }
            }
            if let Some(result) = &n.branch_result {
                out.push_str(result);
                out.push(' ');
            }
            let Some(&next) = n.children.first() else {
                break;
            };
            id = next;
            first = false;
        }
    }
    comments(&mut out, &game.nodes[0].before);
    comments(&mut out, &game.nodes[0].after);
    if let Some(&first) = game.nodes[0].children.first() {
        line(&mut out, game, first, true);
    }
    out.push_str(&game.mainline.record.result);
    out.push('\n');
    if out.len() > MAX_FILE_BYTES {
        return Err("Annotated PGN export exceeds 4 MiB".into());
    }
    Ok(out)
}

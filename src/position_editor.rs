//! Standard-chess draft editing. Invalid temporary boards never become a Chess.
use shakmaty::{CastlingMode, Chess, Color, FromSetup, Piece, Position, Setup, Square, fen::Fen};

pub const MAX_EDITS: usize = 128;
pub const RIGHTS: [Square; 4] = [Square::H1, Square::A1, Square::H8, Square::A8];

#[derive(Clone, Default)]
pub struct Draft {
    pub setup: Setup,
    undo: Vec<Setup>,
    redo: Vec<Setup>,
}
impl Draft {
    pub fn change(&mut self, next: Setup) {
        if next == self.setup {
            return;
        }
        if self.undo.len() == MAX_EDITS {
            self.undo.remove(0);
        }
        self.undo.push(std::mem::replace(&mut self.setup, next));
        self.redo.clear();
    }
    pub fn place(&mut self, square: Square, piece: Option<Piece>) {
        let mut next = self.setup.clone();
        next.board.discard_piece_at(square);
        if let Some(piece) = piece {
            next.board.set_piece_at(square, piece);
        }
        // Rights and EP are explicit fields, never silently repaired by editing.
        self.change(next);
    }
    pub fn relocate(&mut self, from: Square, to: Square) {
        if from == to {
            return;
        }
        if let Some(piece) = self.setup.board.piece_at(from) {
            let mut next = self.setup.clone();
            next.board.discard_piece_at(from);
            next.board.set_piece_at(to, piece);
            self.change(next);
        }
    }
    pub fn undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.setup, s));
        }
    }
    pub fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.setup, s));
        }
    }
    pub fn text(&self) -> String {
        write_setup(&self.setup)
    }
    pub fn validate(&self) -> Result<Chess, String> {
        validate(&self.setup)
    }
}

pub fn write_setup(setup: &Setup) -> String {
    // Fen's flexible castling formatter can use file letters. This editor only
    // permits the four explicit standard rights, including conflicting drafts.
    let placement = Fen::from_setup(setup.clone())
        .to_string()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let rights: String = RIGHTS
        .iter()
        .zip(['K', 'Q', 'k', 'q'])
        .filter_map(|(s, c)| setup.castling_rights.contains(*s).then_some(c))
        .collect();
    format!(
        "{placement} {} {} {} {} {}",
        if setup.turn == Color::White { "w" } else { "b" },
        if rights.is_empty() { "-" } else { &rights },
        setup
            .ep_square
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into()),
        setup.halfmoves,
        setup.fullmoves
    )
}

/// Syntactically exact six-field standard FEN, allowed to remain chess-illegal.
pub fn parse_draft(text: &str) -> Result<Setup, String> {
    let fields: Vec<_> = text.split_whitespace().collect();
    if text.len() > 256 || fields.len() != 6 {
        return Err("FEN: exactly six fields, at most 256 bytes".into());
    }
    if !fields[0]
        .chars()
        .all(|c| "prnbqkPRNBQK12345678/".contains(c))
        || fields[0].split('/').count() != 8
    {
        return Err("FEN: standard piece placement only (no variants)".into());
    }
    if !matches!(fields[1], "w" | "b") {
        return Err("FEN: active color must be w or b".into());
    }
    let mut rights = shakmaty::Bitboard::EMPTY;
    if fields[2] != "-" {
        if fields[2].is_empty() {
            return Err("FEN: castling rights".into());
        }
        for c in fields[2].chars() {
            let i = "KQkq".find(c).ok_or("FEN: standard KQkq castling only")?;
            if rights.contains(RIGHTS[i]) {
                return Err("FEN: duplicate castling right".into());
            }
            rights.add(RIGHTS[i]);
        }
    }
    for value in &fields[4..] {
        if value.is_empty()
            || !value.bytes().all(|c| c.is_ascii_digit())
            || value.parse::<u32>().is_err()
        {
            return Err("FEN: move counters must be unsigned 32-bit decimal integers".into());
        }
    }
    if fields[5].parse::<u32>().unwrap() == 0 {
        return Err("FEN: fullmove number must be at least 1".into());
    }
    // Use '-' for the parser's castling field: it otherwise resolves K/Q using
    // present rooks, which could reinterpret a conflicting draft as Chess960.
    let normalized = format!(
        "{} {} - {} {} {}",
        fields[0], fields[1], fields[3], fields[4], fields[5]
    );
    let mut setup = normalized
        .parse::<Fen>()
        .map_err(|e| format!("FEN: {e}"))?
        .into_setup();
    setup.castling_rights = rights;
    Ok(setup)
}
pub fn validate(setup: &Setup) -> Result<Chess, String> {
    if setup.pockets.is_some() || setup.remaining_checks.is_some() || !setup.promoted.is_empty() {
        return Err("Only standard chess is supported".into());
    }
    if !(setup.castling_rights & !shakmaty::Bitboard::CORNERS).is_empty() {
        return Err("Invalid standard castling rights".into());
    }
    if setup.ep_square.is_some() && setup.halfmoves != 0 {
        return Err(
            "En passant implies the last move was a pawn move: halfmove clock must be 0".into(),
        );
    }
    let pos = Chess::from_setup(setup.clone(), CastlingMode::Standard)
        .map_err(|e| format!("Illegal standard chess position: {e}"))?;
    // Keep the existing strict application FEN entry point in the validation path.
    crate::fen::parse(&write_setup(setup))?;
    if pos.clone().into_setup(shakmaty::EnPassantMode::Always) != *setup {
        return Err("Position fields would be changed by validation".into());
    }
    Ok(pos)
}

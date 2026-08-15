use pleco::bot_prelude::{
    AlphaBetaSearcher, IterativeSearcher, JamboreeSearcher, MiniMaxSearcher, Searcher,
};
use pleco::{Board, PieceType};
use shakmaty::fen::Fen;
use shakmaty::{Chess, EnPassantMode, Move, Position, Role, Square};

enum SearcherKind {
    MiniMax,
    AlphaBeta,
    Jamboree,
    Iterative,
}

pub fn best_move(pos: &Chess, level: u32) -> Move {
    let fen = Fen::from_position(pos.clone(), EnPassantMode::Legal).to_string();
    let board = Board::from_fen(&fen).unwrap_or_else(|_| Board::start_pos());

    let (kind, depth) = match level {
        1 => (SearcherKind::MiniMax, 2u16),
        2 => (SearcherKind::AlphaBeta, 3),
        3 => (SearcherKind::AlphaBeta, 4),
        4 => (SearcherKind::Jamboree, 4),
        _ => (SearcherKind::Iterative, 5),
    };

    let bitmove = match kind {
        SearcherKind::MiniMax => MiniMaxSearcher::best_move(board, depth),
        SearcherKind::AlphaBeta => AlphaBetaSearcher::best_move(board, depth),
        SearcherKind::Jamboree => JamboreeSearcher::best_move(board, depth),
        SearcherKind::Iterative => IterativeSearcher::best_move(board, depth),
    };

    let from = Square::new(bitmove.get_src().0 as u32);
    let to = Square::new(bitmove.get_dest().0 as u32);
    let promo = if bitmove.is_promo() {
        match bitmove.promo_piece() {
            PieceType::N => Some(Role::Knight),
            PieceType::B => Some(Role::Bishop),
            PieceType::R => Some(Role::Rook),
            _ => Some(Role::Queen),
        }
    } else {
        None
    };

    pos.legal_moves()
        .iter()
        .cloned()
        .find(|m| m.from() == Some(from) && m.to() == to && m.promotion() == promo)
        .unwrap_or_else(|| {
            pos.legal_moves()
                .iter()
                .cloned()
                .next()
                .expect("至少存在一个合法着法")
        })
}

#[cfg(test)]
mod tests {
    use super::best_move;
    use shakmaty::{Chess, Position};

    #[test]
    fn suggested_move_is_legal() {
        let pos = Chess::default();
        let suggestion = best_move(&pos, 1);
        assert!(pos.legal_moves().iter().any(|mv| mv == &suggestion));
    }
}

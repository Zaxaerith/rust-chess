use pleco::bot_prelude::{
    AlphaBetaSearcher, IterativeSearcher, JamboreeSearcher, MiniMaxSearcher, Searcher,
};
use pleco::{BitMove, Board};
use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, EnPassantMode, Move, Position};
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchError {
    TerminalPosition,
    InvalidPosition(String),
    MissingMove,
    IllegalMove(String),
    SearchPanicked,
    WorkerDisconnected,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
pub type SearchResult = Result<Move, SearchError>;

pub fn best_move(pos: &Chess, level: u32) -> SearchResult {
    search_with(pos, |board| match level {
        1 => MiniMaxSearcher::best_move(board, 2),
        2 => AlphaBetaSearcher::best_move(board, 3),
        3 => AlphaBetaSearcher::best_move(board, 4),
        4 => JamboreeSearcher::best_move(board, 4),
        _ => IterativeSearcher::best_move(board, 5),
    })
}

fn search_with(pos: &Chess, search: impl FnOnce(Board) -> BitMove) -> SearchResult {
    if pos.is_game_over() || pos.halfmoves() >= 150 {
        return Err(SearchError::TerminalPosition);
    }
    catch_unwind(AssertUnwindSafe(|| {
        let fen = Fen::from_position(pos.clone(), EnPassantMode::Legal).to_string();
        let board = Board::from_fen(&fen)
            .map_err(|error| SearchError::InvalidPosition(format!("{error:?}")))?;
        map_move(pos, search(board))
    }))
    .unwrap_or(Err(SearchError::SearchPanicked))
}

fn map_move(pos: &Chess, bitmove: BitMove) -> SearchResult {
    if bitmove.is_null() {
        return Err(SearchError::MissingMove);
    }
    // Pleco's UCI formatter translates castle rook squares to king targets.
    // shakmaty then resolves castles, en passant and all four promotions.
    let uci = bitmove.stringify();
    uci.parse::<UciMove>()
        .ok()
        .and_then(|mv| mv.to_move(pos).ok())
        .ok_or(SearchError::IllegalMove(uci))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{CastlingMode, Move};

    fn position(fen: &str) -> Chess {
        fen.parse::<Fen>()
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    #[test]
    fn suggested_move_is_legal() {
        let pos = Chess::default();
        assert!(pos.is_legal(&best_move(&pos, 1).unwrap()));
    }

    #[test]
    fn every_engine_move_maps_in_special_positions() {
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
            "4k3/P7/8/8/8/8/7p/4K3 w - - 0 1",
            "4k3/P7/8/8/8/8/7p/4K3 b - - 0 1",
            "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
            "4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1",
        ] {
            let pos = position(fen);
            let board = Board::from_fen(fen).unwrap();
            let mapped: Vec<_> = board
                .generate_moves()
                .iter()
                .map(|mv| map_move(&pos, *mv).unwrap())
                .collect();
            assert_eq!(mapped.len(), pos.legal_moves().len());
            for mv in pos.legal_moves() {
                assert!(mapped.contains(&mv), "unmapped {mv:?} in {fen}");
            }
            if fen.contains("KQkq - 0 1") && !fen.contains("pppppppp") {
                assert_eq!(
                    mapped
                        .iter()
                        .filter(|mv| matches!(mv, Move::Castle { .. }))
                        .count(),
                    2
                );
            }
        }
    }

    #[test]
    fn terminal_positions_do_not_search() {
        for fen in [
            "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1",
            "7k/5K2/6Q1/8/8/8/8/8 b - - 0 1",
            "7k/8/8/8/8/8/8/K7 w - - 0 1",
            "7k/8/8/8/8/8/8/KQ6 w - - 150 80",
        ] {
            assert_eq!(
                search_with(&position(fen), |_| panic!("must not search")),
                Err(SearchError::TerminalPosition)
            );
        }
    }

    #[test]
    fn engine_failures_are_explicit() {
        let pos = Chess::default();
        assert_eq!(
            search_with(&pos, |_| panic!("injected search failure")),
            Err(SearchError::SearchPanicked)
        );
        assert_eq!(
            map_move(&pos, BitMove::null()),
            Err(SearchError::MissingMove)
        );
        let wrong = Board::from_fen("7k/8/8/8/8/8/R7/K7 w - - 0 1").unwrap();
        let moves = wrong.generate_moves();
        let bad = moves
            .iter()
            .find(|mv| mv.stringify() == "a2a8")
            .copied()
            .unwrap();
        assert!(matches!(
            map_move(&pos, bad),
            Err(SearchError::IllegalMove(_))
        ));
    }
}

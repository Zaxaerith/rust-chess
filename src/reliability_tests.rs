use super::*;
use shakmaty::{CastlingMode, fen::Fen, uci::UciMove};

fn app() -> App {
    let mut app = App::new();
    app.settings = Settings::default();
    app.settings.board.chess_clock_enabled = true;
    app.settings.board.clock_warning = false;
    app.screen = Screen::Game;
    app
}

fn mv(app: &App, uci: &str) -> Move {
    uci.parse::<UciMove>().unwrap().to_move(&app.pos).unwrap()
}

fn position(fen: &str) -> Chess {
    fen.parse::<Fen>()
        .unwrap()
        .into_position(CastlingMode::Standard)
        .unwrap()
}

#[test]
fn timeout_ends_game_and_rejects_moves() {
    let mut app = app();
    let next = mv(&app, "e2e4");
    app.clocks.white = Duration::ZERO;
    app.tick_clock();
    assert!(app.game_over());
    app.apply_move(next);
    assert!(app.history.is_empty());
}

#[test]
fn human_undo_restores_previous_clocks() {
    let mut app = app();
    app.clocks.white = Duration::from_secs(590);
    app.apply_move(mv(&app, "e2e4"));
    app.clocks.black = Duration::from_secs(580);
    app.apply_move(mv(&app, "e7e5"));
    app.undo();
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.clocks.black, Duration::from_secs(600));
}

#[test]
fn ai_undo_removes_complete_pair() {
    let mut app = app();
    app.apply_move(mv(&app, "e2e4"));
    app.apply_move(mv(&app, "e7e5"));
    app.ai_mode = Some(Color::Black);
    app.undo();
    assert!(app.history.is_empty());
    assert_eq!(app.pos.turn(), Color::White);
}

#[test]
fn king_two_square_input_castles() {
    let mut app = app();
    app.pos = position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    app.settings.board.castling_method = CastlingMethod::KingTwoSquares;
    app.selected = Some(Square::E1);
    app.handle_board_click(Square::G1);
    assert_eq!(app.pos.board().role_at(Square::G1), Some(Role::King));
}

#[test]
fn repetition_key_ignores_uncapturable_ep_square() {
    let with_ep = position("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    let without_ep = position("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 1 1");
    assert_eq!(position_key(&with_ep), position_key(&without_ep));
}

fn play(app: &mut App, uci: &str) {
    let next = mv(app, uci);
    app.apply_move_at(next, app.clock_last_tick);
}

fn advance(app: &mut App, elapsed: Duration) {
    app.tick_clock_at(app.clock_last_tick + elapsed);
}

fn queue_ai(app: &mut App, result: ai::SearchResult, generation: u64) {
    let (tx, rx) = mpsc::channel();
    tx.send(SearchReply { generation, result }).unwrap();
    app.ai_rx = Some(rx);
    app.ai_reply_at = Some(Instant::now() - Duration::from_secs(1));
    app.ai_thinking = true;
}

fn queue_hint(app: &mut App, result: ai::SearchResult, generation: u64) {
    let (tx, rx) = mpsc::channel();
    tx.send(SearchReply { generation, result }).unwrap();
    app.hint_rx = Some(rx);
    app.hint_thinking = true;
}

#[test]
fn clocks_decrement_exactly_and_switch_turn() {
    let mut app = app();
    advance(&mut app, Duration::from_millis(1234));
    assert_eq!(app.clocks.white, Duration::from_millis(598766));
    assert_eq!(app.clocks.black, Duration::from_secs(600));
    play(&mut app, "e2e4");
    advance(&mut app, Duration::from_millis(5678));
    assert_eq!(app.clocks.black, Duration::from_millis(594322));
    assert_eq!(app.clocks.white, Duration::from_millis(598766));
}

#[test]
fn menu_settings_and_disabled_clock_pause_without_catchup() {
    let mut app = app();
    for screen in [Screen::Menu, Screen::Settings] {
        app.screen = screen;
        advance(&mut app, Duration::from_secs(700));
        assert_eq!(app.clocks, ClockState::default());
    }
    app.screen = Screen::Game;
    app.settings.board.chess_clock_enabled = false;
    advance(&mut app, Duration::from_secs(700));
    assert_eq!(app.clocks, ClockState::default());
    app.settings.board.chess_clock_enabled = true;
    advance(&mut app, Duration::from_secs(1));
    assert_eq!(app.clocks.white, Duration::from_secs(599));
}

#[test]
fn timeout_saturates_once_and_freezes_both_clocks() {
    let mut app = app();
    advance(&mut app, Duration::from_secs(3600));
    assert_eq!(
        app.result,
        Some(GameResult::Timeout {
            loser: Color::White,
            draw: false
        })
    );
    let clocks = app.clocks;
    let end = app.game_over_at;
    let generation = app.generation;
    advance(&mut app, Duration::from_secs(3600));
    assert_eq!(app.clocks, clocks);
    assert_eq!(app.game_over_at, end);
    assert_eq!(app.generation, generation);
    assert_eq!(app.clocks.black, Duration::from_secs(600));
}

#[test]
fn timeout_draw_tests_opponents_material_not_losers() {
    let mut app = app();
    app.pos = position("7k/8/8/8/8/8/8/KQ6 w - - 0 1");
    advance(&mut app, Duration::from_secs(600));
    assert_eq!(
        app.result,
        Some(GameResult::Timeout {
            loser: Color::White,
            draw: true
        })
    );
    assert!(!app.pos.is_insufficient_material());
    let mut app = self::app();
    app.pos = position("7k/8/8/8/8/8/8/KQ6 b - - 0 1");
    advance(&mut app, Duration::from_secs(600));
    assert_eq!(
        app.result,
        Some(GameResult::Timeout {
            loser: Color::Black,
            draw: false
        })
    );
}

#[test]
fn timeout_material_distinguishes_possible_mate_from_forced_mate() {
    // Two knights cannot force mate, but can mate with cooperation.
    let pos = position("7k/8/8/8/8/8/8/KNN5 b - - 0 1");
    assert_eq!(
        GameResult::timeout(&pos),
        GameResult::Timeout {
            loser: Color::Black,
            draw: false
        }
    );
    let pos = position("7k/8/8/8/8/8/8/KN6 b - - 0 1");
    assert_eq!(
        GameResult::timeout(&pos),
        GameResult::Timeout {
            loser: Color::Black,
            draw: true
        }
    );
    // The opponent's pawn can help a single knight deliver mate.
    let pos = position("7k/7p/8/8/8/8/8/KN6 b - - 0 1");
    assert_eq!(
        GameResult::timeout(&pos),
        GameResult::Timeout {
            loser: Color::Black,
            draw: false
        }
    );
}

#[test]
fn timeout_cancels_confirmation_promotion_and_premove() {
    let mut app = app();
    app.pending_move = Some(mv(&app, "e2e4"));
    app.promotion = Some((Square::A7, Square::A8));
    app.premove = Some((Square::D2, Square::D4));
    app.selected = Some(Square::E2);
    advance(&mut app, Duration::from_secs(600));
    assert!(app.pending_move.is_none() && app.promotion.is_none() && app.premove.is_none());
    app.handle_action(UiAction::ConfirmMove);
    app.handle_promotion(Role::Queen);
    app.try_apply_premove();
    assert!(app.history.is_empty());
}

#[test]
fn confirmation_and_promotion_keep_ticking() {
    let mut app = app();
    app.settings.board.move_confirmation = true;
    app.submit_player_move(mv(&app, "e2e4"));
    assert!(app.pending_move.is_some());
    let before = app.clocks.white;
    advance(&mut app, Duration::from_secs(2));
    assert_eq!(app.clocks.white, before - Duration::from_secs(2));
    app.pending_move = None;
    app.pos = position("7k/P7/8/8/8/8/8/4K3 w - - 0 1");
    app.promotion = Some((Square::A7, Square::A8));
    let before = app.clocks.white;
    advance(&mut app, Duration::from_secs(3));
    assert_eq!(app.clocks.white, before - Duration::from_secs(3));
}

#[test]
fn mating_move_before_deadline_wins_at_deadline_times_out() {
    for (elapsed, timed_out) in [(999, false), (1000, true)] {
        let mut app = app();
        app.pos = position("7k/8/5KQ1/8/8/8/8/8 w - - 0 1");
        app.clocks.white = Duration::from_secs(1);
        let mating_move = mv(&app, "g6g7");
        app.apply_move_at(
            mating_move,
            app.clock_last_tick + Duration::from_millis(elapsed),
        );
        if timed_out {
            assert_eq!(
                app.result,
                Some(GameResult::Timeout {
                    loser: Color::White,
                    draw: true
                })
            );
            assert!(app.history.is_empty());
        } else {
            assert_eq!(
                app.result,
                Some(GameResult::Checkmate {
                    winner: Color::White
                })
            );
            advance(&mut app, Duration::from_secs(5));
            assert_eq!(
                app.result,
                Some(GameResult::Checkmate {
                    winner: Color::White
                })
            );
        }
    }
}

#[test]
fn checkmate_stalemate_and_rule_draw_have_distinct_results() {
    for (fen, expected) in [
        (
            "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1",
            GameResult::Checkmate {
                winner: Color::White,
            },
        ),
        ("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1", GameResult::Stalemate),
        (
            "7k/8/8/8/8/8/8/K7 w - - 0 1",
            GameResult::InsufficientMaterial,
        ),
        (
            "7k/8/8/8/8/8/8/KQ6 w - - 150 80",
            GameResult::SeventyFiveMoves,
        ),
    ] {
        assert_eq!(GameResult::from_position(&position(fen), 0), Some(expected));
    }
    let mate = position("7k/6Q1/5K2/8/8/8/8/8 b - - 150 80");
    assert_eq!(
        GameResult::from_position(&mate, 5),
        Some(GameResult::Checkmate {
            winner: Color::White
        })
    );
}

#[test]
fn terminal_undo_restores_playable_state_and_clock_snapshot() {
    let mut app = app();
    for uci in ["f2f3", "e7e5", "g2g4", "d8h4"] {
        play(&mut app, uci);
    }
    assert!(app.game_over());
    app.undo();
    assert!(!app.game_over());
    assert!(app.game_over_at.is_none());
    assert_eq!(app.history.len(), 3);
    assert_eq!(app.clocks, *app.clock_history.last().unwrap());
    app.clocks.black = Duration::ZERO;
    app.tick_clock_at(app.clock_last_tick);
    assert!(app.game_over());
    app.undo();
    assert!(!app.game_over());
    assert!(app.clocks.black > Duration::ZERO);
}

#[test]
fn ai_pair_undo_restores_both_times() {
    let mut app = app();
    for uci in ["e2e4", "e7e5"] {
        play(&mut app, uci);
    }
    let expected = app.clocks;
    advance(&mut app, Duration::from_secs(7));
    play(&mut app, "g1f3");
    advance(&mut app, Duration::from_secs(9));
    play(&mut app, "b8c6");
    app.ai_mode = Some(Color::Black);
    app.undo();
    assert_eq!(app.history.len(), 2);
    assert_eq!(app.clocks, expected);
    assert_eq!(app.clock_history.len(), 3);
}

#[test]
fn undo_during_ai_thinking_only_rewinds_human_move() {
    let mut app = app();
    play(&mut app, "e2e4");
    app.ai_mode = Some(Color::Black);
    let old_generation = app.generation;
    let old_move = mv(&app, "e7e5");
    queue_ai(&mut app, Ok(old_move), old_generation);
    app.undo();
    assert!(app.history.is_empty());
    assert_eq!(app.clocks, ClockState::default());
    assert!(app.ai_rx.is_none());
    assert!(app.generation > old_generation);
}

#[test]
fn undo_with_ai_white_returns_to_human_turn() {
    let mut app = app();
    for uci in ["e2e4", "e7e5", "g1f3"] {
        play(&mut app, uci);
    }
    app.ai_mode = Some(Color::White);
    app.undo();
    assert_eq!(app.history.len(), 1);
    assert_eq!(app.pos.turn(), Color::Black);
}

#[test]
fn new_game_clears_all_history_results_inputs_and_clocks() {
    let mut app = app();
    play(&mut app, "e2e4");
    advance(&mut app, Duration::from_secs(600));
    assert!(app.game_over());
    app.new_game();
    assert!(!app.game_over());
    assert_eq!(app.clocks, ClockState::default());
    assert_eq!(app.clock_history, vec![ClockState::default()]);
    assert!(app.history.is_empty() && app.history_sans.is_empty());
    assert_eq!(app.position_keys, vec![position_key(&Chess::default())]);
}

#[test]
fn stale_ai_and_hint_results_are_rejected_after_state_changes() {
    for transition in 0..4 {
        let mut app = app();
        play(&mut app, "e2e4");
        let old_generation = app.generation;
        let old_move = mv(&app, "e7e5");
        queue_ai(&mut app, Ok(old_move.clone()), old_generation);
        queue_hint(&mut app, Ok(old_move.clone()), old_generation);
        match transition {
            0 => app.new_game(),
            1 => app.undo(),
            2 => app.change_mode(None),
            _ => app.back_to_menu(),
        }
        assert!(app.ai_rx.is_none() && app.hint_rx.is_none());
        assert!(
            app.accept_reply(SearchReply {
                generation: old_generation,
                result: Ok(old_move.clone())
            })
            .is_none()
        );
        let count = app.history.len();
        // Even if a queued response were accidentally reattached, the guard holds.
        app.ai_mode = Some(app.pos.turn());
        queue_ai(&mut app, Ok(old_move.clone()), old_generation);
        queue_hint(&mut app, Ok(old_move), old_generation);
        app.poll_ai();
        app.poll_hint();
        assert_eq!(app.history.len(), count);
        assert!(app.suggestion.is_none());
    }
}

#[test]
fn timeout_while_ai_thinks_discards_ready_response_and_premove() {
    let mut app = app();
    app.ai_mode = Some(Color::White);
    let generation = app.generation;
    let next = mv(&app, "e2e4");
    queue_ai(&mut app, Ok(next.clone()), generation);
    queue_hint(&mut app, Ok(next), generation);
    app.premove = Some((Square::E7, Square::E5));
    app.clocks.white = Duration::ZERO;
    app.poll_ai();
    app.poll_hint();
    assert!(app.game_over());
    assert!(app.history.is_empty());
    assert!(app.ai_rx.is_none() && app.hint_rx.is_none());
    assert!(app.premove.is_none());
}

#[test]
fn current_ai_response_and_hint_are_legal_and_visible() {
    let mut app = app();
    let generation = app.generation;
    let next = mv(&app, "e2e4");
    queue_hint(&mut app, Ok(next.clone()), generation);
    app.poll_hint();
    assert!(app.suggestion.is_some());
    app.ai_mode = Some(Color::White);
    queue_ai(&mut app, Ok(next), generation);
    app.poll_ai();
    assert_eq!(app.history.len(), 1);
    assert!(app.suggestion.is_none());
    assert_eq!(app.pos.turn(), Color::Black);
}

#[test]
fn invalid_async_move_and_worker_errors_do_not_mutate_position() {
    let mut app = app();
    let illegal = "e7e5"
        .parse::<UciMove>()
        .unwrap()
        .to_move(&position(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ))
        .unwrap();
    let generation = app.generation;
    queue_hint(&mut app, Ok(illegal), generation);
    app.poll_hint();
    assert!(matches!(
        app.search_error,
        Some(ai::SearchError::IllegalMove(_))
    ));
    assert!(app.suggestion.is_none());
    app.ai_mode = Some(Color::White);
    queue_ai(&mut app, Err(ai::SearchError::SearchPanicked), generation);
    app.poll_ai();
    assert_eq!(app.search_error, Some(ai::SearchError::SearchPanicked));
    assert!(app.history.is_empty());
    assert!(!app.ai_thinking);
    let (tx, rx) = mpsc::channel();
    drop(tx);
    app.hint_rx = Some(rx);
    app.poll_hint();
    assert_eq!(app.search_error, Some(ai::SearchError::WorkerDisconnected));
}

fn repeat(app: &mut App, cycles: usize) {
    for _ in 0..cycles {
        for uci in ["g1f3", "g8f6", "f3g1", "f6g8"] {
            play(app, uci);
        }
    }
}

#[test]
fn threefold_policy_and_automatic_fivefold() {
    let mut app = app();
    app.settings.board.auto_threefold = AutoThreefold::Always;
    repeat(&mut app, 2);
    assert_eq!(app.result, Some(GameResult::ThreefoldRepetition));
    app.undo();
    assert!(!app.game_over());
    let mut app = self::app();
    repeat(&mut app, 2);
    assert!(!app.game_over());
    repeat(&mut app, 2);
    assert_eq!(app.result, Some(GameResult::FivefoldRepetition));
}

#[test]
fn conditional_threefold_uses_claimants_clock_and_enabled_setting() {
    for (enabled, white, black, expected) in [
        (true, 29, 600, true),
        (true, 600, 29, false),
        (false, 29, 29, false),
        (true, 30, 30, false),
    ] {
        let mut app = app();
        app.settings.board.chess_clock_enabled = enabled;
        app.settings.board.auto_threefold = AutoThreefold::UnderThirtySeconds;
        repeat(&mut app, 1);
        // Set clocks just before the final repetition so no earlier claim occurs.
        for uci in ["g1f3", "g8f6", "f3g1"] {
            play(&mut app, uci);
        }
        app.clocks.white = Duration::from_secs(white);
        app.clocks.black = Duration::from_secs(black);
        play(&mut app, "f6g8");
        assert_eq!(
            app.result == Some(GameResult::ThreefoldRepetition),
            expected
        );
    }
}

#[test]
fn repetition_identity_preserves_turn_castling_and_legal_ep() {
    let a = position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    let b = position("r3k2r/8/8/8/8/8/8/R3K2R w - - 8 5");
    let c = position("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");
    assert_ne!(position_key(&a), position_key(&b));
    assert_ne!(position_key(&a), position_key(&c));
    let a = position("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
    let b = position("4k3/8/8/3pP3/8/8/8/4K3 w - - 0 1");
    assert_ne!(position_key(&a), position_key(&b));
    // Pinned pawn cannot legally capture en passant, so the right is irrelevant.
    let a = position("k3r3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
    let b = position("k3r3/8/8/3pP3/8/8/8/4K3 w - - 0 1");
    assert_eq!(position_key(&a), position_key(&b));
}

#[test]
fn both_castling_methods_work_through_click_events_on_both_wings() {
    for (method, target, king_to, rook_to) in [
        (
            CastlingMethod::KingOntoRook,
            Square::H1,
            Square::G1,
            Square::F1,
        ),
        (
            CastlingMethod::KingOntoRook,
            Square::A1,
            Square::C1,
            Square::D1,
        ),
        (
            CastlingMethod::KingTwoSquares,
            Square::G1,
            Square::G1,
            Square::F1,
        ),
        (
            CastlingMethod::KingTwoSquares,
            Square::C1,
            Square::C1,
            Square::D1,
        ),
    ] {
        let mut app = app();
        app.pos = position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        app.settings.board.castling_method = method;
        app.handle_pointer_down(Square::E1);
        app.handle_pointer_up(Some(Square::E1));
        app.handle_pointer_down(target);
        assert_eq!(app.pos.board().role_at(king_to), Some(Role::King));
        assert_eq!(app.pos.board().role_at(rook_to), Some(Role::Rook));
    }
}

#[test]
fn promotion_policies_manual_auto_and_premove() {
    for policy in [
        AutoPromotion::Never,
        AutoPromotion::Premove,
        AutoPromotion::Always,
    ] {
        let mut app = app();
        app.pos = position("7k/P7/8/8/8/8/8/4K3 w - - 0 1");
        app.settings.board.auto_promotion = policy;
        app.selected = Some(Square::A7);
        app.handle_board_click(Square::A8);
        assert_eq!(app.promotion.is_some(), policy != AutoPromotion::Always);
        if app.promotion.is_some() {
            app.handle_promotion(Role::Knight);
        }
        assert_eq!(
            app.pos.board().role_at(Square::A8),
            Some(if policy == AutoPromotion::Always {
                Role::Queen
            } else {
                Role::Knight
            })
        );
        let mut app = self::app();
        app.pos = position("7k/P7/8/8/8/8/8/4K3 w - - 0 1");
        app.settings.board.auto_promotion = policy;
        app.premove = Some((Square::A7, Square::A8));
        app.try_apply_premove();
        assert_eq!(app.promotion.is_some(), policy == AutoPromotion::Never);
        if policy != AutoPromotion::Never {
            assert_eq!(app.pos.board().role_at(Square::A8), Some(Role::Queen));
        }
    }
}

#[test]
fn premove_respects_confirmation_and_disabled_setting() {
    let mut app = app();
    app.settings.board.move_confirmation = true;
    app.premove = Some((Square::E2, Square::E4));
    app.try_apply_premove();
    assert!(app.pending_move.is_some());
    assert!(app.history.is_empty());
    app.handle_action(UiAction::ConfirmMove);
    assert_eq!(app.history.len(), 1);
    app.settings.board.premoves = false;
    app.premove = Some((Square::E7, Square::E5));
    app.try_apply_premove();
    assert_eq!(app.history.len(), 1);
}

#[test]
fn takeback_and_time_gift_policies_are_enforced() {
    for policy in [
        AccessPolicy::Never,
        AccessPolicy::Casual,
        AccessPolicy::Always,
    ] {
        let mut app = app();
        app.settings.board.chess_clock_enabled = false;
        play(&mut app, "e2e4");
        app.settings.board.takebacks = policy;
        app.undo();
        assert_eq!(app.history.is_empty(), policy != AccessPolicy::Never);
        app.settings.board.give_more_time = policy;
        app.settings.board.chess_clock_enabled = true;
        app.add_clock_time(Color::Black);
        assert_eq!(
            app.clocks.black,
            Duration::from_secs(if policy == AccessPolicy::Never {
                600
            } else {
                615
            })
        );
    }
}

#[test]
fn time_gifts_do_not_revive_terminal_games() {
    let mut app = app();
    app.clocks.white = Duration::ZERO;
    app.tick_clock_at(app.clock_last_tick);
    app.handle_action(UiAction::AddClockTime(Color::White));
    assert_eq!(app.clocks.white, Duration::ZERO);
    assert!(app.game_over());
    app.settings.board.chess_clock_enabled = false;
    app.tick_clock_at(app.clock_last_tick);
    assert!(app.game_over());
}

#[test]
fn clock_warning_crosses_once_and_snapshots_restore_warning_state() {
    let mut clock = ClockState::default();
    assert_eq!(
        clock.elapse(Color::White, Duration::from_secs(569)),
        (false, false)
    );
    assert_eq!(
        clock.elapse(Color::White, Duration::from_secs(1)),
        (false, true)
    );
    assert_eq!(
        clock.elapse(Color::White, Duration::from_secs(1)),
        (false, false)
    );
    assert_eq!(clock.warned, [true, false]);
}

#[test]
fn unresolved_timeout_freezes_game_until_undo_or_restart() {
    let mut app = app();
    play(&mut app, "e2e4");
    app.pending_move = Some(mv(&app, "e7e5"));
    app.finish_game(
        GameResult::TimeoutUnresolved {
            loser: Color::Black,
        },
        app.clock_last_tick,
    );
    let clocks = app.clocks;
    advance(&mut app, Duration::from_secs(600));
    assert_eq!(app.clocks, clocks);
    assert!(app.pending_move.is_none());
    app.apply_move(mv(&app, "e7e5"));
    assert_eq!(app.history.len(), 1);
    app.undo();
    assert!(!app.game_over());
}

use super::*;
use crate::{
    game_record::{GameRecord, MAX_FILE_BYTES, MAX_PLIES},
    persistence,
    records_ui::Action,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn app() -> App {
    let mut a = App::new();
    a.settings = Settings::default();
    a.screen = Screen::Game;
    a.new_game();
    a
}
fn play(a: &mut App, uci: &str) {
    let mv = uci
        .parse::<shakmaty::uci::UciMove>()
        .unwrap()
        .to_move(&a.pos)
        .unwrap();
    a.apply_move_at(mv, a.clock_last_tick);
}
fn temp(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::current_dir()
        .unwrap()
        .join("target/v020-tests")
        .join(format!(
            "{}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed),
            name
        ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn standard_pgn_has_seven_tags_and_numbered_san() {
    let mut a = app();
    for uci in ["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6"] {
        play(&mut a, uci);
    }
    let text = pgn::export(&a.record()).unwrap();
    for tag in ["Event", "Site", "Date", "Round", "White", "Black", "Result"] {
        assert!(text.contains(&format!("[{tag} \"")));
    }
    assert!(text.contains("1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *"));
    assert_eq!(
        fen::write(
            pgn::import(text.as_bytes())
                .unwrap()
                .positions
                .last()
                .unwrap()
        ),
        fen::write(&a.pos)
    );
}
#[test]
fn pgn_roundtrip_preserves_unicode_and_escaped_metadata() {
    let mut game = pgn::import(b"1. e4 e5 *").unwrap().record;
    game.tags.insert("White".into(), "棋手 \"A\" \\ B".into());
    let text = pgn::export(&game).unwrap();
    let back = pgn::import(text.as_bytes()).unwrap();
    assert_eq!(back.record.tags["White"], game.tags["White"]);
    assert_eq!(back.record.moves, game.moves);
}
#[test]
fn export_result_distinguishes_mate_timeout_draw_and_unfinished() {
    use crate::game_record::result_token;
    assert_eq!(result_token(None), "*");
    for (r, token) in [
        (
            GameResult::Checkmate {
                winner: Color::White,
            },
            "1-0",
        ),
        (
            GameResult::Checkmate {
                winner: Color::Black,
            },
            "0-1",
        ),
        (
            GameResult::Timeout {
                loser: Color::White,
                draw: false,
            },
            "0-1",
        ),
        (
            GameResult::Timeout {
                loser: Color::Black,
                draw: false,
            },
            "1-0",
        ),
        (
            GameResult::Timeout {
                loser: Color::White,
                draw: true,
            },
            "1/2-1/2",
        ),
        (GameResult::Stalemate, "1/2-1/2"),
        (GameResult::ThreefoldRepetition, "1/2-1/2"),
        (
            GameResult::TimeoutUnresolved {
                loser: Color::White,
            },
            "*",
        ),
    ] {
        assert_eq!(result_token(Some(r)), token);
        let a = app();
        let record =
            GameRecord::capture(&a.records.initial, &[], a.records.metadata.clone(), Some(r));
        let text = pgn::export(&record).unwrap();
        assert!(text.ends_with(&format!("{token}\n")));
        assert!(text.contains(&format!("[Result \"{token}\"]")));
    }
}
#[test]
fn mate_san_has_suffix_and_correct_result() {
    let mut a = app();
    for uci in ["f2f3", "e7e5", "g2g4", "d8h4"] {
        play(&mut a, uci);
    }
    let text = pgn::export(&a.record()).unwrap();
    assert!(text.contains("Qh4# 0-1"));
    assert!(
        pgn::import(text.as_bytes())
            .unwrap()
            .positions
            .last()
            .unwrap()
            .is_checkmate()
    );
}
#[test]
fn pgn_special_moves_roundtrip() {
    for (initial, moves, needle) in [
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            vec!["e1g1", "e8c8"],
            "O-O",
        ),
        ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", vec!["e5d6"], "exd6"),
        ("7k/P7/8/8/8/8/8/4K3 w - - 0 1", vec!["a7a8q"], "a8=Q+"),
        ("7k/P7/8/8/8/8/8/4K3 w - - 0 1", vec!["a7a8n"], "a8=N"),
    ] {
        let mut a = app();
        a.import_fen(initial).unwrap();
        for m in moves {
            play(&mut a, m);
        }
        let text = pgn::export(&a.record()).unwrap();
        assert!(text.contains(needle), "{text}");
        let imported = pgn::import(text.as_bytes()).unwrap();
        assert_eq!(
            fen::write(imported.positions.last().unwrap()),
            fen::write(&a.pos)
        );
        assert_eq!(imported.record.initial_fen, a.record().initial_fen);
    }
}
#[test]
fn black_start_fullmove_number_is_preserved() {
    let mut a = app();
    a.import_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 17 42")
        .unwrap();
    play(&mut a, "e8g8");
    let text = pgn::export(&a.record()).unwrap();
    assert!(text.contains("42... O-O"));
    assert!(text.contains("[SetUp \"1\"]"));
    assert!(text.contains("[FEN \""));
    assert_eq!(
        pgn::import(text.as_bytes()).unwrap().record.initial_fen,
        a.record().initial_fen
    );
}
#[test]
fn annotations_and_variations_are_explicitly_discarded() {
    let input = b"1. e4 {hello} e5 $1 (1... c5 (1... e6)) 2. Nf3!? Nc6 *";
    let game = pgn::import(input).unwrap();
    assert!(game.record.omitted_annotations);
    assert_eq!(game.moves.len(), 4);
    let text = pgn::export(&game.record).unwrap();
    assert!(!text.contains("hello"));
    assert!(!text.contains("c5"));
    assert!(!text.contains("$1"));
}
#[test]
fn invalid_pgn_is_rejected_with_ply_and_reason() {
    let error = pgn::import(b"1. e4 e5 2. Bh6 *").err().unwrap();
    assert!(error.contains("ply 3"), "{error}");
    for text in [
        "",
        "garbage",
        "1. e4",
        "1. e4 e5 {oops *",
        "1. e4 (e5 *",
        "1. e4 ) *",
        "[Event broken] 1. e4 *",
        "[Result \"1-0\"] 1. e4 *",
        "1. e4+ *",
        "[SetUp \"1\"] *",
        "[FEN \"bad\"] *",
    ] {
        assert!(pgn::import(text.as_bytes()).is_err(), "accepted {text}");
    }
}
#[test]
fn multiple_games_are_rejected() {
    assert!(
        pgn::import(b"1. e4 *\n\n1. d4 *")
            .err()
            .unwrap()
            .contains("Multiple")
    );
}
#[test]
fn bom_crlf_and_empty_valid_game() {
    let game = pgn::import("\u{feff}[Event \"test\"]\r\n\r\n1. e4 e5 *\r\n".as_bytes()).unwrap();
    assert_eq!(game.moves.len(), 2);
    assert!(pgn::import(b"*").unwrap().moves.is_empty());
}
#[test]
fn input_limits_are_enforced() {
    assert!(pgn::import(&vec![b' '; MAX_FILE_BYTES + 1]).is_err());
    let mut game = GameRecord::new(&Chess::default(), None);
    game.moves = vec!["e2e4".into(); MAX_PLIES + 1];
    assert!(game.validate().is_err());
    assert!(pgn::import(format!("{}*{}", "(".repeat(65), ")".repeat(65)).as_bytes()).is_err());
}
#[test]
fn malformed_utf8_pgn_is_rejected() {
    assert!(pgn::import(&[0xff, 0xfe]).is_err());
}
#[test]
fn fen_six_fields_roundtrip_all_state() {
    for f in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 17 42",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 9 20",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
    ] {
        assert_eq!(fen::write(&fen::parse(f).unwrap()), f);
    }
}
#[test]
fn illegal_fens_and_counters_are_rejected() {
    for f in [
        "",
        "8/8/8/8/8/8/8/8 w - - 0 1",
        "7k/8/8/8/8/8/8/K7 w - -",
        "7k/8/8/8/8/8/8/K7 w - - 0 0",
        "7k/8/8/8/8/8/8/K7 x - - 0 1",
        "7k/8/8/8/8/8/8/K7 w KQ - 0 1",
        "7k/8/8/8/8/8/8/K7 w - e3 0 1",
        "7k/8/8/8/8/8/8/K7 w - - -1 1",
    ] {
        assert!(fen::parse(f).is_err(), "{f}");
    }
}
#[test]
fn fen_import_starts_independent_history_and_undo_from_it() {
    let mut a = app();
    play(&mut a, "d2d4");
    let f = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 42";
    a.import_fen(f).unwrap();
    assert!(a.history.is_empty());
    assert_eq!(a.clock_history.len(), 1);
    assert_eq!(a.position_keys.len(), 1);
    play(&mut a, "e8g8");
    a.undo();
    assert_eq!(fen::write(&a.pos), f);
}
#[test]
fn failed_import_does_not_mutate_live_game() {
    let mut a = app();
    play(&mut a, "e2e4");
    let before = a.snapshot();
    assert!(a.import_fen("invalid").is_err());
    let after = a.snapshot();
    assert_eq!(before.record, after.record);
    assert_eq!(before.clocks, after.clocks);
    assert_eq!(before.current_fen, after.current_fen);
}
#[test]
fn save_restore_clocks_history_mode_and_preferences() {
    let mut a = app();
    a.ai_mode = Some(Color::Black);
    a.settings.ai_depth = 4;
    a.settings.board.move_confirmation = true;
    play(&mut a, "e2e4");
    a.clocks.white = Duration::from_secs(511);
    play(&mut a, "e7e5");
    a.clocks.black = Duration::from_secs(488);
    let snapshot = a.snapshot();
    let path = temp("roundtrip").join("save.json");
    persistence::save(&path, &snapshot).unwrap();
    let loaded = persistence::load(&path).unwrap().unwrap();
    let mut restored = app();
    restored.restore_snapshot(loaded).unwrap();
    assert_eq!(fen::write(&restored.pos), fen::write(&a.pos));
    assert_eq!(restored.clocks, a.clocks);
    assert_eq!(restored.clock_history, a.clock_history);
    assert_eq!(restored.ai_mode, a.ai_mode);
    assert_eq!(restored.settings.ai_depth, 4);
    assert!(restored.settings.board.move_confirmation);
    assert_eq!(restored.position_keys, a.position_keys);
    restored.undo();
    assert!(restored.history.is_empty());
    assert_eq!(restored.clocks, ClockState::default());
}
#[test]
fn save_after_undo_restores_correct_position() {
    let mut a = app();
    for m in ["e2e4", "e7e5", "g1f3"] {
        play(&mut a, m);
    }
    a.undo();
    let saved = a.snapshot();
    let mut b = app();
    b.restore_snapshot(saved).unwrap();
    assert_eq!(b.history.len(), 2);
    assert_eq!(b.pos, a.pos);
    assert_eq!(b.clocks, a.clocks);
}
#[test]
fn save_timeout_result_and_zero_clock_survive_restart() {
    let mut a = app();
    play(&mut a, "e2e4");
    a.clocks.black = Duration::ZERO;
    a.settings.board.chess_clock_enabled = true;
    a.tick_clock_at(a.clock_last_tick);
    let saved = a.snapshot();
    assert!(saved.result.is_some());
    let mut b = app();
    b.restore_snapshot(saved).unwrap();
    assert!(b.game_over());
    assert_eq!(b.clocks.black, Duration::ZERO);
    b.undo();
    assert!(!b.game_over());
}
#[test]
fn save_checksum_and_schema_errors_are_explicit() {
    let a = app();
    let path = temp("corrupt").join("save.json");
    persistence::save(&path, &a.snapshot()).unwrap();
    let mut json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    json["sha256"] = "bad".into();
    assert!(
        persistence::decode(&serde_json::to_vec(&json).unwrap())
            .err()
            .unwrap()
            .contains("checksum")
    );
    json["schema_version"] = 2.into();
    assert!(
        persistence::decode(&serde_json::to_vec(&json).unwrap())
            .err()
            .unwrap()
            .contains("schema")
    );
    assert!(persistence::decode(b"{truncated").is_err());
}
#[test]
fn corrupt_primary_is_preserved_and_good_backup_kept() {
    let a = app();
    let path = temp("backup").join("save.json");
    persistence::save(&path, &a.snapshot()).unwrap();
    let original = std::fs::read(&path).unwrap();
    persistence::save(&path, &a.snapshot()).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("json.bak")).unwrap(),
        original
    );
    std::fs::write(&path, b"damaged").unwrap();
    assert!(persistence::save(&path, &a.snapshot()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"damaged");
    assert_eq!(
        std::fs::read(path.with_extension("json.bak")).unwrap(),
        original
    );
}
#[test]
fn save_read_and_replace_failures_preserve_valid_data() {
    let dir = temp("io-errors");
    assert!(persistence::read_limited(&dir.join("missing")).is_err());
    let target = dir.join("target-directory");
    std::fs::create_dir(&target).unwrap();
    assert!(persistence::atomic_write(&target, b"new").is_err());
    assert!(target.is_dir());
    let parent_file = dir.join("file");
    std::fs::write(&parent_file, b"original").unwrap();
    assert!(persistence::atomic_write(&parent_file.join("save.json"), b"new").is_err());
    assert_eq!(std::fs::read(parent_file).unwrap(), b"original");
}
#[test]
fn invalid_restore_is_transactional() {
    let mut a = app();
    play(&mut a, "e2e4");
    let before = a.pos.clone();
    let generation = a.generation;
    let mut save = a.snapshot();
    save.current_fen = fen::write(&Chess::default());
    assert!(a.restore_snapshot(save).is_err());
    assert_eq!(a.pos, before);
    assert_eq!(a.generation, generation);
}
#[test]
fn restored_game_rejects_old_ai_and_hint_generations() {
    let mut a = app();
    play(&mut a, "e2e4");
    let saved = a.snapshot();
    let generation = a.generation;
    let next = "e7e5"
        .parse::<shakmaty::uci::UciMove>()
        .unwrap()
        .to_move(&a.pos)
        .unwrap();
    a.restore_snapshot(saved).unwrap();
    assert!(
        a.accept_reply(SearchReply {
            generation,
            result: Ok(next)
        })
        .is_none()
    );
}
#[test]
fn replay_navigation_and_clicks_are_bounded() {
    let game = pgn::import(b"1. e4 e5 2. Nf3 Nc6 *").unwrap();
    let mut r = replay::Replay::new(game, true);
    assert_eq!(r.cursor, 4);
    r.jump(0);
    r.step(-1);
    assert_eq!(r.cursor, 0);
    r.step(1);
    assert_eq!(r.cursor, 1);
    r.jump(999);
    assert_eq!(r.cursor, 4);
    r.jump(2);
    assert_eq!(
        fen::write(&r.game.positions[2]),
        fen::write(
            pgn::import(b"1. e4 e5 *")
                .unwrap()
                .positions
                .last()
                .unwrap()
        )
    );
}
#[test]
fn replay_freezes_clock_retains_receivers_and_does_not_autosave() {
    let mut a = app();
    play(&mut a, "e2e4");
    a.settings.board.chess_clock_enabled = true;
    let generation = a.generation;
    let clocks = a.clocks;
    let live = a.pos.clone();
    let history = a.history.clone();
    let (tx, rx) = mpsc::channel();
    let mv = "e7e5"
        .parse::<shakmaty::uci::UciMove>()
        .unwrap()
        .to_move(&a.pos)
        .unwrap();
    tx.send(SearchReply {
        generation,
        result: Ok(mv),
    })
    .unwrap();
    a.hint_rx = Some(rx);
    let path = temp("replay").join("save.json");
    a.records.save_path = Some(path.clone());
    a.save_session();
    let bytes = std::fs::read(&path).unwrap();
    a.enter_replay(a.record().validate().unwrap(), false);
    let paused = a.clocks;
    a.tick_clock_at(a.clock_last_tick + Duration::from_secs(1000));
    a.poll_ai();
    a.poll_hint();
    a.records.save_last = Instant::now() - Duration::from_secs(10);
    a.periodic_save();
    assert_eq!(a.clocks, paused);
    assert_eq!(a.generation, generation);
    assert!(a.hint_rx.is_some());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    a.handle_record_action(Action::First);
    assert_eq!(a.records.replay.as_ref().unwrap().cursor, 0);
    assert_eq!(a.pos, live);
    assert_eq!(a.history, history);
    a.leave_replay();
    assert_eq!(a.screen, Screen::Game);
    assert_eq!(a.generation, generation);
    assert!(a.clocks.white <= clocks.white);
    a.poll_hint();
    assert!(a.suggestion.is_some());
}
#[test]
fn imported_replay_preserves_running_game_and_returns_to_it() {
    let mut a = app();
    play(&mut a, "e2e4");
    let before = a.pos.clone();
    let history = a.history.clone();
    a.enter_replay(pgn::import(b"1. d4 d5 *").unwrap(), true);
    let mut view = a.view_state(None, false, false, false);
    a.replay_view(&mut view);
    assert_ne!(view.pos, before);
    a.record_keys(&[Key::Home], &[], false);
    assert_eq!(a.records.replay.as_ref().unwrap().cursor, 0);
    a.record_keys(&[Key::Right], &[], false);
    assert_eq!(a.records.replay.as_ref().unwrap().cursor, 1);
    a.record_keys(&[Key::End], &[], false);
    assert_eq!(a.records.replay.as_ref().unwrap().cursor, 2);
    a.handle_record_action(Action::Back);
    assert_eq!(a.pos, before);
    assert_eq!(a.history, history);
}
#[test]
fn fen_text_input_does_not_trigger_replay_shortcuts() {
    let mut a = app();
    a.enter_replay(pgn::import(b"1. e4 e5 *").unwrap(), true);
    a.records.fen_input = Some(String::new());
    a.record_keys(&[Key::Home, Key::Left], &['a'], false);
    assert_eq!(a.records.replay.as_ref().unwrap().cursor, 2);
    assert_eq!(a.records.fen_input.as_deref(), Some("a"));
}
#[test]
fn replay_and_dialog_render_in_all_languages_and_small_windows() {
    let images = assets::PieceImages::load();
    let font = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &font);
    let mut a = app();
    a.enter_replay(pgn::import(b"1. e4 e5 2. Nf3 Nc6 *").unwrap(), true);
    for language in i18n::Language::ALL {
        a.settings.language = language;
        for size in [(640, 480), (1280, 720)] {
            let mut view = a.view_state(None, false, false, false);
            a.replay_view(&mut view);
            let mut buf = vec![0; size.0 * size.1];
            assert!(renderer.render(&mut buf, size.0, size.1, &view).is_empty());
            assert!(records_ui::draw(&font, &mut buf, size, &a, None, false).is_empty());
            a.records.message =
                Some(records_ui::label(language, records_ui::Text::MainlineOnly).into());
            assert!(records_ui::draw(&font, &mut buf, size, &a, None, false).is_empty());
            a.records.message = None;
        }
    }
}
#[test]
fn malformed_numbers_oversized_tags_and_variants_are_rejected() {
    for input in [
        "1e4 *".to_string(),
        "[SetUp \"0\"] [FEN \"7k/8/8/8/8/8/8/K7 w - - 0 1\"] *".into(),
        "[Variant \"Chess960\"] *".into(),
        format!("[Event \"{}\"] *", "x".repeat(4097)),
    ] {
        assert!(pgn::import(input.as_bytes()).is_err(), "{input}");
    }
    assert!(fen::parse("7k/8/8/8/8/8/8/K7 w - - 0 00").is_err());
}
#[test]
fn fen_editor_keeps_caret_and_replay_navigation_separate() {
    let mut a = app();
    a.handle_record_action(Action::InputFen);
    a.record_keys(&[Key::A], &[], true);
    a.record_keys(&[], &['a', 'c'], false);
    a.record_keys(&[Key::Left], &['b'], false);
    assert_eq!(a.records.fen_input.as_deref(), Some("abc"));
    a.record_keys(&[Key::Home, Key::Delete], &[], false);
    assert_eq!(a.records.fen_input.as_deref(), Some("bc"));
    a.record_keys(&[Key::End, Key::Backspace], &[], false);
    assert_eq!(a.records.fen_input.as_deref(), Some("b"));
}
#[test]
fn fen_editor_pauses_and_retains_ready_ai_result() {
    let mut a = app();
    a.ai_mode = Some(Color::White);
    let generation = a.generation;
    let mv = "e2e4"
        .parse::<shakmaty::uci::UciMove>()
        .unwrap()
        .to_move(&a.pos)
        .unwrap();
    let (tx, rx) = mpsc::channel();
    tx.send(SearchReply {
        generation,
        result: Ok(mv),
    })
    .unwrap();
    a.ai_rx = Some(rx);
    a.ai_reply_at = Some(Instant::now() - Duration::from_secs(1));
    a.records.fen_input = Some(a.display_fen());
    a.poll_ai();
    assert!(a.history.is_empty());
    assert!(a.ai_rx.is_some());
    a.handle_record_action(Action::Cancel);
    a.poll_ai();
    assert_eq!(a.history.len(), 1);
}
#[test]
fn autosave_failures_are_visible_without_repeating_blocking_dialogs() {
    let mut a = app();
    let path = temp("save-error").join("save.json");
    std::fs::write(&path, b"damaged").unwrap();
    a.records.save_path = Some(path.clone());
    a.save_session();
    assert!(a.records.message.is_some());
    assert!(a.records.save_error.is_some());
    a.records.message = None;
    a.save_session();
    assert!(a.records.message.is_none());
    assert!(a.records.save_error.is_some());
    assert_eq!(std::fs::read(path).unwrap(), b"damaged");
}
#[test]
fn menu_continuation_keeps_unsaved_live_history() {
    let mut a = app();
    play(&mut a, "e2e4");
    let before = a.pos.clone();
    a.back_to_menu();
    a.handle_record_action(Action::Resume);
    assert_eq!(a.screen, Screen::Game);
    assert_eq!(a.pos, before);
    assert_eq!(a.history.len(), 1);
}

#[test]
fn terminal_board_cannot_be_exported_or_imported_as_unfinished() {
    assert!(pgn::import(b"1. f3 e5 2. g4 Qh4# *").is_err());
    let mut a = app();
    for m in ["f2f3", "e7e5", "g2g4", "d8h4"] {
        play(&mut a, m);
    }
    let mut record = a.record();
    record.result = "*".into();
    assert!(pgn::export(&record).is_err());
}

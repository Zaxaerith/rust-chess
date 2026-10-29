//! Headless workspace regressions. Only ignored target/ data is written.
use crate::{
    position_editor::{self, Draft},
    position_workspace::{Action as A, Field, Mode, Study},
    *,
};
use shakmaty::{Piece, Setup};
fn mv(p: &Chess, u: &str) -> Move {
    u.parse::<shakmaty::uci::UciMove>()
        .unwrap()
        .to_move(p)
        .unwrap()
}
fn app() -> App {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let mut a = uci_tests::app();
    let root = std::env::current_dir()
        .unwrap()
        .join("target/v029-qa/tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
    std::fs::create_dir_all(&root).unwrap();
    a.records.save_path = Some(root.join("autosave-v1.json"));
    a
}
fn input(a: &mut App, f: Field, text: &str) {
    a.handle_workspace_action(A::Input(f));
    let i = a.workspace.input.as_mut().unwrap();
    i.text = text.into();
    i.cursor = text.len();
    a.handle_workspace_action(A::Apply);
}

#[test]
fn draft_all_pieces_empty_illegal_intermediate_drag_replacement_undo_redo() {
    let mut d = Draft::default();
    assert_eq!(d.validate().unwrap(), Chess::default());
    d.change(Setup::empty());
    assert!(d.validate().is_err());
    for (i, c) in [Color::White, Color::Black].into_iter().enumerate() {
        for (j, r) in [
            Role::King,
            Role::Queen,
            Role::Rook,
            Role::Bishop,
            Role::Knight,
            Role::Pawn,
        ]
        .into_iter()
        .enumerate()
        {
            let sq = Square::new((i * 8 + j + 16) as u32);
            d.place(sq, Some(Piece { color: c, role: r }));
            assert_eq!(
                d.setup.board.piece_at(sq),
                Some(Piece { color: c, role: r })
            );
        }
    }
    let original = d.setup.clone();
    d.relocate(Square::A3, Square::B3);
    assert!(d.setup.board.piece_at(Square::A3).is_none());
    d.undo();
    assert_eq!(d.setup, original);
    d.redo();
    assert!(d.setup.board.piece_at(Square::A3).is_none());
    d.place(Square::B3, None);
    assert!(d.setup.board.piece_at(Square::B3).is_none());
}
#[test]
fn strict_draft_fen_validation_rejects_conflicts_variants_and_counter_coercion() {
    for f in [
        "8/8/8/8/8/8/8/8 w - - 0 1",             // no kings
        "4k3/8/8/8/8/8/4K3/4K3 w - - 0 1",       // two kings
        "4k3/8/8/8/8/8/8/P3K3 w - - 0 1",        // backrank pawn
        "4k3/8/8/8/8/8/8/4K3 w K - 0 1",         // absent rook
        "4k3/8/8/8/8/8/8/R2K3R w KQ - 0 1",      // king displaced
        "4k3/8/8/8/8/8/8/4K1R1 w K - 0 1",       // K must not reinterpret g1 rook
        "4k3/8/8/8/8/8/4R3/4K3 w - - 0 1",       // nonmover in check
        "4k3/pppppppp/p7/8/8/8/8/4K3 w - - 0 1", // too much material
        "4k3/8/8/8/1b6/8/4r3/r3K3 w - - 0 1",    // impossible triple check
        "4k3/8/8/8/8/8/8/4K3 w - e6 0 1",        // EP pawn absent
        "4k3/8/8/3pP3/8/8/8/4K3 b - d6 0 1",     // EP wrong turn
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 1 1",     // pawn clock conflict
    ] {
        let s = position_editor::parse_draft(f).unwrap();
        assert!(position_editor::validate(&s).is_err(), "{f}");
    }
    for f in [
        "4k3/8/8/8/8/8/8/4K3 w - - 0 0",
        "4k3/8/8/8/8/8/8/4K3 w - - 4294967296 1",
        "4k3/8/8/8/8/8/8/4K3 w - - +1 1",
        "4k3/8/8/8/8/8/8/4K3 w KK - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w H - 0 1",
        "4k3/8/8/8/8/8/8/4K3[] w - - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - - 0 1 extra",
    ] {
        assert!(position_editor::parse_draft(f).is_err(), "{f}");
    }
    let text = "r3k2r/8/8/3pP3/8/8/8/R3K2R w KQkq d6 0 37";
    let s = position_editor::parse_draft(text).unwrap();
    let p = position_editor::validate(&s).unwrap();
    assert_eq!(fen::write(&p), text);
    assert_eq!(position_editor::write_setup(&s), text);
    let no_capturer = "4k3/8/8/3p4/8/8/8/4K3 w - d6 0 37";
    let p = position_editor::validate(&position_editor::parse_draft(no_capturer).unwrap()).unwrap();
    assert_eq!(fen::write(&p), no_capturer); // Traditional EP, not a Legal-EP hash.
}
#[test]
fn study_special_moves_navigation_branch_replacement_pgn_and_counter_overflow() {
    for (f, u) in [
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 7", "e1g1"),
        ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 9", "e5d6"),
        ("4k3/P7/8/8/8/8/8/4K3 w - - 0 12", "a7a8n"),
    ] {
        let mut s = Study::new(fen::parse(f).unwrap());
        s.play(mv(s.position(), u)).unwrap();
        let exported = pgn::export(&s.record()).unwrap();
        let read = pgn::import_annotated(exported.as_bytes()).unwrap();
        assert_eq!(read.mainline.moves, s.moves);
        assert_eq!(
            fen::write(read.mainline.positions.last().unwrap()),
            fen::write(s.position())
        );
        assert!(exported.contains("[SetUp \"1\"]"));
    }
    let mut s = Study::new(Chess::default());
    for u in ["e2e4", "e7e5", "g1f3"] {
        s.play(mv(s.position(), u)).unwrap();
    }
    s.cursor = 1;
    s.play(mv(s.position(), "c7c5")).unwrap();
    assert_eq!(s.moves.len(), 2);
    assert_eq!(s.sans, ["e4", "c5"]);
    assert!(!pgn::export(&s.record()).unwrap().contains("SetUp"));
    let mut s = Study::new(fen::parse("4k2r/8/8/8/8/8/8/R3K3 b - - 0 4294967295").unwrap());
    assert!(
        s.play(mv(s.position(), "h8h7"))
            .unwrap_err()
            .contains("overflow")
    );
    let mut s = Study::new(fen::parse("4k2r/8/8/8/8/8/8/R3K3 w - - 4294967295 1").unwrap());
    assert!(s.play(mv(s.position(), "a1a2")).is_err());
}
#[test]
fn app_draft_study_input_guards_and_isolation() {
    let mut a = app();
    let before = a.snapshot();
    a.enter_workspace(None).unwrap();
    a.handle_workspace_action(A::Analyze);
    assert_eq!(a.workspace.mode, Mode::Analyze);
    let live = fen::write(&a.pos);
    a.workspace_play(mv(a.workspace.chess().unwrap(), "e2e4"));
    assert_eq!(fen::write(&a.pos), live);
    assert_eq!(a.history, before.record.validate().unwrap().moves);
    assert_eq!(a.clocks, before.clocks);
    a.handle_workspace_action(A::First);
    a.workspace_play(mv(a.workspace.chess().unwrap(), "d2d4"));
    assert_eq!(a.workspace.study.as_ref().unwrap().sans, ["d4"]);
    a.handle_workspace_action(A::Edit);
    input(&mut a, Field::Fen, "8/8/8/8/8/8/8/8 w - - 0 1");
    a.handle_workspace_action(A::Analyze);
    assert_eq!(a.workspace.mode, Mode::Edit);
    assert!(a.workspace.error.is_some());
    assert!(a.request_analysis().is_err());
    assert_eq!(a.workspace.study.as_ref().unwrap().sans, ["d4"]);
    a.handle_workspace_action(A::Input(Field::Fen));
    let original = a.workspace.draft.text();
    a.record_keys(&[Key::Left], &[], false);
    assert_eq!(a.workspace.draft.text(), original);
    assert_eq!(a.workspace.study.as_ref().unwrap().cursor, 1);
    a.handle_workspace_action(A::CancelInput);
    a.handle_workspace_action(A::Back);
    assert!(a.workspace.pending.is_some());
    a.handle_workspace_action(A::Keep);
    assert_eq!(a.screen, Screen::PositionWorkspace);
    assert!(a.workspace.dirty);
    a.handle_workspace_action(A::Back);
    a.handle_workspace_action(A::Save);
    assert_eq!(a.screen, Screen::Game);
    assert!(!a.workspace.dirty);
    assert_eq!(a.snapshot().record, before.record);
    assert_eq!(a.snapshot().clock_history, before.clock_history);
}

#[test]
fn workspace_persistence_restores_illegal_draft_in_edit_and_preserves_other_data() {
    let mut a = app();
    let path = a.workspace_path().unwrap();
    let root = path.parent().unwrap();
    let names = [
        "autosave-v1.json",
        "puzzle-progress-v1.json",
        "repertoires-v1.json",
        "repertoire-progress-v1.json",
        "opening-stats-v1.json",
        "library-v1/index.json",
        "review-cache-v1/sentinel.json",
    ];
    for name in names {
        let p = root.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"user data sentinel").unwrap();
    }
    a.enter_workspace(None).unwrap();
    a.handle_workspace_action(A::Analyze);
    a.workspace_play(mv(a.workspace.chess().unwrap(), "e2e4"));
    a.handle_workspace_action(A::Edit);
    input(&mut a, Field::Fen, "8/8/8/8/8/8/8/8 b - - 11 29");
    a.save_workspace().unwrap();
    a.handle_workspace_action(A::Back);
    a.workspace = Default::default();
    a.enter_workspace(None).unwrap();
    assert_eq!(a.workspace.mode, Mode::Edit);
    assert!(a.workspace.draft.validate().is_err());
    assert_eq!(a.workspace.draft.setup.turn, Color::Black);
    assert_eq!(a.workspace.study.as_ref().unwrap().sans, ["e4"]);
    for name in names {
        assert_eq!(
            std::fs::read(root.join(name)).unwrap(),
            b"user data sentinel"
        );
    }
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        serde_json::from_slice::<serde_json::Value>(&bytes)
            .unwrap()
            .get("schema_version")
            .is_some()
    );
}
#[test]
fn workspace_bad_schema_corruption_and_write_failure_keep_pending_draft() {
    for contents in [b"broken".as_slice(), br#"{"schema_version":99}"#.as_slice()] {
        let mut a = app();
        let path = a.workspace_path().unwrap();
        std::fs::write(&path, contents).unwrap();
        assert!(a.enter_workspace(None).is_err());
        a.workspace.draft.place(
            Square::A3,
            Some(Piece {
                color: Color::White,
                role: Role::Queen,
            }),
        );
        a.workspace.dirty = true;
        a.handle_workspace_action(A::Back);
        a.handle_workspace_action(A::Save);
        assert!(a.workspace.pending.is_some());
        assert!(a.workspace.dirty);
        assert!(a.workspace.error.is_some());
        assert_eq!(std::fs::read(&path).unwrap(), contents);
        a.handle_workspace_action(A::Keep);
        assert_eq!(a.screen, Screen::PositionWorkspace);
    }
    let mut a = app();
    a.enter_workspace(None).unwrap();
    a.workspace.dirty = true;
    std::fs::create_dir(a.workspace_path().unwrap()).unwrap();
    assert!(a.save_workspace().is_err());
    assert!(a.workspace.dirty);
}
#[test]
fn workspace_pointer_tools_flip_drag_promotion_and_six_field_edit() {
    let mut a = app();
    a.enter_workspace(None).unwrap();
    a.handle_workspace_action(A::Empty);
    for i in 0..12 {
        a.handle_workspace_action(A::Tool(i));
        let square = Square::new((16 + i) as u32);
        a.handle_action(UiAction::PointerDown(square));
        a.handle_action(UiAction::PointerUp(Some(square)));
        assert!(a.workspace.draft.setup.board.piece_at(square).is_some());
    }
    let old = a.workspace.draft.setup.clone();
    a.handle_workspace_action(A::Flip);
    assert_eq!(a.workspace.draft.setup, old);
    a.handle_workspace_action(A::Tool(13));
    a.workspace_down(Square::A3);
    a.workspace_up(Some(Square::E5));
    assert!(a.workspace.draft.setup.board.piece_at(Square::A3).is_none());
    assert_eq!(
        a.workspace
            .draft
            .setup
            .board
            .piece_at(Square::E5)
            .unwrap()
            .role,
        Role::King
    );
    a.handle_workspace_action(A::UndoEdit);
    assert_eq!(a.workspace.draft.setup, old);
    a.handle_workspace_action(A::RedoEdit);
    assert!(a.workspace.draft.setup.board.piece_at(Square::A3).is_none());
    a.handle_workspace_action(A::Tool(12));
    a.workspace_down(Square::E5);
    a.workspace_up(Some(Square::E5));
    assert!(a.workspace.draft.setup.board.piece_at(Square::E5).is_none());
    input(&mut a, Field::Fen, "4k3/P7/8/8/8/8/8/4K3 w - - 0 1");
    a.handle_workspace_action(A::Analyze);
    for role in [Role::Knight, Role::Bishop, Role::Rook, Role::Queen] {
        a.handle_workspace_action(A::First);
        a.workspace_down(Square::A7);
        a.workspace_up(Some(Square::A8));
        assert!(a.workspace.promotion.is_some());
        a.handle_action(UiAction::Promote(role));
        assert_eq!(
            a.workspace
                .chess()
                .unwrap()
                .board()
                .piece_at(Square::A8)
                .unwrap()
                .role,
            role
        );
    }
    a.handle_workspace_action(A::Edit);
    input(&mut a, Field::Fullmove, "42");
    input(&mut a, Field::Halfmove, "17");
    input(&mut a, Field::Ep, "e3");
    a.handle_workspace_action(A::Turn);
    a.handle_workspace_action(A::Castle(0));
    assert_eq!(a.workspace.draft.setup.fullmoves.get(), 42);
    assert_eq!(a.workspace.draft.setup.halfmoves, 17);
    assert_eq!(a.workspace.draft.setup.ep_square, Some(Square::E3));
    assert_eq!(a.workspace.draft.setup.turn, Color::Black);
    assert!(a.workspace.draft.setup.castling_rights.contains(Square::H1));
    assert!(a.workspace.draft.validate().is_err());
}
#[test]
fn replay_copy_uses_actual_variation_and_one_exit_guard_without_mutating_review_identity() {
    let mut a = app();
    let tree = pgn::import_annotated(b"1. e4 (1. d4 d5) e5 *").unwrap();
    a.open_annotated_replay(tree, true);
    let r = a.records.replay.as_mut().unwrap();
    let branch = r.tree.nodes.iter().find(|n| n.san == "d4").unwrap().id;
    r.select_node(branch).unwrap();
    r.dirty = true;
    let source = a.records.replay.as_ref().unwrap().position().clone();
    let identity = a.records.replay.as_ref().unwrap().identity.clone();
    a.handle_workspace_action(A::FromCurrent);
    assert_eq!(a.screen, Screen::Replay);
    assert!(a.advanced.pending.is_some());
    assert!(a.workspace.study.is_none());
    a.handle_advanced_action(advanced_app::Action::CancelExit);
    assert!(a.advanced.pending.is_none());
    assert!(a.records.replay.as_ref().unwrap().dirty);
    a.handle_workspace_action(A::FromCurrent);
    a.handle_advanced_action(advanced_app::Action::Discard);
    assert_eq!(a.screen, Screen::PositionWorkspace);
    assert_eq!(
        fen::write(a.workspace.chess().unwrap()),
        fen::write(&source)
    );
    assert_eq!(a.workspace.study.as_ref().unwrap().moves.len(), 0);
    assert_eq!(a.records.replay.as_ref().unwrap().identity, identity);
    a.handle_workspace_action(A::Back);
    a.handle_workspace_action(A::Discard);
    assert_eq!(a.screen, Screen::Replay);
    assert_eq!(
        fen::write(a.records.replay.as_ref().unwrap().position()),
        fen::write(&source)
    );
}
fn pump(a: &mut App, predicate: impl Fn(&App) -> bool) {
    let until = Instant::now() + Duration::from_secs(6);
    loop {
        a.poll_analysis();
        if predicate(a) {
            return;
        }
        assert!(
            Instant::now() < until,
            "{:?} {:?}",
            a.analysis.report,
            a.analysis.error
        );
        std::thread::sleep(Duration::from_millis(3));
    }
}
#[test]
fn workspace_mock_multipv_cancel_and_context_expiry_never_apply_moves() {
    let mut a = app();
    a.enter_workspace(None).unwrap();
    a.handle_workspace_action(A::Analyze);
    a.analysis.panel = true;
    let (mut cfg, dir) = uci_tests::fixture("multi hold");
    cfg.multipv = 3;
    a.analysis.config = cfg;
    a.enable_analysis().unwrap();
    uci_tests::ready(a.analysis.controller.as_ref().unwrap());
    a.request_analysis().unwrap();
    pump(&mut a, |a| {
        a.analysis.variations.iter().all(|v| v.is_some())
    });
    assert_eq!(a.analysis.variations[0].as_ref().unwrap().error, None);
    assert!(a.analysis_visual().arrow.is_some());
    let first = a.analysis.sequence;
    let live = fen::write(&a.pos);
    a.workspace_play(mv(a.workspace.chess().unwrap(), "e2e4"));
    assert!(a.analysis.variations.iter().all(Option::is_none));
    a.poll_analysis();
    assert!(a.analysis.report.best.is_none());
    assert!(a.analysis_visual().arrow.is_none());
    assert_eq!(fen::write(&a.pos), live);
    a.handle_workspace_action(A::First);
    a.request_analysis().unwrap();
    assert!(a.analysis.sequence > first);
    pump(&mut a, |a| {
        a.analysis.variations.iter().all(|v| v.is_some())
    });
    a.handle_workspace_action(A::Edit);
    a.poll_analysis();
    assert!(a.analysis.report.best.is_none());
    assert!(a.analysis_visual().arrow.is_none());
    assert!(a.request_analysis().is_err());
    assert!(a.shutdown_analysis());
    assert!(
        a.analysis
            .controller
            .as_ref()
            .unwrap()
            .report()
            .pid
            .is_none()
    );
    let log = std::fs::read_to_string(dir.join("commands.log")).unwrap();
    assert!(log.contains("stop\nisready\n"));
    assert!(log.contains("quit"));
}
#[test]
fn workspace_custom_black_fen_history_score_direction_and_terminal_skip() {
    let mut a = app();
    let root = fen::parse("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 37").unwrap();
    a.enter_workspace(Some(root.clone())).unwrap();
    let (cfg, dir) = uci_tests::fixture("review");
    let m = mv(&root, "e7e5");
    let c0 = uci_protocol::position(&root, &[]);
    let c1 = uci_protocol::position(&root, std::slice::from_ref(&m));
    std::fs::write(
        dir.join("review.tsv"),
        format!("{c0}\te7e5\tcp 60 lowerbound\t20\n{c1}\tg1f3\tcp 40\t20\n"),
    )
    .unwrap();
    a.analysis.config = cfg;
    a.analysis.panel = true;
    a.enable_analysis().unwrap();
    uci_tests::ready(a.analysis.controller.as_ref().unwrap());
    a.request_analysis().unwrap();
    pump(&mut a, |a| {
        a.analysis.report.status == uci::Status::Complete
    });
    assert_eq!(
        a.analysis.report.info.score,
        Some((uci_protocol::Score::Cp(-60), uci_protocol::Bound::Upper))
    );
    a.workspace_play(m);
    a.request_analysis().unwrap();
    pump(&mut a, |a| {
        a.analysis.report.status == uci::Status::Complete
    });
    assert_eq!(
        a.analysis.report.info.score,
        Some((uci_protocol::Score::Cp(40), uci_protocol::Bound::Exact))
    );
    let terminal = fen::parse("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
    a.workspace.dirty = false;
    a.enter_workspace(Some(terminal)).unwrap();
    a.request_analysis().unwrap();
    assert_eq!(a.analysis.report.status, uci::Status::Complete);
    assert!(a.analysis.report.best.is_none());
    assert!(a.shutdown_analysis());
    let log = std::fs::read_to_string(dir.join("commands.log")).unwrap();
    assert!(log.contains(&c0));
    assert!(log.contains(&c1));
    assert_eq!(log.matches("go movetime").count(), 2);
    assert_eq!(log.matches("ucinewgame").count(), 1);
}
#[test]
fn workspace_terminal_pgn_results_black_numbering_and_no_previous_repetition_history() {
    let p = fen::parse("7k/8/5KQ1/8/8/8/8/8 w - - 0 19").unwrap();
    let mut s = Study::new(p);
    s.play(mv(s.position(), "g6g7")).unwrap();
    assert!(s.position().is_checkmate());
    let text = pgn::export(&s.record()).unwrap();
    assert!(text.contains("19. Qg7# 1-0"));
    let p = fen::parse("4k3/8/8/8/8/8/8/R3K3 b - - 0 23").unwrap();
    let mut s = Study::new(p);
    s.play(mv(s.position(), "e8f8")).unwrap();
    assert!(pgn::export(&s.record()).unwrap().contains("23... Kf8 *"));
    let s = Study::new(fen::parse("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap());
    assert_eq!(s.record().result, "1/2-1/2");
    s.record().validate().unwrap();
    let mut s = Study::new(Chess::default());
    for _ in 0..4 {
        for u in ["g1f3", "g8f6", "f3g1", "f6g8"] {
            s.play(mv(s.position(), u)).unwrap();
        }
    }
    assert_eq!(s.result(), Some(GameResult::FivefoldRepetition));
    assert_eq!(Study::new(s.position().clone()).result(), None);
}

#[test]
fn workspace_offscreen_eleven_languages_two_sizes_tools_study_engine_and_orientation() {
    let images = assets::PieceImages::load();
    let font = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &font);
    let mut a = app();
    a.enter_workspace(None).unwrap();
    a.workspace.study = Some(Study::new(Chess::default()));
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(
            position_ui::words(lang)
                .iter()
                .all(|s| !s.trim().is_empty())
        );
        for size in [(640, 480), (1280, 720)] {
            let layout =
                render::Layout::for_screen(size.0, size.1, Screen::PositionWorkspace, false);
            assert!(layout.board_left + layout.board_size + 7.0 <= size.0 as f32 - 292.0);
            assert!(layout.board_top + layout.board_size + 20.0 < size.1 as f32 - 96.0);
            for page in 0..4 {
                a.workspace.mode = if page < 2 { Mode::Edit } else { Mode::Analyze };
                a.workspace.parameters = page == 1;
                a.analysis.panel = page == 3;
                let mut view = a.view_state(None, false, false, false);
                a.workspace_view(&mut view);
                let mut buffer = vec![0; size.0 * size.1];
                assert!(
                    renderer
                        .render(&mut buffer, size.0, size.1, &view)
                        .is_empty()
                );
                let base = buffer.clone();
                analysis_ui::draw(&font, &mut buffer, size, &a, None, false);
                assert!(
                    position_ui::draw(&font, &images, &mut buffer, size, &a, None, false)
                        .is_empty()
                );
                for y in (layout.board_top as usize - 8)
                    ..(layout.board_top as usize + layout.board_size as usize + 21)
                {
                    for x in (layout.board_left as usize - 8)
                        ..(layout.board_left as usize + layout.board_size as usize + 8)
                    {
                        assert_eq!(
                            buffer[y * size.0 + x],
                            base[y * size.0 + x],
                            "{lang:?} {size:?} page {page} at {x},{y}"
                        );
                    }
                }
            }
            a.workspace.mode = Mode::Edit;
            a.analysis.panel = false;
            for flip in [false, true] {
                a.workspace.flipped = flip;
                let old = a.workspace.draft.setup.clone();
                let (x, y) = layout.square_center(Square::G1, flip);
                let mut view = a.view_state(Some((x, y)), true, true, false);
                a.workspace_view(&mut view);
                let mut buffer = vec![0; size.0 * size.1];
                let actions = renderer.render(&mut buffer, size.0, size.1, &view);
                assert!(
                    actions
                        .iter()
                        .any(|a| matches!(a, UiAction::PointerDown(Square::G1)))
                );
                assert_eq!(a.workspace.draft.setup, old);
            }
        }
    }
}
#[test]
fn workspace_offscreen_controls_and_modals_do_not_dispatch_to_live_board() {
    let images = assets::PieceImages::load();
    let font = font::TextRenderer::load();
    let mut a = app();
    a.enter_workspace(None).unwrap();
    let size = (640, 480);
    let mut buf = vec![0; 640 * 480];
    for (point, action) in [
        ((361.0, 105.0), A::Tool(0)),
        ((581.0, 149.0), A::Tool(11)),
        ((361.0, 199.0), A::Tool(12)),
        ((500.0, 199.0), A::Tool(13)),
        ((361.0, 235.0), A::Empty),
        ((500.0, 235.0), A::Starting),
        ((361.0, 269.0), A::UndoEdit),
        ((500.0, 269.0), A::RedoEdit),
        ((361.0, 303.0), A::ClearSquare),
        ((361.0, 339.0), A::Input(Field::Fen)),
        ((500.0, 339.0), A::Validate),
        ((361.0, 441.0), A::Save),
        ((500.0, 441.0), A::Back),
    ] {
        assert_eq!(
            position_ui::draw(&font, &images, &mut buf, size, &a, Some(point), true),
            vec![action]
        );
    }
    a.workspace.parameters = true;
    for (point, action) in [
        ((361.0, 105.0), A::Turn),
        ((361.0, 165.0), A::Castle(0)),
        ((427.0, 165.0), A::Castle(1)),
        ((493.0, 165.0), A::Castle(2)),
        ((559.0, 165.0), A::Castle(3)),
        ((361.0, 203.0), A::Input(Field::Ep)),
        ((361.0, 237.0), A::Input(Field::Halfmove)),
        ((361.0, 271.0), A::Input(Field::Fullmove)),
    ] {
        assert_eq!(
            position_ui::draw(&font, &images, &mut buf, size, &a, Some(point), true),
            vec![action]
        );
    }
    a.handle_workspace_action(A::Input(Field::Fen));
    assert!(
        position_ui::draw(
            &font,
            &images,
            &mut buf,
            size,
            &a,
            Some((361.0, 105.0)),
            true
        )
        .is_empty()
    );
    a.handle_workspace_action(A::Back);
    assert!(a.workspace.pending.is_some());
    let actions = position_ui::draw(
        &font,
        &images,
        &mut buf,
        size,
        &a,
        Some((40.0, 350.0)),
        true,
    );
    assert!(actions.is_empty());
    a.handle_workspace_action(A::Keep);
    a.handle_workspace_action(A::CancelInput);
    a.handle_workspace_action(A::Analyze);
    for (point, action) in [
        ((361.0, 77.0), A::Input(Field::Fen)),
        ((361.0, 111.0), A::CopyFen),
        ((361.0, 145.0), A::Export),
        ((361.0, 179.0), A::Engine),
        ((25.0, 401.0), A::First),
        ((65.0, 401.0), A::Previous),
        ((105.0, 401.0), A::Next),
        ((145.0, 401.0), A::Last),
    ] {
        assert_eq!(
            position_ui::draw(&font, &images, &mut buf, size, &a, Some(point), true),
            vec![action]
        );
    }
}
#[test]
fn workspace_copy_live_no_engine_terminal_and_review_ownership() {
    let mut a = app();
    let initial = fen::write(&a.pos);
    a.handle_workspace_action(A::FromCurrent);
    assert_eq!(a.screen, Screen::PositionWorkspace);
    assert_eq!(fen::write(a.workspace.chess().unwrap()), initial);
    assert!(a.request_analysis().is_err());
    assert!(a.analysis.controller.is_none());
    a.workspace_play(mv(a.workspace.chess().unwrap(), "e2e4"));
    assert!(a.workspace.error.is_none());
    let game = pgn::import(b"1. e4 e5 *").unwrap();
    let mut job = game_review::Job::new(
        game,
        "key".into(),
        "cache".into(),
        "mock".into(),
        game_review::Options::default(),
        1,
    );
    job.state = game_review::State::Running;
    a.review.job = Some(job);
    assert!(a.request_analysis().unwrap_err().contains("Review owns"));
    a.cancel_review();
    a.handle_workspace_action(A::Edit);
    input(&mut a, Field::Halfmove, "4294967296");
    assert!(a.workspace.error.is_some());
    assert!(a.workspace.input.is_some());
    assert_eq!(a.workspace.draft.setup.halfmoves, 0);
    a.handle_workspace_action(A::CancelInput);
}

#[test]
fn workspace_study_reselect_cancel_drag_selected_and_castle_input() {
    let mut a = app();
    a.enter_workspace(None).unwrap();
    a.handle_workspace_action(A::Analyze);
    let live = fen::write(&a.pos);
    for sq in [Square::G1, Square::E2] {
        a.handle_action(UiAction::PointerDown(sq));
        a.handle_action(UiAction::PointerUp(Some(sq)));
        assert_eq!(a.workspace.selected, Some(sq));
        assert!(a.workspace.error.is_none());
    }
    a.workspace_down(Square::E2);
    a.workspace_up(Some(Square::E2));
    assert_eq!(a.workspace.selected, None);
    a.workspace_down(Square::E2);
    a.workspace_up(Some(Square::E2));
    // Already-selected pieces still support drag, despite click-to-deselect.
    a.workspace_down(Square::E2);
    a.workspace_up(Some(Square::E4));
    assert_eq!(a.workspace.study.as_ref().unwrap().sans, ["e4"]);
    assert_eq!(fen::write(&a.pos), live);
    a.enter_workspace(Some(
        fen::parse("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap(),
    ))
    .unwrap();
    a.workspace_down(Square::E1);
    a.workspace_up(Some(Square::E1));
    a.workspace_down(Square::G1);
    a.workspace_up(Some(Square::G1));
    assert_eq!(a.workspace.study.as_ref().unwrap().sans, ["O-O"]);
}

#[test]
fn workspace_game_replay_entry_keeps_source_board_coordinates_clear() {
    let images = assets::PieceImages::load();
    let font = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &font);
    for screen in [Screen::Game, Screen::Replay] {
        let mut a = app();
        if screen == Screen::Replay {
            a.enter_replay(pgn::import(b"1. e4 e5 *").unwrap(), true);
        }
        for size in [(640, 480), (1280, 720)] {
            let mut view = a.view_state(None, false, false, false);
            a.replay_view(&mut view);
            let layout = render::Layout::for_screen(size.0, size.1, screen, false);
            let mut buf = vec![0; size.0 * size.1];
            renderer.render(&mut buf, size.0, size.1, &view);
            let before = buf.clone();
            assert!(position_ui::draw(&font, &images, &mut buf, size, &a, None, false).is_empty());
            for y in (layout.board_top as usize - 8)
                ..(layout.board_top as usize + layout.board_size as usize + 21)
            {
                for x in ((layout.board_left - 22.0).max(0.0) as usize)
                    ..(layout.board_left as usize + layout.board_size as usize + 8)
                {
                    assert_eq!(
                        buf[y * size.0 + x],
                        before[y * size.0 + x],
                        "{screen:?} {size:?} at {x},{y}"
                    );
                }
            }
            let (x, y, _, _) = position_ui::source_rect(screen, size);
            assert_eq!(
                position_ui::draw(
                    &font,
                    &images,
                    &mut buf,
                    size,
                    &a,
                    Some((x + 1.0, y + 1.0)),
                    true
                ),
                vec![A::FromCurrent]
            );
        }
    }
}

use crate::{
    puzzle_app::{Action as A, Puzzles},
    puzzle_data::{Puzzle, Validated},
    puzzle_session::{Outcome, Session, State},
    *,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
fn bank() -> Vec<Arc<Validated>> {
    static BANK: OnceLock<Vec<Arc<Validated>>> = OnceLock::new();
    BANK.get_or_init(|| puzzle_data::builtin().unwrap()).clone()
}
fn temp() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let p = std::env::current_dir()
        .unwrap()
        .join("target/v027-qa/tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    std::fs::create_dir_all(&p).unwrap();
    p
}
fn app() -> App {
    let mut a = uci_tests::app();
    let root = temp();
    a.records.save_path = Some(root.join("autosave-v1.json"));
    a.puzzles.root = Some(root);
    a.puzzles.bank = bank();
    a.screen = Screen::Puzzle;
    a.puzzles.select_next(Instant::now());
    a
}
fn fixture(fen: &str, moves: &str) -> Arc<Validated> {
    Arc::new(
        Puzzle {
            id: "QAfixture".into(),
            fen: fen.into(),
            moves: moves.split_whitespace().map(str::to_owned).collect(),
            rating: 1500,
            deviation: 50,
            popularity: 99,
            themes: vec![],
            url: None,
            row: None,
        }
        .validate()
        .unwrap(),
    )
}
fn solve(s: &mut Session, now: Instant) {
    while s.state != State::Solved {
        if s.state == State::Responding {
            s.respond();
        } else {
            let mv = s.puzzle.moves[s.next].clone();
            assert!(s.attempt(mv, now).unwrap());
        }
    }
}
#[test]
fn all_512_official_puzzles_unique_legal_semantics_and_coverage() {
    let b = bank();
    assert_eq!(b.len(), 512);
    let mut ids = std::collections::BTreeSet::new();
    let mut sides = [0; 2];
    for p in &b {
        assert!(ids.insert(&p.data.id));
        assert!(p.data.row.is_some());
        assert_eq!(p.positions.len(), p.moves.len() + 1);
        let mut pos = fen::parse(&p.data.fen).unwrap();
        pos.play_unchecked(&p.moves[0]);
        let s = Session::new(p.clone(), Instant::now());
        assert_eq!(fen::write(&s.pos), fen::write(&pos));
        assert_eq!(s.next, 1);
        assert_eq!(s.pos.turn(), s.solver());
        sides[usize::from(s.solver() == Color::Black)] += 1;
        let mut s = s;
        solve(&mut s, Instant::now());
        assert_eq!(fen::write(&s.pos), fen::write(p.positions.last().unwrap()));
        assert_eq!(s.outcome, Some(Outcome::FirstTry));
    }
    assert!(sides.iter().all(|n| *n > 0));
    for t in puzzle_app::THEMES.iter().skip(1) {
        assert!(
            b.iter()
                .filter(|p| p.data.themes.iter().any(|x| x == t))
                .count()
                >= 16,
            "{t}"
        );
    }
}
#[test]
fn correct_reply_wrong_legal_illegal_and_elapsed() {
    let p = bank().into_iter().find(|p| p.moves.len() > 4).unwrap();
    let now = Instant::now();
    let mut s = Session::new(p.clone(), now);
    let before = fen::write(&s.pos);
    let wrong = s
        .pos
        .legal_moves()
        .into_iter()
        .find(|m| m != &p.moves[1])
        .unwrap();
    assert!(!s.attempt(wrong, now).unwrap());
    assert_eq!(s.errors, 1);
    assert_eq!(fen::write(&s.pos), before);
    let illegal = Move::Normal {
        role: Role::Pawn,
        from: Square::A1,
        capture: None,
        to: Square::A8,
        promotion: None,
    };
    assert!(s.attempt(illegal, now).is_err());
    assert_eq!(s.errors, 1);
    assert!(s.attempt(p.moves[1].clone(), now).unwrap());
    assert_eq!(s.state, State::Responding);
    s.respond();
    assert_eq!(s.next, 3);
    solve(&mut s, now + Duration::from_secs(7));
    assert_eq!(s.elapsed, Duration::from_secs(7));
    assert_eq!(s.outcome, Some(Outcome::Assisted));
}
#[test]
fn mate_in_one_accepts_alternative_actual_checkmate() {
    let (p, mv) = bank()
        .into_iter()
        .filter(|p| p.data.themes.iter().any(|t| t == "mateIn1"))
        .find_map(|p| {
            let m = p.positions[1].legal_moves().into_iter().find(|m| {
                m != &p.moves[1] && {
                    let mut pos = p.positions[1].clone();
                    pos.play_unchecked(m);
                    pos.is_checkmate()
                }
            });
            m.map(|m| (p, m))
        })
        .expect("at least one verified alternate mate in official sample");
    let mut s = Session::new(p, Instant::now());
    assert!(s.attempt(mv, Instant::now()).unwrap());
    assert_eq!(s.state, State::Solved);
    assert!(s.pos.is_checkmate());
}
#[test]
fn promotions_castling_and_en_passant_use_same_rules() {
    for promotion in ["n", "q", "r", "b"] {
        let p = fixture(
            "7k/P7/8/8/8/8/8/7K b - - 0 1",
            &format!("h8g8 a7a8{promotion}"),
        );
        let mut a = app();
        a.puzzles.session = Some(Session::new(p.clone(), Instant::now()));
        a.handle_action(UiAction::PointerDown(Square::A7));
        a.handle_action(UiAction::PointerUp(Some(Square::A8)));
        assert!(a.puzzles.promotion.is_some());
        assert_eq!(a.puzzles.session.as_ref().unwrap().next, 1);
        a.handle_action(UiAction::Promote(p.moves[1].promotion().unwrap()));
        assert_eq!(a.puzzles.session.as_ref().unwrap().state, State::Solved);
    }
    for (fen, moves) in [
        ("7k/3p4/8/4P3/8/8/8/7K b - - 0 1", "d7d5 e5d6"),
        ("4k3/8/8/8/8/8/8/4K2R b K - 0 1", "e8d8 e1g1"),
    ] {
        let p = fixture(fen, moves);
        let mut s = Session::new(p, Instant::now());
        solve(&mut s, Instant::now());
        assert_eq!(s.state, State::Solved);
    }
}
#[test]
fn hints_reveal_replay_and_retry_do_not_rewrite_results() {
    let mut a = app();
    let id = a.puzzles.session.as_ref().unwrap().puzzle.data.id.clone();
    a.handle_puzzle_action(A::Hint);
    let s = a.puzzles.session.as_ref().unwrap();
    assert_eq!(s.hint_square, s.puzzle.moves[1].from());
    a.handle_puzzle_action(A::Hint);
    let s = a.puzzles.session.as_ref().unwrap();
    assert_eq!(s.hint_square, Some(puzzle_app::target(&s.puzzle.moves[1])));
    solve(a.puzzles.session.as_mut().unwrap(), Instant::now());
    a.record_puzzle();
    assert_eq!(a.puzzles.progress.entries[&id].assisted, 1);
    a.handle_puzzle_action(A::Reveal);
    a.handle_puzzle_action(A::Forward);
    a.handle_puzzle_action(A::Previous);
    assert_eq!(
        a.puzzles.session.as_ref().unwrap().outcome,
        Some(Outcome::Assisted)
    );
    a.handle_puzzle_action(A::Retry);
    assert_eq!(a.puzzles.session.as_ref().unwrap().state, State::Waiting);
    solve(a.puzzles.session.as_mut().unwrap(), Instant::now());
    a.record_puzzle();
    a.record_puzzle();
    assert_eq!(a.puzzles.progress.entries.len(), 1);
    assert_eq!(a.puzzles.progress.entries[&id].attempts, 2);
    let path = a
        .puzzles
        .root
        .as_ref()
        .unwrap()
        .join("puzzle-progress-v1.json");
    let restored = puzzle_store::progress(&path).unwrap();
    assert_eq!(restored.entries[&id].first, 1);
}
#[test]
fn filters_unseen_errors_empty_and_finite_rotation() {
    let now = Instant::now();
    let mut p = Puzzles::default();
    p.bank = bank();
    for theme in 1..puzzle_app::THEMES.len() {
        p.theme = theme;
        p.select_next(now);
        assert!(
            p.session
                .as_ref()
                .unwrap()
                .puzzle
                .data
                .themes
                .contains(&puzzle_app::THEMES[theme].into())
        );
    }
    p.theme = 0;
    p.difficulty = 1;
    p.select_next(now);
    assert!(p.session.as_ref().unwrap().puzzle.data.rating <= 1199);
    p.mode = 2;
    p.select_next(now);
    assert!(p.session.is_none());
    let first = p.bank[0].data.id.clone();
    p.difficulty = 0;
    p.progress.entries.entry(first.clone()).or_default().errors = 1;
    p.select_next(now);
    assert_eq!(p.session.as_ref().unwrap().puzzle.data.id, first);
    p.select_next(now);
    assert!(p.session.is_some());
    p.mode = 1;
    p.select_next(now);
    assert_ne!(p.session.as_ref().unwrap().puzzle.data.id, first);
    p.bank.truncate(2);
    p.mode = 0;
    p.visited.clear();
    p.select_next(now);
    let mut previous = p.session.as_ref().unwrap().puzzle.data.id.clone();
    for _ in 0..10 {
        p.select_next(now);
        let id = p.session.as_ref().unwrap().puzzle.data.id.clone();
        assert_ne!(id, previous);
        previous = id;
    }
}
#[test]
fn csv_quotes_bom_optional_unicode_duplicate_and_strict_errors() {
    let p = &bank()[0].data;
    let csv = format!(
        "\u{feff}PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes,GameUrl,OpeningTags,DailyDate\r\n{},\"{}\",{},1500,60,99,200,{},\"https://example.invalid/世界,\"\"quoted\"\"\",,\r\n",
        p.id,
        p.fen,
        p.moves.join(" "),
        p.themes.join(" ")
    );
    let got = puzzle_data::csv(csv.as_bytes()).unwrap();
    assert_eq!(got.len(), 1);
    assert!(got[0].url.as_ref().unwrap().contains("世界,\"quoted\""));
    assert!(got[0].clone().validate().is_ok());
    for data in [
        "",
        "PuzzleId\nhello",
        "\"unterminated",
        "a,a\n1,2",
        "a\n\"x\"oops",
    ] {
        assert!(puzzle_data::csv(data.as_bytes()).is_err());
    }
    assert!(puzzle_data::csv(&[255]).is_err());
    assert!(puzzle_data::csv(&vec![b' '; puzzle_data::MAX_BYTES + 1]).is_err());
}
#[test]
fn malformed_puzzles_limits_unique_and_cancellation() {
    let p = bank()[0].data.clone();
    for what in 0..6 {
        let mut x = p.clone();
        match what {
            0 => x.fen = "bad".into(),
            1 => x.moves[1] = "a1a8".into(),
            2 => x.id = "../bad".into(),
            3 => x.moves.truncate(1),
            4 => x.moves = vec!["e2e4".into(); 34],
            _ => x.themes = vec!["control\n".into()],
        };
        assert!(x.validate().is_err());
    }
    let flag = AtomicBool::new(false);
    let count = AtomicUsize::new(0);
    assert!(puzzle_data::validate_set(vec![p.clone(), p.clone()], &flag, &count).is_err());
    flag.store(true, Ordering::Relaxed);
    assert!(puzzle_data::validate_set(vec![p], &flag, &count).is_err());
    assert!(
        puzzle_data::csv(
            format!(
                "PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes\n{}",
                "x,x,x,1,1,1,1,x\n".repeat(20_001)
            )
            .as_bytes()
        )
        .is_err()
    );
}
fn csv_for(p: &Puzzle, id: &str) -> String {
    format!(
        "PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes,GameUrl\r\n{id},\"{}\",{},1500,60,99,100,{},\"https://example.invalid/test,source\"\r\n",
        p.fen,
        p.moves.join(" "),
        p.themes.join(" ")
    )
}
#[test]
fn csv_import_dedup_restart_atomic_failure_and_original_bytes() {
    let root = temp();
    let path = root.join("source.csv");
    let p = bank()[0].data.clone();
    let text = csv_for(&p, "UserPuzzle1");
    std::fs::write(&path, &text).unwrap();
    let flag = AtomicBool::new(false);
    let progress = AtomicUsize::new(0);
    let first = puzzle_store::import(&root, &path, &flag, &progress).unwrap();
    assert_eq!((first.added, first.duplicates), (1, 0));
    assert_eq!(
        puzzle_store::load_pack(&root, &flag, &progress)
            .unwrap()
            .len(),
        1
    );
    let again = puzzle_store::import(&root, &path, &flag, &progress).unwrap();
    assert_eq!((again.added, again.duplicates), (0, 1));
    assert_eq!(std::fs::read(&path).unwrap(), text.as_bytes());
    let pack = puzzle_store::pack_path(&root);
    let before = std::fs::read(&pack).unwrap();
    std::fs::write(
        &path,
        csv_for(&p, "UserPuzzle2").replace(&p.moves.join(" "), "a1a8 a8a1"),
    )
    .unwrap();
    let e = puzzle_store::import(&root, &path, &flag, &progress)
        .err()
        .unwrap();
    assert!(e.contains("UserPuzzle2") && e.contains("0 archived"));
    assert_eq!(std::fs::read(&pack).unwrap(), before);
    flag.store(true, Ordering::Relaxed);
    std::fs::write(&path, text).unwrap();
    assert!(puzzle_store::import(&root, &path, &flag, &progress).is_err());
    assert_eq!(std::fs::read(&pack).unwrap(), before);
    let other = temp();
    std::fs::create_dir(other.join("puzzles-v1")).unwrap();
    std::fs::create_dir(puzzle_store::pack_path(&other)).unwrap();
    flag.store(false, Ordering::Relaxed);
    assert!(puzzle_store::import(&other, &path, &flag, &progress).is_err());
    assert!(puzzle_store::pack_path(&other).is_dir());
}
#[test]
fn progress_corruption_and_write_failure_preserve_original() {
    let root = temp();
    let path = root.join("puzzle-progress-v1.json");
    std::fs::write(&path, b"damaged-user-progress").unwrap();
    assert!(puzzle_store::progress(&path).is_err());
    assert!(puzzle_store::save_progress(&path, &Default::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"damaged-user-progress");
    let mut a = app();
    a.puzzles.root = Some(root);
    a.handle_puzzle_action(A::Reveal);
    assert!(a.puzzles.progress_error);
    assert!(!a.puzzles.session.as_ref().unwrap().recorded);
    assert_eq!(std::fs::read(&path).unwrap(), b"damaged-user-progress");
    let root = temp();
    let path = root.join("puzzle-progress-v1.json");
    std::fs::create_dir(&path).unwrap();
    assert!(puzzle_store::save_progress(&path, &Default::default()).is_err());
    assert!(path.is_dir());
}
#[test]
fn puzzle_inputs_and_all_controls_isolated_from_live_game_and_files() {
    let mut a = app();
    a.settings.board.chess_clock_enabled = true;
    let root = a.puzzles.root.clone().unwrap();
    let sentinels = [
        "autosave-v1.json",
        "library-v1/index.json",
        "opening-stats-v1.json",
        "review-cache-v1/sentinel",
        "settings.cfg",
    ];
    for name in sentinels {
        let p = root.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"user-sentinel").unwrap();
    }
    let live = fen::write(&a.pos);
    let history = a.history.clone();
    let clocks = a.clocks;
    let record = a.record();
    let opening = fen::write(a.opening.position());
    let generation = a.generation;
    let mv = a.puzzles.session.as_ref().unwrap().puzzle.moves[1].clone();
    a.handle_action(UiAction::PointerDown(mv.from().unwrap()));
    a.handle_action(UiAction::PointerUp(Some(puzzle_app::target(&mv))));
    a.poll_puzzles();
    for action in [
        A::Hint,
        A::Themes,
        A::Reveal,
        A::Forward,
        A::Previous,
        A::Retry,
        A::Next,
        A::Theme,
        A::Difficulty,
        A::Mode,
        A::Details,
        A::Cancel,
    ] {
        a.handle_record_action(records_ui::Action::Puzzle(action));
        a.records.message = None;
    }
    a.handle_record_action(records_ui::Action::Puzzle(A::Back));
    assert_eq!(a.screen, Screen::Menu);
    assert_eq!(fen::write(&a.pos), live);
    assert_eq!(a.history, history);
    assert_eq!(a.clocks, clocks);
    assert_eq!(a.record(), record);
    assert_eq!(fen::write(a.opening.position()), opening);
    assert_eq!(a.generation, generation);
    assert!(a.analysis.controller.is_none());
    for name in sentinels {
        assert_eq!(std::fs::read(root.join(name)).unwrap(), b"user-sentinel");
    }
}
#[test]
fn worker_load_import_cancel_and_shutdown_bounded() {
    let mut a = app();
    a.puzzles.bank.clear();
    a.puzzles.session = None;
    a.start_puzzle_load(None).unwrap();
    assert!(a.start_puzzle_load(None).is_err());
    let until = Instant::now() + Duration::from_secs(6);
    while a.puzzles.busy() {
        a.poll_puzzles();
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(a.puzzles.bank.len(), 512);
    let path = a.puzzles.root.as_ref().unwrap().join("pack.csv");
    std::fs::write(&path, csv_for(&bank()[0].data, "ImportedWorker")).unwrap();
    a.puzzles.session = None; // Loading into an empty filtered/training view must still show the import report.
    a.start_puzzle_load(Some(path.clone())).unwrap();
    while a.puzzles.busy() {
        a.poll_puzzles();
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(a.puzzles.bank.len(), 513);
    assert!(a.puzzles.notice.as_ref().unwrap().contains("+1"));
    a.start_puzzle_load(Some(path)).unwrap();
    a.handle_puzzle_action(A::Cancel);
    assert!(a.puzzles.shutdown(Duration::from_secs(2)));
    assert!(!a.puzzles.busy());
}
#[test]
fn eleven_languages_offscreen_board_orientation_geometry_actions() {
    let mut a = app();
    let images = assets::PieceImages::load();
    let text = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &text);
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(puzzle_ui::words(lang).iter().all(|s| !s.is_empty()));
        for size in [(640, 480), (1280, 720)] {
            for side in [Color::White, Color::Black] {
                let p = bank().into_iter().find(|p| p.solver == side).unwrap();
                a.puzzles.session = Some(Session::new(p, Instant::now()));
                let mut v = a.view_state(None, false, false, false);
                a.puzzle_view(&mut v);
                assert_eq!(v.settings.mode, Some(!side));
                assert!(!v.analysis.active);
                let mut buf = vec![0; size.0 * size.1];
                renderer.render(&mut buf, size.0, size.1, &v);
                let baseline = buf.clone();
                let layout = render::Layout::for_screen(size.0, size.1, Screen::Puzzle, false);
                let actions = puzzle_ui::draw(&text, &mut buf, size, &a, None, false);
                assert!(actions.is_empty());
                for y in layout.board_top as usize..(layout.board_top + layout.board_size) as usize
                {
                    for x in
                        layout.board_left as usize..(layout.board_left + layout.board_size) as usize
                    {
                        assert_eq!(buf[y * size.0 + x], baseline[y * size.0 + x]);
                    }
                }
                // Same production geometry, no OS input/window operation.
                let mv = a.puzzles.session.as_ref().unwrap().puzzle.moves[1].clone();
                let center = layout.square_center(mv.from().unwrap(), side == Color::Black);
                v.mouse = Some(center);
                v.mouse_pressed = true;
                let actions = renderer.render(&mut buf, size.0, size.1, &v);
                assert!(
                    actions
                        .iter()
                        .any(|a| matches!(a,UiAction::PointerDown(sq) if Some(*sq)==mv.from()))
                );
            }
            let x = size.0 as f32 - 292.0;
            for (i, expected) in [
                A::Hint,
                A::Reveal,
                A::Retry,
                A::Next,
                A::Previous,
                A::Forward,
                A::Import,
                A::Details,
            ]
            .into_iter()
            .enumerate()
            {
                let actions = puzzle_ui::draw(
                    &text,
                    &mut buf_for(size),
                    size,
                    &a,
                    Some((
                        x + 12.0 + (i % 2) as f32 * 138.0,
                        284.0 + (i / 2) as f32 * 32.0,
                    )),
                    true,
                );
                assert!(
                    actions
                        .iter()
                        .any(|r| matches!(r,records_ui::Action::Puzzle(got) if *got==expected))
                );
            }
        }
    }
}
fn buf_for(size: (usize, usize)) -> Vec<u32> {
    vec![0; size.0 * size.1]
}
#[test]
fn completed_revealed_skipped_counts_and_unsaved_entry_guard() {
    let mut a = app();
    let id = a.puzzles.session.as_ref().unwrap().puzzle.data.id.clone();
    a.handle_puzzle_action(A::Reveal);
    assert_eq!(a.puzzles.progress.entries[&id].revealed, 1);
    assert_eq!(a.puzzles.completed, 0);
    a.handle_puzzle_action(A::Retry);
    solve(a.puzzles.session.as_mut().unwrap(), Instant::now());
    a.record_puzzle();
    assert_eq!(a.puzzles.completed, 1);
    a.handle_puzzle_action(A::Retry);
    a.handle_puzzle_action(A::Next);
    assert_eq!(a.puzzles.completed, 1);
    assert_eq!(a.puzzles.progress.entries[&id].skipped, 1);
    let game = pgn::import_annotated(b"1. e4 e5 *").unwrap();
    a.open_annotated_replay(game, true);
    a.records.replay.as_mut().unwrap().dirty = true;
    let before = fen::write(a.records.replay.as_ref().unwrap().position());
    a.handle_puzzle_action(A::Enter);
    assert!(a.advanced.pending.is_some());
    assert_eq!(a.screen, Screen::Replay);
    assert_eq!(
        fen::write(a.records.replay.as_ref().unwrap().position()),
        before
    );
}
#[test]
fn empty_answer_hints_promotion_and_all_filter_buttons_offscreen() {
    let mut a = app();
    let images = assets::PieceImages::load();
    let text = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &text);
    for size in [(640, 480), (1280, 720)] {
        let x = size.0 as f32 - 292.0;
        for (i, expected) in [A::Theme, A::Difficulty, A::Mode, A::Themes]
            .into_iter()
            .enumerate()
        {
            let mut buf = buf_for(size);
            let got = puzzle_ui::draw(
                &text,
                &mut buf,
                size,
                &a,
                Some((x + 12.0, 36.0 + i as f32 * 32.0)),
                true,
            );
            assert!(
                got.iter()
                    .any(|g| matches!(g,records_ui::Action::Puzzle(a) if *a==expected))
            );
        }
        for state in 0..4 {
            match state {
                0 => a.puzzles.session = None,
                1 => a.puzzles.select_next(Instant::now()),
                2 => a.handle_puzzle_action(A::Hint),
                _ => a.handle_puzzle_action(A::Reveal),
            };
            let mut v = a.view_state(None, false, false, false);
            a.puzzle_view(&mut v);
            let mut buf = buf_for(size);
            renderer.render(&mut buf, size.0, size.1, &v);
            puzzle_ui::draw(&text, &mut buf, size, &a, None, false);
        }
        let p = fixture("7k/P7/8/8/8/8/8/7K b - - 0 1", "h8g8 a7a8n");
        a.puzzles.session = Some(Session::new(p, Instant::now()));
        a.puzzles.promotion = Some((Square::A7, Square::A8));
        let mut v = a.view_state(None, false, false, false);
        a.puzzle_view(&mut v);
        let layout = render::Layout::for_screen(size.0, size.1, Screen::Puzzle, false);
        let bx = layout.board_left;
        let by = layout.board_top + (layout.board_size - 230.0) / 2.0;
        let bw = (layout.board_size - 40.0) / 4.0;
        let total = bw * 4.0 + 24.0;
        let sx = bx + (layout.board_size - total) / 2.0;
        for (i, role) in [Role::Queen, Role::Rook, Role::Bishop, Role::Knight]
            .into_iter()
            .enumerate()
        {
            let center = (sx + i as f32 * (bw + 8.0) + bw / 2.0, by + 90.0);
            assert!(center.0 < x);
            v.mouse = Some(center);
            v.mouse_pressed = true;
            let mut buf = buf_for(size);
            let actions = renderer.render(&mut buf, size.0, size.1, &v);
            let baseline = buf.clone();
            puzzle_ui::draw(&text, &mut buf, size, &a, None, false);
            assert!(
                actions
                    .iter()
                    .any(|a| matches!(a,UiAction::Promote(r) if *r==role))
            );
            for y in by as usize..(by + 230.0) as usize {
                for xx in bx as usize..(bx + layout.board_size) as usize {
                    assert_eq!(buf[y * size.0 + xx], baseline[y * size.0 + xx]);
                }
            }
        }
        a.puzzles.promotion = None;
    }
}

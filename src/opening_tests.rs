use crate::{
    library::Library,
    opening_app::{Action as A, Status},
    opening_book::Book,
    opening_stats::{self as stats, Stats},
    *,
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
fn book() -> Arc<Book> {
    static B: OnceLock<Arc<Book>> = OnceLock::new();
    B.get_or_init(|| opening_app::shared_book(&AtomicBool::new(false)).unwrap())
        .clone()
}
fn temp() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let p = std::env::current_dir()
        .unwrap()
        .join("target/v026-qa/tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn game(pgn: &str) -> annotated_game::AnnotatedGame {
    pgn::import_annotated(pgn.as_bytes()).unwrap()
}
fn fixture() -> Library {
    let mut lib = Library::open(temp().join("library-v1")).unwrap();
    for (i, result) in [
        "1-0", "1-0", "1-0", "0-1", "0-1", "1/2-1/2", "1/2-1/2", "*", "*", "*",
    ]
    .iter()
    .enumerate()
    {
        lib.add(&game(&format!(
            "[Event \"Game {i}\"]\n\n1. e4 (1. d4 d5) e5 2. Nf3 {result}"
        )))
        .unwrap();
    }
    lib
}
fn build(lib: &Library) -> Stats {
    stats::build(lib, &book(), &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap()
}
fn app(lib: Library) -> App {
    let mut a = uci_tests::app();
    assert!(a.opening.book.is_none());
    a.library.root = Some(lib.root.clone());
    a.records.save_path = Some(lib.root.parent().unwrap().join("autosave-v1.json"));
    a.library.store = Some(lib);
    a.opening.book = Some(book());
    a.update_opening_members();
    a
}
fn pump(a: &mut App) {
    let until = Instant::now() + Duration::from_secs(8);
    while a.opening.busy() {
        a.poll_opening();
        assert!(
            Instant::now() < until,
            "{:?} {:?}",
            a.opening.status,
            a.opening.error
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn official_sources_hashes_all_3865_legal_san_routes_and_mates() {
    let proof: serde_json::Value =
        serde_json::from_str(include_str!("../data/chess-openings/provenance.json")).unwrap();
    assert_eq!(proof["commit"], opening_book::SOURCE);
    assert_eq!(proof["license"], "CC0-1.0");
    assert_eq!(proof["files"].as_array().unwrap().len(), 6);
    for (i, text) in opening_book::TSV.iter().enumerate() {
        assert_eq!(
            library::digest(text.as_bytes()),
            proof["files"][i]["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(
        include_bytes!("../data/chess-openings/COPYING.txt"),
        include_bytes!("../LICENSES/chess-openings-CC0-1.0.txt")
    );
    let b = book();
    assert_eq!(b.entries.len(), 3865);
    for e in &b.entries {
        let mut pos = Chess::default();
        for m in &e.moves {
            assert!(pos.is_legal(m));
            pos.play_unchecked(m);
        }
        assert_eq!(game::position_key(&pos), e.key);
    }
    assert!(
        b.entries
            .iter()
            .filter(|e| e.name.contains("Mate"))
            .any(|e| {
                let mut p = Chess::default();
                for m in &e.moves {
                    p.play_unchecked(m);
                }
                p.is_checkmate()
            })
    );
}
#[test]
fn recognition_uses_position_transposition_exact_history_and_stable_ambiguity() {
    let b = book();
    let g = game("1. e4 e5 2. Nf3 Nc6 3. Bb5 *");
    let m = b.classify(&g.mainline).unwrap();
    assert!(b.entries[m.primary].name.contains("Ruy Lopez"));
    let first = game("1. d4 Nf6 2. c4 e6 3. Nf3 d5 *");
    let second = game("1. d4 d5 2. c4 e6 3. Nf3 Nf6 *");
    let p = first.mainline.positions.last().unwrap();
    assert_eq!(
        game::position_key(p),
        game::position_key(second.mainline.positions.last().unwrap())
    );
    assert_eq!(
        b.identify(p, &first.mainline.moves).map(|m| m.candidates),
        b.identify(p, &second.mainline.moves).map(|m| m.candidates)
    );
    let alternate = game("1. e4 Nc6 2. Nf3 e5 3. Bb5 *");
    let end = g.mainline.positions.last().unwrap();
    assert_eq!(
        game::position_key(end),
        game::position_key(alternate.mainline.positions.last().unwrap())
    );
    let transposed = b.identify(end, &alternate.mainline.moves).unwrap();
    assert_eq!(transposed.primary, m.primary);
    assert!(!transposed.exact);
    // The pinned dataset has unique terminal keys. Exercise ambiguity with a
    // controlled index of legal alternative histories, without changing data.
    let key = game::position_key(end);
    let entry = |name: &str, record: &game_record::ValidatedRecord| opening_book::Entry {
        eco: "C60".into(),
        name: name.into(),
        moves: record.moves.clone(),
        ucis: record.record.moves.clone(),
        key: key.clone(),
    };
    let controlled = Book {
        entries: vec![
            entry("fixture first", &g.mainline),
            entry("fixture alternate", &alternate.mainline),
            entry("fixture tie", &alternate.mainline),
        ],
        positions: std::collections::BTreeMap::from([(key, vec![0, 1, 2])]),
        next: std::collections::BTreeMap::new(),
    };
    assert_eq!(controlled.identify(end, &[]).unwrap().primary, 0);
    let exact = controlled.identify(end, &alternate.mainline.moves).unwrap();
    assert_eq!(exact.primary, 1);
    assert_eq!(exact.candidates, 3);
    assert!(exact.exact);
    assert!(b.identify(&Chess::default(), &[]).is_none());
    assert!(b.classify(&game("[ECO \"C99\"]\n\n*").mainline).is_none());
    let special = game(
        "[SetUp \"1\"]\n[FEN \"rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2\"]\n\n2. Nf3 Nc6 3. Bb5 *",
    );
    assert!(b.classify(&special.mainline).is_none());
    assert!(
        b.identify(
            special.mainline.positions.last().unwrap(),
            &special.mainline.moves
        )
        .is_some()
    );
    assert!(b.search("ruy LOPEZ").len() > 10);
    assert!(b.search("C60").iter().all(|i| b.entries[*i].eco == "C60"));
}
#[test]
fn ten_game_counts_results_denominators_mainline_only_and_actual_next_moves() {
    let lib = fixture();
    let s = build(&lib);
    let root = s.nodes.get(&game::position_key(&Chess::default())).unwrap();
    assert_eq!(
        (
            root.counts.white,
            root.counts.draws,
            root.counts.black,
            root.counts.unfinished
        ),
        (3, 2, 2, 3)
    );
    assert_eq!(root.counts.total(), 10);
    assert_eq!(root.counts.decided(), 7);
    let p = root.counts.percentages().unwrap();
    assert!((p[0] - 300.0 / 7.0).abs() < 1e-9);
    assert!((p.iter().sum::<f64>() - 100.0).abs() < 1e-9);
    assert_eq!(root.moves.len(), 1);
    assert_eq!(root.moves["e2e4"].total(), 10);
    let d4 = game("1. d4 *");
    assert!(
        !s.nodes
            .contains_key(&game::position_key(d4.mainline.positions.last().unwrap()))
    );
    for (key, node) in &s.nodes {
        let pos = fen::parse(&format!("{key} 0 1")).unwrap();
        for m in node.moves.keys() {
            assert!(uci_protocol::legal_move(&pos, m).unwrap().is_some());
        }
    }
    let e4 = game("1. e4 *");
    let node = &s.nodes[&game::position_key(e4.mainline.positions.last().unwrap())];
    assert_eq!(node.moves["e7e5"].total(), 10);
}
#[test]
fn repeated_positions_first_next_transpositions_nonstandard_and_prefix_40() {
    let mut lib = Library::open(temp().join("library-v1")).unwrap();
    let first = game("1. d4 Nf6 2. c4 e6 3. Nf3 d5 *");
    let second = game("1. d4 d5 2. c4 e6 3. Nf3 Nf6 *");
    lib.add(&first).unwrap();
    lib.add(&second).unwrap();
    let mut r = game_record::GameRecord::new(&Chess::default(), None);
    r.moves = ["g1f3", "g8f6", "f3g1", "f6g8"]
        .into_iter()
        .cycle()
        .take(40)
        .map(str::to_owned)
        .collect();
    r.moves.push("e2e4".into());
    let long = annotated_game::AnnotatedGame::from_linear(r.validate().unwrap());
    let long_id = lib.add(&long).unwrap().unwrap();
    let special = game(
        "[SetUp \"1\"]\n[FEN \"rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2\"]\n\n2. Nf3 *",
    );
    lib.add(&special).unwrap();
    let s = build(&lib);
    let node = &s.nodes[&game::position_key(first.mainline.positions.last().unwrap())];
    assert_eq!(node.counts.total(), 2);
    let root = &s.nodes[&game::position_key(&Chess::default())];
    assert_eq!(root.games[&long_id].ply, 0);
    assert_eq!(root.games[&long_id].next.as_deref(), Some("g1f3"));
    assert_eq!(root.counts.total(), 3);
    let after41 = game::position_key(long.mainline.positions.last().unwrap());
    assert!(
        !s.nodes
            .get(&after41)
            .is_some_and(|n| n.games.contains_key(&long_id))
    );
    let special_node = &s.nodes[&game::position_key(&special.mainline.positions[0])];
    assert_eq!(special_node.counts.nonstandard, 1);
}
#[test]
fn empty_library_and_unfinished_only_never_fabricate_global_win_rates() {
    let lib = Library::open(temp().join("library-v1")).unwrap();
    let s = build(&lib);
    assert!(s.nodes.is_empty());
    let mut a = app(lib);
    a.opening.stats = Some(s);
    a.opening.refresh();
    assert!(!a.opening.choices.is_empty());
    assert!(a.opening.choices.iter().all(|m| m.counts.is_none()));
    let id = a
        .library
        .store
        .as_mut()
        .unwrap()
        .add(&game("1. e4 *"))
        .unwrap()
        .unwrap();
    let s = build(a.library.store.as_ref().unwrap());
    assert!(
        s.nodes[&game::position_key(&Chess::default())]
            .counts
            .percentages()
            .is_none()
    );
    assert!(s.classified.contains_key(&id));
}
#[test]
fn cache_roundtrip_favorite_stability_and_membership_content_invalidation() {
    let mut lib = fixture();
    let path = lib.root.parent().unwrap().join("opening-stats-v1.json");
    let s = build(&lib);
    stats::save(&path, &s).unwrap();
    let restored = stats::load(&path, &lib, &book(), &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert_eq!(restored.nodes.len(), s.nodes.len());
    let before = stats::identity(&lib);
    let id = lib.entries[0].id.clone();
    lib.favorite(&id).unwrap();
    assert_eq!(stats::identity(&lib), before);
    assert!(
        stats::load(&path, &lib, &book(), &AtomicBool::new(false))
            .unwrap()
            .is_some()
    );
    let mut g = lib.load(&id).unwrap();
    g.nodes[1].after.push("new metadata".into());
    lib.save(&id, &g).unwrap();
    assert_ne!(stats::identity(&lib), before);
    assert!(
        stats::load(&path, &lib, &book(), &AtomicBool::new(false))
            .unwrap()
            .is_none()
    );
    let before = stats::identity(&lib);
    let add = lib.add(&game("1. d4 *")).unwrap().unwrap();
    assert_ne!(stats::identity(&lib), before);
    lib.remove(&add).unwrap();
    assert_eq!(stats::identity(&lib), before);
    let count = lib.entries.len();
    fs::write(&path, b"corrupt cache").unwrap();
    assert!(stats::load(&path, &lib, &book(), &AtomicBool::new(false)).is_err());
    assert_eq!(
        Library::open(lib.root.clone()).unwrap().entries.len(),
        count
    );
    assert_eq!(fs::read(path).unwrap(), b"corrupt cache");
}
#[test]
fn worker_build_load_cancel_and_shutdown_preserve_previous_cache() {
    let lib = fixture();
    let mut a = app(lib);
    a.screen = Screen::Opening;
    a.start_opening(true).unwrap();
    assert!(a.start_opening(true).is_err());
    pump(&mut a);
    assert_eq!(a.opening.status, Status::Ready);
    let path = a
        .library_root()
        .unwrap()
        .parent()
        .unwrap()
        .join("opening-stats-v1.json");
    let bytes = fs::read(&path).unwrap();
    a.opening.stats = None;
    a.start_opening(false).unwrap();
    pump(&mut a);
    assert!(a.opening.cached);
    assert_eq!(a.opening.status, Status::Ready);
    a.start_opening(true).unwrap();
    a.opening.cancel();
    pump(&mut a);
    assert_eq!(a.opening.status, Status::Cancelled);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    a.start_opening(true).unwrap();
    assert!(a.opening.shutdown(Duration::from_secs(1)));
    assert!(!a.opening.busy());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let preserved = path.with_extension("preserved.json");
    fs::rename(&path, &preserved).unwrap();
    fs::create_dir(&path).unwrap();
    a.start_opening(true).unwrap();
    pump(&mut a);
    assert_eq!(a.opening.status, Status::Ready);
    assert!(a.opening.error.is_some());
    assert!(a.opening.current_stats().is_some());
    assert_eq!(fs::read(&preserved).unwrap(), bytes);
    assert!(fs::read_dir(path.parent().unwrap()).unwrap().all(|p| {
        !p.unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".pending")
    }));
}
#[test]
fn changed_library_snapshot_and_failed_game_cannot_replace_good_cache() {
    let lib = fixture();
    let mut a = app(lib);
    a.start_opening(true).unwrap();
    pump(&mut a);
    let path = a
        .library_root()
        .unwrap()
        .parent()
        .unwrap()
        .join("opening-stats-v1.json");
    let bytes = fs::read(&path).unwrap();
    a.start_opening(true).unwrap();
    let mut other = Library::open(a.library_root().unwrap()).unwrap();
    other.add(&game("1. d4 d5 *")).unwrap();
    pump(&mut a);
    assert_eq!(a.opening.status, Status::Stale);
    assert!(a.opening.current_stats().is_none());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    a.start_opening(true).unwrap();
    pump(&mut a);
    assert_eq!(a.opening.status, Status::Ready);
    let bytes = fs::read(&path).unwrap();
    let entry = a.library.store.as_ref().unwrap().entries[0].clone();
    fs::remove_file(a.library_root().unwrap().join("games").join(entry.file())).unwrap();
    a.start_opening(true).unwrap();
    pump(&mut a);
    assert_eq!(a.opening.status, Status::Failed);
    assert!(a.opening.error.as_ref().unwrap().contains(&entry.id));
    assert_eq!(fs::read(path).unwrap(), bytes);
}
#[test]
fn explorer_navigation_associated_replay_unsaved_guard_and_live_isolation() {
    let lib = fixture();
    let mut a = app(lib);
    a.opening.stats = Some(build(a.library.store.as_ref().unwrap()));
    let data = a.library_root().unwrap().parent().unwrap().to_path_buf();
    fs::write(data.join("autosave-v1.json"), b"autosave sentinel").unwrap();
    fs::create_dir(data.join("review-cache-v1")).unwrap();
    fs::write(
        data.join("review-cache-v1/existing.review.json"),
        b"review sentinel",
    )
    .unwrap();
    a.screen = Screen::Menu;
    let before = a.snapshot();
    a.enter_opening().unwrap();
    a.opening.refresh();
    assert!(a.analysis.controller.is_none());
    let e4 = a
        .opening
        .choices
        .iter()
        .position(|c| c.uci == "e2e4")
        .unwrap();
    a.handle_opening_action(A::Choose(e4));
    let key = game::position_key(a.opening.position());
    let path = a.opening.path.clone();
    assert_eq!(a.opening.cursor, 1);
    a.handle_opening_action(A::Previous);
    assert_eq!(a.opening.cursor, 0);
    a.handle_opening_action(A::Next);
    assert_eq!(a.opening.cursor, 1);
    a.tick_clock_at(a.clock_last_tick + Duration::from_secs(60));
    assert_eq!(a.clocks, before.clocks);
    a.handle_opening_action(A::Game(0));
    assert_eq!(a.screen, Screen::Replay);
    assert_eq!(a.records.replay.as_ref().unwrap().cursor, 1);
    assert_eq!(
        game::position_key(a.records.replay.as_ref().unwrap().position()),
        key
    );
    a.records.replay.as_mut().unwrap().dirty = true;
    a.leave_replay();
    assert!(a.advanced.pending.is_some());
    a.handle_advanced_action(advanced_app::Action::CancelExit);
    assert_eq!(a.screen, Screen::Replay);
    a.leave_replay();
    a.handle_advanced_action(advanced_app::Action::Discard);
    assert_eq!(a.screen, Screen::Opening);
    assert_eq!(a.opening.path, path);
    assert_eq!(a.snapshot().record, before.record);
    assert_eq!(a.clocks, before.clocks);
    assert_eq!(
        fs::read(data.join("autosave-v1.json")).unwrap(),
        b"autosave sentinel"
    );
    assert_eq!(
        fs::read(data.join("review-cache-v1/existing.review.json")).unwrap(),
        b"review sentinel"
    );
    let reference = a.opening.book.as_ref().unwrap().search("C60")[0];
    a.handle_opening_action(A::Reference(reference));
    assert!(a.opening.named.is_some());
    a.handle_opening_action(A::First);
    assert_eq!(a.opening.cursor, 0);
}
#[test]
fn offscreen_eleven_languages_two_sizes_do_not_paint_over_board_or_accept_moves() {
    let mut a = app(fixture());
    a.opening.stats = Some(build(a.library.store.as_ref().unwrap()));
    a.opening.status = Status::Ready;
    a.opening.refresh();
    a.screen = Screen::Opening;
    let images = assets::PieceImages::load();
    let text = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &text);
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(opening_ui::words(lang).iter().all(|s| !s.is_empty()));
        for size in [(640, 480), (1280, 720)] {
            let mut view = a.view_state(Some((100.0, 100.0)), true, true, true);
            a.opening_view(&mut view);
            let mut buf = vec![0; size.0 * size.1];
            assert!(renderer.render(&mut buf, size.0, size.1, &view).is_empty());
            let baseline = buf.clone();
            let layout = render::Layout::for_screen(size.0, size.1, Screen::Opening, false);
            let actions = opening_ui::draw(
                &text,
                &mut buf,
                size,
                &a,
                Some((size.0 as f32 - 270.0, 220.0)),
                true,
            );
            assert!(
                actions
                    .iter()
                    .any(|x| matches!(x, records_ui::Action::Opening(A::Choose(0))))
            );
            for y in
                layout.board_top as usize..(layout.board_top + layout.board_size + 24.0) as usize
            {
                let start = y * size.0 + layout.board_left as usize;
                let end = start + layout.board_size as usize;
                assert_eq!(&buf[start..end], &baseline[start..end]);
            }
            assert!(analysis_ui::draw(&text, &mut buf, size, &a, None, false).is_empty());
            let reference = book()
                .entries
                .iter()
                .enumerate()
                .max_by_key(|(_, e)| e.moves.len())
                .unwrap()
                .0;
            a.opening.reference(reference).unwrap();
            let mut v = a.view_state(None, false, false, false);
            a.opening_view(&mut v);
            renderer.render(&mut buf, size.0, size.1, &v);
            let before = buf.clone();
            opening_ui::draw(&text, &mut buf, size, &a, None, false);
            for y in
                layout.board_top as usize..(layout.board_top + layout.board_size + 24.0) as usize
            {
                let start = y * size.0 + layout.board_left as usize;
                let end = start + layout.board_size as usize;
                assert_eq!(&buf[start..end], &before[start..end]);
            }
            a.handle_opening_action(A::First);
            for tab in [1, 2] {
                a.opening.tab = tab;
                a.opening.found = book().search("C60");
                opening_ui::draw(&text, &mut buf, size, &a, None, false);
            }
            a.opening.tab = 0;
        }
    }
    a.opening.focus = true;
    a.record_keys(&[Key::Home, Key::Left], &['S', 'i'], false);
    assert_eq!(a.opening.query, "Si");
    assert_eq!(a.opening.cursor, 0);
    a.record_keys(&[Key::Escape], &[], false);
    assert!(!a.opening.focus);
    assert_eq!(a.screen, Screen::Opening);
}

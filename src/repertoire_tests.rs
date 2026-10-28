//! App-level repertoire regressions. All persistence is isolated under target/.
use crate::{
    advanced_app::Intent,
    repertoire::{Collection, Side},
    repertoire_app::{Action as A, Confirm, Page},
    repertoire_train::{self, Mode, Session, State},
    *,
};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn temp() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::current_dir()
        .unwrap()
        .join("target/v028-qa/tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&root).unwrap();
    root
}
fn app() -> App {
    let mut a = uci_tests::app();
    let root = temp();
    a.records.save_path = Some(root.join("autosave-v1.json"));
    a.repertoire.root = Some(root.clone());
    a.library.root = Some(root.join("library-v1"));
    a.screen = Screen::Repertoire;
    // The official ECO corpus already has full baseline tests. An empty in-memory
    // reference keeps these state/storage regressions free of background tasks.
    a.repertoire.reference = Some(Arc::new(opening_book::Book {
        entries: vec![],
        positions: BTreeMap::new(),
        next: BTreeMap::new(),
    }));
    a
}
fn path(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_owned).collect()
}
fn mv(position: &Chess, text: &str) -> Move {
    text.parse::<shakmaty::uci::UciMove>()
        .unwrap()
        .to_move(position)
        .unwrap()
}
fn text(a: &mut App, value: &str) {
    let input = a.repertoire.input.as_mut().expect("text editor");
    input.text = value.into();
    input.cursor = value.len();
}
fn create(a: &mut App, side: Side, name: &str) -> String {
    a.handle_repertoire_action(if side == Side::White {
        A::NewWhite
    } else {
        A::NewBlack
    });
    text(a, name);
    a.handle_repertoire_action(A::ApplyText);
    a.repertoire.selected.clone().unwrap()
}
fn seed(a: &mut App, side: Side, lines: &[&str]) -> String {
    let mut c = Collection::default();
    let id = c.create("Prepared book", side, 1).unwrap();
    for (i, line) in lines.iter().enumerate() {
        c.add_route(&id, path(line), &format!("Line {i}"), "saved note", 2)
            .unwrap();
    }
    let root = a.repertoire.root.as_ref().unwrap();
    repertoire_store::save(&root.join("repertoires-v1.json"), &c).unwrap();
    a.repertoire.content = c;
    a.repertoire.selected = Some(id.clone());
    id
}
fn edit_moves(a: &mut App, moves: &str) {
    for uci in moves.split_whitespace() {
        let m = mv(a.repertoire.position(), uci);
        a.repertoire_move(m);
    }
}
#[test]
fn app_create_white_black_edit_shared_prefix_save_and_delete_only_selected_route() {
    let mut a = app();
    let white = create(&mut a, Side::White, "意大利 · White");
    a.handle_repertoire_action(A::Edit);
    edit_moves(&mut a, "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5");
    assert!(a.repertoire.dirty);
    a.handle_repertoire_action(A::RouteNote);
    text(&mut a, "准备说明 中文\nUTF-8");
    a.handle_repertoire_action(A::ApplyText);
    a.handle_repertoire_action(A::SaveRoute);
    assert!(!a.repertoire.dirty);
    assert_eq!(
        a.repertoire.content.get(&white).unwrap().routes[0].note,
        "准备说明 中文\nUTF-8"
    );
    let first = a.repertoire.content.get(&white).unwrap().routes[0].clone();
    a.handle_repertoire_action(A::Previous);
    a.handle_repertoire_action(A::Branch);
    edit_moves(&mut a, "g8f6");
    a.handle_repertoire_action(A::SaveRoute);
    assert_eq!(a.repertoire.content.get(&white).unwrap().routes.len(), 2);
    assert_eq!(a.repertoire.content.get(&white).unwrap().routes[0], first);
    a.handle_repertoire_action(A::RemoveRoute);
    assert_eq!(a.repertoire.confirm, Some(Confirm::Route));
    a.handle_repertoire_action(A::ConfirmRemove);
    assert_eq!(
        a.repertoire.content.get(&white).unwrap().routes,
        vec![first]
    );
    a.handle_repertoire_action(A::Back);
    let black = create(&mut a, Side::Black, "Black preparation");
    assert_eq!(a.repertoire.content.get(&black).unwrap().side, Side::Black);
    let restored = repertoire_store::load(
        &a.repertoire
            .root
            .as_ref()
            .unwrap()
            .join("repertoires-v1.json"),
    )
    .unwrap();
    assert_eq!(restored, a.repertoire.content);
}
#[test]
fn duplicate_non_last_route_save_keeps_matching_id_and_cannot_overwrite_other_route() {
    let mut a = app();
    let id = seed(&mut a, Side::White, &["e2e4 e7e5 g1f3", "d2d4 d7d5 c2c4"]);
    let original = a.repertoire.content.get(&id).unwrap().routes.clone();
    a.handle_repertoire_action(A::Edit);
    a.handle_repertoire_action(A::Branch);
    a.handle_repertoire_action(A::SaveRoute);
    assert_eq!(
        a.repertoire.route.as_deref(),
        Some(original[0].id.as_str()),
        "duplicate branch must bind its existing path, not routes.last()"
    );
    assert_eq!(a.repertoire.content.get(&id).unwrap().routes, original);
    a.handle_repertoire_action(A::RouteNote);
    text(&mut a, "Changed first only");
    a.handle_repertoire_action(A::ApplyText);
    a.handle_repertoire_action(A::SaveRoute);
    let routes = &a.repertoire.content.get(&id).unwrap().routes;
    assert_eq!(routes[0].id, original[0].id);
    assert_eq!(routes[0].note, "Changed first only");
    assert_eq!(routes[1], original[1]);
}
#[test]
fn opening_path_and_library_prefix_use_target_side_and_reject_nonstandard_fen() {
    let mut a = app();
    let white = create(&mut a, Side::White, "White");
    let black = create(&mut a, Side::Black, "Black");
    let source = path("g1f3 d7d5 g2g3");
    let mut pos = Chess::default();
    a.opening.positions = vec![pos.clone()];
    a.opening.moves.clear();
    for u in &source {
        let m = mv(&pos, u);
        pos.play_unchecked(&m);
        a.opening.moves.push(m);
        a.opening.positions.push(pos.clone());
    }
    a.opening.cursor = source.len();
    a.screen = Screen::Opening;
    a.handle_repertoire_action(A::FromOpening);
    assert_eq!(a.repertoire.pending_route, Some(source.clone()));
    a.handle_repertoire_action(A::SelectBook(1));
    a.handle_repertoire_action(A::AddPending);
    assert_eq!(
        a.repertoire.content.get(&black).unwrap().routes[0].moves,
        source
    );
    assert!(a.repertoire.content.get(&white).unwrap().routes.is_empty());
    a.handle_repertoire_action(A::Back);
    assert_eq!(a.screen, Screen::Opening);
    assert_eq!(a.opening.cursor, 3);
    let root = a.repertoire.root.as_ref().unwrap().clone();
    let mut lib = library::Library::open(root.join("library-v1")).unwrap();
    let mut pgn = String::new();
    for i in 0..15 {
        pgn.push_str(&format!("{}. Nf3 Nf6 {}. Ng1 Ng8 ", 2 * i + 1, 2 * i + 2));
    }
    pgn.push('*');
    let game = pgn::import_annotated(pgn.as_bytes()).unwrap();
    let game_id = lib.add(&game).unwrap().unwrap();
    a.library.store = Some(lib);
    a.library.selected = Some(game_id);
    a.library.query = "retain search".into();
    a.screen = Screen::Library;
    a.handle_repertoire_action(A::FromLibrary);
    assert_eq!(a.repertoire.pending_route.as_ref().unwrap().len(), 40);
    a.handle_repertoire_action(A::PrefixMore);
    assert_eq!(a.repertoire.pending_route.as_ref().unwrap().len(), 41);
    a.handle_repertoire_action(A::PrefixLess);
    a.handle_repertoire_action(A::AddPending);
    assert_eq!(
        a.repertoire.content.get(&black).unwrap().routes[1]
            .moves
            .len(),
        40
    );
    assert_eq!(a.repertoire.content.get(&black).unwrap().side, Side::Black);
    a.handle_repertoire_action(A::Back);
    assert_eq!(a.screen, Screen::Library);
    assert_eq!(a.library.query, "retain search");
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
    let external =
        pgn::import_annotated(format!("[SetUp \"1\"]\n[FEN \"{fen}\"]\n\n1... e5 *").as_bytes())
            .unwrap();
    let id = a
        .library
        .store
        .as_mut()
        .unwrap()
        .add(&external)
        .unwrap()
        .unwrap();
    a.library.selected = Some(id);
    let before = a.repertoire.content.clone();
    a.handle_repertoire_action(A::FromLibrary);
    assert_eq!(a.screen, Screen::Library);
    assert!(a.repertoire.error.as_ref().unwrap().contains("standard"));
    assert_eq!(a.repertoire.content, before);
    a.repertoire.session = Some(
        Session::new(
            a.repertoire.content.get(&black).unwrap(),
            Mode::All,
            &a.repertoire.progress,
            100,
            0,
        )
        .unwrap(),
    );
    assert_eq!(a.repertoire.session.as_ref().unwrap().side, Side::Black);
    assert_eq!(
        a.repertoire.session.as_ref().unwrap().pos.turn(),
        Color::Black
    );
}
#[test]
fn unsaved_text_and_dirty_route_guard_global_quit_until_explicit_resolution() {
    let mut a = app();
    create(&mut a, Side::White, "White");
    a.handle_repertoire_action(A::Edit);
    edit_moves(&mut a, "e2e4");
    assert!(a.defer_repertoire_loss(Intent::Quit));
    assert!(!a.exit_requested);
    assert_eq!(a.repertoire.confirm, Some(Confirm::Leave));
    a.handle_repertoire_action(A::KeepEdit);
    assert!(a.repertoire.dirty);
    assert!(!a.exit_requested);
    a.handle_repertoire_action(A::SaveRoute);
    a.handle_repertoire_action(A::RouteNote);
    text(&mut a, "尚未应用文本");
    assert!(a.defer_repertoire_loss(Intent::Quit));
    assert_eq!(a.repertoire.input.as_ref().unwrap().text, "尚未应用文本");
    assert!(!a.exit_requested);
    a.handle_repertoire_action(A::ApplyText);
    assert!(a.repertoire.dirty);
    a.handle_repertoire_action(A::SaveRoute);
    assert!(a.exit_requested);
    assert_eq!(a.repertoire.book().unwrap().routes[0].note, "尚未应用文本");
    let mut b = app();
    b.handle_repertoire_action(A::NewWhite);
    text(&mut b, "Unsaved new book");
    assert!(b.defer_repertoire_loss(Intent::Quit));
    b.handle_repertoire_action(A::CancelText);
    b.handle_repertoire_action(A::DiscardEdit);
    assert!(b.exit_requested);
    assert!(b.repertoire.content.repertoires.is_empty());
}
#[test]
fn failed_content_save_keeps_draft_and_failed_progress_keeps_pending_until_retry() {
    let mut a = app();
    let id = seed(&mut a, Side::White, &["e2e4 e7e5 g1f3"]);
    let root = a.repertoire.root.as_ref().unwrap().clone();
    let content_path = root.join("repertoires-v1.json");
    let original = fs::read(&content_path).unwrap();
    a.handle_repertoire_action(A::Edit);
    edit_moves(&mut a, "g8f6");
    let draft = a.repertoire.moves.clone();
    fs::write(&content_path, b"damaged but valuable").unwrap();
    a.handle_repertoire_action(A::SaveRoute);
    assert!(a.repertoire.dirty);
    assert_eq!(a.repertoire.moves, draft);
    assert_eq!(fs::read(&content_path).unwrap(), b"damaged but valuable");
    assert!(a.repertoire.error.is_some());
    fs::write(&content_path, &original).unwrap();
    a.handle_repertoire_action(A::SaveRoute);
    assert!(!a.repertoire.dirty);
    a.repertoire.mode = 2;
    a.handle_repertoire_action(A::Train);
    assert_eq!(a.repertoire.page, Page::Train);
    assert!(
        !a.repertoire.training_detail.contains("e4"),
        "allowed answers must not be shown without a hint"
    );
    let progress_path = root.join("repertoire-progress-v1.json");
    fs::create_dir(&progress_path).unwrap();
    let correct = mv(&a.repertoire.session.as_ref().unwrap().pos, "e2e4");
    a.repertoire_move(correct);
    assert!(a.repertoire.session.as_ref().unwrap().has_pending());
    assert!(a.repertoire.progress.entries.is_empty());
    a.poll_repertoire();
    assert!(a.repertoire.session.as_ref().unwrap().has_pending());
    a.handle_repertoire_action(A::Back);
    assert_eq!(
        a.repertoire.page,
        Page::Train,
        "failed progress must block silent Session disposal"
    );
    assert!(a.repertoire.session.as_ref().unwrap().has_pending());
    assert!(fs::remove_dir(&progress_path).is_ok());
    a.settle_repertoire().unwrap();
    assert!(!a.repertoire.session.as_ref().unwrap().has_pending());
    let restored = repertoire_train::load_progress(&progress_path).unwrap();
    assert!(!restored.entries.is_empty());
    assert_eq!(a.repertoire.content.get(&id).unwrap().routes.len(), 1);
}
#[test]
fn backup_new_merge_cancel_are_explicit_and_preserve_existing_route_notes() {
    let mut a = app();
    let id = seed(&mut a, Side::White, &["e2e4 e7e5"]);
    let mut incoming = Collection::default();
    let other = incoming.create("Prepared book", Side::White, 1).unwrap();
    incoming
        .rename(&other, "Prepared book", "备份库说明", 2)
        .unwrap();
    incoming
        .add_route(&other, path("d2d4 d7d5"), "Queen pawn", "备份路线说明", 3)
        .unwrap();
    let before = a.repertoire.content.clone();
    a.repertoire.pending_backup = Some(incoming.clone());
    a.handle_repertoire_action(A::CancelImport);
    assert_eq!(a.repertoire.content, before);
    a.repertoire.pending_backup = Some(incoming.clone());
    a.handle_repertoire_action(A::ImportNew);
    assert_eq!(a.repertoire.content.repertoires.len(), 2);
    assert_ne!(a.repertoire.content.repertoires[1].id, id);
    assert_eq!(a.repertoire.content.repertoires[1].note, "备份库说明");
    assert_eq!(
        a.repertoire.content.get(&id).unwrap(),
        before.get(&id).unwrap()
    );
    a.repertoire.pending_backup = Some(incoming.clone());
    a.handle_repertoire_action(A::ImportMerge);
    assert_eq!(a.repertoire.content.get(&id).unwrap().routes.len(), 2);
    assert_eq!(
        a.repertoire.content.get(&id).unwrap().routes[0],
        before.get(&id).unwrap().routes[0]
    );
    a.repertoire.pending_backup = Some(incoming);
    a.handle_repertoire_action(A::ImportMerge);
    assert_eq!(a.repertoire.content.get(&id).unwrap().routes.len(), 2);
    let restored = repertoire_store::load(
        &a.repertoire
            .root
            .as_ref()
            .unwrap()
            .join("repertoires-v1.json"),
    )
    .unwrap();
    assert_eq!(restored, a.repertoire.content);
}
#[test]
fn repertoire_editor_training_and_import_do_not_touch_live_or_other_storage() {
    let mut a = app();
    let id = seed(&mut a, Side::White, &["e2e4 e7e5 g1f3"]);
    let root = a.repertoire.root.as_ref().unwrap().clone();
    let files = [
        "autosave-v1.json",
        "library-v1/index.json",
        "opening-stats-v1.json",
        "puzzle-progress-v1.json",
        "review-cache-v1/sentinel.json",
    ];
    for file in files {
        let p = root.join(file);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"original-user-bytes").unwrap();
    }
    let live = fen::write(&a.pos);
    let clocks = a.clocks;
    let history = a.history.clone();
    let record = a.record();
    let generation = a.generation;
    let opening = fen::write(a.opening.position());
    let puzzle = serde_json::to_vec(&a.puzzles.progress).unwrap();
    let prefs = a.settings.board;
    a.handle_repertoire_action(A::Edit);
    a.handle_repertoire_action(A::First);
    a.handle_repertoire_action(A::Branch);
    edit_moves(&mut a, "d2d4 d7d5");
    a.handle_repertoire_action(A::SaveRoute);
    a.handle_repertoire_action(A::Back);
    a.repertoire.mode = 2;
    a.handle_repertoire_action(A::Train);
    a.handle_repertoire_action(A::Hint);
    a.handle_repertoire_action(A::Reveal);
    a.records.message = None;
    if let Some(s) = &a.repertoire.session
        && s.state == State::Waiting
    {
        let uci = s.allowed_uci()[0].clone();
        let m = mv(&s.pos, &uci);
        a.repertoire_move(m);
    }
    a.poll_repertoire();
    a.handle_repertoire_action(A::Back);
    assert_eq!(fen::write(&a.pos), live);
    assert_eq!(a.clocks, clocks);
    assert_eq!(a.history, history);
    assert_eq!(a.record(), record);
    assert_eq!(a.generation, generation);
    assert_eq!(fen::write(a.opening.position()), opening);
    assert_eq!(serde_json::to_vec(&a.puzzles.progress).unwrap(), puzzle);
    assert_eq!(a.settings.board, prefs);
    assert!(a.analysis.controller.is_none());
    assert!(a.review.job.is_none());
    assert!(a.repertoire.content.get(&id).is_some());
    for file in files {
        assert_eq!(fs::read(root.join(file)).unwrap(), b"original-user-bytes");
    }
}
#[test]
fn unsubmitted_new_book_can_save_then_close_without_saving_an_empty_route() {
    let mut a = app();
    a.handle_repertoire_action(A::NewWhite);
    text(&mut a, "新建并关闭");
    assert!(a.defer_repertoire_loss(Intent::Quit));
    a.handle_repertoire_action(A::SaveRoute);
    assert!(a.exit_requested);
    assert_eq!(a.repertoire.content.repertoires.len(), 1);
    assert!(a.repertoire.content.repertoires[0].routes.is_empty());
    assert!(a.repertoire.error.is_none());
}
#[test]
fn repertoire_pointer_drag_promotion_and_training_orientation_are_independent() {
    for promotion in [Role::Queen, Role::Rook, Role::Bishop, Role::Knight] {
        let mut a = app();
        seed(
            &mut a,
            Side::White,
            &["a2a4 h7h5 a4a5 h5h4 a5a6 h4h3 a6b7 h3g2"],
        );
        a.handle_repertoire_action(A::Edit);
        let live = fen::write(&a.pos);
        let clocks = a.clocks;
        a.handle_action(UiAction::PointerDown(Square::B7));
        a.handle_action(UiAction::PointerUp(Some(Square::A8)));
        assert_eq!(a.repertoire.promotion, Some((Square::B7, Square::A8)));
        a.handle_action(UiAction::Promote(promotion));
        assert_eq!(
            a.repertoire.moves.last().unwrap().promotion(),
            Some(promotion)
        );
        a.handle_repertoire_action(A::SaveRoute);
        assert!(!a.repertoire.dirty);
        assert_eq!(fen::write(&a.pos), live);
        assert_eq!(a.clocks, clocks);
    }
    for side in [Side::White, Side::Black] {
        let mut a = app();
        seed(&mut a, side, &["e2e4 e7e5 g1f3 b8c6"]);
        a.repertoire.mode = 2;
        a.handle_repertoire_action(A::Train);
        let mut view = a.view_state(None, false, false, false);
        a.repertoire_view(&mut view);
        assert_eq!(view.settings.mode, Some(!side.color()));
        assert_eq!(view.pos.turn(), side.color());
        let expected = a.repertoire.session.as_ref().unwrap().allowed_uci()[0].clone();
        let pos = a.repertoire.position().clone();
        let m = mv(&pos, &expected);
        a.handle_action(UiAction::PointerDown(m.from().unwrap()));
        a.handle_action(UiAction::PointerUp(Some(puzzle_app::target(&m))));
        assert_ne!(fen::write(a.repertoire.position()), fen::write(&pos));
        assert!(!a.repertoire.progress.entries.is_empty());
    }
}
#[test]
fn mode_switch_restarts_matching_targets_hint_stages_and_illegal_input_are_honest() {
    let mut a = app();
    seed(&mut a, Side::White, &["e2e4 e7e5 g1f3"]);
    a.repertoire.mode = 0;
    a.handle_repertoire_action(A::Train);
    assert_eq!(a.repertoire.session.as_ref().unwrap().state, State::Empty);
    a.handle_repertoire_action(A::Mode);
    assert_eq!(a.repertoire.mode, 1);
    assert_eq!(a.repertoire.session.as_ref().unwrap().state, State::Waiting);
    a.handle_action(UiAction::PointerDown(Square::E2));
    a.handle_action(UiAction::PointerUp(Some(Square::E5)));
    assert_eq!(a.repertoire.session.as_ref().unwrap().errors, 0);
    assert_eq!(
        a.repertoire.session.as_ref().unwrap().feedback,
        repertoire_train::Feedback::Illegal
    );
    assert_eq!(a.repertoire.position(), &Chess::default());
    a.handle_repertoire_action(A::Hint);
    assert_eq!(
        a.repertoire.session.as_ref().unwrap().hint_square,
        Some(Square::E2)
    );
    assert!(!a.repertoire.training_detail.contains("e4"));
    a.handle_repertoire_action(A::Hint);
    assert_eq!(
        a.repertoire.session.as_ref().unwrap().hint_square,
        Some(Square::E4)
    );
    assert!(!a.repertoire.training_detail.contains("e4"));
    a.handle_repertoire_action(A::Mode);
    assert_eq!(a.repertoire.mode, 2);
    assert_eq!(a.repertoire.session.as_ref().unwrap().state, State::Waiting);
}
#[test]
fn editor_eco_matches_existing_reference_without_writing_opening_data() {
    let mut a = app();
    seed(&mut a, Side::White, &["e2e4 e7e5 g1f3 b8c6 f1b5"]);
    a.repertoire.reference =
        Some(opening_app::shared_book(&std::sync::atomic::AtomicBool::new(false)).unwrap());
    a.handle_repertoire_action(A::Edit);
    assert!(a.repertoire.eco.starts_with("C60 "), "{}", a.repertoire.eco);
    assert!(a.repertoire.eco.contains("Ruy Lopez"));
}

use crate::{
    advanced_app::{Action as Edit, EditKind},
    library::{Library, Sort},
    library_app::Action as A,
    *,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
fn temp() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let p = std::env::current_dir()
        .unwrap()
        .join("target/v025-qa/tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn tree() -> annotated_game::AnnotatedGame {
    pgn::import_annotated("[White \"Émilie 棋手\"]\n[Black \"Bob\"]\n[Event \"Library Cup\"]\n[Date \"2026.10.26\"]\n\n1. e4! {汉字 [Event \"not a tag\"]} (1. d4 d5 (1... Nf6) 2. c4) e5 *".as_bytes()).unwrap()
}
fn collection(n: usize) -> Vec<u8> {
    (0..n).map(|i|format!("[White \"Player {i}\"]\n[Black \"Opponent\"]\n[Result \"{}\"]\n\n1. e4 (1. d4 d5) e5 {{game {i}}} $255 {}\n\n",if i%2==0{"1-0"}else{"0-1"},if i%2==0{"1-0"}else{"0-1"})).collect::<String>().into_bytes()
}
fn app() -> App {
    let mut a = uci_tests::app();
    a.library.root = Some(temp().join("library-v1"));
    a.records.save_path = None;
    a
}
#[test]
fn collections_two_ten_fifty_keep_each_tree_tags_and_result_separate() {
    for n in [1, 2, 10, 50] {
        let mut count = 0;
        assert_eq!(
            pgn::import_collection(&collection(n), |i, g| {
                assert_eq!(g.mainline.record.tags["White"], format!("Player {}", i - 1));
                assert_eq!(
                    g.mainline.record.result,
                    if i % 2 == 1 { "1-0" } else { "0-1" }
                );
                assert_eq!(g.nodes.len(), 5);
                assert_eq!(g.nodes.last().unwrap().nags, [255]);
                let text = pgn::export_annotated(&g).unwrap();
                let restored = pgn::import_annotated(text.as_bytes()).unwrap();
                assert_eq!(pgn::export_annotated(&restored).unwrap(), text);
                count += 1;
                Ok(())
            })
            .unwrap(),
            n
        );
        assert_eq!(count, n);
    }
    // No Event tags or headers are needed to delimit games, and comments cannot split them.
    let text = format!("{}\n\n1. d4 d5 *", pgn::export_annotated(&tree()).unwrap());
    let mut count = 0;
    pgn::import_collection(text.as_bytes(), |_, g| {
        g.validate()?;
        count += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(count, 2);
}
#[test]
fn collection_errors_identify_game_and_preserve_archived_prefix() {
    let mut lib = Library::open(temp()).unwrap();
    let mut bytes = collection(2);
    bytes.extend_from_slice(b"[White \"Bad\"]\n\n1. e5 *");
    let r = lib.import(&bytes, |_| {});
    assert_eq!((r.added, r.failed), (2, 1));
    assert!(r.error.unwrap().contains("Game 3: Illegal SAN"));
    assert_eq!(lib.entries.len(), 2);
    let mut lexical = collection(1);
    lexical.extend_from_slice(b"1. e4 (1. d9) *");
    let r = lib.import(&lexical, |_| {});
    assert_eq!((r.skipped, r.failed), (1, 1));
    assert!(r.error.unwrap().contains("Game 2:"));
}
#[test]
fn collection_limits_and_black_fen_reuse_single_game_validation() {
    let oversized = format!("{}*", " ".repeat(game_record::MAX_FILE_BYTES + 1));
    assert!(
        pgn::import_collection(oversized.as_bytes(), |_, _| Ok(()))
            .unwrap_err()
            .reason
            .contains("4 MiB")
    );
    assert_eq!(
        pgn::import_collection(b"1. e4 *\n\n\xff", |_, _| Ok(()))
            .unwrap_err()
            .game,
        0
    );
    let mut count = 0;
    let e = pgn::import_collection(&collection(201), |_, _| {
        count += 1;
        Ok(())
    })
    .unwrap_err();
    assert_eq!(e.game, 201);
    assert_eq!(count, 200);
    assert!(
        pgn::import_collection(&vec![b' '; pgn::MAX_COLLECTION_BYTES + 1], |_, _| Ok(()))
            .unwrap_err()
            .reason
            .contains("32 MiB")
    );
    let large = format!("{{{}}} *", "x".repeat(4097));
    assert!(pgn::import_collection(large.as_bytes(), |_, _| Ok(())).is_err());
    let text = b"[SetUp \"1\"]\n[FEN \"8/8/8/8/8/4k3/P7/4K3 b - - 7 42\"]\n\n42... Kf3 *\n\n*";
    let mut count = 0;
    pgn::import_collection(text, |_, g| {
        if count == 0 {
            assert_eq!(g.mainline.positions[0].turn(), Color::Black);
            assert_eq!(g.mainline.positions[0].fullmoves().get(), 42);
        }
        count += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(count, 2);
    assert!(pgn::import_annotated(&collection(2)).is_err());
}

#[test]
fn stale_index_and_remove_disk_error_never_overwrite_other_data() {
    let root = temp();
    let mut first = Library::open(root.clone()).unwrap();
    let id = first.add(&tree()).unwrap().unwrap();
    let mut stale = Library::open(root.clone()).unwrap();
    first.favorite(&id).unwrap();
    assert!(stale.favorite(&id).unwrap_err().contains("changed outside"));
    assert!(
        Library::open(root.clone())
            .unwrap()
            .entry(&id)
            .unwrap()
            .favorite
    );
    let file = root.join("games").join(first.entry(&id).unwrap().file());
    fs::rename(&file, root.join("preserved.pgn")).unwrap();
    fs::create_dir(&file).unwrap();
    let before = fs::read(root.join("index.json")).unwrap();
    assert!(first.remove(&id).is_err());
    assert_eq!(fs::read(root.join("index.json")).unwrap(), before);
    assert!(file.is_dir());
    let mut a = app();
    a.library.root = Some(root.join("library-v1"));
    assert!(
        a.guard_library_export(&root.join("AUTOSAVE-v1.json"))
            .is_err()
    );
    assert!(
        a.guard_library_export(&root.join("games/../index.json"))
            .is_err()
    );
}
#[test]
fn canonical_dedup_keeps_different_notes_and_original_source() {
    let root = temp();
    let source = root.join("original.pgn");
    let bytes = pgn::export_annotated(&tree()).unwrap();
    fs::write(&source, &bytes).unwrap();
    let mut lib = Library::open(root.join("library-v1")).unwrap();
    let a = lib.import(bytes.as_bytes(), |_| {});
    assert_eq!(a.added, 1);
    let b = lib.import(bytes.replace("\n", "\r\n").as_bytes(), |_| {});
    assert_eq!(b.skipped, 1);
    let mut g = tree();
    g.nodes[1].after.push("different".into());
    assert!(lib.add(&g).unwrap().is_some());
    assert_eq!(lib.entries.len(), 2);
    let id = lib.entries[0].id.clone();
    lib.remove(&id).unwrap();
    assert_eq!(fs::read_to_string(source).unwrap(), bytes);
    assert_eq!(lib.entries.len(), 1);
}
#[test]
fn save_preserves_stable_id_favorite_import_time_and_mainline_identity() {
    let mut lib = Library::open(temp()).unwrap();
    let mut g = tree();
    let id = lib.add(&g).unwrap().unwrap();
    lib.favorite(&id).unwrap();
    let before = lib.entry(&id).unwrap().clone();
    let key = review_cache::game_key(&g.mainline);
    g.nodes[1].nags.push(6);
    g.nodes[1].after.push("saved note".into());
    lib.save(&id, &g).unwrap();
    let re = Library::open(lib.root.clone()).unwrap();
    let e = re.entry(&id).unwrap();
    assert!(e.favorite);
    assert_eq!(e.imported, before.imported);
    assert_ne!(e.digest, before.digest);
    assert!(!lib.root.join("games").join(before.file()).exists());
    assert_eq!(review_cache::game_key(&re.load(&id).unwrap().mainline), key);
    assert_eq!(
        pgn::export_annotated(&re.load(&id).unwrap()).unwrap(),
        pgn::export_annotated(&g).unwrap()
    );
    let pg = fs::read(lib.root.join("games").join(e.file())).unwrap();
    lib.favorite(&id).unwrap();
    assert!(!lib.entry(&id).unwrap().favorite);
    assert_eq!(fs::read(lib.root.join("games").join(e.file())).unwrap(), pg);
}
#[test]
fn metadata_search_filters_and_all_sort_orders_need_no_pgn_io() {
    let mut lib = Library::open(temp()).unwrap();
    let id = lib.add(&tree()).unwrap().unwrap();
    let mut g = tree();
    g.mainline.record.tags.insert("White".into(), "Adam".into());
    g.mainline
        .record
        .tags
        .insert("Date".into(), "2000.01.01".into());
    let second = lib.add(&g).unwrap().unwrap();
    lib.favorite(&id).unwrap();
    // Prove search/index loading do not parse or even open the managed PGNs.
    for e in &lib.entries {
        fs::remove_file(lib.root.join("games").join(e.file())).unwrap();
    }
    lib = Library::open(lib.root.clone()).unwrap();
    for q in ["ÉMILIE", "bob", "CUP", "2026", "*"] {
        assert!(lib.search(q, false, None, Sort::Recent).contains(&id));
    }
    assert_eq!(
        lib.search("", true, None, Sort::Favorite).as_slice(),
        std::slice::from_ref(&id)
    );
    assert!(lib.search("", false, Some("1-0"), Sort::Recent).is_empty());
    assert_eq!(lib.search("", false, None, Sort::Recent)[0], second);
    assert_eq!(lib.search("", false, None, Sort::Players)[0], second);
    assert_eq!(lib.search("", false, None, Sort::Date)[0], id);
    assert_eq!(lib.search("", false, None, Sort::Favorite)[0], id);
    assert!(lib.load(&id).is_err());
    assert!(lib.recover().unwrap().contains("2 missing/invalid"));
}
#[test]
fn index_corruption_schema_and_untrusted_paths_are_preserved() {
    let root = temp();
    let mut lib = Library::open(root.clone()).unwrap();
    lib.add(&tree()).unwrap();
    let path = root.join("index.json");
    let good = fs::read(&path).unwrap();
    fs::write(&path, b"broken").unwrap();
    assert!(Library::open(root.clone()).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"broken");
    let mut data: serde_json::Value = serde_json::from_slice(&good).unwrap();
    data["schema"] = 2.into();
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(Library::open(root.clone()).is_err());
    data["schema"] = 1.into();
    data["entries"][0]["id"] = "../autosave-v1.json".into();
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(Library::open(root).is_err());
}
#[test]
fn archive_write_failures_do_not_drop_previous_index_or_game() {
    let root = temp();
    let mut lib = Library::open(root.clone()).unwrap();
    let id = lib.add(&tree()).unwrap().unwrap();
    let index = fs::read(root.join("index.json")).unwrap();
    let text = pgn::export_annotated(&lib.load(&id).unwrap()).unwrap();
    // A directory at the atomic replacement target is a deterministic fault.
    fs::rename(root.join("index.json"), root.join("preserved-index.json")).unwrap();
    fs::create_dir(root.join("index.json")).unwrap();
    assert!(lib.favorite(&id).is_err());
    assert!(!lib.entry(&id).unwrap().favorite);
    let mut g = tree();
    g.nodes[1].after.push("uncommitted".into());
    assert!(lib.save(&id, &g).is_err());
    assert_eq!(
        pgn::export_annotated(&lib.load(&id).unwrap()).unwrap(),
        text
    );
    assert_eq!(fs::read(root.join("preserved-index.json")).unwrap(), index);
    let r = lib.import(&collection(2), |_| {});
    assert_eq!(r.added, 0);
    assert_eq!(r.failed, 1);
    assert!(r.error.unwrap().contains("Archive write failed"));
}
#[test]
fn interrupted_archive_check_recovers_orphans_without_deleting_conflicts() {
    let root = temp();
    let mut lib = Library::open(root.clone()).unwrap();
    let id = lib.add(&tree()).unwrap().unwrap();
    fs::rename(root.join("index.json"), root.join("preserved-index.json")).unwrap();
    lib = Library::open(root.clone()).unwrap();
    assert!(lib.entries.is_empty());
    assert!(lib.recover().unwrap().contains("1 recovered"));
    assert_eq!(lib.entries[0].id, id);
    let file = lib.entries[0].file();
    fs::write(root.join("games").join(&file), b"damaged").unwrap();
    assert!(lib.load(&id).err().unwrap().contains("hash mismatch"));
    assert!(lib.recover().unwrap().contains("1 missing/invalid"));
    assert_eq!(fs::read(root.join("games").join(file)).unwrap(), b"damaged");
}
#[test]
fn library_replay_preserves_list_position_live_clocks_and_source_metadata() {
    let mut a = app();
    a.enter_library().unwrap();
    let r = a
        .library
        .store
        .as_mut()
        .unwrap()
        .import(&collection(10), |_| {});
    assert_eq!(r.added, 10);
    a.library.refresh();
    a.library.query = "Player".into();
    a.library.sort = Sort::Players;
    a.library.refresh();
    a.library.scroll = 4;
    let id = a.library.visible[4].clone();
    a.library.selected = Some(id.clone());
    let snap = a.snapshot();
    a.open_library_id(&id).unwrap();
    assert_eq!(a.screen, Screen::Replay);
    assert_eq!(a.records.library_id, Some(id.clone()));
    a.tick_clock_at(a.clock_last_tick + Duration::from_secs(10));
    assert_eq!(a.clocks, snap.clocks);
    a.leave_replay();
    assert_eq!(a.screen, Screen::Library);
    assert_eq!(a.library.scroll, 4);
    assert_eq!(a.library.query, "Player");
    assert_eq!(a.library.selected, Some(id));
    assert_eq!(a.snapshot().record, snap.record);
}
#[test]
fn replay_save_failure_and_unsaved_replacement_use_one_existing_guard() {
    let mut a = app();
    a.enter_library().unwrap();
    let id = a
        .library
        .store
        .as_mut()
        .unwrap()
        .add(&tree())
        .unwrap()
        .unwrap();
    a.open_library_id(&id).unwrap();
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("new text").unwrap();
    a.leave_replay();
    assert!(a.advanced.pending.is_some());
    assert_eq!(a.screen, Screen::Replay);
    a.handle_advanced_action(Edit::CancelExit);
    a.save_replay_library(false).unwrap();
    assert!(!a.needs_replay_save());
    let identity = a.records.replay.as_ref().unwrap().identity.clone();
    let loaded = a.library.store.as_ref().unwrap().load(&id).unwrap();
    assert!(
        loaded
            .nodes
            .last()
            .unwrap()
            .after
            .iter()
            .any(|s| s.contains("new text"))
    );
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("second").unwrap();
    a.apply_annotation_edit().unwrap();
    let root = a.library_root().unwrap();
    fs::rename(root.join("index.json"), root.join("index-safe.json")).unwrap();
    fs::create_dir(root.join("index.json")).unwrap();
    assert!(a.save_replay_library(false).is_err());
    assert!(a.needs_replay_save());
    assert_eq!(a.records.replay.as_ref().unwrap().identity, identity);
    a.open_library_id(&id).unwrap();
    assert!(a.advanced.pending.is_some());
    a.handle_advanced_action(Edit::CancelExit);
    assert!(a.needs_replay_save());
    assert!(a.guard_library_export(&root.join("bad.pgn")).is_err());
}
#[test]
fn ordinary_replay_add_and_export_have_distinct_explicit_effects() {
    let mut a = app();
    a.open_annotated_replay(tree(), true);
    assert!(a.records.library_id.is_none());
    a.records.replay.as_mut().unwrap().dirty = true;
    a.save_replay_library(true).unwrap();
    let id = a.records.library_id.clone().unwrap();
    assert!(!a.needs_replay_save());
    a.save_replay_library(true).unwrap();
    assert_eq!(a.library.store.as_ref().unwrap().entries.len(), 1);
    assert_eq!(a.records.library_id.as_deref(), Some(id.as_str()));
    let target = temp().join("export.pgn");
    a.save_annotated_to(&target).unwrap();
    assert_eq!(
        pgn::export_annotated(&pgn::import_annotated(&fs::read(target).unwrap()).unwrap()).unwrap(),
        pgn::export_annotated(&tree()).unwrap()
    );
    a.leave_replay();
    assert_eq!(a.screen, Screen::Game);
}
#[test]
fn remove_requires_confirmation_and_only_touches_managed_copy() {
    let mut a = app();
    a.enter_library().unwrap();
    let id = a
        .library
        .store
        .as_mut()
        .unwrap()
        .add(&tree())
        .unwrap()
        .unwrap();
    a.library.refresh();
    a.library.selected = Some(id.clone());
    a.handle_library_action(A::Remove);
    assert_eq!(a.library.remove.as_deref(), Some(id.as_str()));
    a.handle_library_action(A::CancelRemove);
    assert_eq!(a.library.store.as_ref().unwrap().entries.len(), 1);
    a.handle_library_action(A::Remove);
    a.handle_library_action(A::ConfirmRemove);
    assert!(a.library.error.is_none());
    assert!(a.library.store.as_ref().unwrap().entries.is_empty());
}
#[test]
fn library_worker_import_reports_partial_failure_and_polls_without_blocking() {
    let mut a = app();
    a.enter_library().unwrap();
    let file = temp().join("many.pgn");
    let mut bytes = collection(10);
    bytes.extend_from_slice(b"1. e5 *");
    fs::write(&file, bytes).unwrap();
    a.start_library_import(&file).unwrap();
    assert!(a.library.busy());
    let until = Instant::now() + Duration::from_secs(5);
    while a.library.busy() {
        a.poll_library();
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    assert_eq!(a.library.store.as_ref().unwrap().entries.len(), 10);
    assert!(a.library.error.as_ref().unwrap().contains("Game 11:"));
    assert!(a.library.notice.as_ref().unwrap().contains("10"));
    let count = a.library.store.as_ref().unwrap().entries.len();
    let bad = temp().join("missing.pgn");
    a.start_library_import(&bad).unwrap();
    while a.library.busy() {
        a.poll_library();
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    assert_eq!(a.library.store.as_ref().unwrap().entries.len(), count);
    assert!(a.library.error.as_ref().unwrap().contains("Read failed"));
}
#[test]
fn library_review_cache_identity_and_request_isolation_survive_metadata_operations() {
    let g = tree();
    let (mut a, dir, _) = review_tests::setup(g.mainline.clone(), "review");
    a.library.root = Some(dir.join("library-v1"));
    a.ensure_library().unwrap();
    let id = a.library.store.as_mut().unwrap().add(&g).unwrap().unwrap();
    a.open_library_id(&id).unwrap();
    a.start_review().unwrap();
    review_tests::completed(&mut a);
    let key = a.review.job.as_ref().unwrap().cache_key.clone();
    let cache = review_cache::path(&dir, &key);
    let bytes = fs::read(&cache).unwrap();
    let identity = a.records.replay.as_ref().unwrap().identity.clone();
    a.library.selected = Some(id.clone());
    a.handle_library_action(A::Favorite);
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("cache unchanged").unwrap();
    a.save_replay_library(false).unwrap();
    a.open_library_id(&id).unwrap();
    assert_eq!(a.records.replay.as_ref().unwrap().identity, identity);
    a.start_review().unwrap();
    assert_eq!(
        a.review.job.as_ref().unwrap().state,
        game_review::State::Completed
    );
    assert_eq!(a.review.job.as_ref().unwrap().cache_key, key);
    assert_eq!(fs::read(cache).unwrap(), bytes);
    a.analysis.panel = true;
    a.request_analysis().unwrap();
    let seq = a.analysis.sequence;
    let mut next = tree();
    next.mainline.record.result = "1-0".into();
    next.mainline
        .record
        .tags
        .insert("Result".into(), "1-0".into());
    let other = a
        .library
        .store
        .as_mut()
        .unwrap()
        .add(&next)
        .unwrap()
        .unwrap();
    a.open_library_id(&other).unwrap();
    a.poll_analysis();
    assert!(a.analysis_visual().score.is_none());
    assert!(a.analysis_visual().arrow.is_none());
    assert!(a.analysis.sequence >= seq);
    assert_eq!(
        fs::read(dir.join("autosave-v1.json")).unwrap(),
        b"user autosave sentinel"
    );
    a.close_analysis_engine();
}
#[test]
fn library_offscreen_languages_sizes_search_focus_and_click_actions() {
    let mut a = app();
    a.enter_library().unwrap();
    a.library.store.as_mut().unwrap().add(&tree()).unwrap();
    a.library.refresh();
    a.library.store.as_mut().unwrap().entries[0].white = "长名字 Very long player ".repeat(100);
    a.library.store.as_mut().unwrap().entries[0].event = "长赛事 Event ".repeat(100);
    let font = font::TextRenderer::load();
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(library_ui::words(lang).iter().all(|s| !s.is_empty()));
        for size in [(640, 480), (1280, 720)] {
            let mut buf = vec![0; size.0 * size.1];
            let acts = library_ui::draw(&font, &mut buf, size, &a, Some((25.0, 190.0)), true);
            assert!(
                acts.iter()
                    .any(|a| matches!(a, records_ui::Action::Library(A::Select(0))))
            );
            assert!(buf.iter().any(|p| *p != 0));
            assert!(analysis_ui::draw(&font, &mut buf, size, &a, None, false).is_empty());
            a.library.remove = a.library.selected.clone();
            let acts = library_ui::draw(&font, &mut buf, size, &a, None, false);
            assert!(acts.is_empty());
            a.library.remove = None;
        }
    }
    a.library.focus = true;
    a.record_keys(&[], &['É', '棋'], false);
    assert_eq!(a.library.query, "É棋");
    a.record_keys(&[Key::Left, Key::Home], &[], false);
    assert_eq!(a.library.query, "É棋");
    a.record_keys(&[Key::Backspace], &[], false);
    assert_eq!(a.library.query, "É");
    a.record_keys(&[Key::A], &[], true);
    assert!(a.library.query.is_empty());
}

use crate::{
    advanced_app::{Action as A, EditKind, Intent},
    uci_tests::app,
    *,
};
fn load() -> App {
    let mut a = app();
    a.import_annotated_bytes(b"1. e4 (1. d4 d5 2. c4) e5 2. Nf3 *")
        .unwrap();
    a
}
#[test]
fn branch_fen_highlight_uci_rejection_and_mainline_review_identity() {
    let mut a = load();
    let before = a.snapshot();
    let key = a.records.replay.as_ref().unwrap().identity.clone();
    let d4 = a
        .records
        .replay
        .as_ref()
        .unwrap()
        .tree
        .nodes
        .iter()
        .find(|n| n.san == "d4")
        .unwrap()
        .id;
    a.handle_advanced_action(A::Node(d4));
    assert!(
        a.request_analysis()
            .unwrap_err()
            .contains(crate::advanced_ui::words(a.settings.language)[21])
    );
    let r = a.records.replay.as_ref().unwrap();
    assert_eq!(a.display_fen(), fen::write(r.position()));
    assert_ne!(a.display_fen(), fen::write(&r.game.positions[r.cursor]));
    let mut view = a.view_state(None, false, false, false);
    a.replay_view(&mut view);
    assert_eq!(view.last_move, Some((Square::D2, Square::D4)));
    assert_eq!(fen::write(&view.pos), a.display_fen());
    a.handle_advanced_action(A::Mainline);
    assert_eq!(a.records.replay.as_ref().unwrap().identity, key);
    assert_eq!(a.snapshot().current_fen, before.current_fen);
    assert_eq!(a.clocks, before.clocks);
}
#[test]
fn unicode_editor_atomic_validation_nags_and_unsaved_exit() {
    let mut a = load();
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("中文 café🙂").unwrap();
    a.advanced_keys(&[Key::Left], &[], false);
    a.advanced_keys(&[Key::Backspace], &[], false);
    a.handle_advanced_action(A::Apply);
    assert!(a.records.replay.as_ref().unwrap().dirty);
    a.leave_replay();
    assert!(a.advanced.pending.is_some());
    assert_eq!(a.screen, Screen::Replay);
    a.handle_advanced_action(A::CancelExit);
    assert!(a.records.replay.as_ref().unwrap().dirty);
    a.begin_annotation_edit(EditKind::After).unwrap();
    let old = a.advanced.editor.as_ref().unwrap().text.clone();
    assert!(a.insert_annotation_text("{bad}").is_err());
    assert_eq!(a.advanced.editor.as_ref().unwrap().text, old);
    a.handle_advanced_action(A::CancelEdit);
    a.begin_annotation_edit(EditKind::Nags).unwrap();
    a.insert_annotation_text("$0 ! ?! $255").unwrap();
    a.handle_advanced_action(A::Apply);
    let r = a.records.replay.as_ref().unwrap();
    assert_eq!(r.tree.nodes[r.node].nags, [0, 1, 6, 255]);
    a.begin_annotation_edit(EditKind::Nags).unwrap();
    a.advanced.editor.as_mut().unwrap().text = "$256".into();
    assert!(a.apply_annotation_edit().is_err());
    a.handle_advanced_action(A::CancelEdit);
    assert!(a.defer_replay_loss(Intent::Quit));
    a.handle_advanced_action(A::Discard);
    assert!(a.exit_requested);
}
#[test]
fn annotated_export_failure_preserves_dirty_and_success_never_mutates_live_record() {
    let mut a = load();
    let before = a.record();
    let identity = a.records.replay.as_ref().unwrap().identity.clone();
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("保存测试").unwrap();
    a.apply_annotation_edit().unwrap();
    let (_, dir) = uci_tests::fixture("normal");
    let blocked = dir.join("file");
    std::fs::write(&blocked, b"sentinel").unwrap();
    assert!(a.save_annotated_to(&blocked.join("x.pgn")).is_err());
    assert!(a.records.replay.as_ref().unwrap().dirty);
    let target = dir.join("annotated.pgn");
    a.save_annotated_to(&target).unwrap();
    assert!(!a.records.replay.as_ref().unwrap().dirty);
    assert_eq!(a.record(), before);
    assert_eq!(a.records.replay.as_ref().unwrap().identity, identity);
    let back = pgn::import_annotated(&std::fs::read(target).unwrap()).unwrap();
    assert!(
        back.nodes
            .iter()
            .any(|n| n.after.contains(&"保存测试".to_owned()))
    );
}

#[test]
fn branch_and_comment_changes_leave_running_mainline_review_and_cache_untouched() {
    use crate::{
        game_review::State,
        review_tests::{completed, pump, setup},
    };
    let tree = pgn::import_annotated(b"1. e4 (1. d4 d5) e5 2. Nf3 Nc6 *").unwrap();
    let branch = tree.nodes.iter().find(|n| n.san == "d4").unwrap().id;
    let (mut a, dir, _) = setup(tree.mainline.clone(), "review");
    a.open_annotated_replay(tree, true);
    a.start_review().unwrap();
    pump(&mut a, |j| j.done() >= 1 && j.pending());
    let key = a.review.job.as_ref().unwrap().cache_key.clone();
    let identity = a.records.replay.as_ref().unwrap().identity.clone();
    a.handle_advanced_action(A::Node(branch));
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("仅注释，不改走棋").unwrap();
    a.apply_annotation_edit().unwrap();
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Running);
    completed(&mut a);
    a.review.panel = true;
    a.analysis.panel = true;
    assert!(a.analysis_visual().score.is_none());
    assert!(a.analysis_visual().arrow.is_none());
    assert_eq!(a.review.job.as_ref().unwrap().cache_key, key);
    let font = font::TextRenderer::load();
    let mut buffer = vec![0; 640 * 480];
    let actions = review_ui::draw(
        &font,
        &mut buffer,
        (640, 480),
        &a,
        Some((400.0, 260.0)),
        true,
    );
    assert!(actions.iter().all(|a| !matches!(
        a,
        analysis_ui::Action::Review(review_ui::ReviewAction::Jump(_))
    )));
    a.handle_advanced_action(A::Mainline);
    assert!(a.analysis_visual().score.is_some());
    assert_eq!(a.records.replay.as_ref().unwrap().identity, identity);
    a.handle_advanced_action(A::Node(branch));
    a.handle_record_action(records_ui::Action::First);
    assert!(a.records.replay.as_ref().unwrap().on_mainline());
    a.handle_review_action(review_ui::ReviewAction::Jump(2));
    let replay = a.records.replay.as_ref().unwrap();
    assert!(replay.on_mainline());
    assert_eq!(
        fen::write(replay.position()),
        fen::write(&replay.game.positions[2])
    );
    let count = std::fs::read_to_string(dir.join("commands.log"))
        .unwrap()
        .matches("go movetime")
        .count();
    a.start_review().unwrap();
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Completed);
    assert_eq!(
        std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .matches("go movetime")
            .count(),
        count
    );
    assert_eq!(
        std::fs::read(dir.join("autosave-v1.json")).unwrap(),
        b"user autosave sentinel"
    );
    assert!(a.shutdown_analysis());
}

#[test]
fn import_failure_pending_replacement_and_export_cancel_never_drop_metadata() {
    let mut a = load();
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("保留").unwrap();
    a.apply_annotation_edit().unwrap();
    let old = a.records.replay.as_ref().unwrap().node;
    assert!(a.import_annotated_bytes(b"1. d4 (1. e5) *").is_err());
    assert!(a.advanced.pending.is_none());
    assert_eq!(a.records.replay.as_ref().unwrap().node, old);
    a.import_annotated_bytes(b"1. d4 d5 *").unwrap();
    assert!(a.advanced.pending.is_some());
    assert_eq!(
        a.records.replay.as_ref().unwrap().game.record.moves[0],
        "e2e4"
    );
    a.finish_export_exit(false).unwrap();
    assert!(a.advanced.pending.is_some());
    assert!(a.records.replay.as_ref().unwrap().dirty);
    a.handle_advanced_action(A::CancelExit);
    assert!(a.records.replay.as_ref().unwrap().dirty);
    a.leave_replay();
    let (_, dir) = uci_tests::fixture("normal");
    a.save_annotated_to(&dir.join("saved.pgn")).unwrap();
    a.finish_export_exit(true).unwrap();
    assert!(a.records.replay.is_none());
    let mut a = load();
    a.begin_annotation_edit(EditKind::After).unwrap();
    a.insert_annotation_text("buffer未应用").unwrap();
    assert!(a.defer_replay_loss(Intent::Quit));
    a.handle_advanced_action(A::CancelExit);
    assert!(a.advanced.editor.as_ref().unwrap().text.contains("buffer"));
    assert!(!a.exit_requested);
}

#[test]
fn review_waits_for_queued_single_pv_setting_instead_of_rejecting_old_stopped_report() {
    use crate::uci::{Controller, Report, Status};
    use std::sync::{Arc, Mutex};
    let mut a = load();
    let report = Arc::new(Mutex::new(Report {
        status: Status::Stopped,
        multipv: 3,
        requested_multipv: 3,
        engine_key: "test-binary".into(),
        ..Default::default()
    }));
    a.analysis.controller = Some(Controller::with_test_report(report.clone()));
    a.start_review().unwrap();
    a.poll_review();
    assert_eq!(
        a.review.job.as_ref().unwrap().state,
        game_review::State::Running
    );
    assert_eq!(a.review.job.as_ref().unwrap().done(), 0);
    {
        let mut r = report.lock().unwrap();
        r.multipv = 1;
        r.requested_multipv = 1;
    }
    a.poll_review();
    assert!(a.review.job.as_ref().unwrap().pending());
    a.cancel_review();
}

#[test]
fn advanced_offscreen_layout_and_edit_modals_cover_eleven_languages_and_two_sizes() {
    let mut a = load();
    let font = font::TextRenderer::load();
    let images = assets::PieceImages::load();
    let renderer = Renderer::new(&images, &font);
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(advanced_ui::words(lang).iter().all(|s| !s.is_empty()));
        for size in [(640, 480), (1280, 720)] {
            let mut view = a.view_state(None, false, false, false);
            a.replay_view(&mut view);
            let mut buf = vec![0; size.0 * size.1];
            renderer.render(&mut buf, size.0, size.1, &view);
            let before = buf.clone();
            records_ui::draw(&font, &mut buf, size, &a, None, false);
            for y in 40..size.1 - 40 {
                let edge = size.0 - 292;
                assert_eq!(
                    &before[y * size.0..y * size.0 + edge],
                    &buf[y * size.0..y * size.0 + edge]
                );
            }
            a.begin_annotation_edit(EditKind::After).unwrap();
            a.insert_annotation_text("中文🙂 — café").unwrap();
            assert!(a.advanced_modal());
            advanced_ui::draw_modal(&font, &mut buf, size, &a, None, false);
            a.advanced_keys(&[Key::Left], &[], false);
            a.advanced_keys(&[Key::Delete], &[], false);
            assert!(
                std::str::from_utf8(a.advanced.editor.as_ref().unwrap().text.as_bytes()).is_ok()
            );
            a.handle_advanced_action(A::CancelEdit);
        }
    }
}

#[test]
fn frame_playback_and_unedited_editor_exit_preserve_the_real_session() {
    let mut a = load();
    let before = a.snapshot();
    a.handle_record_action(records_ui::Action::First);
    a.handle_advanced_action(A::Play);
    let start = a.advanced.last_tick;
    a.tick_replay_at(start + std::time::Duration::from_millis(500));
    assert_eq!(a.records.replay.as_ref().unwrap().path_cursor(), 0);
    a.tick_replay_at(start + std::time::Duration::from_secs(1));
    assert_eq!(a.records.replay.as_ref().unwrap().path_cursor(), 1);
    a.begin_annotation_edit(EditKind::After).unwrap();
    assert!(!a.needs_replay_save());
    assert!(!a.records.replay.as_ref().unwrap().playing);
    a.advanced_keys(&[Key::Home, Key::Right], &[], false);
    assert_eq!(a.records.replay.as_ref().unwrap().path_cursor(), 1);
    a.handle_advanced_action(A::CancelEdit);
    a.leave_replay();
    assert!(a.records.replay.is_none());
    assert_eq!(a.snapshot().record.moves, before.record.moves);
    assert_eq!(a.clocks, before.clocks);
    assert_eq!(a.snapshot().current_fen, before.current_fen);
    assert!(a.analysis.controller.is_none());
    assert!(crate::advanced_app::parse_nags("$$1").is_err());
    assert!(crate::advanced_app::parse_nags("+1").is_err());
}

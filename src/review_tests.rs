use super::*;
use crate::{
    game_review::{Job, Options, Point, State},
    review_scoring::{self as scoring, Quality, Reason},
    uci::{Config, Controller, Report, Status},
    uci_protocol::{self as protocol, Bound, Score},
    uci_tests::{app, fixture, ready, wait},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

fn game(plies: usize) -> game_record::ValidatedRecord {
    let g=pgn::import(b"1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7 *").unwrap();
    let mut r = g.record;
    r.moves.truncate(plies);
    r.validate().unwrap()
}
fn responses(dir: &std::path::Path, game: &game_record::ValidatedRecord, score: &str) {
    let lines = (0..game.positions.len())
        .filter_map(|i| {
            let pos = &game.positions[i];
            let mv = game
                .moves
                .get(i)
                .cloned()
                .or_else(|| pos.legal_moves().first().cloned())?;
            Some(format!(
                "{}\t{}\t{score}\t12",
                protocol::position(&game.positions[0], &game.moves[..i]),
                shakmaty::uci::UciMove::from_standard(&mv)
            ))
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(dir.join("review.tsv"), lines).unwrap();
}
pub(crate) fn setup(game: game_record::ValidatedRecord, mode: &str) -> (App, PathBuf, Config) {
    let (mut config, dir) = fixture(mode);
    config.multipv = 3;
    responses(&dir, &game, "cp 25");
    let mut a = app();
    a.settings.board.chess_clock_enabled = true;
    a.records.save_path = Some(dir.join("autosave-v1.json"));
    std::fs::write(dir.join("autosave-v1.json"), b"user autosave sentinel").unwrap();
    a.enter_replay(game, true);
    a.analysis.config = config.clone();
    a.enable_analysis().unwrap();
    ready(a.analysis.controller.as_ref().unwrap());
    (a, dir, config)
}
pub(crate) fn pump(a: &mut App, predicate: impl Fn(&Job) -> bool) {
    let until = Instant::now() + Duration::from_secs(8);
    loop {
        a.poll_analysis();
        a.poll_review();
        if predicate(a.review.job.as_ref().unwrap()) {
            return;
        }
        assert!(
            Instant::now() < until,
            "{:?} {:?} {:?}",
            a.review.job.as_ref().unwrap().state,
            a.review.job.as_ref().unwrap().error,
            a.analysis.controller.as_ref().unwrap().report()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
pub(crate) fn completed(a: &mut App) {
    pump(a, |j| matches!(j.state, State::Completed | State::Failed));
    assert_eq!(
        a.review.job.as_ref().unwrap().state,
        State::Completed,
        "{:?}",
        a.review.job.as_ref().unwrap().error
    );
}
fn pending_started(a: &App) {
    let id = a.analysis.sequence;
    wait(a.analysis.controller.as_ref().unwrap(), |r| {
        r.request == Some(id) && r.status == Status::Searching && r.info.depth.is_some()
    });
}
fn plain_job(game: game_record::ValidatedRecord) -> Job {
    let game_key = review_cache::game_key(&game);
    Job::new(
        game,
        game_key,
        "test-cache-key".into(),
        "Mock".into(),
        Options::default(),
        1,
    )
}
fn point(score: Option<(Score, Bound)>) -> Point {
    Point {
        fen: fen::write(&Chess::default()),
        score,
        best: Some("e2e4".into()),
        best_san: Some("e4".into()),
        depth: Some(12),
        elapsed_ms: 40,
        reported_ms: Some(7),
        terminal: None,
    }
}
fn cp(n: i32) -> Option<(Score, Bound)> {
    Some((Score::Cp(n), Bound::Exact))
}

#[test]
fn normal_four_ten_twenty_plies_use_n_plus_one_positions_and_one_process() {
    for n in [4, 10, 20] {
        let (mut a, dir, _) = setup(game(n), "review");
        let before = a.snapshot();
        let pid = a.analysis.controller.as_ref().unwrap().report().pid;
        a.start_review().unwrap();
        completed(&mut a);
        let job = a.review.job.as_ref().unwrap();
        assert_eq!(job.done(), n + 1);
        assert_eq!(job.points.len(), n + 1);
        assert_eq!(job.moves.len(), n);
        assert!(job.moves.iter().all(|m| m.matched));
        for (i, p) in job.points.iter().enumerate() {
            let p = p.as_ref().unwrap();
            assert_eq!(p.fen, fen::write(&job.game.positions[i]));
            assert_eq!(p.reported_ms, Some(7));
            assert_eq!(p.score, cp(if i % 2 == 0 { 25 } else { -25 }));
        }
        assert_eq!(a.analysis.controller.as_ref().unwrap().report().pid, pid);
        assert_eq!(a.snapshot().current_fen, before.current_fen);
        assert_eq!(a.snapshot().record.moves, before.record.moves);
        assert_eq!(a.clocks, before.clocks);
        assert_eq!(
            std::fs::read(dir.join("autosave-v1.json")).unwrap(),
            b"user autosave sentinel"
        );
        assert!(a.shutdown_analysis());
        let log = std::fs::read_to_string(dir.join("commands.log")).unwrap();
        assert_eq!(log.lines().filter(|l| *l == "uci").count(), 1);
        assert_eq!(log.matches("go movetime 500").count(), n + 1);
        let positions: Vec<_> = log.lines().filter(|l| l.starts_with("position ")).collect();
        assert_eq!(positions.len(), n + 1);
        assert_eq!(
            positions
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            n + 1
        );
    }
}

#[test]
fn nonstandard_black_start_terminal_and_limited_range_are_explicit() {
    let g=pgn::import(b"[SetUp \"1\"]\n[FEN \"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 17 42\"]\n\n42... e5 43. Nf3 *").unwrap();
    let (mut a, dir, _) = setup(g, "review");
    a.start_review().unwrap();
    completed(&mut a);
    let j = a.review.job.as_ref().unwrap();
    assert_eq!(j.moves[0].side, Color::Black);
    assert_eq!(j.points[0].as_ref().unwrap().score, cp(-25));
    assert!(a.shutdown_analysis());
    assert!(
        std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .contains("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 17 42")
    );
    let (mut a, dir, _) = setup(pgn::import(b"1. f3 e5 2. g4 Qh4# 0-1").unwrap(), "review");
    a.start_review().unwrap();
    completed(&mut a);
    let j = a.review.job.as_ref().unwrap();
    assert_eq!(j.done(), 5);
    let p = j.points[4].as_ref().unwrap();
    assert!(matches!(
        p.terminal,
        Some(game::GameResult::Checkmate {
            winner: Color::Black
        })
    ));
    assert!(p.score.is_none());
    assert_eq!(j.moves[3].quality, Quality::Unrated);
    assert_eq!(j.moves[3].reason, Reason::Terminal);
    assert!(a.shutdown_analysis());
    assert_eq!(
        std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .matches("go movetime")
            .count(),
        4
    );
    let (mut a, _, _) = setup(game(20), "review");
    a.review.options.max_plies = 4;
    a.start_review().unwrap();
    completed(&mut a);
    let j = a.review.job.as_ref().unwrap();
    assert_eq!(j.limit, 4);
    assert_eq!(j.done(), 5);
    assert_eq!(j.game.moves.len(), 20);
    assert!(j.points[5..].iter().all(Option::is_none));
    assert!(j.moves[4..].iter().all(|m| m.quality == Quality::Unrated));
    assert!(a.shutdown_analysis());
}

#[test]
fn heuristic_direction_threshold_boundaries_saturation_and_best_fact() {
    assert_eq!(scoring::classify(f64::NAN), Quality::Unrated);
    assert_eq!(scoring::classify(f64::INFINITY), Quality::Unrated);
    assert_eq!(scoring::loss(Color::White, 0.5, 0.2), 0.3);
    assert_eq!(scoring::loss(Color::Black, 0.2, 0.5), 0.3);
    assert_eq!(scoring::loss(Color::White, 0.2, 0.5), 0.0);
    assert_eq!(scoring::loss(Color::Black, 0.5, 0.2), 0.0);
    for (v, q) in [
        (0.0, Quality::Good),
        (scoring::GOOD_LOSS, Quality::Good),
        (scoring::GOOD_LOSS + 1e-8, Quality::Inaccuracy),
        (scoring::INACCURACY_LOSS, Quality::Inaccuracy),
        (scoring::INACCURACY_LOSS + 1e-8, Quality::Mistake),
        (scoring::MISTAKE_LOSS, Quality::Mistake),
        (scoring::MISTAKE_LOSS + 1e-8, Quality::Blunder),
    ] {
        assert_eq!(scoring::classify(v), q);
    }
    assert!(
        scoring::advantage(Score::Cp(4000)) - scoring::advantage(Score::Cp(2000))
            < scoring::GOOD_LOSS * 2.0
    );
    let mut j = plain_job(game(4));
    let p = point(cp(25));
    j.store(0, p.clone());
    j.store(1, p);
    assert_eq!(j.moves[0].quality, Quality::Best);
    assert!(j.moves[0].reliable && j.moves[0].matched);
    let mut p = point(cp(-900));
    p.depth = Some(1);
    j.store(1, p);
    assert!(j.moves[0].matched);
    assert_eq!(j.moves[0].quality, Quality::Unrated);
    assert_eq!(j.moves[0].reason, Reason::LowDepth);
}

#[test]
fn mate_bounds_missing_depth_and_unfinished_results_are_never_blunders() {
    for (a, b, expected) in [
        (point(cp(100)), None, Reason::Incomplete),
        (point(cp(100)), Some(point(None)), Reason::MissingScore),
        (
            point(Some((Score::Cp(100), Bound::Lower))),
            Some(point(cp(-1000))),
            Reason::NonExact,
        ),
        (
            point(Some((Score::Mate(3), Bound::Exact))),
            Some(point(cp(-1000))),
            Reason::Mate,
        ),
        (
            point(Some((Score::Mate(-3), Bound::Exact))),
            Some(point(Some((Score::Mate(-2), Bound::Exact)))),
            Reason::Mate,
        ),
    ] {
        let mut j = plain_job(game(4));
        j.store(0, a);
        if let Some(b) = b {
            j.store(1, b);
        }
        assert_eq!(j.moves[0].quality, Quality::Unrated);
        assert_eq!(j.moves[0].reason, expected);
    }
    let mut j = plain_job(game(4));
    let p = point(cp(0));
    j.store(0, p.clone());
    let mut p2 = p;
    p2.depth = Some(30);
    j.store(1, p2);
    assert_eq!(j.moves[0].reason, Reason::DepthGap);
    j.points[1].as_mut().unwrap().depth = None;
    j.refresh();
    assert_eq!(j.moves[0].reason, Reason::LowDepth);
    j.points[1].as_mut().unwrap().depth = Some(12);
    j.points[1].as_mut().unwrap().score = cp(-1000);
    j.refresh();
    assert_eq!(j.moves[0].reason, Reason::InconsistentBest);
    assert_eq!(j.moves[0].quality, Quality::Unrated);
    let mut j = plain_job(game(4));
    j.state = State::Running;
    j.begin(99, 0);
    let mut r = Report {
        status: Status::Searching,
        request: Some(99),
        best: Some("e2e4".into()),
        info: protocol::Info {
            score: cp(10),
            depth: Some(12),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(!j.accept(&r).unwrap());
    assert_eq!(j.done(), 0);
    r.status = Status::Complete;
    assert!(j.accept(&r).unwrap());
    assert_eq!(j.done(), 1);
    assert!(!j.accept(&r).unwrap());
    assert_eq!(j.done(), 1);
}

#[test]
fn pause_resume_cancel_restart_preserve_completed_results_and_ownership() {
    let (mut a, dir, _) = setup(game(10), "review pause");
    a.start_review().unwrap();
    pump(&mut a, |j| j.done() == 2 && j.pending());
    pending_started(&a);
    let first = a.review.job.as_ref().unwrap().points[..2].to_vec();
    a.pause_review();
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Paused);
    assert!(a.request_analysis().is_err());
    let original = a.analysis.config.clone();
    a.handle_analysis_action(analysis_ui::Action::MultiPV);
    a.handle_analysis_action(analysis_ui::Action::Threads);
    assert_eq!(a.analysis.config, original);
    let budget = a.review.options;
    a.handle_review_action(review_ui::ReviewAction::Time);
    assert_eq!(a.review.options, budget);
    wait(a.analysis.controller.as_ref().unwrap(), |r| {
        r.status == Status::Stopped
    });
    for _ in 0..5 {
        a.poll_review();
    }
    assert_eq!(a.review.job.as_ref().unwrap().done(), 2);
    a.resume_review().unwrap();
    completed(&mut a);
    assert_eq!(&a.review.job.as_ref().unwrap().points[..2], &first);
    let count = std::fs::read_to_string(dir.join("commands.log"))
        .unwrap()
        .matches("go movetime")
        .count();
    a.request_analysis().unwrap();
    let manual = a.analysis.report.request;
    wait(a.analysis.controller.as_ref().unwrap(), |r| {
        r.status == Status::Complete && r.request == manual
    });
    a.poll_analysis();
    assert!(a.analysis.report.best.is_some());
    a.start_review().unwrap();
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Completed); // Cached, no search.
    assert_eq!(
        std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .matches("go movetime")
            .count(),
        count + 1
    );
    assert!(a.shutdown_analysis());
    let (mut a, _, _) = setup(game(10), "review pause");
    a.start_review().unwrap();
    pump(&mut a, |j| j.done() == 2 && j.pending());
    pending_started(&a);
    a.cancel_review();
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Cancelled);
    a.start_review().unwrap();
    completed(&mut a);
    assert_eq!(a.review.job.as_ref().unwrap().done(), 11);
    assert!(a.shutdown_analysis());
}

#[test]
fn review_failure_crash_missing_score_and_bad_bestmove_are_explicit() {
    for mode in ["review crash", "review missing", "review illegal"] {
        let (mut a, dir, _) = setup(game(4), mode);
        if mode == "review missing" {
            responses(&dir, &game(4), "missing");
        }
        if mode == "review illegal" {
            let p = dir.join("review.tsv");
            let s = std::fs::read_to_string(&p).unwrap();
            std::fs::write(p, s.replace("\te2e4\t", "\ta1a8\t")).unwrap();
        }
        a.start_review().unwrap();
        pump(&mut a, |j| j.state == State::Failed);
        let j = a.review.job.as_ref().unwrap();
        assert!(j.error.is_some());
        assert!(j.done() < 5);
        assert!(!a.review.owns_engine());
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
    }
}

#[test]
fn cursor_navigation_and_old_reports_do_not_retarget_a_batch_or_new_game() {
    let (mut a, _, _) = setup(game(10), "review pause");
    a.start_review().unwrap();
    pump(&mut a, |j| j.done() == 2 && j.pending());
    pending_started(&a);
    let key = a.review.job.as_ref().unwrap().game_key.clone();
    for ply in [0, 10, 3, 7] {
        a.handle_record_action(records_ui::Action::Jump(ply));
        a.poll_review();
        assert_eq!(a.review.job.as_ref().unwrap().game_key, key);
        assert_eq!(a.review.job.as_ref().unwrap().state, State::Running);
    }
    a.pause_review();
    a.resume_review().unwrap();
    completed(&mut a);
    assert!(a.shutdown_analysis());
    let mut a = app();
    a.enter_replay(game(4), true);
    let report = Arc::new(Mutex::new(Report {
        status: Status::Ready,
        multipv: 1,
        engine_key: "controlled".into(),
        ..Default::default()
    }));
    a.analysis.controller = Some(Controller::with_test_report(report.clone()));
    a.start_review().unwrap();
    a.poll_review();
    let old = a.analysis.sequence;
    a.enter_replay(game(10), true);
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Cancelled);
    a.start_review().unwrap();
    a.poll_review();
    assert_ne!(old, a.analysis.sequence);
    *report.lock().unwrap() = Report {
        status: Status::Complete,
        multipv: 1,
        engine_key: "controlled".into(),
        request: Some(old),
        best: Some("e2e4".into()),
        info: protocol::Info {
            depth: Some(12),
            score: cp(10),
            ..Default::default()
        },
        ..Default::default()
    };
    a.poll_review();
    assert_eq!(a.review.job.as_ref().unwrap().done(), 0);
    a.close_analysis_engine();
    assert_eq!(a.review.job.as_ref().unwrap().state, State::Cancelled);
}

#[test]
fn cache_reload_partial_resume_identity_invalidation_and_corrupt_preservation() {
    let (mut a, dir, config) = setup(game(10), "review pause");
    a.start_review().unwrap();
    pump(&mut a, |j| j.done() == 2 && j.pending());
    pending_started(&a);
    a.pause_review();
    let key = a.review.job.as_ref().unwrap().cache_key.clone();
    assert!(a.shutdown_analysis());
    std::fs::write(dir.join("scenario.txt"), "review").unwrap();
    let mut b = app();
    b.records.save_path = Some(dir.join("autosave-v1.json"));
    b.enter_replay(game(10), true);
    b.analysis.config = config;
    b.enable_analysis().unwrap();
    ready(b.analysis.controller.as_ref().unwrap());
    b.start_review().unwrap();
    assert_eq!(b.review.job.as_ref().unwrap().done(), 2);
    completed(&mut b);
    let job = b.review.job.as_ref().unwrap();
    let path = review_cache::path(&dir, &key);
    assert!(path.exists());
    let data = std::fs::read(&path).unwrap();
    let mut fresh = plain_job(game(10));
    fresh.cache_key = key.clone();
    fresh.engine = job.engine.clone();
    assert!(review_cache::load(&path, &mut fresh).unwrap());
    assert_eq!(fresh.done(), 11);
    for field in ["schema", "algorithm"] {
        let mut envelope: serde_json::Value = serde_json::from_slice(&data).unwrap();
        let mut payload: serde_json::Value =
            serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
        payload[field] = serde_json::json!(999);
        let text = serde_json::to_string(&payload).unwrap();
        use sha2::{Digest, Sha256};
        envelope["sha256"] = serde_json::json!(format!("{:x}", Sha256::digest(text.as_bytes())));
        envelope["payload"] = serde_json::json!(text);
        std::fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(review_cache::load(&path, &mut fresh).is_err());
    }
    std::fs::write(&path, &data).unwrap();
    fresh.options.movetime = 1000;
    assert!(review_cache::load(&path, &mut fresh).is_err());
    fresh.options.movetime = 500;
    fresh.cache_key = "different engine/config/algorithm".into();
    assert!(review_cache::load(&path, &mut fresh).is_err());
    std::fs::write(&path, b"broken but valuable cache").unwrap();
    assert!(review_cache::save(&path, job).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"broken but valuable cache");
    b.start_review().unwrap();
    assert!(b.review.cache_blocked);
    assert!(b.review.cache_error.is_some());
    completed(&mut b);
    assert_eq!(std::fs::read(&path).unwrap(), b"broken but valuable cache");
    assert_eq!(
        std::fs::read(dir.join("autosave-v1.json")).unwrap(),
        b"user autosave sentinel"
    );
    assert!(b.shutdown_analysis());
    assert!(data.len() < review_cache::MAX_BYTES);
}

#[test]
fn cache_keys_cover_game_engine_options_and_file_limits_fail_safely() {
    let g = game(4);
    let engine = Report {
        name: "Mock".into(),
        engine_key: "binary-a".into(),
        ..Default::default()
    };
    let cfg = Config::default();
    let o = Options::default();
    let k = review_cache::key(&review_cache::game_key(&g), &cfg, &engine, o);
    let mut cfg2 = cfg.clone();
    cfg2.hash = 128;
    assert_ne!(
        k,
        review_cache::key(&review_cache::game_key(&g), &cfg2, &engine, o)
    );
    let engine2 = Report {
        engine_key: "binary-b".into(),
        ..engine.clone()
    };
    assert_ne!(
        k,
        review_cache::key(&review_cache::game_key(&g), &cfg, &engine2, o)
    );
    assert_ne!(
        review_cache::game_key(&g),
        review_cache::game_key(&game(10))
    );
    let (_, dir) = fixture("review");
    let mut job = plain_job(g);
    let path = review_cache::path(&dir, &job.cache_key);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    for n in 0..review_cache::MAX_FILES {
        std::fs::write(
            path.parent().unwrap().join(format!("{n}.review.json")),
            b"existing cache",
        )
        .unwrap();
    }
    assert!(
        review_cache::save(&path, &job)
            .unwrap_err()
            .contains("full")
    );
    let bad = dir.join("is-a-file");
    std::fs::write(&bad, b"x").unwrap();
    assert!(review_cache::save(&bad.join("cache.json"), &job).is_err());
    let big = dir.join("big.json");
    std::fs::write(&big, vec![b'x'; review_cache::MAX_BYTES + 1]).unwrap();
    assert!(review_cache::load(&big, &mut job).is_err());
}

#[test]
fn chart_gaps_sign_mate_cursor_hit_and_replay_jump_are_consistent() {
    let mut job = plain_job(game(20));
    job.points[0] = Some(point(cp(200)));
    job.points[1] = Some(point(cp(-200)));
    job.points[3] = Some(point(Some((Score::Mate(3), Bound::Exact))));
    job.points[4] = Some(point(Some((Score::Cp(52), Bound::Lower))));
    for size in [(640, 480), (1280, 720)] {
        let r = review_chart::rect(size);
        assert_eq!(review_chart::point(r, 20, 0, 0.0).0, r.0);
        assert_eq!(review_chart::point(r, 20, 20, 0.0).0, r.0 + r.2);
        let a = review_chart::point(r, 20, 0, scoring::advantage(Score::Cp(200)));
        let b = review_chart::point(r, 20, 1, scoring::advantage(Score::Cp(-200)));
        assert!(a.1 < r.1 + r.3 / 2.0 && b.1 > r.1 + r.3 / 2.0);
        assert_eq!(review_chart::hit(&job, r, a), Some(0));
        assert_eq!(review_chart::hit(&job, r, b), Some(1));
        assert!(review_chart::hit(&job, r, (r.0 + r.2, r.1 + 5.0)).is_none());
        assert!(review_chart::value(job.points[4].as_ref().unwrap()).is_none());
        let font = font::TextRenderer::load();
        let mut buf = vec![0; size.0 * size.1];
        let pal = theme::Theme::DarkPlus.palette();
        review_chart::draw(&mut buf, size, &font, &job, 20, &pal);
        // No interpolation through ply 2 to mate at ply 3.
        let gap = review_chart::point(r, 20, 2, 0.5);
        assert_eq!(buf[gap.1 as usize * size.0 + gap.0 as usize], pal.bg);
        let mut a = app();
        a.enter_replay(game(20), true);
        a.review.job = Some(job_for_chart(&job));
        a.review.panel = true;
        a.analysis.panel = true;
        let actions = analysis_ui::draw(&font, &mut buf, size, &a, Some(b), true);
        assert!(
            actions.contains(&analysis_ui::Action::Review(review_ui::ReviewAction::Jump(
                1
            )))
        );
        for action in actions {
            a.handle_analysis_action(action);
        }
        assert_eq!(a.records.replay.as_ref().unwrap().cursor, 1);
    }
}
fn job_for_chart(source: &Job) -> Job {
    let mut j = plain_job(source.game.clone());
    j.points = source.points.clone();
    j.refresh();
    j
}

#[test]
fn review_offscreen_all_languages_sizes_states_and_long_game_keep_board_clear() {
    let font = font::TextRenderer::load();
    let images = assets::PieceImages::load();
    let renderer = Renderer::new(&images, &font);
    let mut a = app();
    a.enter_replay(game(20), true);
    a.review.panel = true;
    a.analysis.panel = true;
    let mut j = plain_job(game(20));
    j.store(0, point(cp(200)));
    j.store(1, point(cp(-200)));
    j.points[3] = Some(point(Some((Score::Mate(-3), Bound::Exact))));
    a.review.job = Some(j);
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(review_ui::words(lang).iter().all(|s| !s.is_empty()));
        for size in [(640, 480), (1280, 720)] {
            for state in [
                State::Idle,
                State::Running,
                State::Paused,
                State::Completed,
                State::Failed,
            ] {
                a.review.job.as_mut().unwrap().state = state;
                let mut view = a.view_state(None, false, false, false);
                a.replay_view(&mut view);
                let mut buf = vec![0; size.0 * size.1];
                renderer.render(&mut buf, size.0, size.1, &view);
                let before = buf.clone();
                analysis_ui::draw(&font, &mut buf, size, &a, None, false);
                for y in 40..size.1 - 40 {
                    let edge = size.0 - 292;
                    assert_eq!(
                        &before[y * size.0..y * size.0 + edge],
                        &buf[y * size.0..y * size.0 + edge]
                    );
                }
            }
        }
    }
    // A long legal record made by repeatable king-safe knight moves; chart mapping
    // covers the full timeline, while analysis range remains explicitly bounded.
    let mut record = game(4).record;
    record.moves = vec!["g1f3".into(), "g8f6".into(), "f3g1".into(), "f6g8".into()]
        .into_iter()
        .cycle()
        .take(800)
        .collect();
    let long = record.validate().unwrap();
    let mut j = plain_job(long);
    assert_eq!(j.limit, 160);
    j.points[0] = Some(point(cp(0)));
    let r = review_chart::rect((640, 480));
    assert_eq!(review_chart::point(r, 800, 800, 0.0).0, r.0 + r.2);
    let mut buf = vec![0; 640 * 480];
    review_chart::draw(
        &mut buf,
        (640, 480),
        &font,
        &j,
        799,
        &theme::Theme::DarkPlus.palette(),
    );
}

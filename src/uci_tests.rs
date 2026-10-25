use super::*;
use crate::{
    analysis_ui::Action,
    uci::{Config, Controller, Request, Status},
    uci_protocol::{self as protocol, Bound, Message, Score},
};
use std::{
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

fn fixture(mode: &str) -> (Config, PathBuf) {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    static N: AtomicU64 = AtomicU64::new(0);
    let base = std::env::current_dir()
        .unwrap()
        .join("target/v021-uci-tests")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&base).unwrap();
    let bin = BIN.get_or_init(|| {
        let bin = base.join("mock-uci.exe");
        assert!(
            std::process::Command::new("rustc")
                .args([
                    "--edition=2024",
                    "-C",
                    "opt-level=1",
                    "tools/mock_uci_engine.rs",
                    "-o"
                ])
                .arg(&bin)
                .status()
                .unwrap()
                .success()
        );
        bin
    });
    let dir = base.join(format!(
        "引擎 path {} {mode}",
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mock engine.exe");
    std::fs::copy(bin, &path).unwrap();
    std::fs::write(dir.join("scenario.txt"), mode).unwrap();
    (
        Config {
            path,
            ..Config::default()
        },
        dir,
    )
}
fn wait(c: &Controller, predicate: impl Fn(&uci::Report) -> bool) -> uci::Report {
    let until = Instant::now() + Duration::from_secs(6);
    loop {
        let r = c.report();
        if predicate(&r) {
            return r;
        }
        assert!(Instant::now() < until, "UCI wait: {r:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn ready(c: &Controller) {
    let r = wait(c, |r| matches!(r.status, Status::Ready | Status::Failed));
    assert_eq!(r.status, Status::Ready, "{r:?}");
}
fn request(id: u64) -> Request {
    Request {
        id,
        context: 1,
        position: "position startpos".into(),
        movetime: 100,
    }
}
fn app() -> App {
    let mut a = App::new();
    a.settings = Settings::default();
    a.screen = Screen::Game;
    a.new_game();
    a
}

#[test]
fn protocol_identity_options_unknown_fields_and_ready() {
    assert_eq!(
        protocol::parse(" id name Example Unicode 棋 ").unwrap(),
        Message::Name("Example Unicode 棋".into())
    );
    assert_eq!(protocol::parse("readyok").unwrap(), Message::ReadyOk);
    assert_eq!(protocol::parse("uciok").unwrap(), Message::UciOk);
    let Message::Option(o) =
        protocol::parse("option name Hash type spin default 16 min 1 max 256").unwrap()
    else {
        panic!()
    };
    assert_eq!(
        (o.name.as_str(), o.kind.as_str(), o.min, o.max),
        ("Hash", "spin", Some(1), Some(256))
    );
    assert_eq!(protocol::parse("registration ok").unwrap(), Message::Other);
    assert!(protocol::parse(&"x".repeat(16385)).is_err());
}
#[test]
fn scores_mate_bounds_multipv_and_tolerant_info() {
    for (text, score, bound) in [
        ("cp 35", Score::Cp(35), Bound::Exact),
        ("mate -3 lowerbound", Score::Mate(-3), Bound::Lower),
    ] {
        let Message::Info(i) = protocol::parse(&format!(
            "info depth 18 seldepth 20 score {text} nps 123 nodes 456 time 12 pv e2e4 e7e5"
        ))
        .unwrap() else {
            panic!()
        };
        assert_eq!(i.score, Some((score, bound)));
        assert_eq!(i.depth, Some(18));
        assert_eq!(i.nodes, Some(456));
        assert_eq!(i.time, Some(12));
        assert_eq!(i.pv, Some(vec!["e2e4".into(), "e7e5".into()]));
    }
    let Message::Info(second) = protocol::parse("info multipv 2 score cp 5 pv d2d4").unwrap()
    else {
        panic!()
    };
    assert_eq!(second.rank, 2);
    assert!(protocol::parse("info score cp NaN").is_err());
    assert!(protocol::parse("info score mate -2147483648").is_err());
    assert_eq!(
        protocol::white_score((Score::Cp(52), Bound::Lower), Color::Black),
        (Score::Cp(-52), Bound::Upper)
    );
    assert_eq!(
        protocol::white_score((Score::Mate(-3), Bound::Upper), Color::Black),
        (Score::Mate(3), Bound::Lower)
    );
}
#[test]
fn legal_moves_san_castling_capture_promotion_en_passant_terminal() {
    for (fen, mv, san) in [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "g1f3",
            "Nf3",
        ),
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "e1g1", "O-O"),
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "e1c1", "O-O-O"),
        ("7k/5P2/8/8/8/8/8/K7 w - - 0 1", "f7f8q", "f8=Q+"),
        ("7k/8/8/3pP3/8/8/8/K7 w - d6 0 1", "e5d6", "exd6"),
        ("7k/8/8/3p4/4P3/8/8/K7 w - - 0 1", "e4d5", "exd5"),
    ] {
        assert_eq!(
            protocol::legal_move(&fen::parse(fen).unwrap(), mv)
                .unwrap()
                .as_deref(),
            Some(san)
        );
    }
    assert!(protocol::legal_move(&Chess::default(), "a1a8").is_err());
    assert!(protocol::legal_move(&Chess::default(), "0000").is_err());
    let pos = fen::parse("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    assert_eq!(protocol::legal_move(&pos, "0000").unwrap(), None);
    assert_eq!(protocol::legal_move(&pos, "(none)").unwrap(), None);
}
#[test]
fn position_preserves_history_fen_and_black_start() {
    let game = pgn::import(b"1. e4 e5 2. Nf3 *").unwrap();
    assert_eq!(
        protocol::position(&game.positions[0], &game.moves),
        "position startpos moves e2e4 e7e5 g1f3"
    );
    let pos = fen::parse("7k/8/8/8/8/8/P7/K7 b - - 17 42").unwrap();
    assert_eq!(
        protocol::position(&pos, &[]),
        "position fen 7k/8/8/8/8/8/P7/K7 b - - 17 42"
    );
    let counted = fen::parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 12 42").unwrap();
    assert_eq!(
        protocol::position(&counted, &[]),
        "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 12 42"
    );
}
#[test]
fn process_handshake_unicode_path_resources_and_complete_multiline_info() {
    let (cfg, dir) = fixture("normal");
    let mut c = Controller::start(cfg);
    ready(&c);
    assert_eq!(c.report().name, "Local Mock UCI 0.1");
    assert_eq!(c.report().author, "RUST CHESS QA");
    c.request(request(1));
    let r = wait(&c, |r| r.status == Status::Complete);
    assert_eq!(r.request, Some(1));
    assert_eq!(r.info.depth, Some(12));
    assert_eq!(r.info.score, Some((Score::Cp(52), Bound::Exact)));
    assert_eq!(r.best.as_deref(), Some("e2e4"));
    assert!(c.shutdown());
    let log = std::fs::read_to_string(dir.join("commands.log")).unwrap();
    for command in [
        "setoption name Threads value 2",
        "setoption name Hash value 64",
        "setoption name MultiPV value 1",
        "ucinewgame",
        "isready",
        "quit",
    ] {
        assert!(log.contains(command), "{log}");
    }
    assert!(c.report().pid.is_none());
}
#[test]
fn missing_engine_and_relative_path_fail_without_ui_blocking() {
    for path in [
        PathBuf::from("missing.exe"),
        std::env::current_dir()
            .unwrap()
            .join("target/no-such-engine.exe"),
    ] {
        let mut c = Controller::start(Config {
            path,
            ..Config::default()
        });
        assert_eq!(
            wait(&c, |r| r.status == Status::Failed).status,
            Status::Failed
        );
        assert!(c.shutdown());
    }
}
#[test]
fn process_startup_timeout_exit_invalid_line_and_ready_timeout() {
    for mode in ["hang", "exit", "longline", "noready"] {
        let (cfg, _) = fixture(mode);
        let mut c = Controller::start(cfg);
        let r = wait(&c, |r| r.status == Status::Failed);
        assert!(r.error.is_some());
        assert!(c.shutdown());
    }
}
#[test]
fn stderr_flood_and_optional_options_do_not_deadlock() {
    let (cfg, _) = fixture("stderr nooptions");
    let mut c = Controller::start(cfg);
    ready(&c);
    assert_eq!(c.report().warnings.len(), 3);
    c.request(request(1));
    wait(&c, |r| r.status == Status::Complete);
    assert!(c.shutdown());
}
#[test]
fn cancel_waits_for_bestmove_then_starts_latest_request() {
    let (cfg, dir) = fixture("hold");
    let mut c = Controller::start(cfg);
    ready(&c);
    c.request(request(1));
    wait(&c, |r| r.info.depth == Some(12));
    for _ in 0..20 {
        c.stop();
    }
    c.request(request(2));
    let r = wait(&c, |r| {
        r.request == Some(2) && r.status == Status::Searching
    });
    assert_eq!(r.request, Some(2));
    c.stop();
    wait(&c, |r| r.status == Status::Stopped);
    assert!(c.shutdown());
    let log = std::fs::read_to_string(dir.join("commands.log")).unwrap();
    let first = log.find("go movetime").unwrap();
    let second = log[first + 1..].find("go movetime").unwrap() + first + 1;
    assert!(
        log[first..second].contains("stop\nisready\nposition startpos"),
        "{log}"
    );
    assert_eq!(log.matches("ucinewgame").count(), 1);
}
#[test]
fn readyok_never_substitutes_for_missing_bestmove() {
    let (cfg, dir) = fixture("nostop");
    let mut c = Controller::start(cfg);
    ready(&c);
    c.request(request(1));
    wait(&c, |r| r.info.depth == Some(12));
    c.request(request(2));
    wait(&c, |r| r.status == Status::Failed);
    assert!(c.shutdown());
    assert_eq!(
        std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .matches("go movetime")
            .count(),
        1
    );
}
#[test]
fn analysis_result_is_white_view_and_never_plays_a_move() {
    let mut a = app();
    a.apply_move(
        "e2e4"
            .parse::<shakmaty::uci::UciMove>()
            .unwrap()
            .to_move(&a.pos)
            .unwrap(),
    );
    let (cfg, _) = fixture("normal bound");
    a.analysis.config = cfg;
    a.enable_analysis().unwrap();
    ready(a.analysis.controller.as_ref().unwrap());
    a.request_analysis().unwrap();
    wait(a.analysis.controller.as_ref().unwrap(), |r| {
        r.status == Status::Complete
    });
    let before = a.snapshot();
    a.poll_analysis();
    assert_eq!(
        a.analysis.report.info.score,
        Some((Score::Cp(-52), Bound::Upper))
    );
    assert_eq!(a.analysis_san("e7e5").as_deref(), Some("e5"));
    assert_eq!(before.current_fen, a.snapshot().current_fen);
    assert_eq!(before.record.moves, a.snapshot().record.moves);
    assert_eq!(before.clocks, a.clocks);
    assert!(a.shutdown_analysis());
}
#[test]
fn illegal_engine_bestmove_and_pv_are_visible_errors() {
    for mode in ["badbest", "badpv"] {
        let mut a = app();
        let (cfg, _) = fixture(mode);
        a.analysis.config = cfg;
        a.enable_analysis().unwrap();
        ready(a.analysis.controller.as_ref().unwrap());
        a.request_analysis().unwrap();
        wait(a.analysis.controller.as_ref().unwrap(), |r| {
            r.status == Status::Complete
        });
        a.poll_analysis();
        assert!(
            a.analysis
                .error
                .as_deref()
                .unwrap()
                .contains("Illegal UCI move")
        );
        assert!(a.history.is_empty());
        assert!(a.analysis.report.best.is_none());
        assert!(a.shutdown_analysis());
    }
}
#[test]
fn stale_searches_after_move_undo_newgame_fen_and_replay() {
    let mut a = app();
    let (cfg, _) = fixture("hold");
    a.analysis.config = cfg;
    a.enable_analysis().unwrap();
    ready(a.analysis.controller.as_ref().unwrap());
    for change in 0..5 {
        a.request_analysis().unwrap();
        wait(a.analysis.controller.as_ref().unwrap(), |r| {
            r.status == Status::Searching
        });
        match change {
            0 => a.apply_move(
                "e2e4"
                    .parse::<shakmaty::uci::UciMove>()
                    .unwrap()
                    .to_move(&a.pos)
                    .unwrap(),
            ),
            1 => a.undo(),
            2 => a.new_game(),
            3 => a.import_fen("7k/8/8/8/8/8/P7/K7 b - - 0 1").unwrap(),
            _ => a.enter_replay(pgn::import(b"1. d4 d5 2. c4 *").unwrap(), true),
        }
        a.poll_analysis();
        assert!(a.analysis.report.info.score.is_none());
        assert!(a.analysis.report.best.is_none());
    }
    assert!(a.shutdown_analysis());
}
#[test]
fn replay_uses_exact_cursor_prefix_and_fast_jumps_cancel() {
    let mut a = app();
    a.enter_replay(pgn::import(b"1. e4 e5 2. Nf3 *").unwrap(), true);
    a.handle_record_action(records_ui::Action::Jump(1));
    let (cfg, dir) = fixture("hold");
    a.analysis.config = cfg;
    a.enable_analysis().unwrap();
    ready(a.analysis.controller.as_ref().unwrap());
    a.request_analysis().unwrap();
    wait(a.analysis.controller.as_ref().unwrap(), |r| {
        r.info.depth == Some(12)
    });
    for n in [0, 3, 1, 2, 0] {
        a.handle_record_action(records_ui::Action::Jump(n));
        a.poll_analysis();
        assert!(a.analysis.report.best.is_none());
    }
    assert!(a.history.is_empty());
    assert!(a.shutdown_analysis());
    assert!(
        std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .contains("position startpos moves e2e4\n")
    );
}
#[test]
fn no_engine_defaults_bounds_old_settings_and_snapshot_exclusion() {
    let mut a = app();
    assert!(a.analysis.controller.is_none());
    assert_eq!(a.analysis.config.threads, 2);
    assert_eq!(a.analysis.config.hash, 64);
    let mut cfg: Config =
        serde_json::from_str(r#"{"threads":99999,"hash":99999,"movetime":1}"#).unwrap();
    cfg.clamp();
    assert_eq!((cfg.threads, cfg.hash, cfg.movetime), (8, 256, 100));
    assert_eq!(cfg.multipv, 1); // v0.2.1 config has no MultiPV field.
    cfg.multipv = u32::MAX;
    cfg.clamp();
    assert_eq!(cfg.multipv, 3);
    cfg.multipv = 0;
    cfg.clamp();
    assert_eq!(cfg.multipv, 1);
    a.analysis.config.path = PathBuf::from("C:/not executed.exe");
    assert!(
        !serde_json::to_string(&a.snapshot())
            .unwrap()
            .contains("not executed")
    );
    assert!(a.request_analysis().is_err());
}
#[test]
fn panel_renders_eleven_languages_without_board_overlap() {
    let images = assets::PieceImages::load();
    let font = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &font);
    let mut a = app();
    a.analysis.panel = true;
    for screen in [Screen::Game, Screen::Replay] {
        if screen == Screen::Replay {
            a.enter_replay(pgn::import(b"1. e4 e5 *").unwrap(), true);
        }
        for language in i18n::Language::ALL {
            a.settings.language = language;
            assert!(analysis_ui::words(language).iter().all(|s| !s.is_empty()));
            for size in [(640, 480), (1280, 720)] {
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
                assert!(analysis_ui::captures(
                    &a,
                    size,
                    Some((size.0 as f32 - 100.0, 250.0)),
                    true
                ));
                assert!(!analysis_ui::captures(&a, size, Some((50.0, 250.0)), true));
            }
        }
    }
    a.handle_analysis_action(Action::Stop);
}

#[test]
fn stalled_stdin_and_invalid_executable_fail_with_bounded_cleanup() {
    let (cfg, dir) = fixture("stallwrite");
    let mut c = Controller::start(cfg);
    ready(&c);
    let mut r = request(1);
    r.position = format!("position startpos moves {}", "a1a2 ".repeat(4096));
    c.request(r);
    wait(&c, |r| r.status == Status::Failed);
    assert!(c.shutdown());
    let bad = dir.join("invalid engine.exe");
    std::fs::write(&bad, b"not an executable").unwrap();
    let mut c = Controller::start(Config {
        path: bad,
        ..Config::default()
    });
    wait(&c, |r| r.status == Status::Failed);
    assert!(c.shutdown());
}
#[test]
fn panel_does_not_pause_clock_and_terminal_does_not_search() {
    let mut a = app();
    a.settings.board.chess_clock_enabled = true;
    a.analysis.panel = true;
    a.tick_clock_at(a.clock_last_tick + Duration::from_secs(3));
    assert_eq!(a.clocks.white, Duration::from_secs(597));
    a.import_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let (cfg, dir) = fixture("normal");
    a.analysis.config = cfg;
    a.enable_analysis().unwrap();
    ready(a.analysis.controller.as_ref().unwrap());
    a.request_analysis().unwrap();
    a.poll_analysis();
    assert_eq!(a.analysis.report.status, Status::Complete);
    assert!(a.shutdown_analysis());
    assert!(
        !std::fs::read_to_string(dir.join("commands.log"))
            .unwrap()
            .contains("go ")
    );
}
#[test]
fn analysis_preferences_load_bounded_and_never_auto_execute() {
    let mut a = app();
    let (cfg, dir) = fixture("normal");
    a.records.save_path = Some(dir.join("autosave-v1.json"));
    std::fs::write(dir.join("analysis.cfg"), serde_json::to_vec(&cfg).unwrap()).unwrap();
    a.init_analysis();
    assert_eq!(a.analysis.config, cfg);
    assert!(a.analysis.controller.is_none());
    a.handle_analysis_action(Action::Time);
    assert_eq!(a.analysis.config.movetime, 3000);
    let loaded: Config =
        serde_json::from_slice(&std::fs::read(dir.join("analysis.cfg")).unwrap()).unwrap();
    assert_eq!(loaded.movetime, 3000);
    a.handle_analysis_action(Action::MultiPV);
    let loaded: Config =
        serde_json::from_slice(&std::fs::read(dir.join("analysis.cfg")).unwrap()).unwrap();
    assert_eq!(loaded.multipv, 2);
    assert!(a.analysis.controller.is_none());
    std::fs::write(dir.join("analysis.cfg"), vec![b'x'; 16385]).unwrap();
    a.init_analysis();
    assert!(a.analysis.error.is_some());
    assert!(a.analysis.controller.is_none());
}
#[test]
fn shutdown_reaps_process_even_during_handshake_or_search() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn WaitForSingleObject(handle: *mut std::ffi::c_void, ms: u32) -> u32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    for mode in ["hang", "nostop"] {
        let (cfg, _) = fixture(mode);
        let mut c = Controller::start(cfg);
        let pid = wait(&c, |r| r.pid.is_some()).pid.unwrap();
        if mode == "nostop" {
            ready(&c);
            c.request(request(1));
            wait(&c, |r| r.status == Status::Searching);
        }
        let handle = unsafe { OpenProcess(0x00100000, 0, pid) };
        assert!(
            !handle.is_null(),
            "Cannot observe test child {pid}: {}",
            std::io::Error::last_os_error()
        );
        assert!(c.shutdown());
        assert_eq!(unsafe { WaitForSingleObject(handle, 0) }, 0);
        unsafe {
            CloseHandle(handle);
        }
    }
}

#[test]
fn search_exit_or_missing_bestmove_deadline_fails_and_engine_can_restart() {
    for mode in ["exitsearch", "nostop"] {
        let (cfg, _) = fixture(mode);
        let mut a = app();
        a.analysis.config = cfg;
        a.enable_analysis().unwrap();
        ready(a.analysis.controller.as_ref().unwrap());
        a.analysis.config.movetime = 100;
        a.request_analysis().unwrap();
        let report = wait(a.analysis.controller.as_ref().unwrap(), |r| {
            r.status == Status::Failed
        });
        assert!(
            report.error.as_deref().is_some_and(|s| s.contains("exited")
                || s.contains("closed")
                || s.contains("timed out")),
            "{report:?}"
        );
        assert!(a.history.is_empty());
        assert!(a.shutdown_analysis());
        a.close_analysis_engine();
        let (cfg, _) = fixture("normal");
        a.analysis.config = cfg;
        a.enable_analysis().unwrap();
        ready(a.analysis.controller.as_ref().unwrap());
        a.request_analysis().unwrap();
        wait(a.analysis.controller.as_ref().unwrap(), |r| {
            r.status == Status::Complete
        });
        a.poll_analysis();
        assert!(a.analysis.report.error.is_none());
        assert_eq!(a.analysis_san("e2e4").as_deref(), Some("e4"));
        assert!(a.shutdown_analysis());
    }
}

#[test]
fn previous_request_failure_does_not_relabel_old_scores_as_current() {
    let report = std::sync::Arc::new(std::sync::Mutex::new(uci::Report {
        status: Status::Ready,
        ..uci::Report::default()
    }));
    let mut a = app();
    a.analysis.controller = Some(Controller::with_test_report(report.clone()));
    a.request_analysis().unwrap();
    let old = a.analysis.report.request;
    a.cancel_analysis();
    a.request_analysis().unwrap();
    assert_ne!(a.analysis.report.request, old);
    *report.lock().unwrap() = uci::Report {
        status: Status::Failed,
        request: old,
        info: protocol::Info {
            depth: Some(12),
            score: Some((Score::Cp(52), Bound::Exact)),
            pv: Some(vec!["e2e4".into()]),
            ..Default::default()
        },
        best: Some("e2e4".into()),
        error: Some("Previous search failed to stop".into()),
        ..uci::Report::default()
    };
    a.poll_analysis();
    assert_eq!(a.analysis.report.status, Status::Failed);
    assert!(a.analysis.report.error.is_some());
    assert_eq!(a.analysis.report.info, protocol::Info::default());
    assert!(a.analysis.report.best.is_none());
    assert!(a.history.is_empty());
}

fn info(text: &str) -> protocol::Info {
    let Message::Info(i) = protocol::parse(text).unwrap() else {
        panic!("{text}")
    };
    i
}
fn words_pv(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

#[test]
fn full_pv_fields_order_limits_and_incomplete_messages() {
    let parsed =
        info("info pv e2e4 e7e5 g1f3 score mate -3 upperbound nodes 20 multipv 3 depth 18 time 7");
    assert_eq!(parsed.rank, 3);
    assert_eq!(parsed.depth, Some(18));
    assert_eq!(parsed.score, Some((Score::Mate(-3), Bound::Upper)));
    assert_eq!(parsed.pv, Some(words_pv("e2e4 e7e5 g1f3")));
    assert_eq!((parsed.nodes, parsed.time), (Some(20), Some(7)));
    assert_eq!(info("info depth 19").pv, None);
    assert_eq!(info("info score cp 0").depth, None);
    for bad in [
        "info multipv 0",
        "info multipv 4",
        "info multipv -1",
        "info multipv X",
        "info pv",
    ] {
        assert!(protocol::parse(bad).is_err(), "{bad}");
    }
    assert!(protocol::parse(&format!("info pv {}", "e2e4 ".repeat(128))).is_ok());
    assert!(protocol::parse(&format!("info pv {}", "e2e4 ".repeat(129))).is_err());
    assert!(protocol::parse(&"info ".repeat(4000)).is_err());
}

#[test]
fn rank_and_depth_updates_never_reuse_previous_depth_score_or_other_rank() {
    let mut slots: [Option<protocol::Info>; 3] = Default::default();
    for message in [
        "info depth 18 multipv 2 score cp 31 pv d2d4 d7d5",
        "info depth 18 multipv 1 score cp 52 pv e2e4 e7e5",
        "info depth 19 multipv 1 pv g1f3 g8f6",
    ] {
        let i = info(message);
        let index = i.rank - 1;
        protocol::update_line(&mut slots[index], i);
    }
    assert!(slots[0].as_ref().unwrap().score.is_none());
    assert_eq!(
        slots[1].as_ref().unwrap().score,
        Some((Score::Cp(31), Bound::Exact))
    );
    assert!(slots[2].is_none());
    protocol::update_line(&mut slots[0], info("info depth 17 score cp 999 pv a1a8"));
    assert_eq!(slots[0].as_ref().unwrap().depth, Some(19));
    protocol::update_line(&mut slots[0], info("info depth 19 score cp 42"));
    assert_eq!(slots[0].as_ref().unwrap().pv, Some(words_pv("g1f3 g8f6")));
    protocol::update_line(&mut slots[0], info("info pv e2e4"));
    assert!(slots[0].as_ref().unwrap().score.is_none());
    assert!(slots[0].as_ref().unwrap().depth.is_none());
}

#[test]
fn full_san_numbering_special_moves_and_failure_prefix_are_read_only() {
    for (fen_text, pv, expected, from, to) in [
        (
            fen::write(&Chess::default()),
            "e2e4 e7e5 g1f3 b8c6 f1b5",
            "1. e4 e5 2. Nf3 Nc6 3. Bb5",
            Square::E2,
            Square::E4,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 17 42".into(),
            "e7e5 g1f3 b8c6",
            "42... e5 43. Nf3 Nc6",
            Square::E7,
            Square::E5,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1".into(),
            "e1g1 e8c8",
            "1. O-O O-O-O",
            Square::E1,
            Square::G1,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 42".into(),
            "e8g8",
            "42... O-O",
            Square::E8,
            Square::G8,
        ),
        (
            "7k/5P2/8/8/8/8/8/K7 w - - 0 1".into(),
            "f7f8q h8h7",
            "1. f8=Q+ Kh7",
            Square::F7,
            Square::F8,
        ),
        (
            "7k/8/8/3pP3/8/8/8/K7 w - d6 0 1".into(),
            "e5d6",
            "1. exd6",
            Square::E5,
            Square::D6,
        ),
        (
            "7k/8/5KQ1/8/8/8/8/8 w - - 0 1".into(),
            "g6g7",
            "1. Qg7#",
            Square::G6,
            Square::G7,
        ),
    ] {
        let pos = fen::parse(&fen_text).unwrap();
        let v = protocol::variation(&pos, &words_pv(pv));
        assert_eq!(v.error, None, "{fen_text}");
        assert_eq!(v.san, expected);
        assert_eq!(v.arrow, Some((from, to)));
        assert_eq!(fen::write(&pos), fen_text);
    }
    let broken = protocol::variation(&Chess::default(), &words_pv("e2e4 e7e5 a1a8"));
    assert_eq!(broken.san, "1. e4 e5");
    assert!(broken.error.unwrap().contains("ply 3"));
    assert!(broken.arrow.is_none());
    assert!(
        protocol::variation(&Chess::default(), &words_pv("0000"))
            .error
            .is_some()
    );
}

#[test]
fn multipv_process_configuration_fallback_limits_and_stop_boundary() {
    for (mode, count, expected) in [
        ("multi", 1, 1),
        ("multi", 2, 2),
        ("multi", 3, 3),
        ("nomultipv", 3, 1),
        ("multi limited", 3, 2),
    ] {
        let (mut cfg, _) = fixture(mode);
        cfg.multipv = count;
        let mut c = Controller::start(cfg);
        ready(&c);
        c.request(request(1));
        let r = wait(&c, |r| r.status == Status::Complete);
        assert_eq!(r.multipv, expected);
        assert_eq!(r.lines.iter().flatten().count(), expected);
        if expected != count as usize {
            assert!(r.warnings.iter().any(|s| s.starts_with("MultiPV:")));
        }
        if mode.starts_with("multi") {
            assert_eq!(r.lines[0].as_ref().unwrap().depth, Some(19));
            assert_eq!(
                r.lines[0].as_ref().unwrap().score,
                Some((Score::Cp(55), Bound::Exact))
            );
            if expected > 1 {
                assert_eq!(
                    r.lines[1].as_ref().unwrap().pv,
                    Some(words_pv("d2d4 d7d5 c2c4"))
                );
            }
        }
        assert!(c.shutdown());
    }
    let (cfg, dir) = fixture("multi hold");
    let mut c = Controller::start(cfg);
    ready(&c);
    c.request(request(1));
    wait(&c, |r| r.info.depth == Some(19));
    c.set_multipv(3);
    c.request(request(2));
    let r = wait(&c, |r| r.request == Some(2) && r.lines[2].is_some());
    assert_eq!(r.multipv, 3);
    c.set_multipv(2);
    c.request(request(3));
    let r = wait(&c, |r| r.request == Some(3) && r.lines[1].is_some());
    assert!(r.lines[2].is_none());
    assert!(c.shutdown());
    let log = std::fs::read_to_string(dir.join("commands.log")).unwrap();
    for count in [2, 3] {
        assert!(log.contains(&format!("stop\nisready\nsetoption name MultiPV value {count}\nisready\nposition startpos\ngo")), "{log}");
    }
}

fn inject_lines(a: &mut App, lines: &[&str]) -> std::sync::Arc<std::sync::Mutex<uci::Report>> {
    let report = std::sync::Arc::new(std::sync::Mutex::new(uci::Report {
        status: Status::Ready,
        ..Default::default()
    }));
    a.analysis.panel = true;
    a.analysis.controller = Some(Controller::with_test_report(report.clone()));
    a.request_analysis().unwrap();
    let mut r = report.lock().unwrap();
    r.status = Status::Complete;
    r.request = a.analysis.report.request;
    r.multipv = lines.len();
    for text in lines {
        let i = info(text);
        let index = i.rank - 1;
        r.lines[index] = Some(i);
    }
    r.info = r.lines[0].clone().unwrap_or_default();
    drop(r);
    a.poll_analysis();
    report
}

#[test]
fn all_candidate_results_invalidate_on_game_replay_and_engine_changes() {
    for change in 0..9 {
        let mut a = app();
        if change == 4 || change == 5 {
            a.enter_replay(pgn::import(b"1. e4 e5 *").unwrap(), true);
            a.handle_record_action(records_ui::Action::First);
        }
        let report = inject_lines(
            &mut a,
            &[
                "info depth 10 score cp 52 pv e2e4 e7e5",
                "info depth 10 multipv 2 score cp 20 pv d2d4 d7d5",
                "info depth 10 multipv 3 score cp 10 pv g1f3 g8f6",
            ],
        );
        assert!(a.analysis_visual().arrow.is_some());
        match change {
            0 => {
                let mv = "e2e4"
                    .parse::<shakmaty::uci::UciMove>()
                    .unwrap()
                    .to_move(&a.pos)
                    .unwrap();
                a.apply_move(mv);
                a.undo();
            }
            1 => a.new_game(),
            2 => a.import_fen("7k/8/8/8/8/8/P7/K7 b - - 0 1").unwrap(),
            3 => a.enter_replay(pgn::import(b"1. d4 *").unwrap(), true),
            4 => a.handle_record_action(records_ui::Action::Next),
            5 => a.leave_replay(),
            6 => a.close_analysis_engine(),
            7 => a.cancel_analysis(),
            _ => {
                a.request_analysis().unwrap();
            }
        }
        a.poll_analysis(); // Re-deliver the entire old three-line report.
        assert!(
            a.analysis.report.lines.iter().all(Option::is_none),
            "change {change}"
        );
        assert!(a.analysis.variations.iter().all(Option::is_none));
        assert!(a.analysis_visual().score.is_none());
        assert!(a.analysis_visual().arrow.is_none());
        drop(report);
    }
}

#[test]
fn replay_full_pv_isolation_black_scores_invalid_candidate_and_selection() {
    let mut a = app();
    let before = a.snapshot();
    a.enter_replay(pgn::import(b"1. e4 e5 2. Nf3 *").unwrap(), true);
    a.handle_record_action(records_ui::Action::Jump(1));
    inject_lines(
        &mut a,
        &[
            "info depth 18 score cp 52 lowerbound pv e7e5 g1f3",
            "info depth 17 multipv 2 score mate -3 pv c7c5 g1f3",
            "info depth 19 multipv 3 score cp 20 pv d7d5 a1a8",
        ],
    );
    assert_eq!(
        a.analysis.report.lines[0].as_ref().unwrap().score,
        Some((Score::Cp(-52), Bound::Upper))
    );
    assert_eq!(
        a.analysis.variations[0].as_ref().unwrap().san,
        "1... e5 2. Nf3"
    );
    assert!(a.analysis.variations[2].as_ref().unwrap().error.is_some());
    assert!(a.analysis.report.lines[2].as_ref().unwrap().score.is_none());
    a.handle_analysis_action(Action::Select(1));
    assert_eq!(
        a.analysis_visual().score,
        Some((Score::Mate(3), Bound::Exact))
    );
    assert_eq!(a.analysis_visual().arrow, Some((Square::C7, Square::C5)));
    a.handle_analysis_action(Action::Select(2));
    assert!(a.analysis_visual().arrow.is_none());
    let cursor = a.records.replay.as_ref().unwrap().cursor;
    a.tick_clock_at(a.clock_last_tick + Duration::from_secs(20));
    assert_eq!(a.clocks, before.clocks);
    assert_eq!(a.history.len(), 0);
    assert_eq!(cursor, 1);
    assert_eq!(fen::write(&a.pos), before.current_fen);
    assert_eq!(a.snapshot().record.moves, before.record.moves);
}

#[test]
fn evaluation_fraction_labels_extremes_bounds_and_mate_zero() {
    use crate::analysis_board::{score_label, white_fraction};
    let cp = |n| Some((Score::Cp(n), Bound::Exact));
    assert_eq!(white_fraction(None), 0.5);
    assert_eq!(white_fraction(cp(0)), 0.5);
    let mut previous = 0.0;
    for n in [i32::MIN, -4000, -1000, -50, 0, 50, 1000, 4000, i32::MAX] {
        let value = white_fraction(cp(n));
        assert!((0.0..=1.0).contains(&value));
        assert!(value >= previous);
        previous = value;
    }
    for (n, fraction) in [(3, 1.0), (-3, 0.0)] {
        assert_eq!(
            white_fraction(Some((Score::Mate(n), Bound::Exact))),
            fraction
        );
    }
    for (side, text, fraction) in [(Color::White, "M-0", 0.0), (Color::Black, "M+0", 1.0)] {
        let value = Some(protocol::white_score((Score::Mate(0), Bound::Exact), side));
        assert_eq!(score_label(value), text);
        assert_eq!(white_fraction(value), fraction);
    }
    assert_eq!(score_label(Some((Score::Cp(52), Bound::Lower))), ">=+0.52");
    assert_eq!(score_label(Some((Score::Mate(-3), Bound::Upper))), "<=M-3");
}

#[test]
fn overlay_pixels_geometry_flips_sizes_and_castling_targets() {
    let font = font::TextRenderer::load();
    let base = 0x00555555;
    for size in [(640, 480), (1280, 720)] {
        let l = render::Layout::new(size.0, size.1, false);
        for flipped in [false, true] {
            let (fx, fy) = l.square_center(Square::E2, flipped);
            let (tx, ty) = l.square_center(Square::E4, flipped);
            assert_eq!(fx, tx);
            assert_eq!(fy > ty, !flipped);
            for (score, white_fraction) in [
                (None, 0.5),
                (Some((Score::Cp(0), Bound::Exact)), 0.5),
                (Some((Score::Mate(3), Bound::Exact)), 1.0),
                (Some((Score::Mate(-3), Bound::Exact)), 0.0),
                (Some((Score::Cp(400), Bound::Lower)), 0.75),
                (Some((Score::Cp(-400), Bound::Upper)), 0.25),
                (
                    Some((Score::Cp(i32::MAX), Bound::Exact)),
                    analysis_board::white_fraction(Some((Score::Cp(4000), Bound::Exact))),
                ),
                (
                    Some((Score::Cp(i32::MIN), Bound::Exact)),
                    analysis_board::white_fraction(Some((Score::Cp(-4000), Bound::Exact))),
                ),
            ] {
                let mut buf = vec![base; size.0 * size.1];
                analysis_board::draw(
                    &mut buf,
                    size,
                    &font,
                    analysis_board::Visual {
                        active: true,
                        score,
                        arrow: Some((Square::E2, Square::E4)),
                    },
                    &l,
                    flipped,
                    &theme::Theme::DarkPlus.palette(),
                );
                let x = (l.board_left - 16.0) as usize;
                let white = (l.board_top as usize..(l.board_top + l.board_size) as usize)
                    .filter(|y| buf[y * size.0 + x] == 0x00eeeeee)
                    .count();
                assert!((white as f64 - f64::from(l.board_size) * white_fraction).abs() <= 1.0);
                let midpoint = ((fy + ty) / 2.0) as usize * size.0 + fx as usize;
                assert_ne!(buf[midpoint], base);
                assert_eq!(
                    buf[l.square_center(Square::A4, flipped).1 as usize * size.0
                        + l.square_center(Square::A4, flipped).0 as usize],
                    base
                );
            }
            for (fen_text, mv, from, to) in [
                (
                    "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
                    "e1c1",
                    Square::E1,
                    Square::C1,
                ),
                (
                    "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
                    "e8g8",
                    Square::E8,
                    Square::G8,
                ),
            ] {
                let v = protocol::variation(&fen::parse(fen_text).unwrap(), &words_pv(mv));
                assert_eq!(v.arrow, Some((from, to)));
                let mut buf = vec![base; size.0 * size.1];
                analysis_board::draw(
                    &mut buf,
                    size,
                    &font,
                    analysis_board::Visual {
                        active: true,
                        score: None,
                        arrow: v.arrow,
                    },
                    &l,
                    flipped,
                    &theme::Theme::DarkPlus.palette(),
                );
                let a = l.square_center(from, flipped);
                let b = l.square_center(to, flipped);
                assert_ne!(
                    buf[((a.1 + b.1) / 2.0) as usize * size.0 + ((a.0 + b.0) / 2.0) as usize],
                    base
                );
            }
        }
    }
}

#[test]
fn candidate_rows_click_wrap_and_eleven_language_offscreen_layouts() {
    let images = assets::PieceImages::load();
    let font = font::TextRenderer::load();
    let renderer = Renderer::new(&images, &font);
    let mut a = app();
    inject_lines(
        &mut a,
        &[
            "info depth 18 score cp 52 pv e2e4 e7e5 g1f3 b8c6 f1b5",
            "info depth 18 multipv 2 score cp 31 pv d2d4 d7d5 c2c4",
            "info depth 17 multipv 3 score cp 20 pv g1f3 g8f6 d2d4",
        ],
    );
    for lang in i18n::Language::ALL {
        a.settings.language = lang;
        assert!(
            analysis_ui::words(lang)
                .iter()
                .chain(analysis_ui::extra_words(lang).iter())
                .all(|s| !s.is_empty())
        );
        for size in [(640, 480), (1280, 720)] {
            for i in 0..3 {
                let rect = analysis_ui::candidate_rect(size, i);
                assert!(rect.1 >= 246.0 && rect.1 + rect.3 < size.1 as f32 - 36.0);
                let mut buf = vec![0; size.0 * size.1];
                let view = a.view_state(None, false, false, false);
                renderer.render(&mut buf, size.0, size.1, &view);
                let before = buf.clone();
                let actions = analysis_ui::draw(
                    &font,
                    &mut buf,
                    size,
                    &a,
                    Some((rect.0 + 5.0, rect.1 + 5.0)),
                    true,
                );
                assert_eq!(actions, vec![Action::Select(i)]);
                a.handle_analysis_action(actions[0]);
                assert_eq!(a.analysis.selected_line, i);
                assert_eq!(
                    a.analysis_visual().arrow,
                    Some(match i {
                        0 => (Square::E2, Square::E4),
                        1 => (Square::D2, Square::D4),
                        _ => (Square::G1, Square::F3),
                    })
                );
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
    let full = a.analysis.variations[0].as_ref().unwrap().san.clone();
    let wrapped = analysis_ui::wrap_pv(&font, &full.repeat(10), 80.0, 2);
    assert_eq!(wrapped.len(), 2);
    assert!(wrapped[1].ends_with('…'));
    assert_eq!(a.analysis.variations[0].as_ref().unwrap().san, full);
}

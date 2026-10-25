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
        assert_eq!(i.pv.as_deref(), Some("e2e4"));
    }
    assert_eq!(
        protocol::parse("info multipv 2 score cp 5 pv d2d4").unwrap(),
        Message::Other
    );
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
            pv: Some("e2e4".into()),
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

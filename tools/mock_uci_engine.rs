//! Local QA fixture only. Never linked into or bundled with RUST CHESS.
use std::{fs::OpenOptions, io::{self, BufRead, Write}, sync::{Arc, Mutex, mpsc}, thread, time::Duration};
fn emit(out: &Mutex<io::Stdout>, value: &str) {
    let mut out = out.lock().unwrap(); writeln!(out,"{value}").unwrap(); out.flush().unwrap();
}
fn main() {
    let mode = std::fs::read_to_string("scenario.txt").unwrap_or_else(|_| "normal".into());
    let out = Arc::new(Mutex::new(io::stdout()));
    if mode.contains("stderr") {
        thread::spawn(|| { let mut e = io::stderr(); for _ in 0..2048 { let _ = e.write_all(&[b'x';4096]); } });
    }
    let mut log = OpenOptions::new().create(true).append(true).open("commands.log").unwrap();
    let mut stop: Option<mpsc::Sender<()>> = None;
    let mut position = String::new();
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break; };
        writeln!(log,"{line}").unwrap(); log.flush().unwrap();
        if line == "uci" {
            if mode == "exit" { return; }
            if mode.contains("hang") { continue; }
            if mode.contains("longline") { emit(&out, &"x".repeat(17000)); continue; }
            emit(&out,"id name Local Mock UCI 0.1"); emit(&out,"id author RUST CHESS QA");
            if !mode.contains("nooptions") {
                emit(&out,"option name Threads type spin default 1 min 1 max 4");
                emit(&out,"option name Hash type spin default 16 min 1 max 128");
                emit(&out,"option name MultiPV type spin default 1 min 1 max 3");
            }
            emit(&out,"info string harmless startup information"); emit(&out,"uciok");
        } else if line == "isready" {
            if !mode.contains("noready") { emit(&out,"readyok"); }
            if mode.contains("stallwrite") { thread::sleep(Duration::from_secs(30)); }
        } else if line.starts_with("position ") { position = line; }
        else if line.starts_with("go ") {
            if mode == "exitsearch" { return; }
            let (tx, rx) = mpsc::channel(); stop = Some(tx);
            let (out, mode, position) = (out.clone(),mode.clone(),position.clone());
            thread::spawn(move || {
                emit(&out,"info depth 8 score cp 35 nodes 100 time 1");
                let best = if mode.contains("badbest") { "a1a8" } else if position.ends_with("e2e4") { "e7e5" } else { "e2e4" };
                let pv = if mode.contains("badpv") { "a1a8" } else { best };
                emit(&out,&format!("info depth 12 score cp 52 {} nodes 200 time 2 pv {pv}", if mode.contains("bound") { "lowerbound" } else { "" }));
                if mode.contains("nostop") { thread::sleep(Duration::from_secs(30)); }
                else if mode.contains("hold") { let _ = rx.recv_timeout(Duration::from_secs(30)); }
                else { let _ = rx.recv_timeout(Duration::from_millis(40)); }
                emit(&out,&format!("bestmove {best}"));
            });
        } else if line == "stop" {
            if !mode.contains("nostop") && let Some(stop) = stop.take() { let _ = stop.send(()); }
        } else if line == "quit" { break; }
    }
}

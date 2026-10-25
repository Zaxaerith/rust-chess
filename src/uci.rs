//! One controller owns one process. UI commands and reports are latest-value
//! mailboxes; pipes have bounded queues, lines and deadlines. No shell involved.
use crate::uci_protocol::{self as protocol, EngineOption, Info, Message};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub path: PathBuf,
    pub threads: u32,
    pub hash: u32,
    pub movetime: u32,
    pub multipv: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            threads: 2,
            hash: 64,
            movetime: 1000,
            multipv: 1,
        }
    }
}
impl Config {
    pub fn clamp(&mut self) {
        self.threads = self.threads.clamp(1, 8);
        self.hash = self.hash.clamp(1, 256);
        self.movetime = self.movetime.clamp(100, 10000);
        self.multipv = self.multipv.clamp(1, 3);
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Off,
    Starting,
    Ready,
    Searching,
    Complete,
    Stopped,
    Failed,
}
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub status: Status,
    pub name: String,
    pub author: String,
    pub error: Option<String>,
    pub request: Option<u64>,
    pub info: Info,
    pub lines: [Option<Info>; 3],
    pub multipv: usize,
    pub best: Option<String>,
    pub warnings: Vec<String>,
    pub pid: Option<u32>,
    pub engine_key: String,
}
#[derive(Clone, Debug)]
pub struct Request {
    pub id: u64,
    pub context: u64,
    pub position: String,
    pub movetime: u32,
}
#[derive(Default)]
struct Desired {
    serial: u64,
    request: Option<Request>,
    quit: bool,
    multipv: u32,
}
pub struct Controller {
    desired: Arc<Mutex<Desired>>,
    report: Arc<Mutex<Report>>,
    worker: Option<JoinHandle<()>>,
}
impl Controller {
    #[cfg(test)]
    pub(crate) fn with_test_report(report: Arc<Mutex<Report>>) -> Self {
        Self {
            desired: Arc::new(Mutex::new(Desired::default())),
            report,
            worker: None,
        }
    }
    pub fn start(mut config: Config) -> Self {
        config.clamp();
        let desired = Arc::new(Mutex::new(Desired {
            multipv: config.multipv,
            ..Desired::default()
        }));
        let report = Arc::new(Mutex::new(Report {
            status: Status::Starting,
            ..Report::default()
        }));
        let (d, r) = (desired.clone(), report.clone());
        let worker = thread::spawn(move || {
            if let Err(error) = run(&config, &d, &r) {
                let mut state = r.lock().expect("UCI report");
                state.status = if d.lock().expect("UCI request").quit {
                    Status::Off
                } else {
                    Status::Failed
                };
                state.error = Some(error);
            }
            r.lock().expect("UCI report").pid = None;
        });
        Self {
            desired,
            report,
            worker: Some(worker),
        }
    }
    pub fn report(&self) -> Report {
        self.report.lock().expect("UCI report").clone()
    }
    pub fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(|w| w.is_finished())
    }
    pub fn request(&self, request: Request) {
        let mut d = self.desired.lock().expect("UCI request");
        d.serial = d.serial.wrapping_add(1);
        d.request = Some(request);
    }
    pub fn stop(&self) {
        let mut d = self.desired.lock().expect("UCI request");
        d.serial = d.serial.wrapping_add(1);
        d.request = None;
    }
    pub fn set_multipv(&self, count: u32) {
        let mut d = self.desired.lock().expect("UCI request");
        d.multipv = count.clamp(1, 3);
        d.serial = d.serial.wrapping_add(1);
        d.request = None;
    }
    pub fn close(&self) {
        let mut d = self.desired.lock().expect("UCI request");
        d.quit = true;
        d.serial = d.serial.wrapping_add(1);
        d.request = None;
    }
    pub fn shutdown(&mut self) -> bool {
        self.close();
        let until = Instant::now() + Duration::from_secs(2);
        while self.worker.as_ref().is_some_and(|w| !w.is_finished()) && Instant::now() < until {
            thread::sleep(Duration::from_millis(5));
        }
        if self.worker.as_ref().is_some_and(|w| w.is_finished())
            && let Some(worker) = self.worker.take()
        {
            return worker.join().is_ok();
        }
        self.worker.is_none()
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.close();
    }
}

struct WriteJob {
    bytes: String,
    ack: SyncSender<Result<(), String>>,
}
struct Engine {
    child: Child,
    input: SyncSender<WriteJob>,
    output: Receiver<Result<String, String>>,
    readers: Vec<JoinHandle<()>>,
}
impl Engine {
    fn spawn(path: &Path) -> Result<Self, String> {
        if !path.is_absolute()
            || !path.is_file()
            || path
                .extension()
                .is_none_or(|e| !e.eq_ignore_ascii_case("exe"))
        {
            return Err("Select an existing absolute local UCI .exe path".into());
        }
        if path.to_string_lossy().starts_with(r"\\") {
            return Err("Network engine paths are not supported".into());
        }
        let mut command = Command::new(path);
        command
            .current_dir(path.parent().ok_or("Engine requires a working directory")?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command
            .spawn()
            .map_err(|e| format!("Start UCI engine: {e}"))?;
        let mut stdin = child.stdin.take().ok_or("Missing engine stdin")?;
        let stdout = child.stdout.take().ok_or("Missing engine stdout")?;
        let mut stderr = child.stderr.take().ok_or("Missing engine stderr")?;
        let (input, jobs) = mpsc::sync_channel::<WriteJob>(1);
        let writer = thread::spawn(move || {
            while let Ok(job) = jobs.recv() {
                let result = stdin
                    .write_all(job.bytes.as_bytes())
                    .and_then(|_| stdin.flush())
                    .map_err(|e| format!("UCI write: {e}"));
                let failed = result.is_err();
                let _ = job.ack.try_send(result);
                if failed {
                    break;
                }
            }
        });
        let (lines, output) = mpsc::sync_channel(64);
        let reader = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut bytes = Vec::with_capacity(1024);
                let result = reader.by_ref().take(16385).read_until(b'\n', &mut bytes);
                let message = match result {
                    Ok(0) => Err("UCI stdout closed / engine exited".into()),
                    Ok(_) if bytes.len() > 16384 => Err("UCI output line exceeds 16 KiB".into()),
                    Ok(_) => {
                        String::from_utf8(bytes).map_err(|_| "Invalid UTF-8 from engine".into())
                    }
                    Err(e) => Err(format!("UCI read: {e}")),
                };
                let failed = message.is_err();
                if lines.send(message).is_err() || failed {
                    break;
                }
            }
        });
        let drain = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut buf) {
                if n == 0 {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            readers: vec![writer, reader, drain],
        })
    }
    fn send(&self, command: &str) -> Result<(), String> {
        if command.len() > 32768 || command.contains(['\n', '\r']) {
            return Err("Invalid/oversized UCI command".into());
        }
        let (ack, receive) = mpsc::sync_channel(1);
        self.input
            .try_send(WriteJob {
                bytes: format!("{command}\n"),
                ack,
            })
            .map_err(|_| "UCI writer unavailable")?;
        receive
            .recv_timeout(Duration::from_millis(200))
            .map_err(|_| "UCI stdin timed out")?
    }
    fn receive(
        &mut self,
        deadline: Instant,
        desired: &Mutex<Desired>,
    ) -> Result<Option<Message>, String> {
        if desired.lock().expect("UCI request").quit {
            return Err("Engine closed".into());
        }
        if self.child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return Err("UCI engine exited".into());
        }
        if Instant::now() >= deadline {
            return Err("UCI response timed out".into());
        }
        match self.output.recv_timeout(Duration::from_millis(10)) {
            Ok(Ok(line)) => protocol::parse(&line).map(Some),
            Ok(Err(e)) => Err(e),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(_) => Err("UCI output disconnected".into()),
        }
    }
    fn barrier(&mut self, desired: &Mutex<Desired>) -> Result<(), String> {
        self.send("isready")?;
        let until = Instant::now() + Duration::from_secs(2);
        loop {
            if matches!(self.receive(until, desired)?, Some(Message::ReadyOk)) {
                return Ok(());
            }
        }
    }
    fn stop_search(&mut self, desired: &Mutex<Desired>) -> Result<(), String> {
        self.send("stop")?;
        let until = Instant::now() + Duration::from_millis(600);
        loop {
            if matches!(self.receive(until, desired)?, Some(Message::Best(_))) {
                break;
            }
        }
        // readyok alone NEVER ends a search. Only after consuming bestmove can
        // we synchronize and submit a replacement position/go command.
        self.barrier(desired)
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.send("stop");
        let _ = self.send("quit");
        let mut until = Instant::now() + Duration::from_millis(300);
        let mut kill_attempted = false;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(e) => {
                    eprintln!("UCI cleanup status failed: {e}");
                    break;
                }
            }
            if Instant::now() >= until {
                if kill_attempted {
                    eprintln!("UCI engine did not exit within the cleanup deadline");
                    break;
                }
                if let Err(e) = self.child.kill() {
                    eprintln!("UCI engine termination failed: {e}");
                }
                kill_attempted = true;
                until = Instant::now() + Duration::from_millis(300);
            }
            thread::sleep(Duration::from_millis(5));
        }
        for reader in self.readers.drain(..) {
            if reader.is_finished() {
                let _ = reader.join();
            }
        }
    }
}
fn configure(
    engine: &Engine,
    options: &[EngineOption],
    cfg: &Config,
) -> Result<(Vec<String>, usize), String> {
    let mut warnings = vec![];
    for (name, value, limit) in [("Threads", cfg.threads, 8), ("Hash", cfg.hash, 256)] {
        if let Some(option) = options
            .iter()
            .find(|o| o.name.eq_ignore_ascii_case(name) && o.kind == "spin")
        {
            let min = option.min.unwrap_or(1).max(1);
            let max = option.max.unwrap_or(i64::from(limit)).min(i64::from(limit));
            if min > max {
                return Err(format!("Engine {name} range exceeds safe resource limit"));
            }
            let safe = i64::from(value).clamp(min, max);
            engine.send(&format!("setoption name {} value {safe}", option.name))?;
        } else {
            warnings.push(format!("{name}: unsupported; engine default"));
        }
    }
    let (count, warning) = configure_multipv(engine, options, cfg.multipv)?;
    warnings.extend(warning);
    Ok((warnings, count))
}
fn configure_multipv(
    engine: &Engine,
    options: &[EngineOption],
    count: u32,
) -> Result<(usize, Option<String>), String> {
    let Some(option) = options
        .iter()
        .find(|o| o.name.eq_ignore_ascii_case("MultiPV") && o.kind == "spin")
    else {
        return Ok((1, Some("MultiPV: unsupported; single PV only".into())));
    };
    let min = option.min.unwrap_or(1).max(1);
    let max = option.max.unwrap_or(3).min(3);
    if min > max {
        return Err("Engine MultiPV range exceeds safe resource limit".into());
    }
    let safe = i64::from(count.clamp(1, 3)).clamp(min, max);
    engine.send(&format!("setoption name {} value {safe}", option.name))?;
    Ok((
        safe as usize,
        (safe != i64::from(count)).then(|| format!("MultiPV: engine range; using {safe}")),
    ))
}
fn run(cfg: &Config, desired: &Mutex<Desired>, report: &Mutex<Report>) -> Result<(), String> {
    let mut engine = Engine::spawn(&cfg.path)?;
    report.lock().expect("UCI report").pid = Some(engine.child.id());
    engine.send("uci")?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut options = vec![];
    loop {
        match engine.receive(deadline, desired)? {
            Some(Message::UciOk) => break,
            Some(Message::Name(name)) => report.lock().expect("UCI report").name = name,
            Some(Message::Author(author)) => report.lock().expect("UCI report").author = author,
            Some(Message::Option(option)) => {
                if options.len() >= 256 {
                    return Err("Too many UCI options".into());
                }
                options.push(option);
            }
            _ => {}
        }
    }
    let (warnings, mut count) = configure(&engine, &options, cfg)?;
    // Identify the selected executable once on the background worker, never
    // hash a potentially large engine on the GUI/render thread.
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(&cfg.path).map_err(|e| format!("Engine identity: {e}"))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 256 * 1024 * 1024 {
        return Err("Engine identity exceeds 256 MiB limit".into());
    }
    let mut digest = Sha256::new();
    let mut chunk = [0u8; 65536];
    loop {
        let n = file.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        digest.update(&chunk[..n]);
    }
    report.lock().expect("UCI report").engine_key = format!("{:x}", digest.finalize());
    let mut configured = cfg.multipv;
    engine.barrier(desired)?;
    {
        let mut r = report.lock().expect("UCI report");
        r.status = Status::Ready;
        r.warnings = warnings;
        r.multipv = count;
    }
    let mut serial = 0;
    let mut context = None;
    let mut active: Option<(Request, Instant)> = None;
    loop {
        let (next_serial, next, quit, multipv) = {
            let d = desired.lock().expect("UCI request");
            (d.serial, d.request.clone(), d.quit, d.multipv)
        };
        if quit {
            return Ok(());
        }
        if next_serial != serial {
            if active.take().is_some() {
                engine.stop_search(desired)?;
            }
            if multipv != configured {
                // stop_search has consumed bestmove and synchronized. No option
                // write can race a running search, including rapid UI changes.
                let (effective, warning) = configure_multipv(&engine, &options, multipv)?;
                engine.barrier(desired)?;
                count = effective;
                configured = multipv;
                let mut r = report.lock().expect("UCI report");
                r.multipv = count;
                r.warnings.retain(|s| !s.starts_with("MultiPV:"));
                r.warnings.extend(warning);
            }
            serial = next_serial;
            if let Some(request) = next {
                if context != Some(request.context) {
                    engine.send("ucinewgame")?;
                    engine.barrier(desired)?;
                    context = Some(request.context);
                }
                engine.send(&request.position)?;
                let ms = request.movetime.clamp(100, 10000);
                engine.send(&format!("go movetime {ms}"))?;
                let mut r = report.lock().expect("UCI report");
                r.request = Some(request.id);
                r.info = Info::default();
                r.lines = Default::default();
                r.best = None;
                r.error = None;
                r.status = Status::Searching;
                active = Some((
                    request,
                    Instant::now() + Duration::from_millis(u64::from(ms) + 1500),
                ));
            } else {
                let mut r = report.lock().expect("UCI report");
                r.status = Status::Stopped;
                r.request = None;
                r.info = Info::default();
                r.lines = Default::default();
                r.best = None;
            }
        }
        let until = active
            .as_ref()
            .map(|(_, deadline)| *deadline)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(60));
        match engine.receive(until, desired)? {
            Some(Message::Info(info)) if active.is_some() => {
                if info.rank > count {
                    continue;
                }
                let mut r = report.lock().expect("UCI report");
                let rank = info.rank;
                protocol::update_line(&mut r.lines[rank - 1], info);
                r.info = r.lines[0].clone().unwrap_or_default();
            }
            Some(Message::Best(best)) => {
                if active.take().is_none() {
                    return Err("Unexpected bestmove outside a search".into());
                }
                let mut r = report.lock().expect("UCI report");
                r.best = Some(best);
                r.status = Status::Complete;
            }
            _ => {}
        }
    }
}

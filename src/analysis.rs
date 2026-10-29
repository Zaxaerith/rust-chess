//! Analysis is a view-only service: no game moves, clocks or snapshots are owned here.
use crate::{
    App, Position, Screen,
    uci::{Config, Controller, Report, Request, Status},
    uci_protocol,
};
use std::{io::Read, path::PathBuf};

#[derive(Default)]
pub struct Analysis {
    pub config: Config,
    pub panel: bool,
    pub controller: Option<Controller>,
    retired: Option<Controller>,
    pub report: Report,
    pub error: Option<String>,
    target: Option<Target>,
    pub(crate) sequence: u64,
    pub context: u64,
    pub external_context: u64,
    pub live_context: u64,
    pub config_path: Option<PathBuf>,
    pub variations: [Option<uci_protocol::Variation>; 3],
    pub selected_line: usize,
    cached: [Option<uci_protocol::Info>; 3],
}
struct Target {
    id: u64,
    stamp: String,
    position: shakmaty::Chess,
}
impl Drop for Analysis {
    fn drop(&mut self) {
        for c in [&mut self.controller, &mut self.retired]
            .into_iter()
            .flatten()
        {
            let _ = c.shutdown();
        }
    }
}
impl App {
    pub(crate) fn init_analysis(&mut self) {
        let path = self
            .records
            .save_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.join("analysis.cfg"));
        if let Some(path) = &path {
            let read = std::fs::File::open(path).and_then(|file| {
                let mut bytes = vec![];
                file.take(16385).read_to_end(&mut bytes)?;
                Ok(bytes)
            });
            match read {
                Ok(bytes) if bytes.len() <= 16384 => match serde_json::from_slice::<Config>(&bytes)
                {
                    Ok(mut config) => {
                        config.clamp();
                        self.analysis.config = config;
                    }
                    Err(e) => self.analysis.error = Some(format!("analysis.cfg: {e}")),
                },
                Ok(_) => self.analysis.error = Some("analysis.cfg exceeds 16 KiB".into()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => self.analysis.error = Some(format!("analysis.cfg: {e}")),
            }
        }
        self.analysis.config_path = path;
        // Paths are preferences, never permission to execute automatically.
    }
    pub(crate) fn save_analysis_config(&mut self) {
        if let Some(path) = &self.analysis.config_path {
            let result = serde_json::to_vec_pretty(&self.analysis.config)
                .map_err(|e| e.to_string())
                .and_then(|bytes| crate::persistence::atomic_write(path, &bytes));
            if let Err(e) = result {
                self.analysis.error = Some(e);
            }
        }
    }
    pub(crate) fn analysis_context(&mut self, external: bool) {
        self.cancel_review();
        self.review.panel = false;
        self.cancel_analysis();
        self.analysis.context = self.analysis.context.wrapping_add(1);
        if external {
            self.analysis.external_context = self.analysis.context;
        } else {
            self.analysis.live_context = self.analysis.context;
        }
    }
    pub(crate) fn analysis_stamp(&self) -> String {
        if self.screen == Screen::PositionWorkspace {
            return format!(
                "Workspace:{}:{}:{}",
                self.analysis.context,
                self.workspace.revision,
                self.display_fen()
            );
        }
        let cursor = self.records.replay.as_ref().map(|r| r.node);
        format!(
            "{:?}:{}:{}:{cursor:?}:{}",
            self.screen,
            self.generation,
            self.analysis.context,
            self.display_fen()
        )
    }
    pub(crate) fn cancel_analysis(&mut self) {
        if !self.review.owns_engine()
            && let Some(c) = &self.analysis.controller
        {
            c.stop();
        }
        self.analysis.target = None;
        self.analysis.report.info = Default::default();
        self.analysis.report.lines = Default::default();
        self.analysis.variations = Default::default();
        self.analysis.cached = Default::default();
        self.analysis.selected_line = 0;
        self.analysis.report.best = None;
        self.analysis.report.request = None;
        if self.analysis.controller.is_some() {
            self.analysis.report.status = Status::Stopped;
        }
    }
    pub(crate) fn close_analysis_engine(&mut self) {
        self.cancel_review();
        self.cancel_analysis();
        if let Some(c) = self.analysis.controller.take() {
            c.close();
            self.analysis.retired = Some(c);
        }
        self.analysis.report = Report::default();
    }
    pub(crate) fn enable_analysis(&mut self) -> Result<(), String> {
        if self.analysis.controller.is_some() {
            return Ok(());
        }
        if self
            .analysis
            .retired
            .as_ref()
            .is_some_and(|c| !c.finished())
        {
            return Err("Previous engine is still closing".into());
        }
        self.analysis.retired = None;
        if self.analysis.config.path.as_os_str().is_empty() {
            return Err("Select a UCI engine first".into());
        }
        self.analysis.error = None;
        self.analysis.controller = Some(Controller::start(self.analysis.config.clone()));
        self.analysis.report = Report {
            status: Status::Starting,
            ..Report::default()
        };
        Ok(())
    }
    pub(crate) fn request_analysis(&mut self) -> Result<(), String> {
        if self.screen == Screen::Replay
            && self
                .records
                .replay
                .as_ref()
                .is_some_and(|r| !r.on_mainline())
        {
            return Err(crate::advanced_ui::words(self.settings.language)[21].into());
        }
        if self.review.owns_engine() {
            return Err("Game Review owns engine; cancel it before manual analysis".into());
        }
        if !matches!(
            self.screen,
            Screen::Game | Screen::Replay | Screen::PositionWorkspace
        ) {
            return Err("Analysis requires a game, replay or position workspace".into());
        }
        if self.screen == Screen::PositionWorkspace
            && (self.workspace.chess().is_none() || self.workspace.modal())
        {
            return Err("Validate the workspace and enter Study before UCI analysis".into());
        }
        let controller = self
            .analysis
            .controller
            .as_ref()
            .ok_or("Enable a UCI engine first")?;
        if matches!(
            controller.report().status,
            Status::Starting | Status::Failed | Status::Off
        ) {
            return Err("Engine is not ready".into());
        }
        let (initial, moves, position, context) = if self.screen == Screen::PositionWorkspace {
            let s = self.workspace.study.as_ref().ok_or("No workspace study")?;
            (
                &s.positions[0],
                &s.moves[..s.cursor],
                s.position().clone(),
                self.analysis.context,
            )
        } else if let Some(r) = &self.records.replay {
            (
                &r.game.positions[0],
                &r.game.moves[..r.cursor],
                r.game.positions[r.cursor].clone(),
                if r.external {
                    self.analysis.external_context
                } else {
                    self.analysis.live_context
                },
            )
        } else {
            (
                &self.records.initial,
                self.history.as_slice(),
                self.pos.clone(),
                self.analysis.live_context,
            )
        };
        let command = uci_protocol::position(initial, moves);
        if command.len() > 32768 {
            return Err("Analysis move history exceeds UCI command limit".into());
        }
        self.analysis.sequence = self.analysis.sequence.wrapping_add(1);
        let id = self.analysis.sequence;
        let stamp = self.analysis_stamp();
        self.analysis.error = None;
        self.analysis.variations = Default::default();
        self.analysis.cached = Default::default();
        self.analysis.selected_line = 0;
        if position.legal_moves().is_empty()
            || (self.screen == Screen::PositionWorkspace
                && self
                    .workspace
                    .study
                    .as_ref()
                    .is_some_and(|s| s.result().is_some()))
        {
            self.cancel_analysis();
            self.analysis.target = Some(Target {
                id,
                stamp,
                position,
            });
            self.analysis.report.status = Status::Complete;
            self.analysis.report.request = Some(id);
            return Ok(());
        }
        self.analysis.target = Some(Target {
            id,
            stamp,
            position,
        });
        controller.request(Request {
            id,
            context,
            position: command,
            movetime: self.analysis.config.movetime,
        });
        self.analysis.report.info = Default::default();
        self.analysis.report.lines = Default::default();
        self.analysis.report.best = None;
        self.analysis.report.status = Status::Searching;
        self.analysis.report.request = Some(id);
        Ok(())
    }
    pub(crate) fn poll_analysis(&mut self) {
        if self.review.owns_engine() {
            return;
        }
        if self
            .analysis
            .target
            .as_ref()
            .is_some_and(|t| t.stamp != self.analysis_stamp())
        {
            self.cancel_analysis();
        }
        let Some(controller) = &self.analysis.controller else {
            return;
        };
        let mut next = controller.report();
        if self.analysis.target.is_none() {
            next.info = Default::default();
            next.lines = Default::default();
            next.best = None;
            next.request = None;
            if matches!(next.status, Status::Searching | Status::Complete) {
                next.status = Status::Stopped;
            }
        } else if let Some(target) = &self.analysis.target {
            if next.request != Some(target.id) {
                if next.status == Status::Failed {
                    // The engine may fail while stopping the previous request.
                    // Surface the transport failure, never its old position data.
                    next.info = Default::default();
                    next.lines = Default::default();
                    next.best = None;
                    next.request = None;
                    self.analysis.report = next;
                }
                return;
            }
            let validate = (|| {
                if let Some(best) = &next.best {
                    uci_protocol::legal_move(&target.position, best)?;
                }
                Ok::<_, String>(())
            })();
            if let Err(e) = validate {
                self.analysis.error = Some(e);
                self.cancel_analysis();
                return;
            }
            // Convert only changed bounded PVs; GUI drawing never parses a line.
            // Keep per-rank errors and valid prefixes without discarding other ranks.
            self.analysis.error = None;
            for i in 0..3 {
                if self.analysis.cached[i].as_ref().and_then(|l| l.pv.as_ref())
                    != next.lines[i].as_ref().and_then(|l| l.pv.as_ref())
                {
                    self.analysis.variations[i] = next.lines[i]
                        .as_ref()
                        .and_then(|line| line.pv.as_ref())
                        .map(|pv| uci_protocol::variation(&target.position, pv));
                }
                self.analysis.cached[i] = next.lines[i].clone();
                if let Some(v) = &self.analysis.variations[i]
                    && let Some(e) = &v.error
                {
                    self.analysis.error = Some(e.clone());
                    if let Some(line) = &mut next.lines[i] {
                        line.score = None;
                    }
                    if i == 0 {
                        next.best = None;
                        next.info.score = None;
                    }
                }
                if let Some(line) = &mut next.lines[i] {
                    line.score = line
                        .score
                        .map(|s| uci_protocol::white_score(s, target.position.turn()));
                }
            }
            next.info.score = next
                .info
                .score
                .map(|s| uci_protocol::white_score(s, target.position.turn()));
        }
        self.analysis.report = next;
    }
    pub(crate) fn analysis_visual(&self) -> crate::analysis_board::Visual {
        if self.review.panel
            && let Some(replay) = &self.records.replay
            && replay.on_mainline()
            && let Some(job) = &self.review.job
            && job.game_key == replay.identity
        {
            let point = job.points.get(replay.cursor).and_then(|p| p.as_ref());
            return crate::analysis_board::Visual {
                active: true,
                score: point.and_then(|p| p.score),
                arrow: point
                    .and_then(|p| p.best.as_ref())
                    .and_then(|m| m.parse::<shakmaty::uci::UciMove>().ok())
                    .and_then(|m| m.to_move(&replay.game.positions[replay.cursor]).ok())
                    .and_then(|m| uci_protocol::move_arrow(&m)),
            };
        }
        let neutral = crate::analysis_board::Visual {
            active: self.analysis.panel
                && (matches!(self.screen, Screen::Game | Screen::Replay)
                    || (self.screen == Screen::PositionWorkspace
                        && self.workspace.chess().is_some())),
            ..Default::default()
        };
        if !self.analysis.panel
            || self
                .analysis
                .target
                .as_ref()
                .is_none_or(|t| t.stamp != self.analysis_stamp())
            || !matches!(
                self.analysis.report.status,
                Status::Searching | Status::Complete
            )
        {
            return neutral;
        }
        let i = self.analysis.selected_line.min(2);
        let invalid = self.analysis.variations[i]
            .as_ref()
            .is_some_and(|v| v.error.is_some());
        let score = if invalid {
            None
        } else {
            self.analysis.report.lines[i].as_ref().and_then(|l| l.score)
        };
        let arrow = self.analysis.variations[i]
            .as_ref()
            .and_then(|v| v.arrow)
            .or_else(|| {
                if i != 0 || invalid {
                    return None;
                }
                let target = self.analysis.target.as_ref()?;
                self.analysis
                    .report
                    .best
                    .as_ref()?
                    .parse::<shakmaty::uci::UciMove>()
                    .ok()?
                    .to_move(&target.position)
                    .ok()
                    .and_then(|mv| uci_protocol::move_arrow(&mv))
            });
        crate::analysis_board::Visual {
            active: true,
            score,
            arrow,
        }
    }
    pub(crate) fn analysis_san(&self, text: &str) -> Option<String> {
        self.analysis
            .target
            .as_ref()
            .and_then(|t| uci_protocol::legal_move(&t.position, text).ok().flatten())
    }
    pub(crate) fn shutdown_analysis(&mut self) -> bool {
        self.cancel_review();
        let mut ok = true;
        for c in [&mut self.analysis.controller, &mut self.analysis.retired]
            .into_iter()
            .flatten()
        {
            ok &= c.shutdown();
        }
        ok
    }
}

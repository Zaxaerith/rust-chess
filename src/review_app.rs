//! Poll-only scheduling over the existing persistent controller.
use crate::{
    App, Screen,
    game_review::{Job, State},
    review_cache,
    uci::{Request, Status},
    uci_protocol,
};
impl App {
    pub(crate) fn save_review_cache(&mut self, force: bool) {
        let Some(job) = self.review.job.as_mut() else {
            return;
        };
        if self.review.cache_blocked || (!force && job.dirty < 8) {
            return;
        }
        if let Some(base) = self.records.save_path.as_ref().and_then(|p| p.parent()) {
            match review_cache::save(&review_cache::path(base, &job.cache_key), job) {
                Ok(()) => self.review.cache_error = None,
                Err(e) => {
                    self.review.cache_error = Some(e);
                    self.review.cache_blocked = true;
                }
            }
        }
        job.dirty = 0;
    }
    pub(crate) fn cancel_review(&mut self) {
        if !self.review.owns_engine() {
            return;
        }
        if let Some(c) = &self.analysis.controller {
            c.stop();
            c.set_multipv(self.analysis.config.multipv);
        }
        if let Some(job) = self.review.job.as_mut() {
            job.cancel();
        }
        self.save_review_cache(true);
    }
    pub(crate) fn start_review(&mut self) -> Result<(), String> {
        if self.review.owns_engine() {
            return Err("Review already owns engine".into());
        }
        let replay = self
            .records
            .replay
            .as_ref()
            .filter(|_| self.screen == Screen::Replay)
            .ok_or("Open a Replay first")?;
        let controller = self
            .analysis
            .controller
            .as_ref()
            .ok_or("Select and enable a UCI engine in the UCI tab first")?;
        let report = controller.report();
        if !matches!(
            report.status,
            Status::Ready | Status::Complete | Status::Stopped | Status::Searching
        ) || report.engine_key.is_empty()
        {
            return Err("Review engine is not ready".into());
        }
        self.review.options.clamp();
        let game_key = replay.identity.clone();
        let key = review_cache::key(
            &game_key,
            &self.analysis.config,
            &report,
            self.review.options,
        );
        let context = if replay.external {
            self.analysis.external_context
        } else {
            self.analysis.live_context
        };
        let mut job = Job::new(
            replay.game.clone(),
            game_key,
            key,
            report.name.clone(),
            self.review.options,
            context,
        );
        self.review.cache_error = None;
        self.review.cache_blocked = false;
        self.review.error = None;
        if let Some(base) = self.records.save_path.as_ref().and_then(|p| p.parent())
            && let Err(e) = review_cache::load(&review_cache::path(base, &job.cache_key), &mut job)
        {
            self.review.cache_error = Some(e);
            self.review.cache_blocked = true;
        }
        self.cancel_analysis();
        job.state = if job.next().is_none() {
            State::Completed
        } else {
            State::Running
        };
        if job.state == State::Running
            && let Some(c) = &self.analysis.controller
        {
            c.set_multipv(1);
        }
        self.review.job = Some(job);
        Ok(())
    }
    pub(crate) fn pause_review(&mut self) {
        if let Some(job) = self.review.job.as_mut()
            && job.state == State::Running
        {
            job.pause();
            if let Some(c) = &self.analysis.controller {
                c.stop();
            }
            self.save_review_cache(true);
        }
    }
    pub(crate) fn resume_review(&mut self) -> Result<(), String> {
        let job = self.review.job.as_mut().ok_or("No review to resume")?;
        if job.state != State::Paused {
            return Err("Review is not paused".into());
        }
        job.state = State::Running;
        Ok(())
    }
    pub(crate) fn poll_review(&mut self) {
        if !self.review.owns_engine() {
            return;
        }
        if self.records.replay.as_ref().is_none_or(|r| {
            self.review
                .job
                .as_ref()
                .is_none_or(|j| j.game_key != r.identity)
        }) || self.screen != Screen::Replay
        {
            self.cancel_review();
            return;
        }
        let result = (|| {
            let c = self
                .analysis
                .controller
                .as_ref()
                .ok_or("Review engine closed")?;
            let report = c.report();
            let job = self.review.job.as_mut().expect("review owner");
            if matches!(report.status, Status::Failed | Status::Off) {
                return Err(report
                    .error
                    .clone()
                    .unwrap_or_else(|| "Review engine disconnected".into()));
            }
            if job.state == State::Paused {
                return Ok(());
            }
            job.accept(&report)?;
            if job.pending() {
                return Ok(());
            }
            let Some(index) = job.next() else {
                job.state = State::Completed;
                return Ok(());
            };
            if report.multipv != 1 {
                if report.status == Status::Stopped {
                    return Err("Engine cannot provide single-PV review".into());
                }
                return Ok(());
            }
            if let Some(point) = job.terminal(index) {
                job.store(index, point);
                return Ok(());
            }
            let command = uci_protocol::position(&job.game.positions[0], &job.game.moves[..index]);
            if command.len() > 32768 {
                return Err("Review UCI history exceeds 32 KiB".into());
            }
            self.analysis.sequence = self.analysis.sequence.wrapping_add(1);
            let id = self.analysis.sequence;
            job.begin(id, index);
            c.request(Request {
                id,
                context: job.context,
                position: command,
                movetime: job.options.movetime,
            });
            Ok::<_, String>(())
        })();
        if let Err(e) = result
            && let Some(job) = self.review.job.as_mut()
        {
            job.fail(e);
        }
        if !self.review.owns_engine() {
            // The final latest-value report has already been copied into the job.
            if let Some(c) = &self.analysis.controller {
                c.stop();
                c.set_multipv(self.analysis.config.multipv);
            }
            self.save_review_cache(true);
        } else {
            self.save_review_cache(false);
        }
    }
}

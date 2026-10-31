use super::*;

/// Fixed simulation step used by the canonical headless model.
pub(crate) const HEADLESS_DT: f64 = 1.0 / 60.0;

impl Session {
    pub(crate) fn queue_headless_command(&mut self, line: String) {
        self.console.commands.push_back(line);
    }
    pub(crate) fn set_headless_quiet(&mut self, quiet: bool) {
        self.console.set_quiet_events(quiet);
    }
    pub(crate) fn headless_commands_pending(&self) -> bool {
        !self.console.commands.is_empty()
    }
    /// Process one debug command, then run one model frame.
    pub(crate) fn headless_step_command(&mut self) -> bool {
        let mut frame = Frame::headless(HEADLESS_DT);
        let processed = self.headless_console_input(&mut frame);
        self.apply_restart_command(&frame);
        self.tick(&mut frame);
        processed
    }
    /// Advance only the model; no command is consumed and no drawing/input occurs.
    pub(crate) fn headless_advance(&mut self) {
        let mut frame = Frame::headless(HEADLESS_DT);
        self.tick(&mut frame);
    }
    /// Stable means no active action, cascade, presentation timer, or AI decision remains.
    pub(crate) fn headless_idle(&self) -> bool {
        self.paused
            || self.hp == 0
            || self.enemy == 0
            || (!self.enemy_turn
                && self.pending.is_empty()
                && !self.action_done
                && !self.presentation.busy()
                && self.deadline <= 0.)
    }
    pub(crate) fn headless_summary(&self, steps: u64) -> String {
        format!(
            "HEADLESS_OK steps={steps} hp={} enemy_hp={} mp={} enemy_mp={} turn={} pending={} paused={} ai={}",
            self.hp,
            self.enemy,
            self.mp,
            self.enemy_mp,
            if self.enemy_turn { "enemy" } else { "player" },
            self.pending.len(),
            self.paused,
            self.difficulty
        )
    }
}

use super::*;

/// Fixed simulation step used by the canonical headless model.
pub(crate) const HEADLESS_DT: f64 = 1.0 / 60.0;
/// Batch runs do not need animation pacing; one tick still resolves one cascade.
pub(crate) const HEADLESS_BATCH_DT: f64 = 1.0;

/// Result of one unattended match; `winner` is 0 for the first side, 1 for the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BatchOutcome {
    pub winner: Option<usize>,
    pub rounds: u32,
    pub steps: u64,
    pub hp: i32,
    pub enemy_hp: i32,
    pub nukes: [u32; 2],
    pub nuke_damage: [i64; 2],
    pub first_nuke_round: [u32; 2],
}

impl Session {
    /// Put the canonical model into unattended mode: no rollback history and no JSONL.
    pub(crate) fn prepare_headless_batch(&mut self) {
        self.zh = false;
        self.paused = false;
        self.choosing = false;
        self.hints_shown = false;
        self.interface.opponent_step = false;
        self.interface.panel = None;
        self.console.open = false;
        self.console.set_batch(true);
        self.console.set_quiet_events(true);
    }

    /// Play one seeded match to completion with the shared strategy on both sides.
    /// The model, event dispatch, cooldowns, pockets and rewards are the normal game rules.
    pub(crate) fn headless_batch_match(
        &mut self,
        hero: usize,
        enemy: usize,
        seed: u64,
        level: usize,
        max_steps: u64,
    ) -> BatchOutcome {
        self.headless_batch_match_at_dt(hero, enemy, seed, level, max_steps, HEADLESS_BATCH_DT)
    }

    /// Same as [`Self::headless_batch_match`] with an explicit frame step, used to test
    /// that skipping animation pacing does not change the seeded result.
    pub(crate) fn headless_batch_match_at_dt(
        &mut self,
        hero: usize,
        enemy: usize,
        seed: u64,
        level: usize,
        max_steps: u64,
        dt: f64,
    ) -> BatchOutcome {
        self.zh = false;
        self.console.open = false;
        self.console.set_batch(true);
        self.console.set_quiet_events(true);
        self.hero_id = hero;
        self.enemy_id = enemy;
        self.difficulty = level;
        self.interface.opponent = Some(enemy);
        rng::srand(seed);
        self.reset_battle();
        self.paused = false;
        self.choosing = false;
        self.hints_shown = false;
        self.interface.panel = None;
        self.targeting = None;
        self.selected = None;

        let mut frame = Frame::headless(dt);
        let mut steps = 0;
        let mut first_nuke_round = [0; 2];
        let mut seen_nukes = [0; 2];
        while self.hp > 0 && self.enemy > 0 && steps < max_steps {
            frame.command_move = None;
            frame.command_skill = None;
            frame.command_target = None;
            self.headless_auto_command(&mut frame);
            self.tick(&mut frame);
            steps += 1;
            for side in 0..2 {
                if self.nuke_casts[side] > seen_nukes[side] {
                    seen_nukes[side] = self.nuke_casts[side];
                    if first_nuke_round[side] == 0 {
                        first_nuke_round[side] = self.round;
                    }
                }
            }
        }
        let winner = if self.hp <= 0 && self.enemy <= 0 {
            None
        } else if self.enemy <= 0 {
            Some(0)
        } else if self.hp <= 0 {
            Some(1)
        } else {
            None
        };
        BatchOutcome {
            winner,
            rounds: self.round,
            steps,
            hp: self.hp,
            enemy_hp: self.enemy,
            nukes: self.nuke_casts,
            nuke_damage: self.nuke_damage,
            first_nuke_round,
        }
    }

    /// Inject one normal player command chosen by the same strategy used by the enemy AI.
    fn headless_auto_command(&mut self, frame: &mut Frame) {
        if self.enemy_turn
            || self.paused
            || self.hp == 0
            || self.enemy == 0
            || !self.pending.is_empty()
            || self.action_done
            || self.presentation.busy()
            || self.console.open
            || self.interface.panel.is_some()
        {
            return;
        }
        self.targeting = None;
        if self.new_role_states[0].force_armed {
            if let Some(m) = strategy::choose_force_move_in(
                &self.b,
                self.hero_id,
                self.hp,
                self.mp,
                self.extra_bonus,
                self.rules(false),
            ) {
                frame.command_move = Some(m);
            } else {
                self.new_role_states[0].force_armed = false;
            }
            return;
        }
        let unavailable: Vec<_> = content::skills(self.hero_id, self.awakened[0])
            .iter()
            .copied()
            .filter(|skill| self.cooldowns.remaining(0, *skill) > 0)
            .collect();
        if let Some(skill) = strategy::choose_skill_available_in(
            &self.b,
            self.hero_id,
            self.hp,
            self.mp,
            self.difficulty,
            self.armed,
            self.extra_bonus,
            self.rules(false),
            &unavailable,
        ) {
            frame.command_target = Some(strategy::choose_target_in(
                skill,
                &self.b,
                self.hero_id,
                self.hp,
                self.mp,
                self.extra_bonus,
                self.rules(false),
            ));
            frame.command_skill = Some(skill);
            return;
        }
        if let Some(m) = strategy::choose_move_in(
            &self.b,
            self.hp,
            self.hero_id,
            self.mp,
            self.enemy_id,
            self.difficulty,
            self.rules(false),
        ) {
            frame.command_move = Some(m);
        } else if self.armed {
            // Match the enemy AI fallback for a three-line swap with no scoring move.
            frame.command_move = Some(((0, 0), (1, 0)));
        } else {
            // Match the enemy AI reroll when neither a skill nor a swap is chosen.
            self.b = board::fresh_for(self.hero_id);
            self.turn_log.current.actions.push(turns::Action::Reroll);
            self.selected = None;
            self.presentation.stop_motion();
        }
    }

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

use super::*;
impl Session {
    pub(super) fn advance_turn(&mut self, frame: &Frame) {
        if let Some((damage, enemy_actor)) = self.charge_damage {
            let before = self.snapshot();
            self.charge_damage = None;
            let hit = self.damage(damage, !enemy_actor, "damage");
            let actual = hit.damage;
            self.turn_log.current.add(0, actual, 0);
            self.presentation.health(
                [before.hp, before.enemy_hp],
                [self.hp, self.enemy],
                content::Skill::Charge.name(self.zh),
            );
            self.feedback = format!(
                "{} · {} {}",
                content::Skill::Charge.name(self.zh),
                tr_key(self.zh, "ui.turn.advance_turn.damage"),
                actual
            );
            self.feedback_time = 2.5;
            self.deadline = 0.;
            self.console.event(
                "skill_hit",
                &format!("Charge expected_damage={} actual_damage={}", damage, actual),
                &before,
                &self.snapshot(),
            );
        }
        if self.pending.is_empty() && !self.presentation.busy() {
            self.deadline = (self.deadline - frame.dt).max(0.);
        }
        if self.action_done
            && self.pending.is_empty()
            && !self.presentation.busy()
            && self.deadline <= 0.
        {
            let actor_id = if self.enemy_turn {
                self.enemy_id
            } else {
                self.hero_id
            };
            if actor_id == content::NUN {
                let before_recovery = self.snapshot();
                let hit = self.apply_rewards((10, 0, 10), 0);
                let heal = if self.enemy_turn {
                    self.enemy - before_recovery.enemy_hp
                } else {
                    self.hp - before_recovery.hp
                };
                let mana = if self.enemy_turn {
                    self.enemy_mp - before_recovery.enemy_mp
                } else {
                    self.mp - before_recovery.mp
                };
                self.turn_log.current.add(heal, hit.damage, mana);
                self.presentation.health(
                    [before_recovery.hp, before_recovery.enemy_hp],
                    [self.hp, self.enemy],
                    content::role_text("AutoRecovery.name", self.zh),
                );
                self.console.event(
                    "auto_recovery",
                    &format!("heal={heal} damage={} mana={mana}", hit.damage),
                    &before_recovery,
                    &self.snapshot(),
                );
                if self.hp == 0 || self.enemy == 0 {
                    return;
                }
            }
            let before = self.snapshot();
            self.bonuses[self.enemy_turn as usize] = self.extra_bonus;
            self.armed_sides[self.enemy_turn as usize] = self.armed;
            let receiver = !self.enemy_turn;
            let receiver_id = if receiver {
                self.enemy_id
            } else {
                self.hero_id
            };
            let cleared = self.new_role_states[self.enemy_turn as usize]
                .direct_tiles
                .clone();
            if receiver_id == content::PRIEST
                && self.new_role_states[receiver as usize]
                    .pockets
                    .iter()
                    .any(Option::is_none)
                && !cleared.is_empty()
            {
                let pick = rng::gen_range(0, cleared.len());
                self.new_role_states[receiver as usize].mind_vision(&cleared, pick);
            }
            self.new_role_states[self.enemy_turn as usize].end_turn();
            self.enemy_turn = !self.enemy_turn;
            self.cooldowns.start_turn(self.enemy_turn as usize);
            self.new_role_states[self.enemy_turn as usize].start_turn();
            self.refresh_copy(self.enemy_turn);
            self.extra_bonus = self.bonuses[self.enemy_turn as usize];
            self.armed = self.armed_sides[self.enemy_turn as usize];
            self.turn_log.next(self.enemy_turn);
            // The last turn has its own row; prompt the new actor afresh.
            self.note = "note.select_adjacent_tiles";
            self.action_done = false;
            self.selected = None;
            self.enemy_clock = 0.;
            self.move_flash = None;
            self.chain = 0;
            self.console.event(
                "turn_start",
                "All cascades resolved; next actor",
                &before,
                &self.snapshot(),
            );
        }
        if self.pending.is_empty()
            && !self.action_done
            && self.targeting.is_none()
            && !self.armed
            && !self.new_role_states[self.enemy_turn as usize].force_armed
        {
            let (id, health, mana) = if self.enemy_turn {
                (self.enemy_id, self.enemy, self.enemy_mp)
            } else {
                (self.hero_id, self.hp, self.mp)
            };
            let no_swaps =
                content::move_options_in(&self.b, id, health, self.rules(self.enemy_turn))
                    .is_empty();
            let automatic = no_swaps
                && content::usable_skills_in(
                    &self.b,
                    id,
                    health,
                    mana,
                    self.rules(self.enemy_turn),
                )
                .iter()
                .all(|skill| self.cooldowns.remaining(self.enemy_turn as usize, *skill) > 0);
            let voluntary = no_swaps
                && !self.enemy_turn
                && (frame.key(KeyCode::F2)
                    || (frame.clicked(MouseButton::Left) && reroll_rect().contains(frame.pointer)));
            if automatic || voluntary {
                let before = self.snapshot();
                if voluntary {
                    self.console.checkpoint(before.clone());
                }
                self.b = board::fresh_for(id);
                self.turn_log.current.actions.push(turns::Action::Reroll);
                self.selected = None;
                self.move_flash = None;
                self.presentation.stop_motion();
                self.note = "note.no_moves";
                self.console.event(
                    "board_refresh",
                    if voluntary {
                        "Player chose reroll instead of a skill"
                    } else {
                        "No role-legal swap or usable skill"
                    },
                    &before,
                    &self.snapshot(),
                );
            }
        }
        if self.enemy_turn && self.pending.is_empty() && !self.action_done {
            self.enemy_clock += frame.dt;
        }
    }
}

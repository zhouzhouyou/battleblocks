use super::*;
impl Session {
    pub(super) fn choose_cast(&mut self, frame: &Frame) -> Option<content::Skill> {
        let mut cast = frame.command_skill;
        if self.pending.is_empty() && self.hp > 0 && !self.enemy_turn && !self.action_done {
            for (i, skill) in content::skills(self.hero_id, self.awakened[0])
                .iter()
                .enumerate()
            {
                let rect = player_skill_rect(i);
                if self.cooldowns.remaining(0, *skill) == 0
                    && content::preparation_available(
                        *skill,
                        self.rules(false),
                        self.armed,
                        self.extra_bonus,
                    )
                    && ((i == 0 && (frame.key(KeyCode::Key1) || frame.key(KeyCode::Space)))
                        || (i == 1 && frame.key(KeyCode::Key2))
                        || (i == 2 && frame.key(KeyCode::Key3))
                        || (frame.clicked(MouseButton::Left) && rect.contains(frame.pointer)))
                {
                    cast = Some(*skill);
                }
            }
        }
        if self.enemy_turn
            && self.enemy_clock >= 1.1
            && self.pending.is_empty()
            && !self.action_done
        {
            if self.new_role_states[1].force_armed {
                if let Some((a, c)) = strategy::choose_force_move_in(
                    &self.b,
                    self.enemy_id,
                    self.enemy,
                    self.enemy_mp,
                    self.extra_bonus,
                    self.rules(true),
                ) {
                    let before = self.snapshot();
                    self.console.checkpoint(before.clone());
                    self.new_role_states[1].complete_force(&mut self.b, a, c, self.enemy_id);
                    self.presentation
                        .swap(&before.board, a, c, false, true, self.enemy_id);
                    self.turn_log.current.actions.push(turns::Action::Swap {
                        from: a,
                        to: c,
                        columns: content::movement_profile(self.enemy_id).columns && a.1 == c.1,
                    });
                    self.console.event(
                        "move",
                        "AI forced non-matching movement; turn continues",
                        &before,
                        &self.snapshot(),
                    );
                } else {
                    self.new_role_states[1].force_armed = false;
                }
            } else {
                let enemy_before = self.snapshot();
                let unavailable: Vec<_> = content::skills(self.enemy_id, self.awakened[1])
                    .iter()
                    .copied()
                    .filter(|skill| self.cooldowns.remaining(1, *skill) > 0)
                    .collect();
                cast = strategy::choose_skill_available_in(
                    &self.b,
                    self.enemy_id,
                    self.enemy,
                    self.enemy_mp,
                    self.difficulty,
                    self.armed,
                    self.extra_bonus,
                    self.rules(true),
                    &unavailable,
                );
                if cast.is_none() {
                    if let Some((a, c)) = strategy::choose_move_in(
                        &self.b,
                        self.enemy,
                        self.enemy_id,
                        self.enemy_mp,
                        self.hero_id,
                        self.difficulty,
                        self.rules(true),
                    )
                    .or_else(|| self.armed.then_some(((0, 0), (1, 0))))
                    {
                        let before = enemy_before.clone();
                        self.console.checkpoint(before.clone());
                        self.presentation.swap(
                            &before.board,
                            a,
                            c,
                            self.armed,
                            self.enemy_turn,
                            self.enemy_id,
                        );
                        self.turn_log.current.actions.push(turns::Action::Swap {
                            from: a,
                            to: c,
                            columns: content::movement_profile(self.enemy_id).columns
                                && !self.armed
                                && a.1 == c.1,
                        });
                        if self.armed {
                            parallel_swap(&mut self.b, a, c);
                            self.armed = false;
                            self.new_role_states[self.enemy_turn as usize]
                                .statuses
                                .consume(content::StatusDuration::UntilSwap);
                        } else {
                            board::hero_swap(&mut self.b, a, c, self.enemy_id);
                        }
                        self.new_role_states[1].record_swap(a, c);
                        self.pending = matches(&self.b);
                        self.deadline = CLEAR_TIME;
                        self.reward = true;
                        self.chain = 1;
                        self.action_done = true;
                        self.move_flash = Some((a, c));
                        self.console.event(
                            "move",
                            &format!(
                                "AI swap [{},{}] -> [{},{}]",
                                a.0 + 1,
                                a.1 + 1,
                                c.0 + 1,
                                c.1 + 1
                            ),
                            &before,
                            &self.snapshot(),
                        );
                    } else {
                        let before = self.snapshot();
                        self.b = board::fresh_for(self.enemy_id);
                        self.turn_log.current.actions.push(turns::Action::Reroll);
                        self.console.event(
                            "board_refresh",
                            "AI chose reroll instead of a skill",
                            &before,
                            &self.snapshot(),
                        );
                    }
                }
            }
            self.enemy_clock = 0.;
        }

        cast
    }
    pub(super) fn cast_skill(
        &mut self,
        frame: &Frame,
        mut cast: Option<content::Skill>,
        commit_force: bool,
    ) -> bool {
        let board_target = if frame.pointer.x >= 256.
            && frame.pointer.x < 768.
            && frame.pointer.y >= 110.
            && frame.pointer.y < 622.
        {
            Some((
                ((frame.pointer.x - 256.) / 64.) as usize,
                ((frame.pointer.y - 110.) / 64.) as usize,
            ))
        } else {
            None
        };
        let cancel_target = frame.key(KeyCode::Escape)
            || frame.clicked(MouseButton::Right)
            || (self.targeting.is_some()
                && frame.clicked(MouseButton::Left)
                && cancel_skill_rect().contains(frame.pointer));
        if cancel_target {
            self.targeting = None;
            self.selected = None;
            cast = None;
        }
        if !self.enemy_turn {
            if let Some(skill) = cast {
                if (content::targeted_in(skill, self.rules(false))
                    || (content::skill_profile(skill, self.rules(false)).force_move
                        && !commit_force))
                    && self.cooldowns.remaining(0, skill) == 0
                    && content::payment_in(self.hero_id, self.hp, self.mp, skill, self.rules(false))
                        .is_some()
                    && frame.command_skill.is_none()
                {
                    self.targeting = toggle_skill_target(self.targeting, skill);
                    self.selected = None;
                    cast = None;
                }
            }
            if let Some(skill) = self.targeting {
                if !content::skill_profile(skill, self.rules(false)).force_move
                    && board_target.is_some()
                    && frame.clicked(MouseButton::Left)
                {
                    cast = Some(skill);
                    self.targeting = None;
                }
            }
        }
        let mut cast_this_frame = false;
        if let Some(skill) = cast {
            let before = self.snapshot();
            let cast_rules = self.rules(self.enemy_turn);
            let id = if self.enemy_turn {
                self.enemy_id
            } else {
                self.hero_id
            };
            let actor_hp = if self.enemy_turn { self.enemy } else { self.hp };
            let actor_mp = if self.enemy_turn {
                self.enemy_mp
            } else {
                self.mp
            };
            let effective = content::effective_skill(skill, cast_rules);
            let target = if content::skill_profile(effective, cast_rules).random_target
                && self.cooldowns.remaining(self.enemy_turn as usize, skill) == 0
                && content::payment_in(id, actor_hp, actor_mp, skill, cast_rules).is_some()
            {
                (
                    rng::gen_range(0, N - 1),
                    rng::gen_range(N - content::reachable_rows(id, actor_hp, cast_rules), N - 1),
                )
            } else if self.enemy_turn {
                strategy::choose_target_in(
                    skill,
                    &self.b,
                    id,
                    actor_hp,
                    actor_mp,
                    self.extra_bonus,
                    cast_rules,
                )
            } else {
                frame.command_target.or(board_target).unwrap_or((3, 3))
            };
            if let Some(pay) = content::payment_in(id, actor_hp, actor_mp, skill, cast_rules)
                .filter(|_| {
                    self.cooldowns.remaining(self.enemy_turn as usize, skill) == 0
                        && !self.new_role_states[self.enemy_turn as usize].force_armed
                        && content::legal_target(id, actor_hp, skill, &self.b, target, cast_rules)
                })
            {
                self.console.checkpoint(before.clone());
                if self.enemy_turn {
                    self.enemy_mp -= pay.mana;
                    self.apply_health_cost(pay.blood, true, effective);
                } else {
                    self.mp -= pay.mana;
                    self.apply_health_cost(pay.blood, false, effective);
                }
                self.cooldowns.cast(self.enemy_turn as usize, skill);
                self.new_role_states[self.enemy_turn as usize].record_cast(skill);
                cast_this_frame = true;
                self.move_flash = None;
                self.last_cast = Some(effective);
                self.last_cast_enemy = self.enemy_turn;
                self.cast_time = presentation::CAST_TIME;
                self.reward_skill = Some(skill);
                self.skill_flash = content::target_cells_in(
                    skill, &self.b, target, id, actor_hp, actor_mp, cast_rules,
                );
                if let Some(effects) = &mut self.skill_effects {
                    effects.capture(effective, self.enemy_turn, &self.skill_flash, &self.b);
                }
                let effect_rules = self.rules(self.enemy_turn);
                let e = content::effect_at_in(
                    skill,
                    &mut self.b,
                    target,
                    id,
                    actor_hp,
                    actor_mp,
                    effect_rules,
                );
                self.new_role_states[self.enemy_turn as usize].on_cast(
                    effective,
                    &before.board,
                    target,
                );
                let mut batch = e.reward_batch(effective);
                if content::skill_profile(effective, effect_rules).delayed_damage {
                    let mut delayed = content::RewardBatch::default();
                    let mut immediate = Vec::new();
                    for event in std::mem::take(&mut batch.events) {
                        if event.resource == content::Resource::Damage
                            && event.recipient == content::EventSide::Opponent
                        {
                            delayed.events.push(event);
                        } else {
                            immediate.push(event);
                        }
                    }
                    delayed.triggers = std::mem::take(&mut batch.triggers);
                    batch.events = immediate;
                    self.charge_damage = Some(content::PendingImpact {
                        rewards: delayed,
                        actor_enemy: self.enemy_turn,
                        skill: effective,
                    });
                    self.presentation
                        .charge(target.0, presentation::CHARGE_TIME);
                }
                let applied = self.apply_batch_rewards(batch);
                let actual_heal = if self.enemy_turn {
                    self.enemy - before.enemy_hp
                } else {
                    self.hp - before.hp
                };
                let actual_damage = applied.hit.damage;
                if effective == content::Skill::EmberBlast {
                    self.nuke_casts[self.enemy_turn as usize] += 1;
                    self.nuke_damage[self.enemy_turn as usize] += actual_damage as i64;
                }
                let actual_mana = if self.enemy_turn {
                    self.enemy_mp - before.enemy_mp + pay.mana
                } else {
                    self.mp - before.mp + pay.mana
                };
                self.turn_log
                    .current
                    .actions
                    .push(turns::Action::Skill(effective));
                self.turn_log
                    .current
                    .add(actual_heal, actual_damage, actual_mana);
                let caster = &HEROES[if self.enemy_turn {
                    self.enemy_id
                } else {
                    self.hero_id
                }];
                self.feedback = format!(
                    "{} · {}  {}",
                    if self.zh { caster.zh } else { caster.en },
                    skill.name(self.zh),
                    reward_summary(self.zh, actual_heal, actual_damage, actual_mana)
                );
                if content::skill_profile(effective, effect_rules).delayed_damage {
                    self.feedback = format!(
                        "{} · {} {}",
                        skill.name(self.zh),
                        tr_key(self.zh, "ui.actions.cast_skill.charging_expected_damage"),
                        e.damage
                            .min(if self.enemy_turn { self.hp } else { self.enemy })
                    );
                }
                if pay.blood > 0 {
                    let (hp_now, mp_now) = if self.enemy_turn {
                        (self.enemy, self.enemy_mp)
                    } else {
                        (self.hp, self.mp)
                    };
                    self.feedback = crate::format_text(
                        self.zh,
                        "feedback.blood_cast",
                        &[
                            &pay.blood.to_string(),
                            &actor_hp.to_string(),
                            &hp_now.to_string(),
                            &actor_mp.to_string(),
                            &mp_now.to_string(),
                        ],
                    );
                }
                let actor_source = if pay.blood > 0 {
                    tr_key(self.zh, "ui.actions.cast_skill.flesh_casting")
                } else {
                    applied
                        .actor_health_source
                        .map(|source| source.name(self.zh))
                        .unwrap_or_else(|| skill.name(self.zh))
                };
                let opponent_source = applied
                    .opponent_health_source
                    .map(|source| source.name(self.zh))
                    .unwrap_or_else(|| skill.name(self.zh));
                self.presentation.health_sources(
                    [before.hp, before.enemy_hp],
                    [self.hp, self.enemy],
                    [actor_source, opponent_source],
                );
                self.feedback_time = 2.5;
                self.action_done = effective.ends_turn();
                self.extra_bonus += e.bonus;
                self.armed |= e.armed;
                self.pending = e.cells;
                self.reward = e.reward;
                if self.pending.is_empty() {
                    self.pending = matches(&self.b);
                    self.reward = true;
                }
                self.deadline = if self.pending.is_empty() {
                    TURN_HOLD
                } else {
                    CLEAR_TIME
                };
                self.chain = 1;
                self.selected = None;
                self.note = "note.skill_cast";
                self.console.event("skill",&format!("{:?} effective={:?} target=[{},{}] cost={} blood_cost={} direct_heal={} direct_damage={} direct_mana={} bonus={} armed={} actual_heal={} actual_damage={} actual_mana={}",skill,effective,target.0+1,target.1+1,pay.mana,pay.blood,e.hp,e.damage,e.mp,e.bonus,e.armed,actual_heal,actual_damage,actual_mana),&before,&self.snapshot());
            } else {
                if !self.enemy_turn
                    && content::reachable_rows(id, actor_hp, cast_rules) < N
                    && !content::legal_target(id, actor_hp, skill, &self.b, target, cast_rules)
                {
                    if self.feedback_time <= 0. || self.feedback != self.rejection_message(false) {
                        self.feedback = self.rejection_message(false).to_string();
                        self.feedback_time = 1.8;
                    }
                    if content::targeted_in(skill, cast_rules) {
                        self.targeting = Some(skill);
                    }
                }
                self.note = "note.not_enough_mana";
            }
        }

        cast_this_frame
    }
}

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
                    && !((matches!(skill, content::Skill::Strike) && self.extra_bonus > 0)
                        || (matches!(skill, content::Skill::Landslide) && self.armed))
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
                        columns: matches!(self.enemy_id, 0 | content::KNIGHT) && a.1 == c.1,
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
                            columns: matches!(self.enemy_id, 0 | content::KNIGHT)
                                && !self.armed
                                && a.1 == c.1,
                        });
                        if self.armed {
                            parallel_swap(&mut self.b, a, c);
                            self.armed = false;
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
    pub(super) fn cast_skill(&mut self, frame: &Frame, mut cast: Option<content::Skill>) -> bool {
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
                if content::targeted_in(skill, self.rules(false))
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
                if board_target.is_some() && frame.clicked(MouseButton::Left) {
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
            let target = if effective == content::Skill::Hammer
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
                    self.damage(pay.blood, true, "blood_payment");
                } else {
                    self.mp -= pay.mana;
                    self.damage(pay.blood, false, "blood_payment");
                }
                self.cooldowns.cast(self.enemy_turn as usize, skill);
                cast_this_frame = true;
                self.move_flash = None;
                self.last_cast = Some(effective);
                self.last_cast_enemy = self.enemy_turn;
                self.cast_time = 1.4;
                self.reward_skill = Some(skill);
                self.skill_flash = content::target_cells_in(
                    skill, &self.b, target, id, actor_hp, actor_mp, cast_rules,
                );
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
                let direct_damage =
                    if matches!(effective, content::Skill::Charge | content::Skill::Rush) {
                        self.charge_damage = Some((e.damage, self.enemy_turn));
                        self.presentation
                            .charge(target.0, presentation::CHARGE_TIME);
                        0
                    } else {
                        e.damage
                    };
                let hit = self.apply_rewards((e.hp, direct_damage, e.mp));
                let actual_heal = if self.enemy_turn {
                    self.enemy - before.enemy_hp
                } else {
                    self.hp - before.hp
                };
                let actual_damage = hit.damage;
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
                if skill == content::Skill::Charge {
                    self.feedback = format!(
                        "{} · {} {}",
                        skill.name(self.zh),
                        tr(
                            self.zh,
                            "Charging · expected damage:",
                            "冲锋中 · 预计伤害："
                        ),
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
                    self.feedback = if self.zh {
                        format!(
                                "血肉施法 生命 -{} · 生命 {actor_hp} → {hp_now} · 魔力 {actor_mp} → {mp_now}",
                                pay.blood
                            )
                    } else {
                        format!(
                                "Flesh casting -{} HP · HP {actor_hp} → {hp_now} · MP {actor_mp} → {mp_now}",
                                pay.blood
                            )
                    };
                }
                self.presentation.health(
                    [before.hp, before.enemy_hp],
                    [self.hp, self.enemy],
                    if pay.blood > 0 {
                        tr(self.zh, "Flesh casting", "血肉施法")
                    } else {
                        skill.name(self.zh)
                    },
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
                self.note = "Skill cast";
                self.console.event("skill",&format!("{:?} effective={:?} target=[{},{}] cost={} blood_cost={} direct_heal={} direct_damage={} direct_mana={} bonus={} armed={} actual_heal={} actual_damage={} actual_mana={}",skill,effective,target.0+1,target.1+1,pay.mana,pay.blood,e.hp,e.damage,e.mp,e.bonus,e.armed,actual_heal,actual_damage,actual_mana),&before,&self.snapshot());
            } else {
                if !self.enemy_turn
                    && id == content::DWARF
                    && !content::legal_target(id, actor_hp, skill, &self.b, target, cast_rules)
                {
                    if self.feedback_time <= 0.
                        || self.feedback != content::role_text("Dwarf.unreachable", self.zh)
                    {
                        self.feedback =
                            content::role_text("Dwarf.unreachable", self.zh).to_string();
                        self.feedback_time = 1.8;
                    }
                    if content::targeted_in(skill, cast_rules) {
                        self.targeting = Some(skill);
                    }
                }
                self.note = "Not enough mana";
            }
        }

        cast_this_frame
    }
}

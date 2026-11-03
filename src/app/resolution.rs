use super::*;
impl Session {
    pub(super) fn resolve_pending(&mut self, dt: f64) {
        if !self.presentation.busy() {
            self.deadline -= dt;
        }
        if self.deadline <= 0. && !self.presentation.busy() {
            if let Some(effects) = &mut self.skill_effects {
                effects.board_cleared(&self.pending);
            }
            if !self.muted {
                if let Some(audio) = &self.audio {
                    audio.crush();
                }
            }
            let before = self.snapshot();
            let cleared: Vec<_> = self.pending.iter().map(|&(x, y)| (x + 1, y + 1)).collect();
            let rewarded = self.reward;
            let bonus = content::reward_scale(self.chain.max(1) as usize, self.extra_bonus);
            let id = if self.enemy_turn {
                self.enemy_id
            } else {
                self.hero_id
            };
            let batch = if self.reward {
                content::clear_batch_in(
                    id,
                    if self.enemy_turn { self.enemy } else { self.hp },
                    if self.enemy_turn {
                        self.enemy_mp
                    } else {
                        self.mp
                    },
                    &self.b,
                    &self.pending,
                    self.chain.max(1) as usize,
                    self.extra_bonus,
                    content::Rules {
                        initial_loot: self.reward_skill.is_some_and(|s| {
                            content::skill_profile(s, self.rules(self.enemy_turn)).initial_loot
                        }),
                        ..self.rules(self.enemy_turn)
                    },
                )
            } else {
                content::RewardBatch::default()
            };
            let (heal, damage, mana) = batch.totals();
            if self.reward && self.chain == 1 {
                self.new_role_states[self.enemy_turn as usize]
                    .direct_tiles
                    .extend(
                        self.pending
                            .iter()
                            .filter_map(|&(x, y)| (self.b[y][x] < 7).then_some(self.b[y][x])),
                    );
            }
            self.reward_skill = None;
            if self.reward {
                self.extra_bonus = 0;
                self.new_role_states[self.enemy_turn as usize]
                    .statuses
                    .consume(content::StatusDuration::UntilClear);
            }
            for &(x, y) in &self.pending {
                self.b[y][x] = 255;
            }
            let applied = self.apply_batch_rewards(batch);
            let fallback = tr_key(self.zh, "ui.resolution.resolve_pending.clear");
            let actor_source = applied
                .actor_health_source
                .map(|source| source.name(self.zh))
                .unwrap_or(fallback);
            let opponent_source = applied
                .opponent_health_source
                .map(|source| source.name(self.zh))
                .unwrap_or(fallback);
            self.presentation.health_sources(
                [before.hp, before.enemy_hp],
                [self.hp, self.enemy],
                [actor_source, opponent_source],
            );
            let actual_heal = if self.enemy_turn {
                self.enemy - before.enemy_hp
            } else {
                self.hp - before.hp
            };
            let actual_damage = applied.hit.damage;
            let actual_mana = if self.enemy_turn {
                self.enemy_mp - before.enemy_mp
            } else {
                self.mp - before.mp
            };
            self.turn_log
                .current
                .add(actual_heal, actual_damage, actual_mana);
            self.turn_log.current.chain = self.turn_log.current.chain.max(self.chain.max(1));
            if self.reward {
                self.feedback = format!(
                    "{} · {}",
                    if self.enemy_turn {
                        tr_key(self.zh, "ui.shared.enemy")
                    } else {
                        tr_key(self.zh, "ui.shared.you")
                    },
                    reward_summary(self.zh, actual_heal, actual_damage, actual_mana)
                );
                self.feedback_time = 2.;
            }
            self.presentation
                .fall(board::drop_board_animated(&mut self.b));
            self.pending = matches(&self.b);
            self.reward = true;
            if !self.pending.is_empty() {
                self.chain += 1;
                self.deadline = CLEAR_TIME;
            } else {
                self.deadline = TURN_HOLD;
            }
            self.console.event("resolution",&format!("cells={:?} reward={} multiplier_percent={} heal={} damage={} mana={} chain={} refreshed={} actual_heal={} actual_damage={} actual_mana={}",cleared,rewarded,bonus,heal,damage,mana,self.chain,false,actual_heal,actual_damage,actual_mana),&before,&self.snapshot());
        }
    }
}

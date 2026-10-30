use super::*;
impl Session {
    pub(in crate::app) fn draw_skill_cards(
        &mut self,
        frame: &Frame,
        player_hover: Option<(content::Skill, bool)>,
    ) {
        for (i, skill) in content::skills(self.enemy_id, self.awakened[1])
            .iter()
            .enumerate()
        {
            let r = enemy_skill_rect(i);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.12, 0.15, 0.2, 1.));
            if r.contains(frame.pointer) {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., ORANGE);
            }
            icons::draw(*skill, r.x + 5., r.y + 5., 28.);
            let color = if content::payment_in(
                self.enemy_id,
                self.enemy,
                self.enemy_mp,
                *skill,
                self.rules(true),
            )
            .is_some()
            {
                GOLD
            } else {
                GRAY
            };
            // Cost right-aligned; the name fits into the space left of it.
            let cost = cost_text(
                self.zh,
                content::skill_cost_in(self.enemy_id, *skill, self.rules(true)),
                0,
            );
            let cost_width = label_width(&cost, 15.);
            label(&cost, r.x + r.w - 7. - cost_width, r.y + 25., 15., color);
            let name = content::skill_name_in(*skill, self.rules(true), self.zh);
            label(
                &name,
                r.x + 40.,
                r.y + 25.,
                fit_size(&name, 17., r.w - 54. - cost_width),
                color,
            );
        }
        for (i, skill) in content::skills(self.hero_id, self.awakened[0])
            .iter()
            .enumerate()
        {
            let rect = player_skill_rect(i);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if self.cooldowns.remaining(0, *skill) == 0
                    && content::payment_in(
                        self.hero_id,
                        self.hp,
                        self.mp,
                        *skill,
                        self.rules(false),
                    )
                    .is_some()
                {
                    Color::new(0.2, 0.28, 0.35, 1.)
                } else {
                    Color::new(0.1, 0.12, 0.16, 1.)
                },
            );
            if self.targeting == Some(*skill) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., GOLD);
            } else if player_hover.is_some_and(|(s, _)| s == *skill) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., SKYBLUE);
            }
            icons::draw(*skill, rect.x + 8., rect.y + 6., 22.);
            let title = format!(
                "{}: {}",
                i + 1,
                content::skill_name_in(*skill, self.rules(false), self.zh)
            );
            label(
                &title,
                rect.x + 40.,
                rect.y + 23.,
                fit_size(&title, 18., rect.w - 48.),
                WHITE,
            );
            let details = format!(
                "{} · {} {}",
                if self.hero_id == 6
                    && self.mp < content::skill_cost_in(self.hero_id, *skill, self.rules(false))
                {
                    cost_text(
                        self.zh,
                        self.mp,
                        content::blood_cost(
                            self.hero_id,
                            self.awakened[0],
                            content::skill_cost_in(self.hero_id, *skill, self.rules(false))
                                - self.mp,
                        ),
                    )
                } else {
                    cost_text(
                        self.zh,
                        content::skill_cost_in(self.hero_id, *skill, self.rules(false)),
                        0,
                    )
                },
                tr(self.zh, "CD", "冷却"),
                skill.cooldown()
            );
            label(
                &details,
                rect.x + 8.,
                rect.y + 40.,
                fit_size(&details, 14., rect.w - 16.),
                if self.hero_id == 6
                    && self.mp < content::skill_cost_in(self.hero_id, *skill, self.rules(false))
                {
                    RED
                } else {
                    SKYBLUE
                },
            );
            for (line, text) in content::skill_description_in(*skill, self.rules(false), self.zh)
                .lines()
                .take(
                    if content::effective_skill(*skill, self.rules(false)).ends_turn() {
                        2
                    } else {
                        1
                    },
                )
                .enumerate()
            {
                // Shrink a long line to the box width instead of letting it spill out.
                label(
                    text,
                    rect.x + 8.,
                    rect.y + 54. + line as f32 * 14.,
                    fit_size(text, 13., rect.w - 16.),
                    LIGHTGRAY,
                );
            }
        }
        for (i, &skill) in content::skills(self.hero_id, self.awakened[0])
            .iter()
            .enumerate()
        {
            if !content::effective_skill(skill, self.rules(false)).ends_turn() {
                let rect = player_skill_rect(i);
                label(
                    content::role_text("Skill.continue", self.zh),
                    rect.x + 8.,
                    rect.y + 68.,
                    13.,
                    GOLD,
                );
            }
        }
        // Text is queued separately from shapes. Flush the cards first so the dimmer
        // covers their text and icons as well as their backgrounds.
        if content::skills(self.hero_id, self.awakened[0])
            .iter()
            .any(|s| self.cooldowns.remaining(0, *s) > 0)
            || content::skills(self.enemy_id, self.awakened[1])
                .iter()
                .any(|s| self.cooldowns.remaining(1, *s) > 0)
        {
            flush_labels();
            for (i, skill) in content::skills(self.hero_id, self.awakened[0])
                .iter()
                .enumerate()
            {
                let remaining = self.cooldowns.remaining(0, *skill);
                if remaining > 0 {
                    draw_cooldown_overlay(player_skill_rect(i), remaining);
                }
            }
            for (i, skill) in content::skills(self.enemy_id, self.awakened[1])
                .iter()
                .enumerate()
            {
                let remaining = self.cooldowns.remaining(1, *skill);
                if remaining > 0 {
                    draw_cooldown_overlay(enemy_skill_rect(i), remaining);
                }
            }
        }
    }
    pub(in crate::app) fn draw_skill_details(
        &mut self,
        frame: &Frame,
        hover_context: Option<(content::Skill, bool)>,
    ) {
        self.presentation.draw_health(self.zh);
        let detail_context = self.targeting.map(|skill| (skill, false)).or(hover_context);
        if let Some((skill, is_enemy)) = detail_context {
            let (id, health, mana, opponent, boost) = if is_enemy {
                (
                    self.enemy_id,
                    self.enemy,
                    self.enemy_mp,
                    self.hp,
                    if self.enemy_turn {
                        self.extra_bonus
                    } else {
                        self.bonuses[1]
                    },
                )
            } else {
                (self.hero_id, self.hp, self.mp, self.enemy, self.extra_bonus)
            };
            // Show details in the opposite sidebar, keeping the highlighted board visible.
            let x = if is_enemy { 14. } else { 790. };
            flush_labels();
            draw_rectangle(x, 208., 220., 438., Color::new(0.04, 0.07, 0.12, 0.98));
            draw_rectangle_lines(
                x,
                208.,
                220.,
                438.,
                2.,
                if is_enemy { ORANGE } else { SKYBLUE },
            );
            label(
                &format!(
                    "{} · {}",
                    if is_enemy {
                        tr(self.zh, "Enemy", "敌方")
                    } else {
                        tr(self.zh, "You", "我方")
                    },
                    content::skill_name_in(skill, self.rules(is_enemy), self.zh)
                ),
                x + 10.,
                239.,
                20.,
                GOLD,
            );
            label(
                &if id == 6 && mana < content::skill_cost_in(id, skill, self.rules(is_enemy)) {
                    cost_text(
                        self.zh,
                        mana,
                        content::blood_cost(
                            id,
                            self.awakened[is_enemy as usize],
                            content::skill_cost_in(id, skill, self.rules(is_enemy)) - mana,
                        ),
                    )
                } else {
                    format!(
                        "{}{}",
                        tr(self.zh, "Cost: ", "消耗："),
                        cost_text(
                            self.zh,
                            content::skill_cost_in(id, skill, self.rules(is_enemy)),
                            0
                        )
                    )
                },
                x + 10.,
                266.,
                18.,
                if id == 6 && mana < content::skill_cost_in(id, skill, self.rules(is_enemy)) {
                    RED
                } else {
                    SKYBLUE
                },
            );
            let description = content::skill_description_in(skill, self.rules(is_enemy), self.zh);
            for (line, text) in wrap_preview_text(description, 12.)
                .iter()
                .take(7)
                .enumerate()
            {
                label(text, x + 10., 292. + line as f32 * 18., 15., LIGHTGRAY);
            }

            let exact_target = if !is_enemy
                && self.targeting == Some(skill)
                && frame.pointer.x >= 256.
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
            let range = (content::targeted_in(skill, self.rules(is_enemy))
                || content::effective_skill(skill, self.rules(is_enemy)) == content::Skill::Hammer)
                && exact_target.is_none();
            let (low, high) = if range {
                content::preview_range_in(
                    skill,
                    &self.b,
                    id,
                    health,
                    if content::payment_in(id, health, mana, skill, self.rules(is_enemy)).is_some()
                    {
                        mana
                    } else {
                        mana.max(content::skill_cost_in(id, skill, self.rules(is_enemy)))
                    },
                    opponent,
                    boost,
                    self.rules(is_enemy),
                )
            } else {
                (
                    content::preview_rewards_in(
                        skill,
                        &self.b,
                        exact_target.unwrap_or((3, 3)),
                        id,
                        health,
                        if content::payment_in(id, health, mana, skill, self.rules(is_enemy))
                            .is_some()
                        {
                            mana
                        } else {
                            mana.max(content::skill_cost_in(id, skill, self.rules(is_enemy)))
                        },
                        opponent,
                        boost,
                        self.rules(is_enemy),
                    ),
                    content::preview_rewards_in(
                        skill,
                        &self.b,
                        exact_target.unwrap_or((3, 3)),
                        id,
                        health,
                        if content::payment_in(id, health, mana, skill, self.rules(is_enemy))
                            .is_some()
                        {
                            mana
                        } else {
                            mana.max(content::skill_cost_in(id, skill, self.rules(is_enemy)))
                        },
                        opponent,
                        boost,
                        self.rules(is_enemy),
                    ),
                )
            };
            let amount = |lo: i32, hi: i32| {
                if lo == hi {
                    lo.to_string()
                } else {
                    // Range over every possible target, e.g. 125～154; not Rust's `..` syntax.
                    format!("{lo}{}{hi}", tr(self.zh, "–", "～"))
                }
            };
            label(
                tr(self.zh, "Expected rewards", "预估收益"),
                x + 10.,
                433.,
                19.,
                GOLD,
            );
            label(
                &format!(
                    "{} {}",
                    tr(self.zh, "HP change:", "生命变化："),
                    amount(low.heal, high.heal)
                ),
                x + 10.,
                460.,
                18.,
                if low.heal < 0 { RED } else { GREEN },
            );
            label(
                &format!(
                    "{} {}",
                    tr(self.zh, "Damage:", "预计伤害："),
                    amount(low.damage, high.damage)
                ),
                x + 10.,
                487.,
                if exact_target.is_some() { 22. } else { 18. },
                ORANGE,
            );
            label(
                &format!(
                    "{} +{}",
                    tr(self.zh, "Mana:", "魔力："),
                    amount(low.mana_gain, high.mana_gain)
                ),
                x + 10.,
                514.,
                18.,
                SKYBLUE,
            );
            if content::payment_in(id, health, mana, skill, self.rules(is_enemy)).is_some() {
                label(
                    &format!(
                        "{} {mana} → {}",
                        tr(self.zh, "MP", "魔力"),
                        amount(low.mana_after, high.mana_after)
                    ),
                    x + 10.,
                    541.,
                    17.,
                    SKYBLUE,
                );
            } else {
                label(
                    tr(
                        self.zh,
                        "If funded; mana too low",
                        "筹足魔力后可获；当前不足",
                    ),
                    x + 10.,
                    541.,
                    14.,
                    ORANGE,
                );
            }
            if id == 6
                && mana < content::skill_cost_in(id, skill, self.rules(is_enemy))
                && content::payment_in(id, health, mana, skill, self.rules(is_enemy)).is_none()
            {
                label(
                    tr(
                        self.zh,
                        "Not enough HP; must leave 1",
                        "生命不足，支付后须剩1点",
                    ),
                    x + 10.,
                    598.,
                    15.,
                    RED,
                );
            }
            if low.blood_cost > 0
                && content::payment_in(id, health, mana, skill, self.rules(is_enemy)).is_some()
            {
                label(
                    &if self.zh {
                        format!("生命 -{} → {} !", low.blood_cost, health - low.blood_cost)
                    } else {
                        format!("-{} HP → {} !", low.blood_cost, health - low.blood_cost)
                    },
                    x + 10.,
                    598.,
                    18.,
                    RED,
                );
                label(
                    &format!(
                        "{} → {}",
                        tr(self.zh, "After: HP", "结算后生命"),
                        amount(low.health_after, high.health_after)
                    ),
                    x + 10.,
                    620.,
                    16.,
                    RED,
                );
            }
            if high.awakens_enemy {
                let text = if self.zh {
                    format!(
                        "敌方将觉醒：生命 → {}",
                        amount(low.enemy_health_after, high.enemy_health_after)
                    )
                } else {
                    format!(
                        "Enemy awakens: HP → {}",
                        amount(low.enemy_health_after, high.enemy_health_after)
                    )
                };
                label(&text, x + 10., 570., 13., GOLD);
            } else if high.awakens_self {
                label(
                    tr(
                        self.zh,
                        "HP payment triggers awakening",
                        "生命支付将触发觉醒",
                    ),
                    x + 10.,
                    570.,
                    13.,
                    GOLD,
                );
            } else if exact_target.is_some() {
                label(
                    tr(self.zh, "Click board to confirm", "点击棋盘确认释放"),
                    x + 10.,
                    570.,
                    15.,
                    WHITE,
                );
            } else if range {
                label(
                    tr(self.zh, "Range depends on target", "收益区间依目标而定"),
                    x + 10.,
                    570.,
                    15.,
                    WHITE,
                );
            }
            if skill == content::Skill::Strike {
                label(
                    tr(self.zh, "Next clear: +100%", "下次消除收益＋100%"),
                    x + 10.,
                    570.,
                    15.,
                    GREEN,
                );
            }
            label(
                tr(self.zh, "Refill cascades excluded", "补充后的连锁未计入"),
                x + 10.,
                641.,
                13.,
                GRAY,
            );
        }
    }
}

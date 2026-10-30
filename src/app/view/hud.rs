use super::*;
impl Session {
    pub(in crate::app) fn draw_actors(&mut self, frame: &Frame) {
        clear_background(Color::from_rgba(25, 27, 38, 255));
        set_camera(&frame.cam);
        procedural_board();
        for enemy_side in [false, true] {
            let panel = side_panel_rect(enemy_side);
            let active_side = enemy_side == self.enemy_turn && self.hp > 0 && self.enemy > 0;
            let accent = if enemy_side { ORANGE } else { SKYBLUE };
            let frame_color = if active_side {
                accent
            } else {
                Color::new(0.25, 0.28, 0.35, 0.6)
            };
            if active_side {
                draw_rectangle(
                    panel.x,
                    panel.y,
                    panel.w,
                    panel.h,
                    Color::new(accent.r, accent.g, accent.b, 0.07),
                );
            }
            draw_rectangle_lines(
                panel.x,
                panel.y,
                panel.w,
                panel.h,
                if active_side { 3. } else { 1. },
                frame_color,
            );
            let (id, health, mana) = if enemy_side {
                (self.enemy_id, self.enemy, self.enemy_mp)
            } else {
                (self.hero_id, self.hp, self.mp)
            };
            let x = panel.x + 12.;
            let name = if self.zh {
                HEROES[id].zh
            } else {
                HEROES[id].en
            };
            let room = panel.w - 24. - if active_side { 88. } else { 0. };
            label(name, x, 116., fit_size(name, 24., room), WHITE);
            if active_side {
                let tag = Rect::new(panel.x + panel.w - 92., 99., 80., 26.);
                draw_rectangle(
                    tag.x,
                    tag.y,
                    tag.w,
                    tag.h,
                    Color::new(accent.r * 0.3, accent.g * 0.3, accent.b * 0.3, 1.),
                );
                label_centered(
                    tr(self.zh, "ACTIVE", "行动中"),
                    tag.x + tag.w / 2.,
                    tag.y + 19.,
                    17.,
                    accent,
                );
            }
            let health_color = if enemy_side {
                Color::new(1., 0.62, 0.62, 1.)
            } else {
                Color::new(0.6, 1., 0.6, 1.)
            };
            for (row, (name, value, max, color)) in [
                (
                    tr(self.zh, "HP", "生命"),
                    health,
                    content::health_cap(id, self.awakened[enemy_side as usize]),
                    health_color,
                ),
                (
                    tr(self.zh, "MP", "魔力"),
                    mana,
                    content::mana_cap(id, self.awakened[enemy_side as usize]),
                    SKYBLUE,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 139. + row as f32 * 33.;
                let (shown, shown_max, extent) = if row == 0 {
                    self.presentation.health_bar(enemy_side, value, max)
                } else {
                    self.presentation
                        .mana_bar(enemy_side, value, content::mana_cap(id, false), max)
                };
                let amount = format!("{} / {}", shown.round() as i32, shown_max.round() as i32);
                label(name, x, y, 18., color);
                label(
                    &amount,
                    panel.x + panel.w - 12. - label_width(&amount, 18.),
                    y,
                    18.,
                    color,
                );
                let visual_max = if row == 1 {
                    content::mana_cap(id, true) as f32
                } else {
                    shown_max
                };
                let fill = (shown / visual_max.max(1.)).clamp(0., 1.) * extent;
                draw_rectangle(
                    x,
                    y + 7.,
                    200. * extent,
                    9.,
                    Color::new(0.1, 0.12, 0.17, 1.),
                );
                draw_rectangle(x, y + 7., 200. * fill, 9., color);
                if row == 1 && content::mana_cap(id, false) < content::mana_cap(id, true) {
                    let lock_x = x + 200. * content::mana_cap(id, false) as f32 / visual_max;
                    if !self.awakened[enemy_side as usize] {
                        draw_rectangle(
                            lock_x,
                            y + 7.,
                            200. - (lock_x - x),
                            9.,
                            Color::new(0.16, 0.19, 0.25, 1.),
                        );
                        draw_lock(lock_x - 5.5, y + 3., SKYBLUE);
                    } else if let Some(t) = self.presentation.awakening_progress(enemy_side) {
                        draw_breaking_lock(lock_x - 5.5, y + 3., t, SKYBLUE);
                    }
                }
            }
            let show_awake = self.awakened[enemy_side as usize]
                && self.presentation.awakened_visible(enemy_side);
            let sprite = if show_awake {
                &sprites::AWAKENED[id]
            } else {
                &sprites::SPRITES[id]
            };
            let casting = id >= content::KNIGHT
                && self.last_cast.is_some()
                && self.last_cast_enemy == enemy_side
                && self.cast_time > 0.;
            let cast_indices: &[usize] = if show_awake {
                &[12, 13, 14, 15]
            } else {
                &[8, 9, 10, 11]
            };
            let mut area = sprite_rect(enemy_side);
            if id == content::DWARF {
                let target = if show_awake {
                    [80, 60, 40]
                        .iter()
                        .filter(|&&v| health * 100 <= content::health_cap(id, true) * v)
                        .count() as f32
                } else {
                    0.
                };
                let shown = &mut self.book_levels[enemy_side as usize];
                *shown += (target - *shown).clamp(-(frame.dt as f32) * 5., (frame.dt as f32) * 5.);
                if *shown > 0.01 {
                    let scale = role_scale(area) * 0.5;
                    let foot_y = area.y + area.h;
                    let height = 8. + 3. * (*shown);
                    let x = area.x + area.w / 2. - 44.;
                    for (tex, y, h) in [
                        (&self.book_cover, foot_y - height * scale, 6. * scale),
                        (
                            &self.book_pages,
                            foot_y - (height - 6.) * scale,
                            3. * (*shown) * scale,
                        ),
                        (&self.book_bottom, foot_y - 2. * scale, 2. * scale),
                    ] {
                        draw_texture_ex(
                            tex,
                            x,
                            y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(88., h)),
                                ..Default::default()
                            },
                        );
                    }
                    area.h -= height * scale;
                }
            }
            if casting
                && content::effective_skill(self.last_cast.unwrap(), self.rules(enemy_side))
                    == content::Skill::Jump
            {
                area.y -= ((1.4 - self.cast_time) as f32 * std::f32::consts::PI / 1.4)
                    .sin()
                    .max(0.)
                    * 12.;
            }
            actor(
                if show_awake {
                    &self.awake_textures[id]
                } else if enemy_side {
                    &self.enemy_texture
                } else {
                    &self.player_texture
                },
                sprite.w,
                sprite.h,
                area,
                if casting {
                    (1.4 - self.cast_time).max(0.).min(0.519)
                } else {
                    get_time()
                },
                if casting {
                    &[0.13; 4]
                } else {
                    sprite.durations
                },
                if casting {
                    cast_indices
                } else {
                    sprite.indices
                },
            );
        }
    }
    pub(in crate::app) fn draw_turns(&mut self, frame: &Frame) {
        // Title centered over the player's sidebar.
        let title = tr(self.zh, "BATTLE BLOCKS", "方块战斗");
        let sidebar = side_panel_rect(false);
        label_centered(
            title,
            sidebar.x + sidebar.w / 2.,
            57.,
            fit_size(title, 32., sidebar.w - 24.),
            GOLD,
        );
        self.interface
            .draw_top(false, self.muted, self.paused, frame.pointer, self.zh);
        // Two-row turn indicator above the board. The current turn, next to the board,
        // stands out: who acts, what was done so far, its running result and the cascade
        // depth. The previous turn sits above it in muted colors.
        let current = current_turn_rect();
        let over = self.hp == 0 || self.enemy == 0;
        let turn_color = if over {
            GOLD
        } else if self.enemy_turn {
            ORANGE
        } else {
            SKYBLUE
        };
        draw_rectangle(
            current.x,
            current.y,
            current.w,
            current.h,
            Color::new(0.07, 0.10, 0.16, 0.98),
        );
        draw_rectangle_lines(current.x, current.y, current.w, current.h, 2., turn_color);
        let chip = Rect::new(current.x + 5., current.y + 5., 104., current.h - 10.);
        draw_rectangle(
            chip.x,
            chip.y,
            chip.w,
            chip.h,
            Color::new(
                turn_color.r * 0.3,
                turn_color.g * 0.3,
                turn_color.b * 0.3,
                1.,
            ),
        );
        label_centered(
            if self.enemy == 0 {
                tr(self.zh, "Victory", "胜利")
            } else if self.hp == 0 {
                tr(self.zh, "Defeat", "失败")
            } else if self.enemy_turn {
                tr(self.zh, "Enemy turn", "敌方回合")
            } else {
                tr(self.zh, "Your turn", "你的回合")
            },
            chip.x + chip.w / 2.,
            chip.y + 23.,
            18.,
            turn_color,
        );
        let chain_text = format!(
            "{} {chain}",
            tr(self.zh, "Chain", "连锁"),
            chain = self.chain
        );
        let chain_x = current.x + current.w - 14. - label_width(&chain_text, 18.);
        label(
            &chain_text,
            chain_x,
            current.y + 28.,
            18.,
            if self.chain > 1 {
                GOLD
            } else if self.chain == 1 {
                WHITE
            } else {
                GRAY
            },
        );
        let mut text_x = chip.x + chip.w + 12.;
        if let Some(skill) = self.turn_log.current.last_skill() {
            icons::draw(skill, text_x, current.y + 10., 22.);
            text_x += 30.;
        }
        let phase = if over {
            tr(self.zh, "Battle over", "战斗结束").to_string()
        } else if self.presentation.charging() {
            tr(self.zh, "Charging", "冲锋中").to_string()
        } else if let Some(swap) = self.presentation.swapping() {
            if swap {
                tr(self.zh, "Swapping", "正在交换")
            } else {
                tr(self.zh, "Tiles falling", "方块掉落")
            }
            .to_string()
        } else if !self.pending.is_empty() {
            tr(self.zh, "Clearing matches", "正在消除").to_string()
        } else if self.action_done {
            tr(self.zh, "Action resolved", "行动结算完成").to_string()
        } else if self.enemy_turn {
            tr(self.zh, "Enemy is choosing", "敌方思考中").to_string()
        } else if self.targeting == Some(content::Skill::Miracle) {
            content::role_text("Skill.force_prompt", self.zh).to_string()
        } else if let Some(skill) = self.targeting {
            format!(
                "{} · {}",
                skill.name(self.zh),
                tr(self.zh, "click a target", "点击棋盘选择目标")
            )
        } else {
            localized_note(self.zh, self.note).to_string()
        };
        // Actions so far, then the running result, or the phase until something changes.
        let actions = self.turn_log.current.actions_text(self.zh);
        let mut segments = Vec::new();
        if !actions.is_empty() {
            segments.push((actions, WHITE));
            segments.push((" · ".to_string(), GRAY));
        }
        segments.push(match self.turn_log.current.results_text(self.zh) {
            Some(results) => (results, GOLD),
            None => (phase, WHITE),
        });
        label_segments(
            &segments,
            text_x,
            current.y + 28.,
            19.,
            chain_x - 12. - text_x,
        );
        let previous = previous_turn_rect();
        draw_rectangle(
            previous.x,
            previous.y,
            previous.w,
            previous.h,
            Color::new(0.05, 0.06, 0.09, 0.92),
        );
        draw_rectangle_lines(
            previous.x,
            previous.y,
            previous.w,
            previous.h,
            1.,
            Color::new(0.25, 0.28, 0.35, 0.8),
        );
        label_centered(
            tr(self.zh, "Last turn", "上一回合"),
            chip.x + chip.w / 2.,
            previous.y + 21.,
            15.,
            GRAY,
        );
        let segments = match &self.turn_log.previous {
            Some(turn) => {
                let mut segments = vec![
                    if turn.enemy {
                        (tr(self.zh, "Enemy", "敌方").to_string(), ORANGE)
                    } else {
                        (tr(self.zh, "You", "我方").to_string(), SKYBLUE)
                    },
                    (tr(self.zh, ": ", "：").to_string(), GRAY),
                ];
                let actions = turn.actions_text(self.zh);
                if !actions.is_empty() {
                    segments.push((actions, LIGHTGRAY));
                    segments.push((" · ".to_string(), GRAY));
                }
                segments.push((
                    turn.results_text(self.zh)
                        .unwrap_or_else(|| tr(self.zh, "no gain", "无收益").to_string()),
                    LIGHTGRAY,
                ));
                if turn.chain > 1 {
                    segments.push((
                        format!(" · {} {}", tr(self.zh, "Chain", "连锁"), turn.chain),
                        GRAY,
                    ));
                }
                segments
            }
            None => vec![(tr(self.zh, "None yet", "暂无").to_string(), GRAY)],
        };
        let text_x = chip.x + chip.w + 12.;
        label_segments(
            &segments,
            text_x,
            previous.y + 21.,
            15.,
            previous.x + previous.w - 14. - text_x,
        );
        label(
            &format!(
                "{}{}",
                tr(self.zh, "AI: ", "AI 难度："),
                [
                    tr(self.zh, "Beginner", "入门"),
                    tr(self.zh, "Standard", "标准"),
                    tr(self.zh, "Advanced", "进阶"),
                ][self.difficulty]
            ),
            808.,
            452.,
            18.,
            GOLD,
        );
        // Below the board frame; the hint legend uses this line while hints are shown.
        if self.new_role_states[self.enemy_turn as usize].force_armed && !self.hints_shown {
            label(
                content::role_text("Skill.force_prompt", self.zh),
                262.,
                670.,
                17.,
                GOLD,
            );
        } else if self.feedback_time > 0. && self.targeting.is_none() && !self.hints_shown {
            label(&self.feedback, 262., 670., 17., GOLD);
        }
    }
}

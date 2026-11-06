use super::*;

/// A falling-block glyph: a 2x2 cluster with motion lines and an arrow pointing
/// where tiles settle. It reads without any aviation or level-training knowledge.
fn draw_gravity_glyph(center: Vec2, gravity: board::Gravity, accent: Color) {
    let (dx, dy) = gravity.vector();
    let d = vec2(dx as f32, dy as f32);
    let n = vec2(-d.y, d.x);
    let block = center - d * 3.;
    for (side, along, color) in [
        (-1., -1., Color::from_rgba(86, 148, 208, 255)),
        (1., -1., Color::from_rgba(101, 180, 108, 255)),
        (-1., 1., Color::from_rgba(206, 123, 72, 255)),
        (1., 1., Color::from_rgba(158, 118, 197, 255)),
    ] {
        let cell = block + n * (side * 3.) + d * (along * 3.);
        draw_rectangle(cell.x - 2.5, cell.y - 2.5, 5., 5., color);
    }
    let tail = block + d * 6.;
    let tip = center + d * 12.;
    draw_line(tail.x, tail.y, tip.x, tip.y, 2., accent);
    draw_triangle(tip, tip - d * 5. + n * 3.5, tip - d * 5. - n * 3.5, accent);
}
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
            outline(
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
                    tr_key(self.zh, "ui.hud.draw_actors.active"),
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
                    tr_key(self.zh, "ui.shared.hp"),
                    health,
                    content::health_cap(id, self.awakened[enemy_side as usize]),
                    health_color,
                ),
                (
                    tr_key(self.zh, "ui.shared.mp"),
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
            let show_shadow = sprites::shadow_form(
                id,
                show_awake,
                self.new_role_states[enemy_side as usize].statuses,
            );
            let sprite = if show_shadow {
                &sprites::AWAKENED[id]
            } else {
                &sprites::SPRITES[id]
            };
            let mut cast_elapsed = presentation::CAST_TIME - self.cast_time;
            let mut cast_clip = sprites::cast_clip(id, show_shadow, self.last_cast);
            let mut casting = cast_clip.is_some()
                && self.last_cast.is_some()
                && self.last_cast_enemy == enemy_side
                && self.cast_time > 0.;
            let synchronized =
                sprite.legacy_awake() || cast_clip.is_some_and(|clip| clip.impact.is_some());
            if let Some((clip, moment)) = cast_clip.filter(|clip| clip.impact.is_some()).zip(
                self.skill_effects.as_ref().and_then(|effects| {
                    effects.actor_moment(enemy_side, self.hp <= 0 || self.enemy <= 0)
                }),
            ) {
                cast_elapsed = clip.aligned_time(moment);
                casting = cast_elapsed < clip.duration();
            }
            if sprite.legacy_awake() {
                let sample = self.skill_effects.as_ref().and_then(|effects| {
                    effects.actor_sample(
                        enemy_side,
                        self.console.gameplay_action(),
                        self.hp <= 0 || self.enemy <= 0,
                    )
                });
                casting = sample.is_some();
                if let Some((clip, elapsed)) = sample {
                    cast_clip = Some(clip);
                    cast_elapsed = elapsed;
                }
            }
            let idle_elapsed = if synchronized {
                self.skill_effects
                    .as_mut()
                    .map(|effects| effects.idle_elapsed(enemy_side, casting, show_shadow))
                    .unwrap_or(0.)
            } else {
                get_time()
            };
            let role_state = &self.new_role_states[enemy_side as usize];
            let sleeping = effects::sleeping(
                role_state.statuses.contains(content::StatusId::Sleep),
                role_state.sleep_pending,
                (self.cast_time > 0. && self.last_cast_enemy == enemy_side)
                    || self
                        .skill_effects
                        .as_ref()
                        .is_some_and(|effects| effects.holds_board()),
            );
            let sleep_clip = sprite.sleep.filter(|_| sleeping);
            let sleep_elapsed = self
                .skill_effects
                .as_mut()
                .map(|effects| effects.sleep_elapsed(enemy_side, sleeping))
                .unwrap_or(0.);
            let (clip, elapsed) = if casting {
                (cast_clip.unwrap(), cast_elapsed)
            } else if let Some(clip) = sleep_clip {
                (clip, sleep_elapsed)
            } else {
                (sprite.idle(), idle_elapsed)
            };
            let mut area = sprite_rect(enemy_side);
            if content::ability_capabilities(id, self.rules(enemy_side)).reach_visual {
                let target = if show_awake {
                    [80, 60, 40]
                        .iter()
                        .filter(|&&v| health * 100 <= content::health_cap(id, true) * v)
                        .count() as f32
                } else {
                    0.
                };
                let cover = self.book_cover().clone();
                let pages = self.book_pages().clone();
                let bottom = self.book_bottom().clone();
                let shown = &mut self.book_levels[enemy_side as usize];
                *shown += (target - *shown).clamp(-(frame.dt as f32) * 5., (frame.dt as f32) * 5.);
                if *shown > 0.01 {
                    let scale = role_scale(area) * 0.5;
                    let foot_y = area.y + area.h;
                    let height = 8. + 3. * (*shown);
                    let x = area.x + area.w / 2. - 44.;
                    for (tex, y, h) in [
                        (&cover, foot_y - height * scale, 6. * scale),
                        (
                            &pages,
                            foot_y - (height - 6.) * scale,
                            3. * (*shown) * scale,
                        ),
                        (&bottom, foot_y - 2. * scale, 2. * scale),
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
            let foot = vec2(area.x + area.w / 2., area.y + area.h);
            let jump = self
                .skill_effects
                .as_ref()
                .and_then(|effects| effects.jump_elapsed(enemy_side));
            if let Some(elapsed) = jump {
                area.y -= effects::jump_lift(elapsed);
                if let Some(effects) = &self.skill_effects {
                    effects.draw_landing(foot, elapsed, false, enemy_side);
                }
            }
            if let Some(effects) = &self.skill_effects {
                effects.draw_heal(foot, false, enemy_side);
            }
            actor(
                if show_shadow {
                    self.awake_texture(enemy_side)
                } else if enemy_side {
                    self.enemy_texture()
                } else {
                    self.player_texture()
                },
                sprite,
                area,
                elapsed,
                clip,
            );
            if let Some(elapsed) = jump {
                if let Some(effects) = &self.skill_effects {
                    effects.draw_landing(foot, elapsed, true, enemy_side);
                }
            }
            if let Some(effects) = &self.skill_effects {
                effects.draw_heal(foot, true, enemy_side);
            }
            if sleeping {
                if let Some(effects) = &self.skill_effects {
                    effects.draw_sleep(area, foot, sleep_clip.is_some(), enemy_side);
                }
            }
        }
    }
    pub(in crate::app) fn draw_turns(&mut self, frame: &Frame) {
        self.draw_battle_status();
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
        outline(current.x, current.y, current.w, current.h, 2., turn_color);
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
                tr_key(self.zh, "battle.outcome.victory")
            } else if self.hp == 0 {
                tr_key(self.zh, "ui.hud.draw_turns.defeat")
            } else if self.enemy_turn {
                tr_key(self.zh, "ui.hud.draw_turns.enemy_turn")
            } else {
                tr_key(self.zh, "ui.hud.draw_turns.your_turn")
            },
            chip.x + chip.w / 2.,
            chip.y + 23.,
            18.,
            turn_color,
        );
        let chain_text = format!(
            "{} {chain}",
            tr_key(self.zh, "ui.hud.draw_turns.chain"),
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
            tr_key(self.zh, "ui.hud.draw_turns.battle_over").to_string()
        } else if self.presentation.charging() {
            tr_key(self.zh, "ui.hud.draw_turns.charging").to_string()
        } else if let Some(swap) = self.presentation.swapping() {
            if swap {
                tr_key(self.zh, "ui.hud.draw_turns.swapping")
            } else {
                tr_key(self.zh, "ui.hud.draw_turns.tiles_falling")
            }
            .to_string()
        } else if !self.pending.is_empty() {
            tr_key(self.zh, "ui.hud.draw_turns.clearing_matches").to_string()
        } else if self.action_done {
            tr_key(self.zh, "ui.hud.draw_turns.action_resolved").to_string()
        } else if self.enemy_turn {
            tr_key(self.zh, "ui.hud.draw_turns.enemy_is_choosing").to_string()
        } else if self
            .targeting
            .is_some_and(|s| content::skill_profile(s, self.rules(false)).force_move)
        {
            content::role_text("Skill.force_prompt", self.zh).to_string()
        } else if let Some(skill) = self.targeting {
            format!(
                "{} · {}",
                skill.name(self.zh),
                tr_key(self.zh, "ui.hud.draw_turns.click_a_target")
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
        outline(
            previous.x,
            previous.y,
            previous.w,
            previous.h,
            1.,
            Color::new(0.25, 0.28, 0.35, 0.8),
        );
        label_centered(
            tr_key(self.zh, "ui.hud.draw_turns.last_turn"),
            chip.x + chip.w / 2.,
            previous.y + 21.,
            15.,
            GRAY,
        );
        let segments = match &self.turn_log.previous {
            Some(turn) => {
                let mut segments = vec![
                    if turn.enemy {
                        (tr_key(self.zh, "ui.shared.enemy").to_string(), ORANGE)
                    } else {
                        (tr_key(self.zh, "ui.shared.you").to_string(), SKYBLUE)
                    },
                    (tr_key(self.zh, "ui.hud.draw_turns.text").to_string(), GRAY),
                ];
                let actions = turn.actions_text(self.zh);
                if !actions.is_empty() {
                    segments.push((actions, LIGHTGRAY));
                    segments.push((" · ".to_string(), GRAY));
                }
                segments.push((
                    turn.results_text(self.zh).unwrap_or_else(|| {
                        tr_key(self.zh, "ui.hud.draw_turns.no_gain").to_string()
                    }),
                    LIGHTGRAY,
                ));
                if turn.chain > 1 {
                    segments.push((
                        format!(
                            " · {} {}",
                            tr_key(self.zh, "ui.hud.draw_turns.chain"),
                            turn.chain
                        ),
                        GRAY,
                    ));
                }
                segments
            }
            None => vec![(
                tr_key(self.zh, "ui.hud.draw_turns.none_yet").to_string(),
                GRAY,
            )],
        };
        let text_x = chip.x + chip.w + 12.;
        label_segments(
            &segments,
            text_x,
            previous.y + 21.,
            15.,
            previous.x + previous.w - 14. - text_x,
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
            let y = if self
                .skill_effects
                .as_ref()
                .is_some_and(|fx| fx.healing_active(false) || fx.healing_active(true))
            {
                690.
            } else {
                670.
            };
            label(&self.feedback, 262., y, 17., GOLD);
        }
    }
    /// The old title area now carries round, gravity and global modifiers.
    /// The gravity row is absent when neither side can ever change gravity.
    fn gravity_change_available(&self) -> bool {
        let available = |id: usize| {
            content::skills(id, false)
                .iter()
                .chain(content::skills(id, true))
                .any(|&skill| {
                    content::skill_profile(skill, content::Rules::default()).gravity_change
                })
        };
        available(self.hero_id)
            || available(self.enemy_id)
            || self.board_mechanics.gravity_effect.is_some()
    }
    fn draw_battle_status(&self) {
        let gravity_visible = self.gravity_change_available();
        let (x, w) = (12., 224.);
        let h = if gravity_visible { 78. } else { 50. };
        let y = if gravity_visible { 6. } else { 20. };
        draw_rectangle(x, y, w, h, Color::from_rgba(20, 27, 38, 240));
        outline(x, y, w, h, 1., Color::new(0.25, 0.28, 0.35, 0.8));
        let round = crate::format_text(self.zh, "ui.hud.status.round", &[&self.round.to_string()]);
        label(&round, x + 10., y + 16., 13., LIGHTGRAY);
        let (heading, heading_color) = if self.round == content::FATIGUE_START_ROUND {
            (tr_key(self.zh, "ui.hud.status.next_round"), GOLD)
        } else {
            (tr_key(self.zh, "ui.hud.status.global"), GRAY)
        };
        label(
            heading,
            x + w - 10. - label_width(heading, 11.),
            y + 15.,
            11.,
            heading_color,
        );
        draw_line(
            x + 8.,
            y + 22.,
            x + w - 8.,
            y + 22.,
            1.,
            Color::from_rgba(45, 53, 66, 255),
        );
        let mut row = y + 22.;
        if gravity_visible {
            self.draw_gravity_row(x, row);
            row += 28.;
            draw_line(
                x + 8.,
                row,
                x + w - 8.,
                row,
                1.,
                Color::from_rgba(45, 53, 66, 255),
            );
        }
        self.draw_global_row(x, row, w);
    }
    fn draw_gravity_row(&self, x: f32, y: f32) {
        let gravity = self.board_mechanics.gravity();
        let active = self.board_mechanics.gravity_effect;
        let accent = match active {
            Some(effect) if effect.owner => ORANGE,
            Some(_) => SKYBLUE,
            None => GRAY,
        };
        draw_gravity_glyph(vec2(x + 20., y + 14.), gravity, accent);
        let name = match gravity {
            board::Gravity::Up => tr_key(self.zh, "ui.gravity.up"),
            board::Gravity::Down => tr_key(self.zh, "ui.gravity.down"),
            board::Gravity::Left => tr_key(self.zh, "ui.gravity.left"),
            board::Gravity::Right => tr_key(self.zh, "ui.gravity.right"),
        };
        let title = crate::format_text(self.zh, "ui.gravity.label", &[name]);
        label(
            &title,
            x + 38.,
            y + 14.,
            14.,
            if active.is_some() { accent } else { WHITE },
        );
        let state = if let Some(effect) = active {
            tr_key(
                self.zh,
                if effect.owner {
                    "ui.gravity.until_enemy"
                } else {
                    "ui.gravity.until_yours"
                },
            )
        } else {
            tr_key(self.zh, "ui.gravity.default")
        };
        label(
            state,
            x + 38.,
            y + 25.,
            10.,
            if active.is_some() { accent } else { GRAY },
        );
    }
    fn draw_global_row(&self, x: f32, y: f32, w: f32) {
        let displayed_round = if self.round == content::FATIGUE_START_ROUND {
            self.round + 1
        } else {
            self.round
        };
        let (damage_scale, healing_scale) = content::fatigue_scales(displayed_round);
        let damage = (damage_scale - 100).to_string();
        let healing = (100 - healing_scale).to_string();
        let color = if self.round < content::FATIGUE_START_ROUND {
            GRAY
        } else if self.round == content::FATIGUE_START_ROUND {
            GOLD
        } else {
            ORANGE
        };
        let segments = [
            (
                crate::format_text(self.zh, "ui.hud.status.damage", &[&damage]),
                color,
            ),
            ("   ".to_string(), GRAY),
            (
                crate::format_text(self.zh, "ui.hud.status.healing", &[&healing]),
                color,
            ),
        ];
        label_segments(&segments, x + 10., y + 18., 13., w - 20.);
    }
}

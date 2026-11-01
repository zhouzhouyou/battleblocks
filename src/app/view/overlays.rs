use super::*;
impl Session {
    pub(in crate::app) fn draw_status_tooltips(&mut self, frame: &Frame, show_moves: bool) {
        if !self.console.open && self.interface.panel.is_none() && !self.paused && !show_moves {
            for side in [false, true] {
                let panel = side_panel_rect(side);
                let id = if side { self.enemy_id } else { self.hero_id };
                let bar = Rect::new(panel.x + 12., 124., 200., 39.);
                if !self.awakened[side as usize] {
                    let tx = bar.x + 200. * content::AWAKENING_LINES[id] as f32 / 100.;
                    let cx = bar.x + 200. * content::AWAKENING_CAPS[id] as f32 / 100.;
                    // Independent upper/lower lanes preserve exact values even when equal.
                    draw_line(tx, 142., tx, 146., 1., GOLD);
                    draw_diamond(tx, 141., 5., GOLD, true);
                    draw_line(cx, 155., cx, 159., 1., SKYBLUE);
                    draw_diamond(cx, 160., 4., SKYBLUE, false);
                }
                if bar.contains(frame.pointer) {
                    let x = if side { 548. } else { 256. };
                    let r = Rect::new(
                        x,
                        128.,
                        220.,
                        if self.awakened[side as usize] {
                            98.
                        } else {
                            146.
                        },
                    );
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(25, 33, 46, 255));
                    outline(r.x, r.y, r.w, r.h, 1., SKYBLUE);
                    let line = content::awakening_line(id).to_string();
                    let threshold = content::AWAKENING_LINES[id].to_string();
                    let cap = content::health_cap(id, true).to_string();
                    let cap_percent = content::AWAKENING_CAPS[id].to_string();
                    let text = if self.awakened[side as usize] {
                        crate::format_text(self.zh, "hero.current_hp_cap", &[&cap])
                    } else {
                        let effect = content::awakening_effect(id, self.zh);
                        crate::format_text(
                            self.zh,
                            "hero.awakening_overlay",
                            &[&line, &threshold, &cap, &cap_percent, effect],
                        )
                    };
                    for (i, line) in wrap_preview_text(&text, 14.).iter().enumerate() {
                        label(line, x + 10., 150. + i as f32 * 18., 14., GOLD);
                    }
                }
                if content::mana_cap(id, false) < content::mana_cap(id, true)
                    && !self.awakened[side as usize]
                    && Rect::new(panel.x + 12., 165., 200., 30.).contains(frame.pointer)
                {
                    let x = if side { 548. } else { 256. };
                    draw_rectangle(x, 176., 220., 64., Color::from_rgba(25, 33, 46, 255));
                    label(
                        &crate::format_text(
                            self.zh,
                            "hero.mana_locked",
                            &[&content::mana_cap(id, false).to_string()],
                        ),
                        x + 10.,
                        199.,
                        16.,
                        SKYBLUE,
                    );
                    label(
                        &crate::format_text(
                            self.zh,
                            "hero.mana_unlocks",
                            &[&content::mana_cap(id, true).to_string()],
                        ),
                        x + 10.,
                        223.,
                        15.,
                        GOLD,
                    );
                }
            }
        }
    }
    pub(in crate::app) fn draw_effects(&mut self) {
        self.presentation.draw_awakening(self.zh);
        if self.presentation.charging() {
            let id = if self.last_cast_enemy {
                self.enemy_id
            } else {
                self.hero_id
            };
            let is_awake = self.awakened[self.last_cast_enemy as usize];
            let asset = if is_awake {
                &sprites::AWAKENED[id]
            } else {
                &sprites::SPRITES[id]
            };
            self.presentation.draw_charge(
                if is_awake {
                    &self.awake_textures[id]
                } else if self.last_cast_enemy {
                    self.enemy_texture()
                } else {
                    self.player_texture()
                },
                asset.w,
                asset.h,
                asset.indices,
            );
        }
        if let Some(skill) = self.last_cast.filter(|s| {
            self.cast_time > 0.
                && !content::skill_profile(*s, self.rules(self.last_cast_enemy)).delayed_damage
        }) {
            let progress = 1. - self.cast_time as f32 / presentation::CAST_TIME as f32;
            effects::draw(skill, &self.skill_flash, progress, self.last_cast_enemy);
        }
    }
    pub(in crate::app) fn draw_pockets(&mut self, frame: &Frame) {
        for side in [false, true] {
            let id = if side { self.enemy_id } else { self.hero_id };
            if content::ability_capabilities(id, self.rules(side)).pockets
                || self.new_role_states[side as usize]
                    .pockets
                    .iter()
                    .any(Option::is_some)
            {
                for slot in 0..2 {
                    let r = Rect::new(
                        if side {
                            796. + slot as f32 * 105.
                        } else {
                            20. + slot as f32 * 105.
                        },
                        if side { 610. } else { 670. },
                        100.,
                        25.,
                    );
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(39, 42, 50, 255));
                    if !side && self.new_role_states[0].pocket_slot == slot {
                        outline(r.x, r.y, r.w, r.h, 1.5, GOLD);
                    }
                    if let Some(tile) = self.new_role_states[side as usize].pockets[slot] {
                        draw_texture_ex(
                            &self.tiles()[tile as usize],
                            r.x + 4.,
                            r.y + 2.,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(21., 21.)),
                                ..Default::default()
                            },
                        );
                    }
                    let text = format!(
                        "{} {}",
                        tr_key(self.zh, "ui.overlays.draw_pockets.pocket"),
                        slot + 1
                    );
                    label(&text, r.x + 29., r.y + 18., 13., LIGHTGRAY);
                    if !side
                        && !frame.block_input
                        && !self.paused
                        && !self.enemy_turn
                        && !self.action_done
                        && frame.clicked(MouseButton::Left)
                        && r.contains(frame.pointer)
                    {
                        self.new_role_states[0].pocket_slot = slot;
                    }
                }
            }
        }
    }
    pub(in crate::app) fn draw_passives(
        &mut self,
        frame: &Frame,
        show_moves: bool,
        hover_context: Option<(content::Skill, bool)>,
    ) {
        // Each passive has its own identity, icon, state and hit target.
        let mut passive_popup = None;
        let mut status_popup: Option<Vec<String>> = None;
        for enemy_side in [false, true] {
            let id = if enemy_side {
                self.enemy_id
            } else {
                self.hero_id
            };
            let awake = self.awakened[enemy_side as usize];
            let area = passive_rect(enemy_side);
            let (health, mana) = if enemy_side {
                (self.enemy, self.enemy_mp)
            } else {
                (self.hp, self.mp)
            };
            let status_names: Vec<_> = self.new_role_states[enemy_side as usize]
                .statuses
                .0
                .iter()
                .flatten()
                .map(|s| {
                    let duration = match s.duration {
                        content::StatusDuration::OwnerTurnEnd(n) => {
                            if self.zh {
                                format!("{n}回合")
                            } else {
                                format!("{n} turn")
                            }
                        }
                        content::StatusDuration::UntilSwap => {
                            if self.zh {
                                "下次交换".into()
                            } else {
                                "next swap".into()
                            }
                        }
                        content::StatusDuration::UntilClear => {
                            if self.zh {
                                "下次消除".into()
                            } else {
                                "next clear".into()
                            }
                        }
                    };
                    format!("{}·{}", s.id.name(self.zh), duration)
                })
                .collect();
            if !status_names.is_empty() {
                let text = format!(
                    "{} {}",
                    if self.zh { "状态" } else { "Effects" },
                    status_names.join(" / ")
                );
                draw_rectangle(area.x, 430., 208., 24., Color::from_rgba(38, 29, 54, 255));
                label(
                    &text,
                    area.x + 4.,
                    447.,
                    fit_size(&text, 12., 200.),
                    Color::from_rgba(211, 181, 255, 255),
                );
                if !show_moves
                    && !frame.block_input
                    && Rect::new(area.x, 430., 208., 24.).contains(frame.pointer)
                {
                    let mut lines = Vec::new();
                    for status in self.new_role_states[enemy_side as usize]
                        .statuses
                        .0
                        .iter()
                        .flatten()
                    {
                        lines.push(format!(
                            "{} · {} {}",
                            status.id.name(self.zh),
                            if self.zh { "来源：" } else { "From:" },
                            status.applied_by.name(self.zh)
                        ));
                        lines.extend(wrap_preview_text(status.id.description(self.zh), 22.));
                    }
                    status_popup = Some(lines);
                }
            }
            for (i, &passive) in content::passives(id, awake).iter().enumerate() {
                let rect = Rect::new(
                    area.x + (i % 2) as f32 * 106.,
                    area.y + (i / 2) as f32 * 26.,
                    102.,
                    24.,
                );
                let hovered = !show_moves
                    && self.targeting.is_none()
                    && hover_context.is_none()
                    && !frame.block_input
                    && !self.paused
                    && self.interface.panel.is_none()
                    && rect.contains(frame.pointer);
                let status = if content::ability_enabled(
                    &content::EventContext::new(id, health, mana, self.rules(enemy_side)),
                    content::AbilityId::Passive(passive),
                ) {
                    passive.status(id, health, mana, awake, self.zh)
                } else {
                    Some((if self.zh { "已禁用" } else { "Disabled" }, false))
                };
                let color = if status.is_some_and(|(_, on)| !on) {
                    GRAY
                } else {
                    GOLD
                };
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::from_rgba(43, 40, 29, 255),
                );
                if hovered {
                    outline(rect.x, rect.y, rect.w, rect.h, 1., GOLD);
                }
                icons::draw_passive(passive, rect.x + 3., rect.y + 3., 18.);
                let name = passive.badge_name(self.zh, status);
                label(
                    &name,
                    rect.x + 25.,
                    rect.y + 16.,
                    fit_size(&name, 13., 73.),
                    color,
                );
                if passive.badge_highlight(status) {
                    draw_circle(rect.x + rect.w - 4., rect.y + 4., 2.5, ORANGE);
                }
                if hovered {
                    passive_popup = Some((passive, id, health, mana, awake, enemy_side));
                }
            }
        }
        if let Some((passive, id, health, mana, awake, enemy_side)) = passive_popup {
            let lines = wrap_preview_text(passive.description(self.zh), 18.);
            let status = if content::ability_enabled(
                &content::EventContext::new(id, health, mana, self.rules(enemy_side)),
                content::AbilityId::Passive(passive),
            ) {
                passive.status(id, health, mana, awake, self.zh)
            } else {
                Some((if self.zh { "已禁用" } else { "Disabled" }, false))
            };
            let relation = passive.replaces().map(|old| {
                crate::format_text(self.zh, "battle.replace_passive", &[old.name(self.zh)])
            });
            let height = 65.
                + lines.len() as f32 * 26.
                + if status.is_some() { 26. } else { 0. }
                + if relation.is_some() { 24. } else { 0. };
            flush_labels();
            draw_rectangle(270., 225., 485., height, Color::from_rgba(15, 23, 34, 255));
            icons::draw_passive(passive, 286., 239., 28.);
            label(passive.name(self.zh), 326., 261., 22., GOLD);
            let mut y = 292.;
            for line in &lines {
                label(line, 287., y, 18., WHITE);
                y += 26.;
            }
            if let Some((text, on)) = status {
                label(text, 287., y, 17., if on { ORANGE } else { GRAY });
                y += 26.;
            }
            if let Some(text) = relation {
                label(&text, 287., y, 15., SKYBLUE);
            }
        }
        if let Some(lines) = status_popup {
            flush_labels();
            draw_rectangle(
                270.,
                225.,
                485.,
                30. + lines.len() as f32 * 24.,
                Color::from_rgba(23, 20, 36, 250),
            );
            for (i, line) in lines.iter().enumerate() {
                label(
                    line,
                    286.,
                    251. + i as f32 * 24.,
                    17.,
                    Color::from_rgba(211, 181, 255, 255),
                );
            }
        }
    }
    pub(in crate::app) fn draw_reroll(&mut self) {
        if !self.enemy_turn
            && !self.action_done
            && self.pending.is_empty()
            && !self.presentation.busy()
            && !self.armed
            && self.hp > 0
            && self.enemy > 0
            && self.targeting.is_none()
            && board::hero_options(&self.b, self.hero_id).is_empty()
        {
            let rect = reroll_rect();
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.18, 0.24, 0.30, 1.),
            );
            outline(rect.x, rect.y, rect.w, rect.h, 2., GOLD);
            label(
                tr_key(
                    self.zh,
                    "ui.overlays.draw_reroll.no_swaps_cast_a_skill_reroll_f2",
                ),
                rect.x + 12.,
                rect.y + 25.,
                19.,
                GOLD,
            );
        }
    }
    pub(in crate::app) fn draw_overlays(&mut self, frame: &Frame) {
        if !self.paused && (self.hp == 0 || self.enemy == 0) {
            flush_labels();
            draw_rectangle(256., 190., 512., 340., Color::new(0.02, 0.02, 0.03, 0.96));
            if self.hp == 0 && self.enemy > 0 {
                draw_texture_ex(
                    self.defeat(),
                    422.,
                    205.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(180., 183.)),
                        ..Default::default()
                    },
                );
                label_centered(
                    tr_key(self.zh, "ui.overlays.draw_overlays.defeat"),
                    512.,
                    436.,
                    38.,
                    GOLD,
                );
            } else {
                label_centered(
                    tr_key(self.zh, "battle.banner.victory"),
                    512.,
                    345.,
                    40.,
                    GOLD,
                );
            }
            ui::draw_game_over_buttons(self.zh, frame.pointer);
        }
        if self.paused {
            ui::draw_pause(self.zh, frame.pointer);
        }
        self.interface
            .draw_panel(self.zh, self.muted, self.tiles(), frame.pointer);
        self.console.draw();
        flush_labels();
        set_default_camera();
    }
}

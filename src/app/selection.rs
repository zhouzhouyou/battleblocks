use super::*;
impl Session {
    pub(super) fn menu(&mut self, frame: &Frame) -> bool {
        if self.choosing {
            if let Some(audio) = &self.audio {
                audio.playing(false);
                audio.stop_speech();
            }
            self.sync_selection_groups();
            // Warm the visible roster in the background so switching heroes
            // rarely waits on the network. The selected atlas jumps the queue.
            for &id in self.interface.hero_group.roster() {
                crate::assets::want(sprites::SPRITES[id].asset.path);
            }
            let opponents = self.interface.opponent_step;
            let group = if opponents {
                self.interface.opponent_group
            } else {
                self.interface.hero_group
            };
            let mut requested_group = frame.key(KeyCode::Tab).then(|| group.other());
            if frame.clicked(MouseButton::Left) {
                for (i, candidate) in content::HeroGroup::ALL.into_iter().enumerate() {
                    if ui::group_tab_rect(opponents, i).contains(frame.pointer) {
                        requested_group = Some(candidate);
                    }
                }
            }
            if let Some(group) = requested_group {
                if opponents {
                    self.set_opponent_group(group);
                } else {
                    self.set_hero_group(group);
                }
            }
            if !self.interface.opponent_step {
                let roster = self.interface.hero_group.roster();
                let old_page = self.character_page;
                if frame.clicked(MouseButton::Left) && menu::page_rect(0).contains(frame.pointer) {
                    self.character_page = self.character_page.saturating_sub(1);
                }
                if frame.clicked(MouseButton::Left) && menu::page_rect(1).contains(frame.pointer) {
                    self.character_page =
                        (self.character_page + 1).min(menu::CHARACTER_PAGE_COUNT - 1);
                }
                if self.character_page != old_page {
                    // The awakening page lists different actives; preview the new list.
                    self.refresh_preview(0);
                }
                let roster_len = roster.len();
                self.interface.hero_scroll = ui::clamp_scroll(
                    self.interface.hero_scroll,
                    roster_len,
                    menu::HERO_VISIBLE_ROWS,
                );
                if frame.interactive
                    && !frame.block_input
                    && menu::hero_list_rect().contains(frame.pointer)
                {
                    let wheel = mouse_wheel().1;
                    if wheel != 0. {
                        self.interface.hero_scroll = ui::scroll_by(
                            self.interface.hero_scroll,
                            if wheel < 0. { 1 } else { -1 },
                            roster_len,
                            menu::HERO_VISIBLE_ROWS,
                        );
                    }
                }
                let old_hero = self.hero_id;
                if frame.key(KeyCode::Down) {
                    self.hero_id = roster[(roster
                        .iter()
                        .position(|id| *id == self.hero_id)
                        .unwrap_or(0)
                        + 1)
                        % roster.len()];
                }
                if frame.key(KeyCode::Up) {
                    self.hero_id = roster[(roster
                        .iter()
                        .position(|id| *id == self.hero_id)
                        .unwrap_or(0)
                        + roster.len()
                        - 1)
                        % roster.len()];
                }
                if frame.clicked(MouseButton::Left) {
                    for row in 0..menu::HERO_VISIBLE_ROWS {
                        let Some(&id) = roster.get(self.interface.hero_scroll + row) else {
                            break;
                        };
                        if menu::hero_rect(row).contains(frame.pointer) {
                            self.hero_id = id;
                        }
                    }
                }
                if self.hero_id != old_hero {
                    self.refresh_actor_textures();
                    self.refresh_preview(0);
                    self.reveal_hero();
                }
                let skill_rects =
                    menu::skill_rects(self.hero_id, self.character_page == 2, self.zh);
                for (i, rect) in skill_rects.iter().enumerate() {
                    if (frame.clicked(MouseButton::Left) && rect.contains(frame.pointer))
                        || (i == 0 && frame.key(KeyCode::Key1))
                        || (i == 1 && frame.key(KeyCode::Key2))
                        || (i == 2 && frame.key(KeyCode::Key3))
                    {
                        self.refresh_preview(i);
                    }
                }
            }
            let next = frame.key(KeyCode::Enter)
                || (frame.clicked(MouseButton::Left) && menu::start_rect().contains(frame.pointer));
            let start_battle = self.interface.opponent_step && next;
            if self.interface.opponent_step {
                let roster = self.interface.opponent_group.roster();
                let roster_len = roster.len();
                let opponent_rows = ui::opponent_row_count(roster.len());
                self.interface.opponent_scroll = ui::clamp_scroll(
                    self.interface.opponent_scroll,
                    opponent_rows,
                    ui::OPPONENT_VISIBLE_ROWS,
                );
                if frame.interactive
                    && !frame.block_input
                    && ui::opponent_list_rect().contains(frame.pointer)
                {
                    let wheel = mouse_wheel().1;
                    if wheel != 0. {
                        self.interface.opponent_scroll = ui::scroll_by(
                            self.interface.opponent_scroll,
                            if wheel < 0. { 1 } else { -1 },
                            opponent_rows,
                            ui::OPPONENT_VISIBLE_ROWS,
                        );
                    }
                }
                if frame.key(KeyCode::Escape)
                    || (frame.clicked(MouseButton::Left) && ui::back_rect().contains(frame.pointer))
                {
                    self.interface.opponent_step = false;
                    self.reveal_hero();
                }
                if frame.clicked(MouseButton::Left) {
                    for i in 0..3 {
                        if ui::difficulty_rect(i).contains(frame.pointer) {
                            self.difficulty = i;
                        }
                    }
                    for row in 0..ui::OPPONENT_VISIBLE_ROWS {
                        for column in 0..2 {
                            let slot = (self.interface.opponent_scroll + row) * 2 + column;
                            if slot > roster_len {
                                break;
                            }
                            if ui::opponent_rect(row, column).contains(frame.pointer) {
                                self.interface.opponent = if slot == 0 {
                                    None
                                } else {
                                    Some(roster[slot - 1])
                                };
                            }
                        }
                    }
                }
                let index = self
                    .interface
                    .opponent
                    .and_then(|id| roster.iter().position(|v| *v == id))
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let delta = if frame.key(KeyCode::Right) {
                    1
                } else if frame.key(KeyCode::Left) {
                    -1
                } else if frame.key(KeyCode::Down) {
                    2
                } else if frame.key(KeyCode::Up) {
                    -2
                } else {
                    0
                };
                if delta != 0 {
                    let i = (index as i32 + delta).rem_euclid(roster.len() as i32 + 1) as usize;
                    self.interface.opponent = if i == 0 { None } else { Some(roster[i - 1]) };
                    self.reveal_opponent();
                }
            } else if next {
                self.interface.opponent_step = true;
                self.reveal_opponent();
            }
            if start_battle {
                let before = self.snapshot();
                self.choosing = false;
                self.reset_battle();
                self.console.event(
                    "battle_start",
                    "Selected hero and opponent",
                    &before,
                    &self.snapshot(),
                );
            }
            if self.choosing {
                clear_background(Color::from_rgba(25, 27, 38, 255));
                set_camera(&frame.cam);
                if self.interface.opponent_step {
                    self.interface.draw_opponents(
                        self.hero_id,
                        self.zh,
                        self.thumbs(),
                        frame.pointer,
                        self.difficulty,
                        self.interface.opponent_scroll,
                    );
                } else {
                    // The awakening page and dark skill previews are the only menu
                    // views that draw the awakening sheet.
                    if self.wants_awake_texture() {
                        self.ensure_player_awake_texture();
                    }
                    menu::draw(
                        self.hero_id,
                        self.interface.hero_group,
                        self.zh,
                        self.player_sprite(),
                        self.thumbs(),
                        self.tiles(),
                        &self.preview,
                        frame.pointer,
                        self.character_page,
                        self.interface.hero_scroll,
                        self.awake_textures[0].as_ref(),
                    );
                }
                self.interface
                    .draw_top(true, self.muted, self.paused, frame.pointer, self.zh);
                self.interface
                    .draw_panel(self.zh, self.muted, self.tiles(), frame.pointer);
                self.console.draw();
                flush_labels();
                set_default_camera();
                return true;
            }
        }

        false
    }
    /// Environment overrides, console edits and rollback may select a hero outside
    /// the last menu group. Follow that hero without changing the other page.
    fn sync_selection_groups(&mut self) {
        let group = content::HeroGroup::for_hero(self.hero_id).expect("registered hero");
        if self.interface.hero_group != group {
            self.interface.hero_group = group;
            self.interface.hero_scroll = 0;
            self.reveal_hero();
        }
        if let Some(id) = self.interface.opponent {
            let group = content::HeroGroup::for_hero(id).expect("registered opponent");
            if self.interface.opponent_group != group {
                self.interface.opponent_group = group;
                self.interface.opponent_scroll = 0;
                self.reveal_opponent();
            }
        }
    }
    fn set_hero_group(&mut self, group: content::HeroGroup) {
        if self.interface.hero_group == group {
            return;
        }
        self.interface.hero_group = group;
        self.interface.hero_scroll = 0;
        self.hero_id = group.roster()[0];
        self.refresh_actor_textures();
        self.refresh_preview(0);
    }
    fn set_opponent_group(&mut self, group: content::HeroGroup) {
        if self.interface.opponent_group == group {
            return;
        }
        self.interface.opponent_group = group;
        self.interface.opponent_scroll = 0;
        self.interface.opponent = None;
    }
    /// Keep the selected hero inside the visible window, e.g. after arrow-key navigation.
    fn reveal_hero(&mut self) {
        let roster = self.interface.hero_group.roster();
        let index = roster
            .iter()
            .position(|id| *id == self.hero_id)
            .unwrap_or(0);
        self.interface.hero_scroll = ui::scroll_to_show(
            self.interface.hero_scroll,
            index,
            roster.len(),
            menu::HERO_VISIBLE_ROWS,
        );
    }
    /// Same for the opponent grid, whose random entry occupies index 0.
    fn reveal_opponent(&mut self) {
        let roster = self.interface.opponent_group.roster();
        let index = self
            .interface
            .opponent
            .and_then(|id| roster.iter().position(|v| *v == id))
            .map(|i| i + 1)
            .unwrap_or(0);
        self.interface.opponent_scroll = ui::scroll_to_show(
            self.interface.opponent_scroll,
            index / 2,
            ui::opponent_row_count(roster.len()),
            ui::OPPONENT_VISIBLE_ROWS,
        );
    }
}

#[cfg(test)]
mod tests;

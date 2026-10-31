use super::*;
impl Session {
    pub(super) fn menu(&mut self, frame: &Frame) -> bool {
        if self.choosing {
            if let Some(audio) = &self.audio {
                audio.playing(false);
            }
            if !self.interface.opponent_step {
                if frame.clicked(MouseButton::Left) && menu::page_rect(0).contains(frame.pointer) {
                    self.character_page = self.character_page.saturating_sub(1);
                }
                if frame.clicked(MouseButton::Left) && menu::page_rect(1).contains(frame.pointer) {
                    self.character_page = (self.character_page + 1).min(2);
                }
                let roster_len = content::ROSTER.len();
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
                    self.hero_id = content::ROSTER[(content::ROSTER
                        .iter()
                        .position(|id| *id == self.hero_id)
                        .unwrap_or(0)
                        + 1)
                        % content::ROSTER.len()];
                }
                if frame.key(KeyCode::Up) {
                    self.hero_id = content::ROSTER[(content::ROSTER
                        .iter()
                        .position(|id| *id == self.hero_id)
                        .unwrap_or(0)
                        + content::ROSTER.len()
                        - 1)
                        % content::ROSTER.len()];
                }
                if frame.clicked(MouseButton::Left) {
                    for row in 0..menu::HERO_VISIBLE_ROWS {
                        let Some(&id) = content::ROSTER.get(self.interface.hero_scroll + row)
                        else {
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
                for i in 0..content::skills(self.hero_id, self.character_page == 2).len() {
                    if (frame.clicked(MouseButton::Left)
                        && menu::skill_rect(i).contains(frame.pointer))
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
                let roster_len = content::ROSTER.len();
                let opponent_rows = ui::opponent_row_count();
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
                                    Some(content::ROSTER[slot - 1])
                                };
                            }
                        }
                    }
                }
                let index = self
                    .interface
                    .opponent
                    .and_then(|id| content::ROSTER.iter().position(|v| *v == id))
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
                    let i = (index as i32 + delta).rem_euclid(content::ROSTER.len() as i32 + 1)
                        as usize;
                    self.interface.opponent = if i == 0 {
                        None
                    } else {
                        Some(content::ROSTER[i - 1])
                    };
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
                    menu::draw(
                        self.hero_id,
                        self.zh,
                        self.player_texture(),
                        self.thumbs(),
                        self.tiles(),
                        &self.preview,
                        frame.pointer,
                        self.character_page,
                        self.interface.hero_scroll,
                        &self.awake_textures[self.hero_id],
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
    /// Keep the selected hero inside the visible window, e.g. after arrow-key navigation.
    fn reveal_hero(&mut self) {
        let index = content::ROSTER
            .iter()
            .position(|id| *id == self.hero_id)
            .unwrap_or(0);
        self.interface.hero_scroll = ui::scroll_to_show(
            self.interface.hero_scroll,
            index,
            content::ROSTER.len(),
            menu::HERO_VISIBLE_ROWS,
        );
    }
    /// Same for the opponent grid, whose random entry occupies index 0.
    fn reveal_opponent(&mut self) {
        let index = self
            .interface
            .opponent
            .and_then(|id| content::ROSTER.iter().position(|v| *v == id))
            .map(|i| i + 1)
            .unwrap_or(0);
        self.interface.opponent_scroll = ui::scroll_to_show(
            self.interface.opponent_scroll,
            index / 2,
            ui::opponent_row_count(),
            ui::OPPONENT_VISIBLE_ROWS,
        );
    }
}

use super::*;
impl Session {
    pub(in crate::app) fn draw_hints(&mut self, frame: &Frame, show_moves: bool) {
        let hints_rect = move_hints_rect();
        let hint_actor = if self.enemy_turn {
            self.enemy_id
        } else {
            self.hero_id
        };
        let hints_playable = show_moves && !self.enemy_turn;
        // Hints are derived only from the settled board; drawing never changes
        // selection, RNG, turn state, resources or the action log.
        let hint_health = if self.enemy_turn { self.enemy } else { self.hp };
        let hint_mana = if self.enemy_turn {
            self.enemy_mp
        } else {
            self.mp
        };
        let hint_opponent = if self.enemy_turn { self.hp } else { self.enemy };
        // Best total reward first; numbers on the board follow the same order.
        let (legal_hints, move_previews): (Vec<Move>, Vec<_>) = if show_moves {
            content::ranked_moves_in(
                &self.b,
                hint_actor,
                hint_health,
                hint_mana,
                hint_opponent,
                self.extra_bonus,
                self.armed,
                self.rules(self.enemy_turn),
            )
            .into_iter()
            .unzip()
        } else {
            (Vec::new(), Vec::new())
        };
        let pages = legal_hints.len().div_ceil(7).max(1);
        let previous_page_rect = Rect::new(798., 590., 52., 24.);
        let next_page_rect = Rect::new(950., 590., 52., 24.);
        if show_moves {
            let wheel = mouse_wheel().1;
            if wheel < 0.
                || frame.key(KeyCode::Right)
                || (frame.clicked(MouseButton::Left) && next_page_rect.contains(frame.pointer))
            {
                self.move_hint_page = (self.move_hint_page + 1).min(pages - 1);
            }
            if wheel > 0.
                || frame.key(KeyCode::Left)
                || (frame.clicked(MouseButton::Left) && previous_page_rect.contains(frame.pointer))
            {
                self.move_hint_page = self.move_hint_page.saturating_sub(1);
            }
            self.move_hint_page = self.move_hint_page.min(pages - 1);
        } else {
            self.move_hint_page = 0;
        }
        let row_focus = show_moves
            .then(|| hint_row_at(frame.pointer, self.move_hint_page, legal_hints.len()))
            .flatten();
        let arrow_focus = show_moves
            .then(|| hint_badge_at(frame.pointer, &legal_hints))
            .flatten();
        let focused_hint = row_focus.or(arrow_focus);
        if let Some(i) = arrow_focus {
            self.move_hint_page = i / 7;
        }
        if show_moves {
            for (i, &(a, c)) in legal_hints.iter().enumerate() {
                presentation::draw_move_hint(
                    a,
                    c,
                    hint_actor == 0 && !self.armed && a.1 == c.1,
                    i + 1,
                    focused_hint == Some(i),
                );
            }
            if let Some(p) = focused_hint.and_then(|i| move_previews[i].as_ref()) {
                for &(x, y) in &p.cells {
                    draw_rectangle_lines(
                        259. + x as f32 * 64.,
                        113. + y as f32 * 64.,
                        58.,
                        58.,
                        3.,
                        GREEN,
                    );
                }
            }
            if !legal_hints.is_empty() {
                label(
                    if hints_playable {
                        tr(
                            self.zh,
                            "Blue: tiles · Gold: columns · hold H + click a number",
                            "蓝色：两格交换 / 金色：整列交换 · 按住 H 点编号即可执行",
                        )
                    } else {
                        tr(
                            self.zh,
                            "Blue: tiles / Gold: whole columns",
                            "蓝色：两格交换 / 金色：整列交换",
                        )
                    },
                    256.,
                    674.,
                    17.,
                    GOLD,
                );
            }
        }
        if self.targeting.is_none() {
            draw_rectangle(
                hints_rect.x,
                hints_rect.y,
                hints_rect.w,
                hints_rect.h,
                Color::new(0.10, 0.17, 0.23, 1.),
            );
            draw_rectangle_lines(
                hints_rect.x,
                hints_rect.y,
                hints_rect.w,
                hints_rect.h,
                2.,
                if show_moves { GOLD } else { GRAY },
            );
            label(
                tr(self.zh, "Moves · hold H", "可行移动 · 按住 H"),
                hints_rect.x + 8.,
                hints_rect.y + 19.,
                16.,
                if self.hints_ready(frame) {
                    SKYBLUE
                } else {
                    GRAY
                },
            );
            let detail = if show_moves {
                format!(
                    "{} · {} {}",
                    if self.zh {
                        HEROES[hint_actor].zh
                    } else {
                        HEROES[hint_actor].en
                    },
                    legal_hints.len(),
                    tr(self.zh, "moves", "种交换")
                )
            } else {
                tr(
                    self.zh,
                    "Hover to list · click to play",
                    "悬停查看 · 点击条目执行",
                )
                .to_string()
            };
            label(
                &detail,
                hints_rect.x + 8.,
                hints_rect.y + 36.,
                13.,
                if show_moves { GOLD } else { GRAY },
            );
        }

        if show_moves {
            flush_labels();
            draw_rectangle(790., 208., 220., 430., Color::new(0.04, 0.07, 0.12, 0.98));
            draw_rectangle_lines(790., 208., 220., 430., 2., SKYBLUE);
            label(
                tr(self.zh, "Move rewards", "移动收益"),
                800.,
                232.,
                20.,
                GOLD,
            );
            label(
                if self.zh {
                    HEROES[hint_actor].zh
                } else {
                    HEROES[hint_actor].en
                },
                800.,
                252.,
                15.,
                WHITE,
            );
            label(
                tr(
                    self.zh,
                    "First clear; no refill chains",
                    "首轮收益，不含补充连锁",
                ),
                800.,
                270.,
                13.,
                GRAY,
            );
            for i in self.move_hint_page * 7..((self.move_hint_page + 1) * 7).min(legal_hints.len())
            {
                let row_y = 278. + (i % 7) as f32 * 44.;
                if focused_hint == Some(i) {
                    draw_rectangle(798., row_y, 204., 43., Color::new(0.16, 0.25, 0.30, 1.));
                    if hints_playable {
                        draw_rectangle_lines(798., row_y, 204., 43., 1., GOLD);
                        label(
                            tr(self.zh, "click", "点击执行"),
                            950.,
                            row_y + 35.,
                            12.,
                            GOLD,
                        );
                    }
                }
                let (a, c) = legal_hints[i];
                let column = hint_actor == 0 && !self.armed && a.1 == c.1;
                label(
                    &format!("#{} {}", i + 1, turns::swap_text(self.zh, a, c, column)),
                    800.,
                    row_y + 16.,
                    13.,
                    if column { GOLD } else { SKYBLUE },
                );
                if let Some(p) = &move_previews[i] {
                    label(
                        &format!(
                            "{}{}  {}{}  {}{}",
                            tr(self.zh, "Dmg ", "伤"),
                            p.damage,
                            tr(self.zh, "HP+", "治"),
                            p.heal,
                            tr(self.zh, "MP+", "魔"),
                            p.mana_gain
                        ),
                        800.,
                        row_y + 35.,
                        14.,
                        WHITE,
                    );
                }
            }
            if legal_hints.is_empty() {
                label(
                    tr(self.zh, "No legal exchanges", "当前无可行交换"),
                    800.,
                    305.,
                    17.,
                    GOLD,
                );
            }
            for (rect, enabled, text) in [
                (
                    previous_page_rect,
                    self.move_hint_page > 0,
                    tr(self.zh, "< Prev", "上一页"),
                ),
                (
                    next_page_rect,
                    self.move_hint_page + 1 < pages,
                    tr(self.zh, "Next >", "下一页"),
                ),
            ] {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.13, 0.20, 0.28, 1.),
                );
                draw_rectangle_lines(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    1.,
                    if enabled { SKYBLUE } else { GRAY },
                );
                label(
                    text,
                    rect.x + 5.,
                    rect.y + 17.,
                    13.,
                    if enabled { WHITE } else { GRAY },
                );
            }
            label(
                &format!("{} / {}", self.move_hint_page + 1, pages),
                867.,
                607.,
                14.,
                GOLD,
            );
            let focused_preview = focused_hint.and_then(|i| move_previews[i].as_ref());
            let footer = focused_preview
                .map(|p| {
                    format!(
                        "{} {hint_health}→{}  {} {hint_mana}→{}",
                        tr(self.zh, "HP", "生命"),
                        p.health_after,
                        tr(self.zh, "MP", "魔力"),
                        p.mana_after
                    )
                })
                .unwrap_or_else(|| {
                    if hints_playable {
                        tr(
                            self.zh,
                            "Click a row to play · ← → pages",
                            "点击条目即可执行 · ← → 翻页",
                        )
                    } else {
                        tr(self.zh, "Enemy moves: view only", "敌方的交换，仅供查看")
                    }
                    .to_string()
                });
            label(&footer, 800., 628., 13., LIGHTGRAY);
            if let Some(p) = focused_preview {
                let text = if self.zh {
                    format!(
                        "敌方生命 → {}{}",
                        p.enemy_health_after,
                        if p.awakens_enemy { " · 将觉醒" } else { "" }
                    )
                } else {
                    format!(
                        "Enemy HP → {}{}",
                        p.enemy_health_after,
                        if p.awakens_enemy { " · Awakening" } else { "" }
                    )
                };
                label(&text, 800., 646., 12., GOLD);
            }
        }
    }
}

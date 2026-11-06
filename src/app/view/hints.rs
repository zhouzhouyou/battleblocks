use super::*;
impl Session {
    pub(in crate::app) fn draw_hints(&mut self, frame: &Frame, show_moves: bool) {
        if self.advanced_hints && show_moves && !self.enemy_turn {
            self.draw_plan_hints(frame);
            return;
        }
        let hints_rect = move_hints_rect();
        let hint_actor = if self.enemy_turn {
            self.enemy_id
        } else {
            self.hero_id
        };
        let hints_playable = show_moves && !self.enemy_turn;
        let (hint_health, hint_mana) = if self.enemy_turn {
            (self.enemy, self.enemy_mp)
        } else {
            (self.hp, self.mp)
        };
        // Hints are derived only from the settled board; drawing never changes
        // selection, RNG, turn state, resources or the action log.
        // Best total reward first; numbers on the board follow the same order.
        let ranked = show_moves.then(|| self.ranked_hints(self.enemy_turn));
        let (legal_hints, move_previews): (Vec<Move>, Vec<_>) = ranked
            .as_deref()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|(m, preview)| (*m, preview.as_ref()))
            .unzip();
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
                    outline(
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
                        tr_key(
                            self.zh,
                            "ui.hints.draw_hints.blue_tiles_gold_columns_hold_h_cli",
                        )
                    } else {
                        tr_key(self.zh, "ui.hints.draw_hints.blue_tiles_gold_whole_columns")
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
            outline(
                hints_rect.x,
                hints_rect.y,
                hints_rect.w,
                hints_rect.h,
                2.,
                if show_moves { GOLD } else { GRAY },
            );
            let heading = if self.zh {
                "提示 H · 组合 Shift+H"
            } else {
                "Hints H · Plans Shift+H"
            };
            label(
                heading,
                hints_rect.x + 8.,
                hints_rect.y + 19.,
                fit_size(heading, 16., hints_rect.w - 16.),
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
                    tr_key(self.zh, "ui.hints.draw_hints.moves")
                )
            } else {
                if self.zh {
                    "悬停交换 · 点击组合提示"
                } else {
                    "Hover: swaps · Click: plans"
                }
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
            draw_rectangle(790., 208., 220., 454., Color::new(0.04, 0.07, 0.12, 0.98));
            outline(790., 208., 220., 454., 2., SKYBLUE);
            label(
                tr_key(self.zh, "ui.hints.draw_hints.move_rewards"),
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
                tr_key(self.zh, "ui.hints.draw_hints.first_clear_no_refill_chains"),
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
                        outline(798., row_y, 204., 43., 1., GOLD);
                        label(
                            tr_key(self.zh, "ui.hints.draw_hints.click"),
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
                            tr_key(self.zh, "ui.hints.draw_hints.dmg"),
                            p.damage,
                            tr_key(self.zh, "ui.hints.draw_hints.hp"),
                            p.heal,
                            tr_key(self.zh, "ui.hints.draw_hints.mp"),
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
                    tr_key(self.zh, "ui.hints.draw_hints.no_legal_exchanges"),
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
                    tr_key(self.zh, "ui.shared.prev"),
                ),
                (
                    next_page_rect,
                    self.move_hint_page + 1 < pages,
                    tr_key(self.zh, "ui.shared.next"),
                ),
            ] {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.13, 0.20, 0.28, 1.),
                );
                outline(
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
                        tr_key(self.zh, "ui.shared.hp"),
                        p.health_after,
                        tr_key(self.zh, "ui.shared.mp"),
                        p.mana_after
                    )
                })
                .unwrap_or_else(|| {
                    if hints_playable {
                        tr_key(self.zh, "ui.hints.draw_hints.click_a_row_to_play_pages")
                    } else {
                        tr_key(self.zh, "ui.hints.draw_hints.enemy_moves_view_only")
                    }
                    .to_string()
                });
            label(&footer, 800., 628., 13., LIGHTGRAY);
            if let Some(p) = focused_preview {
                let health = p.enemy_health_after.to_string();
                let awakening = if p.awakens_enemy {
                    crate::tr_key(self.zh, "battle.enemy_hp_awakening_suffix")
                } else {
                    ""
                };
                let text =
                    crate::format_text(self.zh, "battle.enemy_hp_preview", &[&health, awakening]);
                label(&text, 800., 646., 12., GOLD);
            }
        }
    }
}

impl Session {
    fn draw_plan_hints(&mut self, frame: &Frame) {
        let ranked = self.ranked_ai_plans(false);
        let plans = &ranked[..ranked.len().min(6)];
        let pages = crate::app::hints::plan_pages(plans);
        self.move_hint_page = self.move_hint_page.min(pages.len() - 1);
        let current_bottom = pages[self.move_hint_page]
            .last()
            .map_or(340., |(_, r)| r.y + r.h)
            .max(340.);
        let prev = Rect::new(584., current_bottom + 10., 70., 24.);
        let next = Rect::new(930., current_bottom + 10., 70., 24.);
        if frame.key(KeyCode::Right)
            || mouse_wheel().1 < 0.
            || (frame.clicked(MouseButton::Left) && next.contains(frame.pointer))
        {
            self.move_hint_page = (self.move_hint_page + 1).min(pages.len() - 1);
        }
        if frame.key(KeyCode::Left)
            || mouse_wheel().1 > 0.
            || (frame.clicked(MouseButton::Left) && prev.contains(frame.pointer))
        {
            self.move_hint_page = self.move_hint_page.saturating_sub(1);
        }
        self.move_hint_page = self.move_hint_page.min(pages.len() - 1);
        let rows = &pages[self.move_hint_page];
        let bottom = rows.last().map_or(340., |(_, r)| r.y + r.h).max(340.);
        // Border, rows and hit testing share geometry. Height follows the actual
        // step count; pagination bounds even the longest supported sequence.
        let panel = Rect::new(574., 208., 436., bottom - 208. + 46.);
        let footer_y = bottom + 14.;
        flush_labels();
        draw_rectangle(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            Color::new(0.04, 0.07, 0.12, 0.98),
        );
        outline(panel.x, panel.y, panel.w, panel.h, 2., SKYBLUE);
        let line = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            label(text, x, y, fit_size(text, size, 402.), color);
        };
        line(
            if self.zh {
                "AI 组合提示 · Shift+H / Esc 关闭"
            } else {
                "AI plans · Shift+H / Esc to close"
            },
            584.,
            232.,
            19.,
            GOLD,
        );
        line(
            if self.zh {
                "按 AI 收益排序 · 含补充连锁推演 · 点击方案全部执行"
            } else {
                "AI reward order · simulated cascades · click to execute"
            },
            584.,
            253.,
            13.,
            GRAY,
        );
        for &(i, rect) in rows {
            let plan = &plans[i];
            let focused = rect.contains(frame.pointer);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.10, 0.17, 0.23, 1.),
            );
            outline(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                1.,
                if focused { GOLD } else { GRAY },
            );
            let last = plan.steps.last().unwrap();
            let mana: i32 = plan.steps.iter().map(|s| s.mana_cost).sum();
            let blood: i32 = plan.steps.iter().map(|s| s.blood_cost).sum();
            let title = if self.zh {
                format!(
                    "#{} 评分 {:.1} · 伤害 {} · 消耗 魔力 {} / 血量 {}",
                    i + 1,
                    plan.score,
                    plan.damage,
                    mana,
                    blood
                )
            } else {
                format!(
                    "#{} Score {:.1} · DMG {} · Cost MP {} / HP {}",
                    i + 1,
                    plan.score,
                    plan.damage,
                    mana,
                    blood
                )
            };
            line(&title, rect.x + 7., rect.y + 19., 15., GOLD);
            for (n, step) in plan.steps.iter().enumerate() {
                let action = match step.action {
                    crate::app::planning::PlanAction::Swap((a, b)) => turns::swap_text(
                        self.zh,
                        a,
                        b,
                        content::movement_profile(self.hero_id).columns && a.1 == b.1,
                    ),
                    crate::app::planning::PlanAction::Cast(_, target, anchor) => {
                        if let Some(a) = anchor {
                            format!(
                                "{} ({},{}) → ({},{})",
                                step.name,
                                a.0 + 1,
                                a.1 + 1,
                                target.0 + 1,
                                target.1 + 1
                            )
                        } else {
                            format!("{} ({},{})", step.name, target.0 + 1, target.1 + 1)
                        }
                    }
                };
                let y = rect.y + 40. + n as f32 * 42.;
                line(
                    &format!("{}. {}", n + 1, action),
                    rect.x + 7.,
                    y,
                    15.,
                    SKYBLUE,
                );
                let details = if self.zh {
                    format!(
                        "消耗 MP {} / HP {} · 剩余 HP {} / MP {} · 敌 HP {}",
                        step.mana_cost, step.blood_cost, step.hp, step.mp, step.enemy_hp
                    )
                } else {
                    format!(
                        "Cost MP {} / HP {} · HP {} / MP {} · Enemy HP {}",
                        step.mana_cost, step.blood_cost, step.hp, step.mp, step.enemy_hp
                    )
                };
                line(&details, rect.x + 7., y + 17., 12., WHITE);
            }
            line(
                &format!(
                    "HP {} → {}   MP {} → {}   {} {}",
                    self.hp,
                    last.hp,
                    self.mp,
                    last.mp,
                    if self.zh { "敌方 HP" } else { "Enemy HP" },
                    last.enemy_hp
                ),
                rect.x + 7.,
                rect.y + rect.h - 9.,
                13.,
                LIGHTGRAY,
            );
        }
        if plans.is_empty() {
            line(
                if self.zh {
                    "当前没有可执行方案"
                } else {
                    "No executable plans"
                },
                584.,
                300.,
                17.,
                GOLD,
            );
        }
        // Draw and click the footer at the same adaptive position.
        let footer = format!(
            "{} / {} · {} {} · ← / →",
            self.move_hint_page + 1,
            pages.len(),
            plans.len(),
            if self.zh { "方案" } else { "plans" }
        );
        line(&footer, 664., footer_y + 12., 13., GOLD);
        for (x, text) in [
            (584., if self.zh { "上一页" } else { "Prev" }),
            (930., if self.zh { "下一页" } else { "Next" }),
        ] {
            let r = Rect::new(x, footer_y - 4., 70., 24.);
            outline(r.x, r.y, r.w, r.h, 1., SKYBLUE);
            label(text, r.x + 5., r.y + 17., 13., WHITE);
        }
    }
}

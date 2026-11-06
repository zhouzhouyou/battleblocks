use super::*;

impl Session {
    fn draw_board_mechanics(&self, frame: &Frame) {
        let draw_edge = |edge: board::PortalEdge, color: Color| {
            if edge.cells().is_none() {
                return;
            }
            let (x, y) = edge.cell;
            let px = 256. + x as f32 * 64.;
            let py = 110. + y as f32 * 64.;
            let (left, top, w, h) = if edge.horizontal {
                (px + 4., py + 60., 56., 8.)
            } else {
                (px + 60., py + 4., 8., 56.)
            };
            draw_rectangle(
                left - 2.,
                top - 2.,
                w + 4.,
                h + 4.,
                Color::from_rgba(22, 24, 32, 255),
            );
            draw_rectangle(left, top, w, h, color);
            draw_rectangle(left + 2., top + 2., w - 4., h - 4., WHITE);
        };
        for pair in self.board_mechanics.portals.iter().flatten() {
            draw_edge(pair.a, SKYBLUE);
            draw_edge(pair.b, ORANGE);
        }
        if let Some(anchor) = self.target_anchor {
            draw_edge(
                board::PortalEdge::at(anchor, self.board_mechanics.gravity()),
                GOLD,
            );
        }
        if let Some(skill) = self.targeting {
            if content::skill_profile(skill, self.rules(false)).paired_target {
                label_centered(
                    if self.zh {
                        if self.target_anchor.is_some() {
                            "选择第二扇门 · 不同重力行列"
                        } else {
                            "选择第一扇门 · 方块下边／右边"
                        }
                    } else {
                        if self.target_anchor.is_some() {
                            "Choose second portal on another gravity line"
                        } else {
                            "Choose first portal: bottom / right tile edge"
                        }
                    },
                    512.,
                    100.,
                    16.,
                    GOLD,
                );
            }
            if content::effective_skill(skill, self.rules(false)) == content::Skill::GravityShift {
                for (text, x, y) in if self.zh {
                    [
                        ("向上", 384., 230.),
                        ("向右", 640., 230.),
                        ("向左", 384., 486.),
                        ("向下", 640., 486.),
                    ]
                } else {
                    [
                        ("UP", 384., 230.),
                        ("RIGHT", 640., 230.),
                        ("LEFT", 384., 486.),
                        ("DOWN", 640., 486.),
                    ]
                } {
                    label_centered(text, x, y, 24., GOLD);
                }
                draw_line(512., 110., 512., 622., 2., GOLD);
                draw_line(256., 366., 768., 366., 2., GOLD);
            }
            if frame.pointer.x >= 256.
                && frame.pointer.x < 768.
                && frame.pointer.y >= 110.
                && frame.pointer.y < 622.
                && content::skill_profile(skill, self.rules(false)).paired_target
            {
                let target = (
                    ((frame.pointer.x - 256.) / 64.) as usize,
                    ((frame.pointer.y - 110.) / 64.) as usize,
                );
                let allowed = content::legal_target(
                    self.hero_id,
                    self.hp,
                    skill,
                    &self.b,
                    target,
                    self.rules(false),
                );
                draw_edge(
                    board::PortalEdge::at(target, self.board_mechanics.gravity()),
                    if allowed { GREEN } else { RED },
                );
            }
        }
    }
    /// Shade the settling side the actor cannot reach; the axis follows gravity.
    fn draw_reach_mask(&self) {
        let depth = content::reachable_depth(self.hero_id, self.hp, self.rules(false));
        if depth >= N || self.enemy_turn || !self.pending.is_empty() || self.action_done {
            return;
        }
        let excluded = (N - depth) as f32 * 64.;
        let grain = Color::new(0., 0., 0., 0.35);
        match self.board_mechanics.gravity() {
            board::Gravity::Down => {
                draw_rectangle(256., 110., 512., excluded, grain);
                draw_line(256., 110. + excluded, 768., 110. + excluded, 2., GOLD);
            }
            board::Gravity::Up => {
                draw_rectangle(256., 622. - excluded, 512., excluded, grain);
                draw_line(256., 622. - excluded, 768., 622. - excluded, 2., GOLD);
            }
            board::Gravity::Left => {
                draw_rectangle(768. - excluded, 110., excluded, 512., grain);
                draw_line(768. - excluded, 110., 768. - excluded, 622., 2., GOLD);
            }
            board::Gravity::Right => {
                draw_rectangle(256., 110., excluded, 512., grain);
                draw_line(256. + excluded, 110., 256. + excluded, 622., 2., GOLD);
            }
        }
    }

    pub(in crate::app) fn hints_ready(&self, frame: &Frame) -> bool {
        self.targeting.is_none()
            && !self.action_done
            && self.pending.is_empty()
            && !self.presentation.busy()
            && !frame.block_input
            && !self.paused
            && self.interface.panel.is_none()
            && self.hp > 0
            && self.enemy > 0
    }

    pub(in crate::app) fn hover_context(
        &mut self,
        frame: &Frame,
    ) -> (
        bool,
        Option<(content::Skill, bool)>,
        Option<(content::Skill, bool)>,
    ) {
        let hints_ready = self.hints_ready(frame);
        let hints_rect = move_hints_rect();
        // The continuous hover region bridges the entry button and its panel,
        // so a mouse/trackpad can reach rows and pagination without holding H.
        let hints_region = Rect::new(790., 208., 220., 484.);
        self.hints_hover_open = hints_ready
            && !frame.key(KeyCode::Escape)
            && (hints_rect.contains(frame.pointer)
                || ((self.hints_hover_open || is_key_down(KeyCode::H))
                    && hints_region.contains(frame.pointer)));
        let show_moves = hints_ready
            && !frame.key(KeyCode::Escape)
            && (is_key_down(KeyCode::H) || self.hints_hover_open);
        self.hints_shown = show_moves;
        // Hints can be played only on the player's own turn; the enemy's are view-only.
        let player_hover = if !show_moves
            && !self.enemy_turn
            && !self.action_done
            && self.pending.is_empty()
            && self.hp > 0
            && self.enemy > 0
            && !self.paused
            && !self.console.open
        {
            content::skills(self.hero_id, self.awakened[0])
                .iter()
                .enumerate()
                .find(|(i, _)| player_skill_rect(*i).contains(frame.pointer))
                .map(|(_, s)| (*s, false))
        } else {
            None
        };
        let enemy_hover = if !show_moves && !self.console.open {
            content::skills(self.enemy_id, self.awakened[1])
                .iter()
                .enumerate()
                .find(|(i, _)| enemy_skill_rect(*i).contains(frame.pointer))
                .map(|(_, s)| (*s, true))
        } else {
            None
        };
        let hover_context = player_hover.or(enemy_hover);

        (show_moves, player_hover, hover_context)
    }
    pub(in crate::app) fn draw_board(
        &mut self,
        frame: &Frame,
        show_moves: bool,
        hover_context: Option<(content::Skill, bool)>,
    ) {
        let hover_skill = hover_context.map(|(skill, _)| skill);
        if !diagnostic("legacy_batches") {
            for y in 0..N {
                for x in 0..N {
                    let pos = vec2(256. + x as f32 * 64., 110. + y as f32 * 64.);
                    procedural_cell(pos.x, pos.y);
                }
            }
        }
        let mut preview_cells = self
            .targeting
            .or(hover_skill.filter(|s| {
                !content::targeted_in(*s, self.rules(hover_context.is_some_and(|(_, side)| side)))
            }))
            .map(|skill| {
                if content::targeted_in(
                    skill,
                    self.rules(hover_context.is_some_and(|(_, side)| side)),
                ) && !(frame.pointer.x >= 256.
                    && frame.pointer.x < 768.
                    && frame.pointer.y >= 110.
                    && frame.pointer.y < 622.)
                {
                    return Vec::new();
                }
                let x = ((frame.pointer.x - 256.) / 64.).clamp(0., 7.) as usize;
                let y = ((frame.pointer.y - 110.) / 64.).clamp(0., 7.) as usize;
                let side = hover_context.is_some_and(|(_, side)| side);
                let (id, health, mana) = if side {
                    (self.enemy_id, self.enemy, self.enemy_mp)
                } else {
                    (self.hero_id, self.hp, self.mp)
                };
                if !content::legal_target(id, health, skill, &self.b, (x, y), self.rules(side)) {
                    return Vec::new();
                }
                content::target_cells_in(skill, &self.b, (x, y), id, health, mana, self.rules(side))
            })
            .unwrap_or_default();
        if !show_moves
            && (self.targeting.is_none()
                || self
                    .targeting
                    .is_some_and(|s| content::skill_profile(s, self.rules(false)).force_move))
            && !self.enemy_turn
            && !self.action_done
        {
            if let Some(a) = self.selected {
                if frame.pointer.x >= 256.
                    && frame.pointer.x < 768.
                    && frame.pointer.y >= 110.
                    && frame.pointer.y < 622.
                {
                    let c = (
                        ((frame.pointer.x - 256.) / 64.) as usize,
                        ((frame.pointer.y - 110.) / 64.) as usize,
                    );
                    let mut copy = self.b;
                    if board::hero_swap_in(&mut copy, a, c, self.hero_id, self.board_mechanics) {
                        preview_cells = self.board_mechanics.matches(&copy);
                    }
                    if content::movement_profile(self.hero_id).columns
                        && board::adjacent(a, c, self.hero_id)
                        && a.1 == c.1
                    {
                        for y in 0..N {
                            for x in [a.0, c.0] {
                                outline(
                                    256. + x as f32 * 64.,
                                    110. + y as f32 * 64.,
                                    64.,
                                    64.,
                                    2.,
                                    GOLD,
                                );
                            }
                        }
                    }
                }
            }
        }
        if self.targeting.is_some() {
            let cancel = cancel_skill_rect();
            draw_rectangle(
                cancel.x,
                cancel.y,
                cancel.w,
                cancel.h,
                Color::new(0.24, 0.16, 0.18, 1.),
            );
            outline(cancel.x, cancel.y, cancel.w, cancel.h, 1., GOLD);
            label(
                tr_key(self.zh, "ui.battlefield.draw_board.cancel_esc_right_click"),
                805.,
                665.,
                18.,
                GOLD,
            );
        }
        for kind in 0..if !diagnostic("legacy_batches") { 7 } else { 1 } {
            for y in 0..N {
                for x in 0..N {
                    if !diagnostic("legacy_batches") && self.b[y][x] != kind {
                        continue;
                    }
                    let pos = vec2(256. + x as f32 * 64., 110. + y as f32 * 64.);
                    let charge_progress = self.presentation.destruction(x, y);
                    let clearing = self.pending.contains(&(x, y))
                        && (!self.presentation.busy() || charge_progress.is_some_and(|p| p > 0.));
                    let hold_until_impact = effects::holds_tiles(
                        self.last_cast,
                        (presentation::CAST_TIME - self.cast_time) as f32,
                    ) && self.skill_flash.contains(&(x, y))
                        || self.skill_effects.as_ref().is_some_and(|effects| {
                            effects.holds_board() || effects.holds_cell(x, y)
                        });
                    let clear_progress = if hold_until_impact {
                        0.
                    } else if let Some(progress) = charge_progress {
                        progress
                    } else if clearing {
                        (1. - self.deadline as f32 / CLEAR_TIME as f32).clamp(0., 1.)
                    } else {
                        0.
                    };
                    let color = Color::new(1., 1., 1., 1. - clear_progress);
                    let tile_size = 64. * (1. - clear_progress * 0.65);
                    if diagnostic("legacy_batches") {
                        procedural_cell(pos.x, pos.y);
                    }
                    if !self.presentation.hides(x, y) {
                        draw_texture_ex(
                            &self.tiles()[self.b[y][x] as usize],
                            pos.x + (64. - tile_size) / 2.,
                            pos.y + (64. - tile_size) / 2.,
                            color,
                            DrawTextureParams {
                                dest_size: Some(vec2(tile_size, tile_size)),
                                ..Default::default()
                            },
                        );
                    }
                    if preview_cells.contains(&(x, y)) {
                        outline(pos.x + 3., pos.y + 3., 58., 58., 3., SKYBLUE);
                    }
                    if clearing && !hold_until_impact {
                        let pulse = (get_time() as f32 * 14.).sin().abs();
                        draw_rectangle(
                            pos.x,
                            pos.y,
                            64.,
                            64.,
                            Color::new(1., 0.7, 0.2, 0.15 + pulse * 0.35),
                        );
                        draw_circle_lines(pos.x + 32., pos.y + 32., 15. + pulse * 15., 2., GOLD);
                    }
                    if !self.pending.is_empty()
                        && self
                            .move_flash
                            .is_some_and(|(a, c)| a == (x, y) || c == (x, y))
                    {
                        outline(
                            pos.x + 2.,
                            pos.y + 2.,
                            60.,
                            60.,
                            4.,
                            if self.enemy_turn { ORANGE } else { SKYBLUE },
                        );
                    }
                    if self.selected == Some((x, y)) {
                        outline(pos.x, pos.y, 62., 62., 3., GOLD);
                    }
                }
            }
        }
        self.presentation.draw_tiles(self.tiles());
        self.draw_board_mechanics(frame);
        self.draw_reach_mask();
        if content::reachable_depth(self.hero_id, self.hp, self.rules(false)) < N
            && self.feedback_time > 0.
            && self.feedback == self.rejection_message(false)
        {
            draw_rectangle(22., 202., 204., 30., Color::from_rgba(20, 27, 38, 245));
            outline(22., 202., 204., 30., 1., GOLD);
            label_centered(&self.feedback, 124., 223., 16., WHITE);
        }
        for (side, reaction) in self.reactions.iter().enumerate() {
            if let Some((text, _)) = reaction {
                let x = if side == 0 { 22. } else { 798. };
                draw_rectangle(x, 202., 204., 30., Color::from_rgba(20, 27, 38, 245));
                outline(x, 202., 204., 30., 1., GOLD);
                label_centered(text, x + 102., 223., 16., WHITE);
            }
        }
    }
}

use super::*;
impl Session {
    pub(super) fn hint_action(
        &mut self,
        frame: &mut Frame,
        cast_this_frame: bool,
    ) -> Option<usize> {
        // Clicking a listed exchange (a reward-panel row, or a numbered badge on the
        // board while H is held) plays it through the normal player swap below.
        let mut hint_number = None;
        if self.hints_shown
            && frame.command_move.is_none()
            && self.pending.is_empty()
            && !self.enemy_turn
            && !self.action_done
            && self.targeting.is_none()
            && !cast_this_frame
            && frame.clicked(MouseButton::Left)
        {
            // Same ranking as the drawn list, so row and badge numbers match.
            let hints: Vec<Move> = content::ranked_moves_in(
                &self.b,
                self.hero_id,
                self.hp,
                self.mp,
                self.enemy,
                self.extra_bonus,
                self.armed,
                self.rules(false),
            )
            .into_iter()
            .map(|(m, _)| m)
            .collect();
            if let Some(i) =
                hint_row_at(frame.pointer, self.move_hint_page, hints.len()).or_else(|| {
                    is_key_down(KeyCode::H)
                        .then(|| hint_badge_at(frame.pointer, &hints))
                        .flatten()
                })
            {
                frame.command_move = Some(hints[i]);
                hint_number = Some(i + 1);
            }
        }

        hint_number
    }
    pub(super) fn player_swap(
        &mut self,
        frame: &Frame,
        hint_number: Option<usize>,
        cast_this_frame: bool,
    ) {
        if (frame.clicked(MouseButton::Left) || frame.command_move.is_some())
            && self.hp > 0
            && !self.enemy_turn
            && !self.action_done
            && (self.targeting.is_none()
                || self
                    .targeting
                    .is_some_and(|s| content::skill_profile(s, self.rules(false)).force_move))
            && !cast_this_frame
        {
            let (mx, my) = mouse_position();
            let world = frame.cam.screen_to_world(vec2(mx, my));
            let x = (world.x - 256.) / 64.;
            let y = (world.y - 110.) / 64.;
            if frame.command_move.is_some() || (x >= 0. && x < 8. && y >= 0. && y < 8.) {
                let c = frame
                    .command_move
                    .map(|(_, c)| c)
                    .unwrap_or((x as usize, y as usize));
                if let Some((a, _)) = frame.command_move {
                    self.selected = Some(a);
                }
                if content::reachable_rows(self.hero_id, self.hp, self.rules(false)) < N
                    && c.1 < N - content::reachable_rows(self.hero_id, self.hp, self.rules(false))
                {
                    self.selected = None;
                    if self.feedback_time <= 0. || self.feedback != self.rejection_message(false) {
                        self.feedback = self.rejection_message(false).to_string();
                        self.feedback_time = 1.8;
                    }
                } else if let Some(a) = self.selected {
                    if self
                        .targeting
                        .is_some_and(|s| content::skill_profile(s, self.rules(false)).force_move)
                    {
                        let mut preview = self.b;
                        if content::legal_swap_region(
                            self.hero_id,
                            self.hp,
                            a,
                            c,
                            self.rules(false),
                        ) && role_state::force_swap(&mut preview, a, c, self.hero_id)
                        {
                            let force_skill = self.targeting;
                            self.targeting = None;
                            if !self.cast_skill(frame, force_skill, true) {
                                return;
                            }
                        } else {
                            self.selected = Some(c);
                            self.note = "note.no_match";
                            return;
                        }
                    }
                    let before = self.snapshot();
                    let forced = self.new_role_states[0].force_armed;
                    let reachable =
                        content::legal_swap_region(self.hero_id, self.hp, a, c, self.rules(false));
                    let legal = if !reachable {
                        false
                    } else if forced {
                        let ok =
                            self.new_role_states[0].complete_force(&mut self.b, a, c, self.hero_id);
                        if ok {
                            self.new_role_states[0].force_armed = false;
                        }
                        ok
                    } else if self.armed && a.0.abs_diff(c.0) + a.1.abs_diff(c.1) == 1 {
                        parallel_swap(&mut self.b, a, c);
                        self.armed = false;
                        self.new_role_states[self.enemy_turn as usize]
                            .statuses
                            .consume(content::StatusDuration::UntilSwap);
                        true
                    } else {
                        board::hero_swap(&mut self.b, a, c, self.hero_id)
                    };
                    if legal {
                        self.presentation.swap(
                            &before.board,
                            a,
                            c,
                            before.armed,
                            self.enemy_turn,
                            self.hero_id,
                        );
                        self.turn_log.current.actions.push(turns::Action::Swap {
                            from: a,
                            to: c,
                            columns: content::movement_profile(self.hero_id).columns
                                && !before.armed
                                && a.1 == c.1,
                        });
                        self.console.checkpoint(before.clone());
                        self.new_role_states[0].record_swap(a, c);
                        self.action_done = !forced;
                        self.move_flash = Some((a, c));
                        self.pending = matches(&self.b);
                        self.reward = true;
                        self.deadline = CLEAR_TIME;
                        self.chain = 1;
                        self.selected = None;
                        self.note = "note.match";
                        self.console.event(
                            "move",
                            &format!(
                                "swap [{},{}] -> [{},{}]{}",
                                a.0 + 1,
                                a.1 + 1,
                                c.0 + 1,
                                c.1 + 1,
                                hint_number
                                    .map(|n| format!(" via hint #{n}"))
                                    .unwrap_or_default()
                            ),
                            &before,
                            &self.snapshot(),
                        );
                    } else {
                        self.selected = Some(c);
                        if !reachable
                            && (self.feedback_time <= 0.
                                || self.feedback != self.rejection_message(false))
                        {
                            self.feedback = self.rejection_message(false).to_string();
                            self.feedback_time = 1.8;
                        }
                        self.note = "note.no_match";
                        self.console.event(
                            "move_rejected",
                            &format!(
                                "swap [{},{}] -> [{},{}]: no match or nonadjacent",
                                a.0 + 1,
                                a.1 + 1,
                                c.0 + 1,
                                c.1 + 1
                            ),
                            &before,
                            &self.snapshot(),
                        );
                    }
                } else {
                    self.selected = Some(c);
                }
            }
        }
    }
}

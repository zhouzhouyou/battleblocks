use super::*;
impl Session {
    pub(super) fn tick(&mut self, frame: &mut Frame) {
        if frame.key(KeyCode::Escape) || frame.clicked(MouseButton::Right) {
            self.hint_queue.clear();
        }
        if let Some(audio) = &self.audio {
            audio.playing(
                !self.paused
                    && !self.console.open
                    && self.interface.panel.is_none()
                    && self.hp > 0
                    && self.enemy > 0,
            );
        }
        // Visual lifetimes continue after victory/defeat and are independent of rewards.
        self.feedback_time = (self.feedback_time - frame.dt).max(0.);
        for reaction in &mut self.reactions {
            if let Some((_, remaining)) = reaction {
                *remaining = (*remaining - frame.dt).max(0.);
                if *remaining == 0. {
                    *reaction = None;
                }
            }
        }
        self.cast_time = (self.cast_time - frame.dt).max(0.);
        if self.cast_time == 0. {
            self.skill_flash.clear();
            self.last_cast = None;
        }
        if !self.paused && !self.console.open && self.interface.panel.is_none() {
            self.presentation.tick(frame.dt);
        }
        let active = !self.paused
            && !self.console.open
            && !frame.ui_consumed
            && !frame.pause_click
            && self.hp > 0
            && self.enemy > 0
            && !self.presentation.busy();
        if active {
            self.advance_turn(frame);
            if self.hp > 0 && self.enemy > 0 {
                self.drive_hint_queue(frame);
                let cast = self.choose_cast(frame);
                let cast_this_frame = self.cast_skill(frame, cast, false);
                // A lethal direct hit ends battle before a large frame step can
                // settle another clear from the same action.
                if self.hp > 0 && self.enemy > 0 {
                    let hint_number = self.hint_action(frame, cast_this_frame);
                    if !self.pending.is_empty() {
                        self.resolve_pending(frame.dt);
                    } else {
                        self.player_swap(frame, hint_number, cast_this_frame);
                    }
                }
            }
        }

        if (self.hp == 0 || self.enemy == 0) && !self.game_over_logged {
            self.console.event(
                "battle_end",
                if self.enemy == 0 {
                    "Player victory"
                } else {
                    "Player defeat"
                },
                &self.snapshot(),
                &self.snapshot(),
            );
            self.game_over_logged = true;
        }
    }
}

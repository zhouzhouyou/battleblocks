use super::*;
impl Session {
    pub(super) fn speak_role(&self, id: usize, cue: &str) -> bool {
        let Some(audio) = &self.audio else {
            return false;
        };
        if !self.muted {
            if let Some(text) = content::role_lore(id).and_then(|lore| lore.voice(cue, self.zh)) {
                audio.speak(text, self.zh);
                return true;
            }
        }
        false
    }
    pub(super) fn tick(&mut self, frame: &mut Frame) {
        if frame.key(KeyCode::Escape) || frame.clicked(MouseButton::Right) {
            self.hint_queue.clear();
        }
        if let Some(audio) = &self.audio {
            if self.paused || self.console.open || self.interface.panel.is_some() {
                audio.stop_speech();
            }
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
            if self.presentation.tick(frame.dt) {
                if let Some(audio) = &self.audio {
                    audio.portal();
                }
            }
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
            if let Some(audio) = &self.audio {
                audio.stop_speech();
            }
            let (winner, loser) = if self.enemy == 0 {
                (self.hero_id, self.enemy_id)
            } else {
                (self.enemy_id, self.hero_id)
            };
            // One result cue per battle, including a mirror match.
            if !self.speak_role(winner, "victory") {
                self.speak_role(loser, "defeat");
            }
        }
    }
}

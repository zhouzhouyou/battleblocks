use super::*;
impl Session {
    pub(super) fn controls(&mut self, frame: &mut Frame) {
        let old_zh = self.zh;
        frame.ui_consumed = if !frame.was_console_open && !self.console.open {
            self.interface.input(
                frame.pointer,
                self.choosing,
                &mut self.muted,
                &mut self.paused,
                &mut self.zh,
            )
        } else {
            false
        };
        if let Some(audio) = &self.audio {
            audio.muted(self.muted);
            audio.volume(self.interface.volume);
            audio.channel_volumes(self.interface.sound_levels());
        }
        frame.pause_click = false;
        if self.paused
            && !self.choosing
            && !frame.was_console_open
            && !self.console.open
            && !frame.ui_consumed
            && self.interface.panel.is_none()
            && is_mouse_button_pressed(MouseButton::Left)
        {
            for i in 0..4 {
                if ui::pause_rect(i).contains(frame.pointer) {
                    frame.pause_click = true;
                    match i {
                        0 => self.paused = false,
                        1 => {
                            self.choosing = true;
                            self.interface.opponent_step = false;
                            self.paused = false;
                            self.targeting = None;
                            self.selected = None;
                        }
                        2 => frame.restart_command = true,
                        _ => self.interface.panel = Some(ui::Panel::Settings),
                    }
                    break;
                }
            }
        }
        // Buttons on the victory/defeat box, which is drawn only while not paused.
        frame.over_click = false;
        if (self.hp == 0 || self.enemy == 0)
            && !self.paused
            && !self.choosing
            && !frame.pause_click
            && !frame.was_console_open
            && !self.console.open
            && !frame.ui_consumed
            && self.interface.panel.is_none()
            && is_mouse_button_pressed(MouseButton::Left)
        {
            if ui::game_over_rect(0).contains(frame.pointer) {
                frame.over_click = true;
                frame.restart_command = true;
            } else if ui::game_over_rect(1).contains(frame.pointer) {
                frame.over_click = true;
                self.choosing = true;
                self.interface.opponent_step = false;
                self.targeting = None;
                self.selected = None;
            }
        }
        frame.block_input = frame.was_console_open
            || self.console.open
            || frame.ui_consumed
            || frame.pause_click
            || frame.over_click;
        if frame.key(KeyCode::L) {
            self.zh = !self.zh;
        }
        if frame.key(KeyCode::P) {
            self.paused = !self.paused;
        }
        if frame.key(KeyCode::M) {
            self.muted = !self.muted;
            if let Some(audio) = &self.audio {
                audio.muted(self.muted);
                audio.volume(self.interface.volume);
                audio.channel_volumes(self.interface.sound_levels());
            }
        }
        if frame.key(KeyCode::E) {
            self.choosing = true;
            self.interface.opponent_step = false;
        }
        if old_zh != self.zh {
            if let Some(audio) = &self.audio {
                audio.stop_speech();
            }
        }
        self.apply_restart_command(frame);
    }
    pub(super) fn apply_restart_command(&mut self, frame: &Frame) {
        if frame.key(KeyCode::R) || frame.restart_command {
            let before = self.snapshot();
            self.reset_battle();
            self.console
                .event("restart", "Restart battle", &before, &self.snapshot());
        }
    }
}

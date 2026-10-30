use super::*;
mod battlefield;
mod hints;
mod hud;
mod overlays;
mod skills;
impl Session {
    pub(super) fn draw(&mut self, frame: &Frame) {
        self.draw_actors(frame);
        self.draw_turns(frame);
        let (show_moves, player_hover, hover_context) = self.hover_context(frame);
        self.draw_board(frame, show_moves, hover_context);
        self.draw_status_tooltips(frame, show_moves);
        self.draw_effects();
        self.draw_skill_cards(frame, player_hover);
        self.draw_pockets(frame);
        self.draw_passives(frame, show_moves, hover_context);
        self.draw_reroll();
        self.draw_hints(frame, show_moves);
        self.draw_skill_details(frame, hover_context);
        self.draw_overlays(frame);
    }
}

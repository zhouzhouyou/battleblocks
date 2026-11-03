use super::*;
impl Session {
    pub(super) fn reset_battle(&mut self) {
        self.b = fresh();
        self.hp = HEROES[self.hero_id].hp;
        self.enemy_id = self.interface.opponent.unwrap_or_else(|| {
            let ids: Vec<_> = self
                .interface
                .opponent_group
                .roster()
                .iter()
                .copied()
                .filter(|id| *id != self.hero_id)
                .collect();
            ids[rng::gen_range(0, ids.len())]
        });
        self.refresh_actor_textures();
        self.enemy = HEROES[self.enemy_id].hp;
        self.enemy_mp = 0;
        self.enemy_turn = false;
        self.action_done = false;
        self.bonuses = [0, 0];
        self.awakened = [false; 2];
        self.cooldowns = content::Cooldowns::default();
        self.new_role_states = [
            role_state::RoleState::default(),
            role_state::RoleState::default(),
        ];
        self.reward_skill = None;
        self.armed_sides = [false, false];
        self.targeting = None;
        self.presentation.clear();
        self.charge_damage = None;
        self.skill_flash.clear();
        self.last_cast = None;
        self.cast_time = 0.;
        self.reactions = [None, None];
        self.feedback.clear();
        self.feedback_time = 0.;
        self.move_flash = None;
        self.mp = 0;
        self.pending.clear();
        self.selected = None;
        self.enemy_clock = 0.;
        self.paused = false;
        self.chain = 0;
        self.note = "note.select_adjacent_tiles";
        self.armed = false;
        self.extra_bonus = 0;
        self.reward = true;
        self.game_over_logged = false;
        self.nuke_casts = [0; 2];
        self.nuke_damage = [0; 2];
        self.turn_log = turns::Log::new();
        self.round = 1;
        self.console.reset_history();
    }
}

use crate::*;

mod actions;
mod combat;
mod console;
mod controls;
mod exchange;
mod lifecycle;
mod resolution;
mod selection;
mod turn;
mod view;

pub(crate) struct Session {
    defeat: Texture2D,
    zh: bool,
    tiles: [Texture2D; 7],
    hero_id: usize,
    player_texture: Texture2D,
    thumbs: Texture2D,
    book_cover: Texture2D,
    book_pages: Texture2D,
    book_bottom: Texture2D,
    book_levels: [f32; 2],
    preview: menu::Preview,
    character_page: usize,
    enemy_id: usize,
    enemy_texture: Texture2D,
    choosing: bool,
    audio: Option<audio::Audio>,
    b: Board,
    selected: Option<(usize, usize)>,
    move_hint_page: usize,
    hints_hover_open: bool,
    hints_shown: bool,
    hp: i32,
    enemy: i32,
    mp: i32,
    pending: Vec<(usize, usize)>,
    deadline: f64,
    enemy_clock: f64,
    enemy_mp: i32,
    enemy_turn: bool,
    action_done: bool,
    cooldowns: content::Cooldowns,
    new_role_states: [role_state::RoleState; 2],
    difficulty: usize,
    bonuses: [i32; 2],
    armed_sides: [bool; 2],
    feedback: String,
    feedback_time: f64,
    move_flash: Option<Move>,
    targeting: Option<content::Skill>,
    skill_flash: Vec<(usize, usize)>,
    last_cast: Option<content::Skill>,
    last_cast_enemy: bool,
    cast_time: f64,
    paused: bool,
    muted: bool,
    interface: ui::Ui,
    chain: i32,
    note: &'static str,
    reward: bool,
    extra_bonus: i32,
    armed: bool,
    console: debug::Console,
    game_over_logged: bool,
    awakened: [bool; 2],
    reward_skill: Option<content::Skill>,
    awake_textures: Vec<Texture2D>,
    charge_damage: Option<(i32, bool)>,
    presentation: presentation::Presentation,
    turn_log: turns::Log,
}

struct Frame {
    dt: f64,
    cam: Camera2D,
    pointer: Vec2,
    was_console_open: bool,
    command_move: Option<Move>,
    command_skill: Option<content::Skill>,
    command_target: Option<(usize, usize)>,
    restart_command: bool,
    ui_consumed: bool,
    pause_click: bool,
    over_click: bool,
    block_input: bool,
}
impl Frame {
    fn new() -> Self {
        let scale = (screen_width() / 1024.).min(screen_height() / 720.);
        let cam = Camera2D {
            target: vec2(512., 360.),
            zoom: vec2(2. * scale / screen_width(), 2. * scale / screen_height()),
            ..Default::default()
        };
        let (mx, my) = mouse_position();
        let pointer = cam.screen_to_world(vec2(mx, my));
        Self {
            dt: get_frame_time() as f64,
            cam,
            pointer,
            was_console_open: false,
            command_move: None,
            command_skill: None,
            command_target: None,
            restart_command: false,
            ui_consumed: false,
            pause_click: false,
            over_click: false,
            block_input: false,
        }
    }
    fn key(&self, code: KeyCode) -> bool {
        !self.block_input && is_key_pressed(code)
    }
    fn clicked(&self, button: MouseButton) -> bool {
        !self.block_input && is_mouse_button_pressed(button)
    }
}
impl Session {
    pub(crate) fn new() -> Self {
        let defeat = texture(include_bytes!("../../assets/defeat.webp"));
        let zh = true;
        let tiles = [
            texture(include_bytes!("../../assets/balanced.webp")),
            texture(include_bytes!("../../assets/hp.webp")),
            texture(include_bytes!("../../assets/damage.webp")),
            texture(include_bytes!("../../assets/mp.webp")),
            texture(include_bytes!("../../assets/hd.webp")),
            texture(include_bytes!("../../assets/hm.webp")),
            texture(include_bytes!("../../assets/dm.webp")),
        ];
        #[cfg(not(target_arch = "wasm32"))]
        let bench = std::env::var("BATTLEBLOCKS_BENCH_HERO")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&i| i < HEROES.len());
        #[cfg(target_arch = "wasm32")]
        let bench: Option<usize> = None;
        let mut hero_id = bench.unwrap_or(content::KNIGHT);
        #[cfg(not(target_arch = "wasm32"))]
        let startup_commands = std::env::args().any(|arg| arg == "--command");
        #[cfg(target_arch = "wasm32")]
        let startup_commands = false;
        if startup_commands && bench.is_none() {
            hero_id = 0;
        }
        let player_texture = diagnostic_texture(sprites::SPRITES[hero_id].bytes, "no_actors");
        let thumbs = texture(include_bytes!("../../assets/hero-thumbnails.webp"));
        let book_cover = texture(sprites::BOOK_COVER);
        let book_pages = texture(sprites::BOOK_PAGES);
        let book_bottom = texture(sprites::BOOK_BOTTOM);
        let book_levels = [0f32; 2];
        let preview = menu::Preview::new(hero_id, 0);
        let character_page = 0usize;
        let enemy_id = (hero_id + 1) % HEROES.len();
        let enemy_texture = diagnostic_texture(sprites::SPRITES[enemy_id].bytes, "no_actors");
        let choosing = bench.is_none() && !startup_commands;
        let audio = audio::Audio::new(!diagnostic("no_music"), !diagnostic("no_sound"))
            .map_err(|error| eprintln!("Audio unavailable: {error}"))
            .ok();
        rng::srand((get_time() * 1e9) as u64);
        let b = fresh();
        let selected: Option<(usize, usize)> = None;
        let move_hint_page = 0usize;
        let hints_hover_open = false;
        // Whether the move hints were on screen last frame; clicks act on what was shown.
        let hints_shown = false;
        let (hp, enemy, mp) = (HEROES[hero_id].hp, HEROES[enemy_id].hp, 0i32);
        let pending: Vec<(usize, usize)> = Vec::new();
        let deadline = 0.;
        let enemy_clock = 0.;
        let enemy_mp = 0;
        let enemy_turn = false;
        let action_done = false;
        let cooldowns = content::Cooldowns::default();
        let new_role_states = [
            role_state::RoleState::default(),
            role_state::RoleState::default(),
        ];
        let difficulty = 1usize;
        let bonuses = [0, 0];
        let armed_sides = [false, false];
        let feedback = String::new();
        let feedback_time = 0.;
        let move_flash = None;
        let targeting = None;
        let skill_flash = Vec::new();
        let last_cast = None;
        let last_cast_enemy = false;
        let cast_time = 0.;
        let paused = false;
        let muted = false;
        let interface = ui::Ui::default();
        let chain = 0;
        let note = "Select two adjacent tiles";
        let reward = true;
        let extra_bonus = 0;
        let armed = false;
        let console = debug::Console::new(zh);
        let game_over_logged = false;
        let awakened = [false; 2];
        let reward_skill = None;
        let awake_textures: Vec<_> = sprites::AWAKENED
            .iter()
            .map(|s| diagnostic_texture(s.bytes, "no_actors"))
            .collect();
        let charge_damage: Option<(i32, bool)> = None;
        let presentation = presentation::Presentation::default();
        let turn_log = turns::Log::new();

        Self {
            defeat,
            zh,
            tiles,
            hero_id,
            player_texture,
            thumbs,
            book_cover,
            book_pages,
            book_bottom,
            book_levels,
            preview,
            character_page,
            enemy_id,
            enemy_texture,
            choosing,
            audio,
            b,
            selected,
            move_hint_page,
            hints_hover_open,
            hints_shown,
            hp,
            enemy,
            mp,
            pending,
            deadline,
            enemy_clock,
            enemy_mp,
            enemy_turn,
            action_done,
            cooldowns,
            new_role_states,
            difficulty,
            bonuses,
            armed_sides,
            feedback,
            feedback_time,
            move_flash,
            targeting,
            skill_flash,
            last_cast,
            last_cast_enemy,
            cast_time,
            paused,
            muted,
            interface,
            chain,
            note,
            reward,
            extra_bonus,
            armed,
            console,
            game_over_logged,
            awakened,
            reward_skill,
            awake_textures,
            charge_damage,
            presentation,
            turn_log,
        }
    }
    fn rules(&self, side: bool) -> content::Rules {
        content::Rules {
            jumped: self.new_role_states[side as usize].jumped,
            miracle: self.new_role_states[side as usize].miracle,
            horizontal: self.new_role_states[side as usize].horizontal,
            upper_swap: self.new_role_states[side as usize].upper_swap,
            pockets: self.new_role_states[side as usize].pockets,
            pocket_slot: self.new_role_states[side as usize].pocket_slot,
            copied_skill: self.new_role_states[side as usize]
                .copied_skill
                .and_then(content::Skill::from_index),
            awake: self.awakened[side as usize],
            initial_loot: false,
            opponent_hp: Some(if side { self.hp } else { self.enemy }),
            opponent: Some((
                if side { self.hero_id } else { self.enemy_id },
                self.awakened[(!side) as usize],
            )),
        }
    }
    fn refresh_copy(&mut self, side: bool) {
        let id = if side { self.enemy_id } else { self.hero_id };
        if id == content::PRIEST && self.awakened[side as usize] {
            let other = if side { self.hero_id } else { self.enemy_id };
            let options: Vec<_> = content::skills(other, self.awakened[(!side) as usize])
                .iter()
                .copied()
                .map(|s| content::effective_skill(s, self.rules(!side)))
                .filter(|s| *s != content::Skill::Copy)
                .collect();
            self.new_role_states[side as usize].copied_skill = if options.is_empty() {
                None
            } else {
                Some(options[rng::gen_range(0, options.len())].index())
            };
        }
    }
    fn damage(&mut self, amount: i32, side: bool, cause: &str) -> content::DamageResult {
        let old = self.snapshot();
        let result = content::take_damage(
            if side { self.enemy_id } else { self.hero_id },
            if side { &mut self.enemy } else { &mut self.hp },
            &mut self.awakened[side as usize],
            amount,
        );
        if result.awakened {
            self.refresh_copy(side);
            let id = if side { self.enemy_id } else { self.hero_id };
            self.presentation.awakening(
                side,
                if side { old.enemy_hp } else { old.hp },
                content::health_cap(id, false),
                content::health_cap(id, true),
                result.restored,
            );
            self.console.event(
                "awakening",
                &format!(
                    "side={} hero={} damage={} restored={} cap={} cause={}",
                    if side { "enemy" } else { "player" },
                    id,
                    result.damage,
                    result.restored,
                    content::health_cap(id, true),
                    cause
                ),
                &old,
                &self.snapshot(),
            );
        }
        result
    }
    fn apply_rewards(&mut self, gains: (i32, i32, i32)) -> content::DamageResult {
        let side = self.enemy_turn;
        let (id, hp, mp) = if side {
            (self.enemy_id, self.enemy, self.enemy_mp)
        } else {
            (self.hero_id, self.hp, self.mp)
        };
        let (hp, mp, damage) = content::settle_rewards(id, hp, mp, gains, self.rules(side));
        if side {
            self.enemy = hp;
            self.enemy_mp = mp;
        } else {
            self.hp = hp;
            self.mp = mp;
        }
        self.damage(damage, !side, "damage")
    }
    fn snapshot(&self) -> debug::Snapshot {
        debug::Snapshot {
            cooldowns: self.cooldowns.clone(),
            new_role_states: self.new_role_states.clone(),
            awakened: self.awakened,
            reward_skill: self.reward_skill,
            board: self.b,
            hero: self.hero_id,
            enemy: self.enemy_id,
            hp: self.hp,
            enemy_hp: self.enemy,
            mp: self.mp,
            enemy_mp: self.enemy_mp,
            enemy_turn: self.enemy_turn,
            action_done: self.action_done,
            pending: self.pending.clone(),
            deadline: self.deadline,
            enemy_clock: self.enemy_clock,
            bonuses: self.bonuses,
            armed_sides: self.armed_sides,
            bonus: self.extra_bonus,
            armed: self.armed,
            chain: self.chain,
            reward: self.reward,
            difficulty: self.difficulty,
            paused: self.paused,
            rng_state: rng::state(),
            charge_damage: self.charge_damage,
            charge_remaining: self.presentation.charge_remaining(),
            turns: self.turn_log.clone(),
        }
    }
    fn restore(&mut self, state: debug::Snapshot) {
        self.awakened = state.awakened;
        self.cooldowns = state.cooldowns;
        self.new_role_states = state.new_role_states;
        self.book_levels = [0.; 2];
        self.reward_skill = state.reward_skill;
        self.b = state.board;
        self.hero_id = state.hero;
        self.enemy_id = state.enemy;
        self.hp = state.hp;
        self.enemy = state.enemy_hp;
        self.mp = state.mp;
        self.enemy_mp = state.enemy_mp;
        self.enemy_turn = state.enemy_turn;
        self.action_done = state.action_done;
        self.pending = state.pending;
        self.deadline = state.deadline;
        self.enemy_clock = state.enemy_clock;
        self.bonuses = state.bonuses;
        self.armed_sides = state.armed_sides;
        self.extra_bonus = state.bonus;
        self.armed = state.armed;
        self.chain = state.chain;
        rng::restore(state.rng_state);
        self.reward = state.reward;
        self.difficulty = state.difficulty;
        self.paused = state.paused;
        self.selected = None;
        self.targeting = None;
        self.presentation.clear();
        self.charge_damage = state.charge_damage;
        self.turn_log = state.turns;
        self.last_cast_enemy = self.charge_damage.map(|(_, enemy)| enemy).unwrap_or(false);
        if self.charge_damage.is_some() {
            let column = self.pending.first().map(|c| c.0).unwrap_or(0);
            self.presentation
                .charge(column, state.charge_remaining.max(0.001));
        }
        self.skill_flash.clear();
        self.last_cast = None;
        self.cast_time = 0.;
        self.feedback.clear();
        self.feedback_time = 0.;
        self.move_flash = None;
        self.player_texture = diagnostic_texture(sprites::SPRITES[self.hero_id].bytes, "no_actors");
        self.enemy_texture = diagnostic_texture(sprites::SPRITES[self.enemy_id].bytes, "no_actors");
        self.preview = menu::Preview::new(self.hero_id, 0);
        self.choosing = false;
        self.game_over_logged = false;
    }
    pub(crate) fn start(&mut self) {
        self.console.event(
            "session_start",
            "New session",
            &self.snapshot(),
            &self.snapshot(),
        );
    }
    pub(crate) fn frame(&mut self) {
        let mut frame = Frame::new();
        if !self.console_input(&mut frame) {
            return;
        }
        self.controls(&mut frame);
        if self.menu(&frame) {
            return;
        }
        self.tick(&mut frame);
        self.draw(&frame);
    }
}

use crate::*;

mod actions;
mod combat;
mod console;
mod controls;
mod exchange;
mod headless;
mod hints;
mod planning;
mod projectile;
pub(crate) use headless::BatchOutcome;
mod lifecycle;
mod resolution;
mod selection;
mod turn;
mod view;

pub(crate) struct Session {
    rendering: bool,
    defeat: Option<Texture2D>,
    zh: bool,
    tiles: Option<[Texture2D; 7]>,
    skill_effects: Option<effects::Textures>,
    hero_id: usize,
    player_texture: Option<Texture2D>,
    /// Transparent stand-in until the web build fetches an actor atlas.
    blank: Option<Texture2D>,
    thumbs: Option<Texture2D>,
    book_cover: Option<Texture2D>,
    book_pages: Option<Texture2D>,
    book_bottom: Option<Texture2D>,
    book_levels: [f32; 2],
    preview: menu::Preview,
    character_page: usize,
    enemy_id: usize,
    enemy_texture: Option<Texture2D>,
    choosing: bool,
    audio: Option<audio::Audio>,
    b: Board,
    board_mechanics: board::BoardMechanics,
    target_anchor: Option<(usize, usize)>,
    selected: Option<(usize, usize)>,
    move_hint_page: usize,
    hints_hover_open: bool,
    hints_shown: bool,
    hint_cache: Option<hints::HintCache>,
    advanced_hints: bool,
    plan_cache: Option<planning::AiPlanCache>,
    hint_queue: std::collections::VecDeque<planning::PlanStep>,
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
    reactions: [Option<(String, f64)>; 2],
    move_flash: Option<Move>,
    targeting: Option<content::Skill>,
    projectile_aim: Option<content::projectile::Aim>,
    projectile_drag: bool,
    projectile_preview: Option<projectile::PreviewCache>,
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
    awake_textures: [Option<Texture2D>; 2],
    charge_damage: Option<content::PendingImpact>,
    presentation: presentation::Presentation,
    turn_log: turns::Log,
    round: u32,
    /// Analysis-only counter for balance screens; not part of snapshots or play.
    nuke_casts: [u32; 2],
    nuke_damage: [i64; 2],
}

/// Result of one reward settlement, including the latest health-affecting source per side.
struct AppliedRewards {
    hit: content::DamageResult,
    actor_health_source: Option<content::EventSource>,
    opponent_health_source: Option<content::EventSource>,
}

struct Frame {
    dt: f64,
    interactive: bool,
    cam: Camera2D,
    pointer: Vec2,
    was_console_open: bool,
    command_move: Option<Move>,
    command_skill: Option<content::Skill>,
    command_target: Option<(usize, usize)>,
    command_aim: Option<content::projectile::Aim>,
    restart_command: bool,
    ui_consumed: bool,
    pause_click: bool,
    over_click: bool,
    block_input: bool,
}
impl Frame {
    fn new() -> Self {
        let scale = (screen_width() / ui::CANVAS.w).min(screen_height() / ui::CANVAS.h);
        let cam = Camera2D {
            target: vec2(512., 360.),
            zoom: vec2(2. * scale / screen_width(), 2. * scale / screen_height()),
            ..Default::default()
        };
        let (mx, my) = mouse_position();
        let pointer = cam.screen_to_world(vec2(mx, my));
        Self {
            dt: get_frame_time() as f64,
            interactive: true,
            cam,
            pointer,
            was_console_open: false,
            command_move: None,
            command_skill: None,
            command_target: None,
            command_aim: None,
            restart_command: false,
            ui_consumed: false,
            pause_click: false,
            over_click: false,
            block_input: false,
        }
    }
    fn headless(dt: f64) -> Self {
        Self {
            dt,
            interactive: false,
            cam: Camera2D {
                target: vec2(512., 360.),
                zoom: vec2(2. / ui::CANVAS.w, 2. / ui::CANVAS.h),
                ..Default::default()
            },
            pointer: vec2(-1., -1.),
            was_console_open: false,
            command_move: None,
            command_skill: None,
            command_target: None,
            command_aim: None,
            restart_command: false,
            ui_consumed: false,
            pause_click: false,
            over_click: false,
            block_input: false,
        }
    }
    fn key(&self, code: KeyCode) -> bool {
        self.interactive && !self.block_input && is_key_pressed(code)
    }
    fn clicked(&self, button: MouseButton) -> bool {
        self.interactive && !self.block_input && is_mouse_button_pressed(button)
    }
}
impl Session {
    pub(crate) fn new() -> Self {
        Self::create(true, false)
    }
    /// Build the canonical headless game model without touching a graphics or audio context.
    pub(crate) fn new_headless() -> Self {
        Self::create(false, false)
    }
    fn new_simulation() -> Self {
        Self::create(false, true)
    }
    fn create(rendering: bool, simulation: bool) -> Self {
        let zh = std::env::var("BATTLEBLOCKS_LANG")
            .map(|value| value != "en")
            .unwrap_or(true);
        let defeat = rendering.then(|| texture(include_bytes!("../../assets/ui/defeat.webp")));
        let skill_effects = rendering.then(effects::Textures::load);
        let tiles = rendering.then(|| {
            [
                texture(include_bytes!("../../assets/ui/balanced.webp")),
                texture(include_bytes!("../../assets/ui/hp.webp")),
                texture(include_bytes!("../../assets/ui/damage.webp")),
                texture(include_bytes!("../../assets/ui/mp.webp")),
                texture(include_bytes!("../../assets/ui/hd.webp")),
                texture(include_bytes!("../../assets/ui/hm.webp")),
                texture(include_bytes!("../../assets/ui/dm.webp")),
            ]
        });
        #[cfg(not(target_arch = "wasm32"))]
        let bench = std::env::var("BATTLEBLOCKS_BENCH_HERO")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&i| i < HEROES.len());
        #[cfg(target_arch = "wasm32")]
        let bench: Option<usize> = None;
        let menu_hero = std::env::var("BATTLEBLOCKS_MENU_HERO")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|&id| id < HEROES.len());
        let mut hero_id = bench.or(menu_hero).unwrap_or(content::KNIGHT);
        #[cfg(not(target_arch = "wasm32"))]
        let startup_commands = std::env::args().any(|arg| arg == "--command");
        #[cfg(target_arch = "wasm32")]
        let startup_commands = false;
        if startup_commands && bench.is_none() {
            hero_id = 0;
        }
        // The selection menu needs only its own hero: the opponent and awakening
        // atlases are battle assets and load when the battle starts.
        let choosing = rendering && bench.is_none() && !startup_commands;
        let blank = rendering.then(blank_texture);
        let player_texture = rendering
            .then(|| asset_texture(&sprites::SPRITES[hero_id].asset, "no_actors"))
            .flatten();
        let thumbs =
            rendering.then(|| texture(include_bytes!("../../assets/ui/hero-thumbnails.webp")));
        let book_cover = rendering.then(|| texture(sprites::BOOK_COVER));
        let book_pages = rendering.then(|| texture(sprites::BOOK_PAGES));
        let book_bottom = rendering.then(|| texture(sprites::BOOK_BOTTOM));
        let book_levels = [0f32; 2];
        let character_page = std::env::var("BATTLEBLOCKS_MENU_PAGE")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0)
            .min(menu::CHARACTER_PAGE_COUNT - 1);
        let preview = if rendering {
            menu::Preview::new(hero_id, 0, character_page == 2)
        } else {
            menu::Preview::new_static(hero_id, 0, character_page == 2)
        };
        let enemy_id = (hero_id + 1) % HEROES.len();
        let enemy_texture = (rendering && !choosing)
            .then(|| asset_texture(&sprites::SPRITES[enemy_id].asset, "no_actors"))
            .flatten();
        let audio = rendering
            .then(|| {
                audio::Audio::new(!diagnostic("no_music"), !diagnostic("no_sound"))
                    .map_err(|error| eprintln!("Audio unavailable: {error}"))
                    .ok()
            })
            .flatten();
        let headless_seed = std::env::var("BATTLEBLOCKS_HEADLESS_SEED")
            .ok()
            .and_then(|seed| seed.parse().ok())
            .unwrap_or(1);
        rng::srand(if rendering {
            (get_time() * 1e9) as u64
        } else {
            headless_seed
        });
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
        let note = "note.select_adjacent_tiles";
        let reward = true;
        let extra_bonus = 0;
        let armed = false;
        let console = if simulation {
            debug::Console::detached(zh)
        } else {
            debug::Console::new(zh)
        };
        let game_over_logged = false;
        let awakened = [false; 2];
        let reward_skill = None;
        let awake_textures = if rendering && !choosing {
            [
                player_texture
                    .as_ref()
                    .and_then(|t| Self::load_awake_texture(hero_id, t)),
                enemy_texture
                    .as_ref()
                    .and_then(|t| Self::load_awake_texture(enemy_id, t)),
            ]
        } else {
            [None, None]
        };
        let charge_damage: Option<content::PendingImpact> = None;
        let presentation = presentation::Presentation::default();
        let turn_log = turns::Log::new();
        let nuke_casts = [0; 2];
        let nuke_damage = [0; 2];
        let round = 1u32;

        let mut session = Self {
            rendering,
            defeat,
            zh,
            tiles,
            skill_effects,
            hero_id,
            player_texture,
            blank,
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
            board_mechanics: Default::default(),
            target_anchor: None,
            selected,
            move_hint_page,
            hints_hover_open,
            hints_shown,
            hint_cache: None,
            advanced_hints: false,
            plan_cache: None,
            hint_queue: Default::default(),
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
            reactions: [None, None],
            move_flash,
            targeting,
            projectile_aim: None,
            projectile_drag: false,
            projectile_preview: None,
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
            round,
            nuke_casts,
            nuke_damage,
        };
        if rendering && !choosing {
            session.prefetch_battle_effects();
        }
        session
    }
    fn required_texture(texture: &Option<Texture2D>) -> &Texture2D {
        texture
            .as_ref()
            .expect("render-only texture used in headless mode")
    }
    fn tiles(&self) -> &[Texture2D; 7] {
        self.tiles
            .as_ref()
            .expect("render-only texture used in headless mode")
    }
    /// A transparent stand-in stands in while the web build fetches the atlas.
    fn blank(&self) -> &Texture2D {
        Self::required_texture(&self.blank)
    }
    fn player_texture(&self) -> &Texture2D {
        self.player_texture.as_ref().unwrap_or_else(|| self.blank())
    }
    /// Selected hero atlas, or `None` while the web build is still fetching it.
    fn player_sprite(&self) -> Option<&Texture2D> {
        self.player_texture.as_ref()
    }
    /// Whether the battle actor of this side can be drawn from loaded artwork.
    fn battle_sprite_ready(&self, enemy: bool, awakened: bool) -> bool {
        if awakened {
            self.awake_textures[enemy as usize].is_some()
        } else if enemy {
            self.enemy_texture.is_some()
        } else {
            self.player_texture.is_some()
        }
    }
    fn enemy_texture(&self) -> &Texture2D {
        self.enemy_texture.as_ref().unwrap_or_else(|| self.blank())
    }
    fn awake_texture(&self, enemy: bool) -> &Texture2D {
        self.awake_textures[enemy as usize].as_ref().unwrap_or_else(|| {
            if enemy {
                self.enemy_texture()
            } else {
                self.player_texture()
            }
        })
    }
    fn load_awake_texture(id: usize, normal: &Texture2D) -> Option<Texture2D> {
        if sprites::AWAKENED[id].asset.path == sprites::SPRITES[id].asset.path {
            Some(normal.clone())
        } else {
            asset_texture(&sprites::AWAKENED[id].asset, "no_actors")
        }
    }
    fn thumbs(&self) -> &Texture2D {
        Self::required_texture(&self.thumbs)
    }
    fn book_cover(&self) -> &Texture2D {
        Self::required_texture(&self.book_cover)
    }
    fn book_pages(&self) -> &Texture2D {
        Self::required_texture(&self.book_pages)
    }
    fn book_bottom(&self) -> &Texture2D {
        Self::required_texture(&self.book_bottom)
    }
    fn defeat(&self) -> &Texture2D {
        Self::required_texture(&self.defeat)
    }
    fn refresh_actor_textures(&mut self) {
        if !self.rendering {
            return;
        }
        self.player_texture = asset_texture(&sprites::SPRITES[self.hero_id].asset, "no_actors");
        if self.choosing {
            // The menu draws one hero; opponent and awakening sheets are battle
            // resources and stay out of memory until the battle starts.
            self.enemy_texture = None;
            self.awake_textures = [None, None];
            return;
        }
        self.enemy_texture = if self.hero_id == self.enemy_id {
            self.player_texture.clone()
        } else {
            asset_texture(&sprites::SPRITES[self.enemy_id].asset, "no_actors")
        };
        let player = Self::load_awake_texture(self.hero_id, self.player_texture());
        let enemy = if self.hero_id == self.enemy_id {
            player.clone()
        } else {
            Self::load_awake_texture(self.enemy_id, self.enemy_texture())
        };
        self.awake_textures = [player, enemy];
    }

    /// The menu only needs the selected hero's awakening sheet on the awakening
    /// page or when a preview turns the hero dark. Battle loads them eagerly.
    fn ensure_player_awake_texture(&mut self) {
        if !self.rendering || self.awake_textures[0].is_some() {
            return;
        }
        self.awake_textures[0] = Self::load_awake_texture(self.hero_id, self.player_texture());
    }

    /// Whether the menu shows the awakening sheet right now.
    fn wants_awake_texture(&self) -> bool {
        self.character_page == 2 || self.preview.dark || self.preview.base_dark
    }

    /// The web loader finished a file: upload anything that was blank so far.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) fn refresh_textures(&mut self) {
        if !self.rendering {
            return;
        }
        if self.player_texture.is_none() {
            self.player_texture = asset_texture(&sprites::SPRITES[self.hero_id].asset, "no_actors");
        }
        if self.choosing {
            if self.wants_awake_texture() && self.awake_textures[0].is_none() {
                self.awake_textures[0] =
                    Self::load_awake_texture(self.hero_id, self.player_texture());
            }
            return;
        }
        if self.enemy_texture.is_none() {
            self.enemy_texture =
                asset_texture(&sprites::SPRITES[self.enemy_id].asset, "no_actors");
        }
        if self.awake_textures[0].is_none() {
            self.awake_textures[0] = Self::load_awake_texture(self.hero_id, self.player_texture());
        }
        if self.awake_textures[1].is_none() {
            self.awake_textures[1] = Self::load_awake_texture(self.enemy_id, self.enemy_texture());
        }
        self.prefetch_battle_effects();
    }

    /// Return to hero selection and drop the artwork only a battle draws.
    fn enter_selection(&mut self) {
        self.choosing = true;
        self.interface.opponent_step = false;
        self.targeting = None;
        self.selected = None;
        self.enemy_texture = None;
        self.awake_textures = [None, None];
        if let Some(effects) = &mut self.skill_effects {
            effects.release();
        }
    }

    /// A battle frame needs both actors and its matchup sheets. Console commands
    /// can leave the menu without `reset_battle`, so repair that here.
    fn ensure_battle_resources(&mut self) {
        if !self.rendering || self.choosing {
            return;
        }
        if self.enemy_texture.is_none() || self.awake_textures.iter().any(Option::is_none) {
            self.refresh_actor_textures();
            self.prefetch_battle_effects();
        }
    }

    /// Upload this matchup's effect sheets before the first cast, so a cast does
    /// not stall on decoding mid-animation. Skills that arrive by copying or debug
    /// commands outside the two loadouts stay lazy and load on first use.
    fn prefetch_battle_effects(&mut self) {
        let Some(effects) = &self.skill_effects else {
            return;
        };
        let mut skills = Vec::new();
        for id in [self.hero_id, self.enemy_id] {
            for awake in [false, true] {
                skills.extend_from_slice(content::skills(id, awake));
            }
        }
        effects.prefetch(skills);
    }
    fn refresh_preview(&mut self, skill: usize) {
        let awakened = self.character_page == 2;
        self.preview = if self.rendering {
            menu::Preview::new(self.hero_id, skill, awakened)
        } else {
            menu::Preview::new_static(self.hero_id, skill, awakened)
        };
    }
    fn rules(&self, side: bool) -> content::Rules {
        content::Rules {
            projectile_aim: if side { None } else { self.projectile_aim },
            board_mechanics: self.board_mechanics,
            actor_enemy: side,
            target_anchor: if side { None } else { self.target_anchor },
            horizontal: self.new_role_states[side as usize].horizontal,
            swap_clear: self.new_role_states[side as usize].swap_clear,
            pockets: self.new_role_states[side as usize].pockets,
            pocket_slot: self.new_role_states[side as usize].pocket_slot,
            copied_skill: self.new_role_states[side as usize]
                .copied_skill
                .and_then(content::Skill::from_index),
            statuses: self.new_role_states[side as usize].statuses,
            repeat_casts: self.new_role_states[side as usize].repeat_casts,
            awake: self.awakened[side as usize],
            initial_loot: false,
            opponent_hp: Some(if side { self.hp } else { self.enemy }),
            opponent_mp: if side { self.mp } else { self.enemy_mp },
            opponent_statuses: self.new_role_states[(!side) as usize].statuses,
            opponent: Some((
                if side { self.hero_id } else { self.enemy_id },
                self.awakened[(!side) as usize],
            )),
            round: self.round,
        }
    }
    fn rejection_message(&self, side: bool) -> &'static str {
        let id = if side { self.enemy_id } else { self.hero_id };
        content::ability_capabilities(id, self.rules(side))
            .rejection
            .map(|key| content::role_text(key, self.zh))
            .unwrap_or("")
    }
    fn refresh_copy(&mut self, side: bool) {
        let id = if side { self.enemy_id } else { self.hero_id };
        let other = if side { self.hero_id } else { self.enemy_id };
        let options: Vec<_> = content::skills(other, self.awakened[(!side) as usize])
            .iter()
            .copied()
            .map(|s| content::effective_skill(s, self.rules(!side)))
            .filter(|s| !content::skill_profile(*s, self.rules(!side)).copy_slot)
            .collect();
        let mut commands = Vec::new();
        let mut triggers = Vec::new();
        content::dispatch_rules(
            &content::EventContext::new(id, 0, 0, self.rules(side)),
            &mut content::RuleEvent::RefreshAbilities {
                options: &options,
                commands: &mut commands,
            },
            &mut triggers,
        );
        self.apply_state_commands(side, commands);
        self.present_ability_triggers(side, &triggers);
    }
    fn apply_state_commands(&mut self, side: bool, commands: Vec<content::StateCommand>) {
        for command in commands {
            match command {
                content::StateCommand::FillPocket { slot, options } => {
                    if !options.is_empty() {
                        let pick = rng::gen_range(0, options.len());
                        self.new_role_states[side as usize].pockets[slot] = Some(options[pick]);
                        self.new_role_states[side as usize].select_occupied_pocket();
                    }
                }
                content::StateCommand::CopySkill { options } => {
                    self.new_role_states[side as usize].copied_skill = if options.is_empty() {
                        None
                    } else {
                        Some(options[rng::gen_range(0, options.len())].index())
                    }
                }
            }
        }
    }
    fn apply_health_cost(
        &mut self,
        amount: i32,
        side: bool,
        skill: content::Skill,
    ) -> content::DamageResult {
        let id = if side { self.enemy_id } else { self.hero_id };
        let hp = if side { self.enemy } else { self.hp };
        let mp = if side { self.enemy_mp } else { self.mp };
        let mut event =
            content::RewardBatch::gains((0, amount, 0), 0, content::EventSource::Cost(skill))
                .events
                .into_iter()
                .next();
        if let Some(reward) = event.as_mut() {
            reward.recipient = content::EventSide::Actor;
            content::dispatch_rules(
                &content::EventContext::new(id, hp, mp, self.rules(side)),
                &mut content::RuleEvent::Incoming { reward },
                &mut Vec::new(),
            );
        }
        self.damage(event.map(|e| e.amount).unwrap_or(0), side, "blood_payment")
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
    fn apply_batch_rewards(&mut self, batch: content::RewardBatch) -> AppliedRewards {
        self.apply_batch_rewards_for(batch, self.enemy_turn)
    }
    fn apply_batch_rewards_for(
        &mut self,
        mut batch: content::RewardBatch,
        side: bool,
    ) -> AppliedRewards {
        for reward in &mut batch.events {
            if reward.probability < 100 && rng::gen_range(0, 100) >= reward.probability as i32 {
                reward.amount = 0;
            }
            reward.probability = 100;
        }
        let before_rewards = self.snapshot();
        let (id, hp, mp) = if side {
            (self.enemy_id, self.enemy, self.enemy_mp)
        } else {
            (self.hero_id, self.hp, self.mp)
        };
        let opponent_hp = if side { self.hp } else { self.enemy };
        let settled = content::settle_batch_in(id, hp, mp, opponent_hp, batch, self.rules(side));
        let actor_health_source =
            settled.source_for(content::Resource::Healing, content::EventSide::Actor);
        let opponent_health_source = settled
            .source_for(content::Resource::Damage, content::EventSide::Opponent)
            .or_else(|| {
                settled.source_for(content::Resource::Healing, content::EventSide::Opponent)
            });
        if let Some(effects) = &mut self.skill_effects {
            effects.healing_rewards(&settled.events, side);
        }
        self.present_ability_triggers(side, &settled.triggers);
        if side {
            self.enemy = settled.actor_hp;
            self.enemy_mp = settled.actor_mp;
            self.hp = settled.opponent_hp;
        } else {
            self.hp = settled.actor_hp;
            self.mp = settled.actor_mp;
            self.enemy = settled.opponent_hp;
        }
        let hit = self.damage(settled.damage, !side, "damage");
        for event in &settled.events {
            // Reward Actor/Opponent are relative to this batch's owner, which
            // can differ from the current turn when an off-turn passive fires.
            let event_json = event.json();
            let payload = format!("{{\"actor_enemy\":{side},{}", &event_json[1..]);
            self.console.event_payload(
                "reward_event",
                &format!("{:?}", event),
                Some(&payload),
                &before_rewards,
                &self.snapshot(),
            );
        }
        AppliedRewards {
            hit,
            actor_health_source,
            opponent_health_source,
        }
    }
    fn present_ability_triggers(&mut self, side: bool, triggers: &[content::AbilityTrigger]) {
        for trigger in triggers {
            let target_side = if trigger.side == content::EventSide::Actor {
                side
            } else {
                !side
            };
            let text = trigger
                .message
                .map(|key| content::role_text(key, self.zh).to_string())
                .unwrap_or_else(|| trigger.ability.name(self.zh).to_string());
            self.reactions[target_side as usize] = Some((text, 2.));
            self.console.event(
                "ability_trigger",
                &format!(
                    "side={} ability={:?} message={:?}",
                    target_side, trigger.ability, trigger.message
                ),
                &self.snapshot(),
                &self.snapshot(),
            );
        }
    }
    fn snapshot(&self) -> debug::Snapshot {
        debug::Snapshot {
            cooldowns: self.cooldowns.clone(),
            new_role_states: self.new_role_states.clone(),
            awakened: self.awakened,
            reward_skill: self.reward_skill,
            board: self.b,
            board_mechanics: self.board_mechanics,
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
            charge_damage: self.charge_damage.clone(),
            charge_remaining: if self
                .charge_damage
                .as_ref()
                .is_some_and(|p| p.projectile.is_some())
            {
                self.presentation.remaining()
            } else {
                self.presentation.charge_remaining()
            },
            turns: self.turn_log.clone(),
            round: self.round,
        }
    }
    fn restore(&mut self, state: debug::Snapshot) {
        if let Some(audio) = &self.audio {
            audio.stop_speech();
        }
        self.projectile_aim = None;
        self.projectile_drag = false;
        self.projectile_preview = None;
        self.hint_cache = None;
        self.plan_cache = None;
        self.hint_queue.clear();
        self.awakened = state.awakened;
        self.cooldowns = state.cooldowns;
        self.new_role_states = state.new_role_states;
        self.book_levels = [0.; 2];
        self.reward_skill = state.reward_skill;
        self.b = state.board;
        self.board_mechanics = state.board_mechanics;
        self.target_anchor = None;
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
        self.round = state.round;
        self.last_cast_enemy = self
            .charge_damage
            .as_ref()
            .map(|impact| impact.actor_enemy)
            .unwrap_or(false);
        if self
            .charge_damage
            .as_ref()
            .is_some_and(|p| p.projectile.is_some())
        {
            self.presentation.hold(state.charge_remaining.max(0.001));
        } else if self.charge_damage.is_some() {
            let column = self.pending.first().map(|c| c.0).unwrap_or(0);
            self.presentation
                .charge(column, state.charge_remaining.max(0.001));
        }
        self.skill_flash.clear();
        if let Some(effects) = &mut self.skill_effects {
            effects.clear();
            if let Some(impact) = &self.charge_damage {
                let id = if impact.actor_enemy {
                    self.enemy_id
                } else {
                    self.hero_id
                };
                if self.awakened[impact.actor_enemy as usize] {
                    effects.resume_actor(
                        sprites::AWAKENED[id].actor_cast(impact.skill),
                        impact.skill,
                        impact.actor_enemy,
                        self.console.gameplay_action(),
                        &self.pending,
                        (presentation::CHARGE_TIME - state.charge_remaining)
                            .clamp(0., presentation::CHARGE_TIME),
                    );
                }
            }
        }
        self.last_cast = None;
        self.cast_time = 0.;
        self.reactions = [None, None];
        self.feedback.clear();
        self.feedback_time = 0.;
        self.move_flash = None;
        self.refresh_actor_textures();
        self.refresh_preview(0);
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
        self.ensure_battle_resources();
        self.tick(&mut frame);
        self.draw(&frame);
    }
}
#[cfg(test)]
mod portal_tests;

#[cfg(test)]
mod projectile_tests;

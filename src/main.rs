mod board;
#[cfg(test)]
use board::drop_board;
mod content;
#[cfg(test)]
use board::valid_swap;
use board::{fresh, matches, parallel_swap, swap, Board, N};
mod debug;
mod effects;
mod icons;
mod menu;
mod presentation;
mod ui;
use presentation::{CLEAR_TIME, TURN_HOLD};
mod rng;
mod strategy;
use content::{effect, effect_at, HEROES, VALUES};
struct SpriteAsset {
    bytes: &'static [u8],
    w: f32,
    h: f32,
    durations: &'static [f64],
    indices: &'static [usize],
}
mod audio;
mod sprites;
use macroquad::prelude::*;
use std::cell::RefCell;
fn wrap_preview_text(text: &str, limit: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source in text.lines() {
        let mut line = String::new();
        let mut width = 0.;
        for c in source.chars() {
            let w = if c.is_ascii() { 0.55 } else { 1. };
            if width + w > limit && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                width = 0.;
            }
            line.push(c);
            width += w;
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}
fn reroll_rect() -> Rect {
    Rect::new(256., 650., 512., 38.)
}
fn cancel_skill_rect() -> Rect {
    Rect::new(805., 642., 195., 40.)
}
fn toggle_skill_target(
    current: Option<content::Skill>,
    skill: content::Skill,
) -> Option<content::Skill> {
    if current == Some(skill) {
        None
    } else {
        Some(skill)
    }
}
fn diagnostic(flag: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static FLAGS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        FLAGS
            .get_or_init(|| std::env::var("BATTLEBLOCKS_MEMORY_FLAGS").unwrap_or_default())
            .split(',')
            .any(|s| s == flag)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = flag;
        false
    }
}
fn diagnostic_texture(bytes: &[u8], flag: &str) -> Texture2D {
    if diagnostic(flag) {
        Texture2D::empty()
    } else {
        texture(bytes)
    }
}
fn procedural_board() {
    let dark = Color::from_rgba(43, 37, 33, 255);
    let mid = Color::from_rgba(172, 145, 126, 255);
    let light = Color::from_rgba(214, 193, 167, 255);
    draw_rectangle(236., 90., 552., 552., dark);
    draw_rectangle(241., 95., 542., 542., mid);
    draw_rectangle_lines(241., 95., 542., 542., 3., light);
    for i in 0..18 {
        let p = i as f32 * 30.;
        draw_line(241. + p, 95., 241. + p, 108., 3., dark);
        draw_line(241. + p, 624., 241. + p, 637., 3., dark);
        draw_line(241., 95. + p, 254., 95. + p, 3., dark);
        draw_line(770., 95. + p, 783., 95. + p, 3., dark);
    }
}
fn procedural_cell(x: f32, y: f32) {
    draw_rectangle(x, y, 64., 64., Color::from_rgba(172, 127, 115, 255));
    draw_rectangle_lines(x, y, 64., 64., 3., Color::from_rgba(43, 37, 33, 255));
    draw_rectangle_lines(
        x + 4.,
        y + 4.,
        56.,
        56.,
        2.,
        Color::from_rgba(214, 193, 167, 255),
    );
}
fn conf() -> macroquad::conf::Conf {
    let window = Conf {
        window_title: "BattleBlocks".into(),
        window_width: if diagnostic("small_window") {
            512
        } else {
            1024
        },
        window_height: if diagnostic("small_window") { 360 } else { 720 },
        high_dpi: diagnostic("hidpi"),
        ..Default::default()
    };
    macroquad::conf::Conf {
        miniquad_conf: window,
        draw_call_vertex_capacity: if diagnostic("large_buffers") {
            10000
        } else {
            512
        },
        draw_call_index_capacity: if diagnostic("large_buffers") {
            5000
        } else {
            768
        },
        ..Default::default()
    }
}
fn texture(bytes: &[u8]) -> Texture2D {
    let t = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    t.set_filter(FilterMode::Nearest);
    t
}
struct LabelJob {
    text: String,
    x: f32,
    y: f32,
    font_size: u16,
    font_scale: f32,
    font_scale_aspect: f32,
    color: Color,
}
struct UiFont {
    font: Font,
    cached: std::collections::HashSet<(char, u16)>,
    scratch: Vec<char>,
    jobs: Vec<LabelJob>,
    pending: usize,
}
thread_local! { static UI_FONT: RefCell<Option<UiFont>> = const { RefCell::new(None) }; }
fn tr(zh: bool, en: &'static str, cn: &'static str) -> &'static str {
    if zh {
        cn
    } else {
        en
    }
}
fn localized_note(zh: bool, note: &str) -> &str {
    if !zh {
        return note;
    }
    match note {
        "Skill cast" => "技能已释放",
        "Not enough mana" => "魔力不足",
        "Blade slash: 100 damage" => "剑刃斩击：造成100伤害",
        "No moves: board refreshed" => "无可用交换，棋盘已刷新",
        "Match!" => "消除成功！",
        "No match: choose another tile" => "未形成三消，请重新选择",
        _ => "选择两个相邻方块",
    }
}
fn font_raster_bucket(requested: u16) -> u16 {
    [16u16, 24, 32, 48, 64]
        .into_iter()
        .find(|&size| size >= requested)
        .unwrap_or(64)
}
fn label(text: &str, x: f32, y: f32, size: f32, color: Color) {
    if diagnostic("no_text") {
        return;
    }
    let (requested, font_scale, font_scale_aspect) = camera_font_scale(size);
    let font_size = font_raster_bucket(requested);
    let font_scale = font_scale * requested.max(1) as f32 / font_size as f32;
    UI_FONT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(ui) = slot.as_mut() {
            let n = ui.pending;
            ui.pending += 1;
            if n == ui.jobs.len() {
                ui.jobs.push(LabelJob {
                    text: String::new(),
                    x,
                    y,
                    font_size,
                    font_scale,
                    font_scale_aspect,
                    color,
                });
            }
            let job = &mut ui.jobs[n];
            job.text.clear();
            job.text.push_str(text);
            job.x = x;
            job.y = y;
            job.font_size = font_size;
            job.font_scale = font_scale;
            job.font_scale_aspect = font_scale_aspect;
            job.color = color;
        }
    });
}
fn flush_labels() {
    UI_FONT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(ui) = slot.as_mut() {
            let UiFont {
                font,
                cached,
                scratch,
                jobs,
                pending,
            } = ui;
            // Populate the whole frame before rendering any font texture.
            // Atlas growth must not invalidate text batches already queued this frame.
            for job in jobs.iter().take(*pending) {
                let raster_size =
                    (job.font_size as f32 * miniquad::window::dpi_scale()).ceil() as u16;
                scratch.clear();
                for c in job.text.chars() {
                    if cached.insert((c, raster_size)) {
                        scratch.push(c);
                    }
                }
                if !scratch.is_empty() {
                    font.populate_font_cache(scratch, raster_size);
                }
            }
            for job in jobs.iter().take(*pending) {
                draw_text_ex(
                    &job.text,
                    job.x,
                    job.y,
                    TextParams {
                        font: Some(font),
                        font_size: job.font_size,
                        font_scale: job.font_scale,
                        font_scale_aspect: job.font_scale_aspect,
                        color: job.color,
                        ..Default::default()
                    },
                );
            }
            *pending = 0;
        }
    });
}

fn actor(t: &Texture2D, w: f32, h: f32, x: f32, time: f64, durations: &[f64], indices: &[usize]) {
    if diagnostic("no_actors") {
        return;
    }
    let total: f64 = durations.iter().sum();
    let mut elapsed = time % total;
    let mut frame = 0;
    for (i, d) in durations.iter().enumerate() {
        frame = i;
        if elapsed < *d {
            break;
        }
        elapsed -= d;
    }
    draw_texture_ex(
        t,
        x,
        200.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(w * 0.55, h * 0.55)),
            source: Some(Rect::new(indices[frame] as f32 * w, 0., w, h)),
            ..Default::default()
        },
    );
}
#[macroquad::main(conf)]
async fn main() {
    if !diagnostic("no_text") {
        let mut font =
            load_ttf_font_from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/ui-subset.ttf")))
                .expect("UI font");
        font.set_filter(FilterMode::Linear);
        UI_FONT.with(|slot| {
            *slot.borrow_mut() = Some(UiFont {
                font,
                cached: std::collections::HashSet::new(),
                scratch: Vec::new(),
                jobs: Vec::new(),
                pending: 0,
            })
        });
    }
    let defeat = texture(include_bytes!("../assets/defeat.png"));
    let mut zh = true;
    let tiles = [
        texture(include_bytes!("../assets/balanced.png")),
        texture(include_bytes!("../assets/hp.png")),
        texture(include_bytes!("../assets/damage.png")),
        texture(include_bytes!("../assets/mp.png")),
        texture(include_bytes!("../assets/hd.png")),
        texture(include_bytes!("../assets/hm.png")),
        texture(include_bytes!("../assets/dm.png")),
    ];
    #[cfg(not(target_arch = "wasm32"))]
    let bench = std::env::var("BATTLEBLOCKS_BENCH_HERO")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&i| i < HEROES.len());
    #[cfg(target_arch = "wasm32")]
    let bench: Option<usize> = None;
    let mut hero_id = bench.unwrap_or(0);
    #[cfg(not(target_arch = "wasm32"))]
    let startup_commands = std::env::args().any(|arg| arg == "--command");
    #[cfg(target_arch = "wasm32")]
    let startup_commands = false;
    let mut knight = diagnostic_texture(sprites::SPRITES[hero_id].bytes, "no_actors");
    let thumbs = texture(include_bytes!("../assets/hero-thumbnails.png"));
    let mut preview = menu::Preview::new(hero_id, 0);
    let mut enemy_id = (hero_id + 1) % HEROES.len();
    let mut demon = diagnostic_texture(sprites::SPRITES[enemy_id].bytes, "no_actors");
    let mut choosing = bench.is_none() && !startup_commands;
    let audio = audio::Audio::new(!diagnostic("no_music"), !diagnostic("no_sound"))
        .map_err(|error| eprintln!("Audio unavailable: {error}"))
        .ok();
    rng::srand((get_time() * 1e9) as u64);
    let mut b = fresh();
    let mut selected: Option<(usize, usize)> = None;
    let (mut hp, mut enemy, mut mp) = (HEROES[hero_id].hp, HEROES[enemy_id].hp, 0i32);
    let mut pending: Vec<(usize, usize)> = Vec::new();
    let mut deadline = 0.;
    let mut enemy_clock = 0.;
    let mut enemy_mp = 0;
    let mut enemy_turn = false;
    let mut action_done = false;
    let mut difficulty = 1usize;
    let mut bonuses = [0, 0];
    let mut armed_sides = [false, false];
    let mut feedback = String::new();
    let mut feedback_time = 0.;
    let mut move_flash = None;
    let mut targeting = None;
    let mut skill_flash = Vec::new();
    let mut last_cast = None;
    let mut last_cast_enemy = false;
    let mut cast_time = 0.;
    let mut paused = false;
    let mut muted = false;
    let mut interface = ui::Ui::default();
    let mut chain = 0;
    let mut note = "Select two adjacent tiles";
    let mut reward = true;
    let mut extra_bonus = 0;
    let mut armed = false;
    let mut console = debug::Console::new();
    let mut game_over_logged = false;
    let mut charge_damage: Option<(i32, bool)> = None;
    let mut presentation = presentation::Presentation::default();
    macro_rules! snapshot {
        () => {
            debug::Snapshot {
                board: b,
                hero: hero_id,
                enemy: enemy_id,
                hp,
                enemy_hp: enemy,
                mp,
                enemy_mp,
                enemy_turn,
                action_done,
                pending: pending.clone(),
                deadline,
                enemy_clock,
                bonuses,
                armed_sides,
                bonus: extra_bonus,
                armed,
                chain,
                reward,
                difficulty,
                paused,
                rng_state: rng::state(),
                charge_damage,
                charge_remaining: presentation.charge_remaining(),
            }
        };
    }
    macro_rules! restore {
        ($state:expr) => {{
            let state = $state;
            b = state.board;
            hero_id = state.hero;
            enemy_id = state.enemy;
            hp = state.hp;
            enemy = state.enemy_hp;
            mp = state.mp;
            enemy_mp = state.enemy_mp;
            enemy_turn = state.enemy_turn;
            action_done = state.action_done;
            pending = state.pending;
            deadline = state.deadline;
            enemy_clock = state.enemy_clock;
            bonuses = state.bonuses;
            armed_sides = state.armed_sides;
            extra_bonus = state.bonus;
            armed = state.armed;
            chain = state.chain;
            rng::restore(state.rng_state);
            reward = state.reward;
            difficulty = state.difficulty;
            paused = state.paused;
            selected = None;
            targeting = None;
            presentation.clear();
            charge_damage = state.charge_damage;
            last_cast_enemy = charge_damage.map(|(_, enemy)| enemy).unwrap_or(false);
            if charge_damage.is_some() {
                let column = pending.first().map(|c| c.0).unwrap_or(0);
                presentation.charge(column, state.charge_remaining.max(0.001));
            }
            skill_flash.clear();
            last_cast = None;
            cast_time = 0.;
            feedback.clear();
            feedback_time = 0.;
            move_flash = None;
            knight = diagnostic_texture(sprites::SPRITES[hero_id].bytes, "no_actors");
            demon = diagnostic_texture(sprites::SPRITES[enemy_id].bytes, "no_actors");
            preview = menu::Preview::new(hero_id, 0);
            choosing = false;
            game_over_logged = false;
        }};
    }
    console.event("session_start", "New session", &snapshot!(), &snapshot!());
    loop {
        let dt = get_frame_time() as f64;
        let scale = (screen_width() / 1024.).min(screen_height() / 720.);
        let cam = Camera2D {
            target: vec2(512., 360.),
            zoom: vec2(2. * scale / screen_width(), 2. * scale / screen_height()),
            ..Default::default()
        };
        let (mx, my) = mouse_position();
        let pointer = cam.screen_to_world(vec2(mx, my));
        let was_console_open = console.open;
        console.input();
        let mut command_move = None;
        let mut command_skill = None;
        let mut command_target = None;
        let mut restart_command = false;
        // Process one command per frame so multiple CLI commands retain their order.
        if let Some(line) = console.commands.pop_front() {
            console.event("command", &line, &snapshot!(), &snapshot!());
            match debug::parse(&line) {
                Err(error) => console.say(&error),
                Ok(command) => {
                    use debug::Command;
                    let before = snapshot!();
                    match command {
                        Command::Help=>{
                            for text in [
                                "status | board | export | undo | undo player | restart",
                                "tile x y type | board load <64 types in row order> | shuffle | seed n",
                                "hp player/enemy n | mp player/enemy n | turn player/enemy | ai 0/1/2",
                                "hero player/enemy 0..6 | buff player/enemy bonus 0..10 | buff player/enemy armed on/off",
                                "move x1 y1 x2 y2 | cast 1/2 [x y] | resolve | pause on/off",
                                "Coordinates: 1..8; tile: 0 balanced,1 heal,2 attack,3 mana,4 HP+ATK,5 HP+MP,6 ATK+MP",
                                "Console freezes combat. Move/cast/resolve close it and run normal rules.",
                                "Undo restores up to 256 action snapshots. Random refill state is also restored.",
                            ]{console.say(text);}
                        },
                        Command::Status=>console.say(&format!("Hero={} Enemy={} HP={}/{} MP={}/{} turn={} pending={} buff={} armed={} AI={} paused={}",hero_id,enemy_id,hp,enemy,mp,enemy_mp,if enemy_turn{"enemy"}else{"player"},pending.len(),extra_bonus,armed,difficulty,paused)),
                        Command::Board=>{for (y,row) in b.iter().enumerate(){console.say(&format!("{}: {:?}",y+1,row));}},
                        Command::Export=>console.export(),
                        Command::Undo(player)=>{
                            if let Some((state,action))=console.undo(player) {restore!(state);console.event("rollback",&format!("Restored before action {action}; command: {line}"),&before,&snapshot!());}
                            else{console.say("No matching action to undo");}
                        },
                        Command::Restart=>restart_command=true,
                        Command::Move(a,c)=>{
                            if choosing||enemy_turn||action_done||presentation.busy()||!pending.is_empty()||hp==0||enemy==0||paused {console.say("Move requires a live, unpaused player turn with no pending resolution");}
                            else if !board::adjacent(a,c,hero_id){console.say("Move requires adjacent tiles");}
                            else{command_move=Some((a,c));console.open=false;targeting=None;}
                        },
                        Command::Cast(slot,target)=>{
                            let skill=HEROES[hero_id].skills[slot];
                            if choosing||enemy_turn||action_done||presentation.busy()||!pending.is_empty()||hp==0||enemy==0||paused {console.say("Cast requires a live, unpaused player turn with no pending resolution");}
                            else if content::payment(hero_id,hp,mp,skill).is_none(){console.say("Not enough mana");}
                            else if (skill==content::Skill::Strike&&extra_bonus>0)||(skill==content::Skill::Landslide&&armed){console.say("Preparation skill is already active");}
                            else{command_skill=Some(skill);command_target=Some(target);console.open=false;targeting=None;}
                        },
                        Command::Ai(level)=>{
                            if choosing {difficulty=level;console.say("Pre-battle AI difficulty updated");}
                            else {console.say("AI difficulty is locked during battle; return to the main menu to choose a new difficulty");}
                        },
                        command=>{
                            let allowed=!action_done&&pending.is_empty()&&!presentation.busy();
                            if !allowed && !matches!(command,Command::Hp(..)|Command::Mp(..)|Command::Ai(..)|Command::Pause(..)) {
                                console.say("Wait for resolution, or undo before editing board/turn");
                            } else {
                                console.checkpoint_edit(before.clone());
                                match command {
                                    Command::Hero(side,id)=>{
                                        if side {enemy_id=id;enemy=HEROES[id].hp;enemy_mp=0;demon=diagnostic_texture(sprites::SPRITES[id].bytes,"no_actors");}
                                        else {hero_id=id;hp=HEROES[id].hp;mp=0;knight=diagnostic_texture(sprites::SPRITES[id].bytes,"no_actors");preview=menu::Preview::new(id,0);}
                                        bonuses[side as usize]=0;armed_sides[side as usize]=false;
                                        if enemy_turn==side {extra_bonus=0;armed=false;}
                                    },
                                    Command::Bonus(side,value)=>{bonuses[side as usize]=value;if enemy_turn==side{extra_bonus=value;}},
                                    Command::Armed(side,value)=>{armed_sides[side as usize]=value;if enemy_turn==side{armed=value;}},
                                    Command::SetTile(x,y,kind)=>b[y][x]=kind,
                                    Command::LoadBoard(board)=>b=board,
                                    Command::Shuffle=>b=fresh(),
                                    Command::Seed(seed)=>rng::srand(seed),
                                    Command::Hp(side,value)=>if side{enemy=value.min(HEROES[enemy_id].hp)}else{hp=value.min(HEROES[hero_id].hp)},
                                    Command::Mp(side,value)=>if side{enemy_mp=value.min(HEROES[enemy_id].mp)}else{mp=value.min(HEROES[hero_id].mp)},
                                    Command::Turn(side)=>{
                                        bonuses[enemy_turn as usize]=extra_bonus;armed_sides[enemy_turn as usize]=armed;
                                        enemy_turn=side;extra_bonus=bonuses[side as usize];armed=armed_sides[side as usize];enemy_clock=0.;
                                    },
                                    Command::Pause(value)=>paused=value,
                                    Command::Resolve=>{pending=matches(&b);action_done=true;deadline=0.28;reward=true;chain=1;console.open=false;},
                                    _=>unreachable!(),
                                }
                                selected=None;targeting=None;choosing=false;game_over_logged=false;
                                console.event("debug_edit",&line,&before,&snapshot!());
                            }
                        }
                    }
                }
            }
        }
        let ui_consumed = if !was_console_open && !console.open {
            interface.input(pointer, choosing, &mut muted, &mut paused, &mut zh)
        } else {
            false
        };
        if let Some(audio) = &audio {
            audio.muted(muted);
            audio.volume(interface.volume);
        }
        let mut pause_click = false;
        if paused
            && !choosing
            && !was_console_open
            && !console.open
            && !ui_consumed
            && interface.panel.is_none()
            && is_mouse_button_pressed(MouseButton::Left)
        {
            for i in 0..4 {
                if ui::pause_rect(i).contains(pointer) {
                    pause_click = true;
                    match i {
                        0 => paused = false,
                        1 => {
                            choosing = true;
                            interface.opponent_step = false;
                            paused = false;
                            targeting = None;
                            selected = None;
                        }
                        2 => restart_command = true,
                        _ => interface.panel = Some(ui::Panel::Settings),
                    }
                    break;
                }
            }
        }
        let block_input = was_console_open || console.open || ui_consumed || pause_click;
        let key = |code| !block_input && is_key_pressed(code);
        let clicked = |button| !block_input && is_mouse_button_pressed(button);
        if key(KeyCode::L) {
            zh = !zh;
        }
        if key(KeyCode::P) {
            paused = !paused;
        }
        if key(KeyCode::M) {
            muted = !muted;
            if let Some(audio) = &audio {
                audio.muted(muted);
                audio.volume(interface.volume);
            }
        }
        if key(KeyCode::E) {
            choosing = true;
            interface.opponent_step = false;
        }
        if key(KeyCode::R) || restart_command {
            let before = snapshot!();
            b = fresh();
            hp = HEROES[hero_id].hp;
            enemy_id = interface.opponent.unwrap_or_else(|| {
                (hero_id + 1 + rng::gen_range(0, HEROES.len() - 1)) % HEROES.len()
            });
            demon = diagnostic_texture(sprites::SPRITES[enemy_id].bytes, "no_actors");
            enemy = HEROES[enemy_id].hp;
            enemy_mp = 0;
            enemy_turn = false;
            action_done = false;
            bonuses = [0, 0];
            armed_sides = [false, false];
            targeting = None;
            presentation.clear();
            charge_damage = None;
            skill_flash.clear();
            last_cast = None;
            cast_time = 0.;
            feedback.clear();
            feedback_time = 0.;
            move_flash = None;
            mp = 0;
            pending.clear();
            selected = None;
            enemy_clock = 0.;
            paused = false;
            chain = 0;
            note = "Select two adjacent tiles";
            armed = false;
            extra_bonus = 0;
            reward = true;
            game_over_logged = false;
            console.reset_history();
            console.event("restart", "Restart battle", &before, &snapshot!());
        }
        if choosing {
            if let Some(audio) = &audio {
                audio.playing(false);
            }
            if !interface.opponent_step {
                let old_hero = hero_id;
                if key(KeyCode::Down) {
                    hero_id = (hero_id + 1) % HEROES.len();
                }
                if key(KeyCode::Up) {
                    hero_id = (hero_id + HEROES.len() - 1) % HEROES.len();
                }
                if clicked(MouseButton::Left) {
                    for i in 0..HEROES.len() {
                        if menu::hero_rect(i).contains(pointer) {
                            hero_id = i;
                        }
                    }
                }
                if hero_id != old_hero {
                    knight = diagnostic_texture(sprites::SPRITES[hero_id].bytes, "no_actors");
                    preview = menu::Preview::new(hero_id, 0);
                }
                for i in 0..HEROES[hero_id].skills.len() {
                    if (clicked(MouseButton::Left) && menu::skill_rect(i).contains(pointer))
                        || (i == 0 && key(KeyCode::Key1))
                        || (i == 1 && key(KeyCode::Key2))
                    {
                        preview = menu::Preview::new(hero_id, i);
                    }
                }
            }
            let next = key(KeyCode::Enter)
                || (clicked(MouseButton::Left) && menu::start_rect().contains(pointer));
            let start_battle = interface.opponent_step && next;
            if interface.opponent_step {
                if key(KeyCode::Escape)
                    || (clicked(MouseButton::Left) && ui::back_rect().contains(pointer))
                {
                    interface.opponent_step = false;
                }
                if clicked(MouseButton::Left) {
                    for i in 0..3 {
                        if ui::difficulty_rect(i).contains(pointer) {
                            difficulty = i;
                        }
                    }
                    for i in 0..8 {
                        if ui::opponent_rect(i).contains(pointer) {
                            interface.opponent = if i == 0 { None } else { Some(i - 1) };
                        }
                    }
                }
                let index = interface.opponent.map(|i| i + 1).unwrap_or(0);
                let delta = if key(KeyCode::Right) {
                    1
                } else if key(KeyCode::Left) {
                    -1
                } else if key(KeyCode::Down) {
                    2
                } else if key(KeyCode::Up) {
                    -2
                } else {
                    0
                };
                if delta != 0 {
                    let i = (index as i32 + delta).rem_euclid(8) as usize;
                    interface.opponent = if i == 0 { None } else { Some(i - 1) };
                }
            } else if next {
                interface.opponent_step = true;
            }
            if start_battle {
                let before = snapshot!();
                choosing = false;
                b = fresh();
                hp = HEROES[hero_id].hp;
                mp = 0;
                enemy_id = interface.opponent.unwrap_or_else(|| {
                    (hero_id + 1 + rng::gen_range(0, HEROES.len() - 1)) % HEROES.len()
                });
                demon = diagnostic_texture(sprites::SPRITES[enemy_id].bytes, "no_actors");
                enemy = HEROES[enemy_id].hp;
                enemy_mp = 0;
                enemy_turn = false;
                action_done = false;
                bonuses = [0, 0];
                armed_sides = [false, false];
                targeting = None;
                presentation.clear();
                charge_damage = None;
                skill_flash.clear();
                last_cast = None;
                cast_time = 0.;
                feedback.clear();
                feedback_time = 0.;
                move_flash = None;
                pending.clear();
                selected = None;
                enemy_clock = 0.;
                paused = false;
                armed = false;
                extra_bonus = 0;
                reward = true;
                chain = 0;
                note = "Select two adjacent tiles";
                game_over_logged = false;
                console.reset_history();
                console.event(
                    "battle_start",
                    "Selected hero and opponent",
                    &before,
                    &snapshot!(),
                );
            }
            if choosing {
                clear_background(Color::from_rgba(25, 27, 38, 255));
                set_camera(&cam);
                if interface.opponent_step {
                    interface.draw_opponents(hero_id, zh, &thumbs, pointer, difficulty);
                } else {
                    menu::draw(hero_id, zh, &knight, &thumbs, &tiles, &preview, pointer);
                }
                interface.draw_top(true, muted, paused, pointer, zh);
                interface.draw_panel(zh, muted, &tiles, pointer);
                console.draw();
                flush_labels();
                set_default_camera();
                next_frame().await;
                continue;
            }
        }
        if let Some(audio) = &audio {
            audio.playing(
                !paused && !console.open && interface.panel.is_none() && hp > 0 && enemy > 0,
            );
        }
        // Visual lifetimes continue after victory/defeat and are independent of rewards.
        feedback_time = (feedback_time - dt).max(0.);
        cast_time = (cast_time - dt).max(0.);
        if cast_time == 0. {
            skill_flash.clear();
            last_cast = None;
        }
        if !paused && !console.open && interface.panel.is_none() {
            presentation.tick(dt);
        }
        let active = !paused
            && !console.open
            && !ui_consumed
            && !pause_click
            && hp > 0
            && enemy > 0
            && !presentation.busy();
        if active {
            if let Some((damage, enemy_actor)) = charge_damage {
                let before = snapshot!();
                charge_damage = None;
                if enemy_actor {
                    hp = (hp - damage).max(0);
                } else {
                    enemy = (enemy - damage).max(0);
                }
                let actual = if enemy_actor {
                    before.hp - hp
                } else {
                    before.enemy_hp - enemy
                };
                presentation.health(
                    [before.hp, before.enemy_hp],
                    [hp, enemy],
                    content::Skill::Charge.name(zh),
                );
                feedback = format!(
                    "{} · {} {}",
                    content::Skill::Charge.name(zh),
                    tr(zh, "Damage:", "伤害："),
                    actual
                );
                feedback_time = 2.5;
                deadline = 0.;
                console.event(
                    "skill_hit",
                    &format!("Charge expected_damage={} actual_damage={}", damage, actual),
                    &before,
                    &snapshot!(),
                );
            }
            if pending.is_empty() && !presentation.busy() {
                deadline = (deadline - dt).max(0.);
            }
            if action_done && pending.is_empty() && !presentation.busy() && deadline <= 0. {
                let before = snapshot!();
                bonuses[enemy_turn as usize] = extra_bonus;
                armed_sides[enemy_turn as usize] = armed;
                enemy_turn = !enemy_turn;
                extra_bonus = bonuses[enemy_turn as usize];
                armed = armed_sides[enemy_turn as usize];
                action_done = false;
                selected = None;
                enemy_clock = 0.;
                move_flash = None;
                chain = 0;
                console.event(
                    "turn_start",
                    "All cascades resolved; next actor",
                    &before,
                    &snapshot!(),
                );
            }
            if pending.is_empty() && !action_done && targeting.is_none() && !armed {
                let (id, health, mana) = if enemy_turn {
                    (enemy_id, enemy, enemy_mp)
                } else {
                    (hero_id, hp, mp)
                };
                let no_swaps = board::hero_options(&b, id).is_empty();
                let automatic = no_swaps && content::must_refresh(&b, id, health, mana, armed);
                let voluntary = no_swaps
                    && !enemy_turn
                    && (key(KeyCode::F2)
                        || (clicked(MouseButton::Left) && reroll_rect().contains(pointer)));
                if automatic || voluntary {
                    let before = snapshot!();
                    if voluntary {
                        console.checkpoint(before.clone());
                    }
                    b = board::fresh_for(id);
                    selected = None;
                    move_flash = None;
                    presentation.stop_motion();
                    note = "No moves: board refreshed";
                    console.event(
                        "board_refresh",
                        if voluntary {
                            "Player chose reroll instead of a skill"
                        } else {
                            "No role-legal swap or usable skill"
                        },
                        &before,
                        &snapshot!(),
                    );
                }
            }
            if enemy_turn && pending.is_empty() && !action_done {
                enemy_clock += dt;
            }
            let mut cast = command_skill;
            if pending.is_empty() && hp > 0 && !enemy_turn && !action_done {
                for (i, skill) in HEROES[hero_id].skills.iter().enumerate() {
                    let rect = Rect::new(20., 445. + i as f32 * 118., 210., 112.);
                    if !((matches!(skill, content::Skill::Strike) && extra_bonus > 0)
                        || (matches!(skill, content::Skill::Landslide) && armed))
                        && ((i == 0 && (key(KeyCode::Key1) || key(KeyCode::Space)))
                            || (i == 1 && key(KeyCode::Key2))
                            || (clicked(MouseButton::Left) && rect.contains(pointer)))
                    {
                        cast = Some(*skill);
                    }
                }
            }
            if enemy_turn && enemy_clock >= 1.1 && pending.is_empty() && !action_done {
                let enemy_before = snapshot!();
                cast = strategy::choose_skill(
                    &b,
                    enemy_id,
                    enemy,
                    enemy_mp,
                    difficulty,
                    armed,
                    extra_bonus,
                );
                if cast.is_none() {
                    if let Some((a, c)) =
                        strategy::choose_move(&b, enemy, enemy_id, enemy_mp, hero_id, difficulty)
                            .or_else(|| armed.then_some(((0, 0), (1, 0))))
                    {
                        let before = enemy_before.clone();
                        console.checkpoint(before.clone());
                        presentation.swap(&before.board, a, c, armed, enemy_turn, enemy_id);
                        if armed {
                            parallel_swap(&mut b, a, c);
                            armed = false;
                        } else {
                            board::hero_swap(&mut b, a, c, enemy_id);
                        }
                        pending = matches(&b);
                        deadline = CLEAR_TIME;
                        reward = true;
                        chain = 1;
                        action_done = true;
                        move_flash = Some((a, c));
                        console.event(
                            "move",
                            &format!(
                                "AI swap [{},{}] -> [{},{}]",
                                a.0 + 1,
                                a.1 + 1,
                                c.0 + 1,
                                c.1 + 1
                            ),
                            &before,
                            &snapshot!(),
                        );
                    } else {
                        let before = snapshot!();
                        b = board::fresh_for(enemy_id);
                        console.event(
                            "board_refresh",
                            "AI chose reroll instead of a skill",
                            &before,
                            &snapshot!(),
                        );
                    }
                }
                enemy_clock = 0.;
            }
            let board_target =
                if pointer.x >= 256. && pointer.x < 768. && pointer.y >= 110. && pointer.y < 622. {
                    Some((
                        ((pointer.x - 256.) / 64.) as usize,
                        ((pointer.y - 110.) / 64.) as usize,
                    ))
                } else {
                    None
                };
            let cancel_target = key(KeyCode::Escape)
                || clicked(MouseButton::Right)
                || (targeting.is_some()
                    && clicked(MouseButton::Left)
                    && cancel_skill_rect().contains(pointer));
            if cancel_target {
                targeting = None;
                selected = None;
                cast = None;
            }
            if !enemy_turn {
                if let Some(skill) = cast {
                    if (content::targeted(skill)
                        || content::payment(hero_id, hp, mp, skill).is_some_and(|p| p.blood > 0))
                        && content::payment(hero_id, hp, mp, skill).is_some()
                        && command_skill.is_none()
                    {
                        targeting = toggle_skill_target(targeting, skill);
                        selected = None;
                        cast = None;
                    }
                }
                if let Some(skill) = targeting {
                    if board_target.is_some() && clicked(MouseButton::Left) {
                        cast = Some(skill);
                        targeting = None;
                    }
                }
            }
            let mut cast_this_frame = false;
            if let Some(skill) = cast {
                let before = snapshot!();
                let id = if enemy_turn { enemy_id } else { hero_id };
                let actor_hp = if enemy_turn { enemy } else { hp };
                let actor_mp = if enemy_turn { enemy_mp } else { mp };
                if let Some(pay) = content::payment(id, actor_hp, actor_mp, skill) {
                    if enemy_turn {
                        enemy_mp -= pay.mana;
                        enemy -= pay.blood;
                    } else {
                        mp -= pay.mana;
                        hp -= pay.blood;
                    }
                    console.checkpoint(before.clone());
                    cast_this_frame = true;
                    move_flash = None;
                    last_cast = Some(skill);
                    last_cast_enemy = enemy_turn;
                    cast_time = 1.4;
                    let target = if enemy_turn {
                        strategy::choose_target(
                            skill,
                            &b,
                            enemy_id,
                            before.enemy_hp,
                            before.enemy_mp,
                            extra_bonus,
                        )
                    } else {
                        command_target.or(board_target).unwrap_or((3, 3))
                    };
                    skill_flash = content::target_cells(skill, &b, target);
                    let e = effect_at(skill, &mut b, target);
                    let direct_damage = if skill == content::Skill::Charge {
                        charge_damage = Some((e.damage, enemy_turn));
                        presentation.charge(target.0, presentation::CHARGE_TIME);
                        0
                    } else {
                        e.damage
                    };
                    if enemy_turn {
                        enemy = (enemy + e.hp).min(HEROES[enemy_id].hp);
                        hp = (hp - direct_damage).max(0);
                        enemy_mp = (enemy_mp + e.mp).min(HEROES[enemy_id].mp);
                    } else {
                        hp = (hp + e.hp).min(HEROES[hero_id].hp);
                        enemy = (enemy - direct_damage).max(0);
                        mp = (mp + e.mp).min(HEROES[hero_id].mp);
                    }
                    let actual_heal = if enemy_turn {
                        enemy - before.enemy_hp
                    } else {
                        hp - before.hp
                    };
                    let actual_damage = if enemy_turn {
                        before.hp - hp
                    } else {
                        before.enemy_hp - enemy
                    };
                    let actual_mana = if enemy_turn {
                        enemy_mp - before.enemy_mp + pay.mana
                    } else {
                        mp - before.mp + pay.mana
                    };
                    feedback = format!(
                        "{} · {}  +{} HP / {} damage / +{} MP",
                        if enemy_turn {
                            HEROES[enemy_id].zh
                        } else {
                            HEROES[hero_id].zh
                        },
                        skill.name(zh),
                        actual_heal,
                        actual_damage,
                        actual_mana
                    );
                    if skill == content::Skill::Charge {
                        feedback = format!(
                            "{} · {} {}",
                            skill.name(zh),
                            tr(zh, "Charging · expected damage:", "冲锋中 · 预计伤害："),
                            e.damage.min(if enemy_turn { hp } else { enemy })
                        );
                    }
                    if pay.blood > 0 {
                        feedback = format!(
                            "{} -{} HP · HP {} → {} · MP {} → {}",
                            tr(zh, "Flesh casting", "血肉施法"),
                            pay.blood,
                            actor_hp,
                            if enemy_turn { enemy } else { hp },
                            actor_mp,
                            if enemy_turn { enemy_mp } else { mp }
                        );
                    }
                    presentation.health(
                        [before.hp, before.enemy_hp],
                        [hp, enemy],
                        if pay.blood > 0 {
                            tr(zh, "Flesh casting", "血肉施法")
                        } else {
                            skill.name(zh)
                        },
                    );
                    feedback_time = 2.5;
                    action_done = !e.armed && e.bonus == 0;
                    extra_bonus += e.bonus;
                    armed |= e.armed;
                    pending = e.cells;
                    reward = e.reward;
                    if pending.is_empty() {
                        pending = matches(&b);
                        reward = true;
                    }
                    deadline = if pending.is_empty() {
                        TURN_HOLD
                    } else {
                        CLEAR_TIME
                    };
                    chain = 1;
                    selected = None;
                    note = "Skill cast";
                    console.event("skill",&format!("{:?} target=[{},{}] cost={} blood_cost={} direct_heal={} direct_damage={} direct_mana={} bonus={} armed={} actual_heal={} actual_damage={} actual_mana={}",skill,target.0+1,target.1+1,pay.mana,pay.blood,e.hp,e.damage,e.mp,e.bonus,e.armed,actual_heal,actual_damage,actual_mana),&before,&snapshot!());
                } else {
                    note = "Not enough mana";
                }
            }
            if !pending.is_empty() {
                if !presentation.busy() {
                    deadline -= dt;
                }
                if deadline <= 0. && !presentation.busy() {
                    if !muted {
                        if let Some(audio) = &audio {
                            audio.crush();
                        }
                    }
                    let before = snapshot!();
                    let cleared: Vec<_> = pending.iter().map(|&(x, y)| (x + 1, y + 1)).collect();
                    let rewarded = reward;
                    let bonus = content::reward_scale(chain.max(1) as usize, extra_bonus);
                    let id = if enemy_turn { enemy_id } else { hero_id };
                    let (heal, damage, mana) = if reward {
                        content::clear_rewards(
                            id,
                            if enemy_turn { enemy } else { hp },
                            if enemy_turn { enemy_mp } else { mp },
                            &b,
                            &pending,
                            chain.max(1) as usize,
                            extra_bonus,
                        )
                    } else {
                        (0, 0, 0)
                    };
                    if reward {
                        extra_bonus = 0;
                    }
                    for &(x, y) in &pending {
                        b[y][x] = 255;
                    }
                    if enemy_turn {
                        enemy = (enemy + heal).min(HEROES[enemy_id].hp);
                        hp = (hp - damage).max(0);
                        enemy_mp = (enemy_mp + mana).min(HEROES[enemy_id].mp);
                    } else {
                        hp = (hp + heal).min(HEROES[hero_id].hp);
                        enemy = (enemy - damage).max(0);
                        mp = (mp + mana).min(HEROES[hero_id].mp);
                    }
                    presentation.health(
                        [before.hp, before.enemy_hp],
                        [hp, enemy],
                        tr(zh, "Clear", "消除"),
                    );
                    let actual_heal = if enemy_turn {
                        enemy - before.enemy_hp
                    } else {
                        hp - before.hp
                    };
                    let actual_damage = if enemy_turn {
                        before.hp - hp
                    } else {
                        before.enemy_hp - enemy
                    };
                    let actual_mana = if enemy_turn {
                        enemy_mp - before.enemy_mp
                    } else {
                        mp - before.mp
                    };
                    if reward {
                        feedback = format!(
                            "{} +{} HP / {} damage / +{} MP",
                            if enemy_turn {
                                tr(zh, "Enemy", "敌方")
                            } else {
                                tr(zh, "You", "我方")
                            },
                            actual_heal,
                            actual_damage,
                            actual_mana
                        );
                        feedback_time = 2.;
                    }
                    presentation.fall(board::drop_board_animated(&mut b));
                    pending = matches(&b);
                    reward = true;
                    if !pending.is_empty() {
                        chain += 1;
                        deadline = CLEAR_TIME;
                    } else {
                        deadline = TURN_HOLD;
                    }
                    console.event("resolution",&format!("cells={:?} reward={} multiplier_percent={} heal={} damage={} mana={} chain={} refreshed={} actual_heal={} actual_damage={} actual_mana={}",cleared,rewarded,bonus,heal,damage,mana,chain,false,actual_heal,actual_damage,actual_mana),&before,&snapshot!());
                }
            } else if (clicked(MouseButton::Left) || command_move.is_some())
                && hp > 0
                && !enemy_turn
                && !action_done
                && targeting.is_none()
                && !cast_this_frame
            {
                let (mx, my) = mouse_position();
                let world = cam.screen_to_world(vec2(mx, my));
                let x = (world.x - 256.) / 64.;
                let y = (world.y - 110.) / 64.;
                if command_move.is_some() || (x >= 0. && x < 8. && y >= 0. && y < 8.) {
                    let c = command_move
                        .map(|(_, c)| c)
                        .unwrap_or((x as usize, y as usize));
                    if let Some((a, _)) = command_move {
                        selected = Some(a);
                    }
                    if let Some(a) = selected {
                        let before = snapshot!();
                        let legal = if armed && a.0.abs_diff(c.0) + a.1.abs_diff(c.1) == 1 {
                            parallel_swap(&mut b, a, c);
                            armed = false;
                            true
                        } else {
                            board::hero_swap(&mut b, a, c, hero_id)
                        };
                        if legal {
                            presentation.swap(
                                &before.board,
                                a,
                                c,
                                before.armed,
                                enemy_turn,
                                hero_id,
                            );
                            console.checkpoint(before.clone());
                            action_done = true;
                            move_flash = Some((a, c));
                            pending = matches(&b);
                            reward = true;
                            deadline = CLEAR_TIME;
                            chain = 1;
                            selected = None;
                            note = "Match!";
                            console.event(
                                "move",
                                &format!(
                                    "swap [{},{}] -> [{},{}]",
                                    a.0 + 1,
                                    a.1 + 1,
                                    c.0 + 1,
                                    c.1 + 1
                                ),
                                &before,
                                &snapshot!(),
                            );
                        } else {
                            selected = Some(c);
                            note = "No match: choose another tile";
                            console.event(
                                "move_rejected",
                                &format!(
                                    "swap [{},{}] -> [{},{}]: no match or nonadjacent",
                                    a.0 + 1,
                                    a.1 + 1,
                                    c.0 + 1,
                                    c.1 + 1
                                ),
                                &before,
                                &snapshot!(),
                            );
                        }
                    } else {
                        selected = Some(c);
                    }
                }
            }
        }
        if (hp == 0 || enemy == 0) && !game_over_logged {
            console.event(
                "battle_end",
                if enemy == 0 {
                    "Player victory"
                } else {
                    "Player defeat"
                },
                &snapshot!(),
                &snapshot!(),
            );
            game_over_logged = true;
        }
        clear_background(Color::from_rgba(25, 27, 38, 255));
        set_camera(&cam);
        procedural_board();
        for (side, x) in [(false, 12.), (true, 788.)] {
            let active_side = side == enemy_turn && hp > 0 && enemy > 0;
            let color = if active_side {
                if side {
                    ORANGE
                } else {
                    SKYBLUE
                }
            } else {
                Color::new(0.25, 0.28, 0.35, 0.6)
            };
            if active_side {
                draw_rectangle(
                    x,
                    92.,
                    224.,
                    551.,
                    Color::new(color.r, color.g, color.b, 0.07),
                );
            }
            draw_rectangle_lines(x, 92., 224., 551., if active_side { 3. } else { 1. }, color);
            if active_side {
                let bx = if side { 918. } else { 142. };
                draw_rectangle(
                    bx,
                    94.,
                    88.,
                    24.,
                    Color::new(color.r * 0.3, color.g * 0.3, color.b * 0.3, 1.),
                );
                label(tr(zh, "ACTIVE", "行动中"), bx + 6., 112., 18., color);
            }
        }
        label(tr(zh, "BATTLE BLOCKS", "方块战斗"), 300., 55., 40., GOLD);
        interface.draw_top(false, muted, paused, pointer, zh);
        actor(
            &knight,
            sprites::SPRITES[hero_id].w,
            sprites::SPRITES[hero_id].h,
            50.,
            get_time(),
            sprites::SPRITES[hero_id].durations,
            sprites::SPRITES[hero_id].indices,
        );
        actor(
            &demon,
            sprites::SPRITES[enemy_id].w,
            sprites::SPRITES[enemy_id].h,
            805.,
            get_time(),
            sprites::SPRITES[enemy_id].durations,
            sprites::SPRITES[enemy_id].indices,
        );
        label(
            if zh {
                HEROES[hero_id].zh
            } else {
                HEROES[hero_id].en
            },
            25.,
            110.,
            32.,
            WHITE,
        );
        label(
            &format!("{} {hp} / {}", tr(zh, "HP", "生命"), HEROES[hero_id].hp),
            25.,
            150.,
            30.,
            Color::new(0.7, 1., 0.7, 1.),
        );
        label(
            &format!("{} {mp} / {}", tr(zh, "MP", "魔力"), HEROES[hero_id].mp),
            25.,
            185.,
            30.,
            SKYBLUE,
        );
        label(
            if zh {
                HEROES[enemy_id].zh
            } else {
                HEROES[enemy_id].en
            },
            805.,
            110.,
            32.,
            WHITE,
        );
        label(
            &format!("MP {enemy_mp} / {}", HEROES[enemy_id].mp),
            805.,
            185.,
            28.,
            SKYBLUE,
        );
        label(tr(zh, "AI difficulty", "AI 难度"), 805., 445., 24., WHITE);
        label(
            [
                tr(zh, "Beginner", "入门"),
                tr(zh, "Standard", "标准"),
                tr(zh, "Advanced", "进阶"),
            ][difficulty],
            805.,
            480.,
            28.,
            GOLD,
        );
        label(
            if enemy_turn {
                tr(zh, "Enemy turn", "敌方回合")
            } else {
                tr(zh, "Your turn", "你的回合")
            },
            805.,
            525.,
            28.,
            if enemy_turn { ORANGE } else { GREEN },
        );
        if feedback_time > 0. && targeting.is_none() {
            label(&feedback, 256., 640., 16., GOLD);
        }
        label(
            &format!("{} {enemy} / {}", tr(zh, "HP", "生命"), HEROES[enemy_id].hp),
            805.,
            150.,
            30.,
            Color::new(1., 0.7, 0.7, 1.),
        );
        if !diagnostic("legacy_batches") {
            for y in 0..N {
                for x in 0..N {
                    let pos = vec2(256. + x as f32 * 64., 110. + y as f32 * 64.);
                    procedural_cell(pos.x, pos.y);
                }
            }
        }
        let player_hover = if !enemy_turn
            && !action_done
            && pending.is_empty()
            && hp > 0
            && enemy > 0
            && !paused
            && !console.open
        {
            HEROES[hero_id]
                .skills
                .iter()
                .enumerate()
                .find(|(i, _)| {
                    Rect::new(20., 445. + *i as f32 * 118., 210., 112.).contains(pointer)
                })
                .map(|(_, s)| (*s, false))
        } else {
            None
        };
        let enemy_hover = if !console.open {
            HEROES[enemy_id]
                .skills
                .iter()
                .enumerate()
                .find(|(i, _)| Rect::new(805., 565. + *i as f32 * 38., 195., 34.).contains(pointer))
                .map(|(_, s)| (*s, true))
        } else {
            None
        };
        let hover_context = player_hover.or(enemy_hover);
        let hover_skill = hover_context.map(|(skill, _)| skill);
        let mut preview_cells = targeting
            .or(hover_skill.filter(|s| !content::targeted(*s)))
            .map(|skill| {
                if content::targeted(skill)
                    && !(pointer.x >= 256.
                        && pointer.x < 768.
                        && pointer.y >= 110.
                        && pointer.y < 622.)
                {
                    return Vec::new();
                }
                let x = ((pointer.x - 256.) / 64.).clamp(0., 7.) as usize;
                let y = ((pointer.y - 110.) / 64.).clamp(0., 7.) as usize;
                content::target_cells(skill, &b, (x, y))
            })
            .unwrap_or_default();
        if targeting.is_none() && !enemy_turn && !action_done {
            if let Some(a) = selected {
                if pointer.x >= 256. && pointer.x < 768. && pointer.y >= 110. && pointer.y < 622. {
                    let c = (
                        ((pointer.x - 256.) / 64.) as usize,
                        ((pointer.y - 110.) / 64.) as usize,
                    );
                    let mut copy = b;
                    if board::hero_swap(&mut copy, a, c, hero_id) {
                        preview_cells = matches(&copy);
                    }
                    if hero_id == 0 && board::adjacent(a, c, hero_id) && a.1 == c.1 {
                        for y in 0..N {
                            for x in [a.0, c.0] {
                                draw_rectangle_lines(
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
        if targeting.is_some() {
            let cancel = cancel_skill_rect();
            draw_rectangle(
                cancel.x,
                cancel.y,
                cancel.w,
                cancel.h,
                Color::new(0.24, 0.16, 0.18, 1.),
            );
            draw_rectangle_lines(cancel.x, cancel.y, cancel.w, cancel.h, 1., GOLD);
            label(
                tr(zh, "Cancel · Esc / right click", "取消 · Esc / 右键"),
                805.,
                665.,
                18.,
                GOLD,
            );
        }
        for kind in 0..if !diagnostic("legacy_batches") { 7 } else { 1 } {
            for y in 0..N {
                for x in 0..N {
                    if !diagnostic("legacy_batches") && b[y][x] != kind {
                        continue;
                    }
                    let pos = vec2(256. + x as f32 * 64., 110. + y as f32 * 64.);
                    let charge_progress = presentation.destruction(x, y);
                    let clearing = pending.contains(&(x, y))
                        && (!presentation.busy() || charge_progress.is_some_and(|p| p > 0.));
                    let clear_progress = if let Some(progress) = charge_progress {
                        progress
                    } else if clearing {
                        (1. - deadline as f32 / CLEAR_TIME as f32).clamp(0., 1.)
                    } else {
                        0.
                    };
                    let color = Color::new(1., 1., 1., 1. - clear_progress);
                    let tile_size = 64. * (1. - clear_progress * 0.65);
                    if diagnostic("legacy_batches") {
                        procedural_cell(pos.x, pos.y);
                    }
                    if !presentation.hides(x, y) {
                        draw_texture_ex(
                            &tiles[b[y][x] as usize],
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
                        draw_rectangle_lines(pos.x + 3., pos.y + 3., 58., 58., 3., SKYBLUE);
                    }
                    if clearing {
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
                    if !pending.is_empty()
                        && move_flash.is_some_and(|(a, c)| a == (x, y) || c == (x, y))
                    {
                        draw_rectangle_lines(
                            pos.x + 2.,
                            pos.y + 2.,
                            60.,
                            60.,
                            4.,
                            if enemy_turn { ORANGE } else { SKYBLUE },
                        );
                    }
                    if selected == Some((x, y)) {
                        draw_rectangle_lines(pos.x, pos.y, 62., 62., 3., GOLD);
                    }
                }
            }
        }
        presentation.draw_tiles(&tiles);
        if presentation.charging() {
            let id = if last_cast_enemy { enemy_id } else { hero_id };
            let asset = &sprites::SPRITES[id];
            presentation.draw_charge(
                if last_cast_enemy { &demon } else { &knight },
                asset.w,
                asset.h,
                asset.indices,
            );
        }
        if let Some(skill) = last_cast.filter(|s| cast_time > 0. && *s != content::Skill::Charge) {
            let progress = 1. - cast_time as f32 / 1.4;
            effects::draw(skill, &skill_flash, progress, last_cast_enemy);
            draw_rectangle(352., 82., 320., 26., Color::new(0.04, 0.06, 0.1, 0.9));
            icons::draw(skill, 356., 84., 22.);
            label(
                &format!(
                    "{} · {}",
                    if last_cast_enemy {
                        tr(zh, "Enemy", "敌方")
                    } else {
                        tr(zh, "You", "我方")
                    },
                    skill.name(zh)
                ),
                386.,
                102.,
                18.,
                GOLD,
            );
        }
        for (i, skill) in HEROES[enemy_id].skills.iter().enumerate() {
            let r = Rect::new(805., 565. + i as f32 * 38., 195., 34.);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.12, 0.15, 0.2, 1.));
            if r.contains(pointer) {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., ORANGE);
            }
            icons::draw(*skill, 805., 565. + i as f32 * 38., 28.);
            label(
                &format!("{} · {} MP", skill.name(zh), skill.cost()),
                840.,
                585. + i as f32 * 38.,
                17.,
                if content::payment(enemy_id, enemy, enemy_mp, *skill).is_some() {
                    GOLD
                } else {
                    GRAY
                },
            );
        }
        for (i, skill) in HEROES[hero_id].skills.iter().enumerate() {
            let rect = Rect::new(20., 445. + i as f32 * 118., 210., 112.);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if content::payment(hero_id, hp, mp, *skill).is_some() {
                    Color::new(0.2, 0.28, 0.35, 1.)
                } else {
                    Color::new(0.1, 0.12, 0.16, 1.)
                },
            );
            icons::draw(*skill, rect.x + 8., rect.y + 8., 26.);
            label(
                &format!("{}: {}", i + 1, skill.name(zh)),
                rect.x + 40.,
                rect.y + 28.,
                21.,
                WHITE,
            );
            label(
                &if hero_id == 6 && mp < skill.cost() {
                    format!("{} MP + {} HP !", mp, (skill.cost() - mp) * 2)
                } else {
                    format!("{} MP", skill.cost())
                },
                rect.x + 8.,
                rect.y + 49.,
                17.,
                if hero_id == 6 && mp < skill.cost() {
                    RED
                } else {
                    SKYBLUE
                },
            );
            for (line, text) in skill.description(zh).lines().enumerate() {
                label(
                    text,
                    rect.x + 8.,
                    rect.y + 69. + line as f32 * 18.,
                    16.,
                    LIGHTGRAY,
                );
            }
        }
        for (id, x) in [(hero_id, 20.), (enemy_id, 800.)] {
            label(
                tr(zh, "PASSIVE (hover)", "被动（悬停查看）"),
                x,
                415.,
                16.,
                GOLD,
            );
            if Rect::new(x, 394., 210., 36.).contains(pointer) {
                flush_labels();
                draw_rectangle(270., 225., 485., 145., Color::new(0.04, 0.07, 0.12, 0.98));
                for (i, line) in content::passive(id, zh).lines().enumerate() {
                    label(line, 287., 260. + i as f32 * 30., 22., GOLD);
                }
            }
        }
        if !enemy_turn
            && !action_done
            && pending.is_empty()
            && !presentation.busy()
            && !armed
            && hp > 0
            && enemy > 0
            && targeting.is_none()
            && board::hero_options(&b, hero_id).is_empty()
        {
            let rect = reroll_rect();
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.18, 0.24, 0.30, 1.),
            );
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., GOLD);
            label(
                tr(
                    zh,
                    "No swaps: cast a skill / reroll (F2)",
                    "无可交换：可施法 / 点击重排（F2）",
                ),
                rect.x + 12.,
                rect.y + 25.,
                19.,
                GOLD,
            );
        }
        presentation.draw_health(zh);
        let detail_context = targeting.map(|skill| (skill, false)).or(hover_context);
        if let Some((skill, is_enemy)) = detail_context {
            let (id, health, mana, opponent, boost) = if is_enemy {
                (
                    enemy_id,
                    enemy,
                    enemy_mp,
                    hp,
                    if enemy_turn { extra_bonus } else { bonuses[1] },
                )
            } else {
                (hero_id, hp, mp, enemy, extra_bonus)
            };
            // Show details in the opposite sidebar, keeping the highlighted board visible.
            let x = if is_enemy { 14. } else { 790. };
            flush_labels();
            draw_rectangle(x, 208., 220., 438., Color::new(0.04, 0.07, 0.12, 0.98));
            draw_rectangle_lines(
                x,
                208.,
                220.,
                438.,
                2.,
                if is_enemy { ORANGE } else { SKYBLUE },
            );
            label(
                &format!(
                    "{} · {}",
                    if is_enemy {
                        tr(zh, "Enemy", "敌方")
                    } else {
                        tr(zh, "You", "我方")
                    },
                    skill.name(zh)
                ),
                x + 10.,
                239.,
                20.,
                GOLD,
            );
            label(
                &if id == 6 && mana < skill.cost() {
                    format!("{} MP + {} HP !", mana, (skill.cost() - mana) * 2)
                } else {
                    format!("{} {} MP", tr(zh, "Cost:", "消耗："), skill.cost())
                },
                x + 10.,
                266.,
                18.,
                if id == 6 && mana < skill.cost() {
                    RED
                } else {
                    SKYBLUE
                },
            );
            for (line, text) in wrap_preview_text(skill.description(zh), 12.)
                .iter()
                .take(7)
                .enumerate()
            {
                label(text, x + 10., 292. + line as f32 * 18., 15., LIGHTGRAY);
            }
            let exact_target = if !is_enemy
                && targeting == Some(skill)
                && pointer.x >= 256.
                && pointer.x < 768.
                && pointer.y >= 110.
                && pointer.y < 622.
            {
                Some((
                    ((pointer.x - 256.) / 64.) as usize,
                    ((pointer.y - 110.) / 64.) as usize,
                ))
            } else {
                None
            };
            let range = content::targeted(skill) && exact_target.is_none();
            let (low, high) = if range {
                content::preview_range(
                    skill,
                    &b,
                    id,
                    health,
                    if content::payment(id, health, mana, skill).is_some() {
                        mana
                    } else {
                        mana.max(skill.cost())
                    },
                    opponent,
                    boost,
                )
            } else {
                (
                    content::preview_rewards(
                        skill,
                        &b,
                        exact_target.unwrap_or((3, 3)),
                        id,
                        health,
                        if content::payment(id, health, mana, skill).is_some() {
                            mana
                        } else {
                            mana.max(skill.cost())
                        },
                        opponent,
                        boost,
                    ),
                    content::preview_rewards(
                        skill,
                        &b,
                        exact_target.unwrap_or((3, 3)),
                        id,
                        health,
                        if content::payment(id, health, mana, skill).is_some() {
                            mana
                        } else {
                            mana.max(skill.cost())
                        },
                        opponent,
                        boost,
                    ),
                )
            };
            let amount = |lo: i32, hi: i32| {
                if lo == hi {
                    lo.to_string()
                } else {
                    format!("{lo}..{hi}")
                }
            };
            label(
                tr(zh, "Expected rewards", "预估收益"),
                x + 10.,
                433.,
                19.,
                GOLD,
            );
            label(
                &format!(
                    "{} {}",
                    tr(zh, "HP change:", "生命变化："),
                    amount(low.heal, high.heal)
                ),
                x + 10.,
                460.,
                18.,
                if low.heal < 0 { RED } else { GREEN },
            );
            label(
                &format!(
                    "{} {}",
                    tr(zh, "Damage:", "预计伤害："),
                    amount(low.damage, high.damage)
                ),
                x + 10.,
                487.,
                if exact_target.is_some() { 22. } else { 18. },
                ORANGE,
            );
            label(
                &format!(
                    "{} +{}",
                    tr(zh, "Mana:", "魔力："),
                    amount(low.mana_gain, high.mana_gain)
                ),
                x + 10.,
                514.,
                18.,
                SKYBLUE,
            );
            if content::payment(id, health, mana, skill).is_some() {
                label(
                    &format!("MP {mana} → {}", amount(low.mana_after, high.mana_after)),
                    x + 10.,
                    541.,
                    17.,
                    SKYBLUE,
                );
            } else {
                label(
                    tr(zh, "If funded; mana too low", "筹足魔力后可获；当前不足"),
                    x + 10.,
                    541.,
                    14.,
                    ORANGE,
                );
            }
            if id == 6 && mana < skill.cost() && content::payment(id, health, mana, skill).is_none()
            {
                label(
                    tr(zh, "Not enough HP; must leave 1", "生命不足，支付后须剩1点"),
                    x + 10.,
                    598.,
                    15.,
                    RED,
                );
            }
            if low.blood_cost > 0 && content::payment(id, health, mana, skill).is_some() {
                label(
                    &format!("-{} HP → {} !", low.blood_cost, health - low.blood_cost),
                    x + 10.,
                    598.,
                    18.,
                    RED,
                );
                label(
                    &format!(
                        "{} HP → {}",
                        tr(zh, "After:", "结算后："),
                        amount(low.health_after, high.health_after)
                    ),
                    x + 10.,
                    620.,
                    16.,
                    RED,
                );
            }
            if exact_target.is_some() {
                label(
                    tr(zh, "Click board to confirm", "点击棋盘确认释放"),
                    x + 10.,
                    570.,
                    15.,
                    WHITE,
                );
            } else if range {
                label(
                    tr(zh, "Range depends on target", "收益区间依目标而定"),
                    x + 10.,
                    570.,
                    15.,
                    WHITE,
                );
            }
            if skill == content::Skill::Strike {
                label(
                    tr(zh, "Next clear: +100%", "下次消除收益＋100%"),
                    x + 10.,
                    570.,
                    15.,
                    GREEN,
                );
            }
            label(
                tr(zh, "Refill cascades excluded", "补充后的连锁未计入"),
                x + 10.,
                641.,
                13.,
                GRAY,
            );
        }
        label(
            &format!(
                "{}  {}: {chain}",
                if presentation.charging() {
                    if enemy_turn {
                        tr(zh, "Enemy · knight charge", "敌方 · 骑士冲锋")
                    } else {
                        tr(zh, "You · knight charge", "我方 · 骑士冲锋")
                    }
                } else if let Some(swap) = presentation.swapping() {
                    if enemy_turn {
                        if swap {
                            tr(zh, "Enemy · swapping", "敌方 · 正在交换")
                        } else {
                            tr(zh, "Enemy · falling", "敌方 · 方块掉落")
                        }
                    } else {
                        if swap {
                            tr(zh, "You · swapping", "我方 · 正在交换")
                        } else {
                            tr(zh, "You · falling", "我方 · 方块掉落")
                        }
                    }
                } else if !pending.is_empty() {
                    if enemy_turn {
                        tr(zh, "Enemy · clearing", "敌方 · 正在消除")
                    } else {
                        tr(zh, "You · clearing", "我方 · 正在消除")
                    }
                } else if action_done {
                    tr(zh, "Action resolved", "行动结算完成")
                } else {
                    localized_note(zh, note)
                },
                tr(zh, "Chain", "连锁")
            ),
            256.,
            80.,
            26.,
            WHITE,
        );
        if !paused && (hp == 0 || enemy == 0) {
            flush_labels();
            draw_rectangle(256., 190., 512., 340., Color::new(0.02, 0.02, 0.03, 0.96));
            if hp == 0 && enemy > 0 {
                draw_texture_ex(
                    &defeat,
                    422.,
                    205.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(180., 183.)),
                        ..Default::default()
                    },
                );
                label(tr(zh, "DEFEAT", "你输了"), 395., 435., 38., GOLD);
                label(
                    tr(zh, "R: Try again", "按 R 再来一局"),
                    355.,
                    485.,
                    32.,
                    WHITE,
                );
            } else {
                label(
                    if enemy == 0 {
                        tr(zh, "VICTORY!", "胜利！")
                    } else {
                        tr(zh, "PAUSED", "已暂停")
                    },
                    365.,
                    335.,
                    40.,
                    GOLD,
                );
                label(
                    if enemy == 0 {
                        tr(zh, "R: Play again", "按 R 再来一局")
                    } else {
                        tr(zh, "P: Continue", "按 P 继续游戏")
                    },
                    345.,
                    390.,
                    32.,
                    WHITE,
                );
            }
        }
        if paused {
            ui::draw_pause(zh, pointer);
        }
        interface.draw_panel(zh, muted, &tiles, pointer);
        console.draw();
        flush_labels();
        set_default_camera();
        next_frame().await;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Board {
        let mut b = [[0; N]; N];
        for y in 0..N {
            for x in 0..N {
                b[y][x] = ((x + y) % 7) as u8;
            }
        }
        b
    }
    #[test]
    fn selecting_same_skill_cancels_and_different_skill_switches() {
        use content::Skill;
        let selected = toggle_skill_target(None, Skill::Lightning);
        assert_eq!(selected, Some(Skill::Lightning));
        assert_eq!(toggle_skill_target(selected, Skill::Lightning), None);
        assert_eq!(
            toggle_skill_target(selected, Skill::Charge),
            Some(Skill::Charge)
        );
    }
    #[test]
    fn raster_sizes_stay_bounded_while_visual_size_is_preserved() {
        let mut used = std::collections::BTreeSet::new();
        for requested in 1..=400u16 {
            let bucket = font_raster_bucket(requested);
            used.insert(bucket);
            let adjusted = requested as f32 / bucket as f32;
            assert!((bucket as f32 * adjusted - requested as f32).abs() < 0.001);
        }
        assert_eq!(used.len(), 5);
        assert_eq!(font_raster_bucket(400), 64);
    }
    #[test]
    fn legal_swap_creates_a_line() {
        let mut b = base();
        b[0][0] = 6;
        b[0][1] = 6;
        b[1][2] = 6;
        assert!(matches(&b).is_empty());
        assert!(valid_swap(&mut b, (2, 0), (2, 1)));
        assert!(matches(&b).contains(&(0, 0)));
    }
    #[test]
    fn no_initial_match() {
        for _ in 0..100 {
            assert!(matches(&fresh()).is_empty());
        }
    }
    #[test]
    fn straight_and_cross_deduplicate() {
        let mut b = base();
        for x in 1..=3 {
            b[3][x] = 6;
        }
        for y in 2..=4 {
            b[y][2] = 6;
        }
        assert_eq!(matches(&b).len(), 5);
    }
    #[test]
    fn bent_three_do_not_match() {
        let mut b = base();
        b[0][0] = 6;
        b[0][1] = 6;
        b[1][1] = 6;
        assert!(matches(&b).is_empty());
    }
    #[test]
    fn illegal_swap_restores_board() {
        let mut b = base();
        let original = b;
        assert!(!valid_swap(&mut b, (0, 0), (1, 0)));
        assert_eq!(b, original);
    }
    #[test]
    fn gravity_preserves_survivor_order() {
        let mut b = base();
        b[7][0] = 255;
        b[5][0] = 255;
        let survivors: Vec<_> = (0..N)
            .filter_map(|y| if b[y][0] != 255 { Some(b[y][0]) } else { None })
            .collect();
        drop_board(&mut b);
        assert_eq!((2..N).map(|y| b[y][0]).collect::<Vec<_>>(), survivors);
    }
}

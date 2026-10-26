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
mod turns;
use content::{effect, HEROES, VALUES};
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
fn move_hints_rect() -> Rect {
    Rect::new(796., 648., 208., 44.)
}
fn reroll_rect() -> Rect {
    Rect::new(256., 650., 512., 38.)
}
fn cancel_skill_rect() -> Rect {
    move_hints_rect()
}
// Battle sidebars: the player on the left, the enemy on the right. Every element
// below stays inside its sidebar frame.
fn side_panel_rect(enemy: bool) -> Rect {
    Rect::new(if enemy { 788. } else { 12. }, 90., 224., 610.)
}
fn sprite_rect(enemy: bool) -> Rect {
    Rect::new(if enemy { 796. } else { 20. }, 212., 208., 156.)
}
fn passive_rect(enemy: bool) -> Rect {
    Rect::new(if enemy { 796. } else { 20. }, 376., 208., 50.)
}
fn player_skill_rect(i: usize) -> Rect {
    Rect::new(20., 434. + i as f32 * 132., 208., 126.)
}
fn enemy_skill_rect(i: usize) -> Rect {
    Rect::new(796., 470. + i as f32 * 44., 208., 38.)
}
// Turn indicator rows above the board frame, between the two sidebars.
fn previous_turn_rect() -> Rect {
    Rect::new(236., 6., 552., 32.)
}
fn current_turn_rect() -> Rect {
    Rect::new(236., 42., 552., 42.)
}
type Move = ((usize, usize), (usize, usize));
/// Row of the move-reward panel under the pointer; the panel lists 7 hints per page.
fn hint_row_at(pointer: Vec2, page: usize, count: usize) -> Option<usize> {
    // Half-open bounds: `Rect::contains` includes the bottom edge, which would reach a
    // row of the next page.
    (pointer.x >= 798. && pointer.x < 1002. && pointer.y >= 278. && pointer.y < 586.)
        .then(|| page * 7 + ((pointer.y - 278.) / 44.) as usize)
        .filter(|&i| i < count)
}
/// Numbered hint badge on the board under the pointer, nearest first.
fn hint_badge_at(pointer: Vec2, hints: &[Move]) -> Option<usize> {
    if !(pointer.x >= 256. && pointer.x < 768. && pointer.y >= 110. && pointer.y < 622.) {
        return None;
    }
    hints
        .iter()
        .enumerate()
        .map(|(i, &(a, c))| (i, presentation::move_hint_anchor(a, c).distance(pointer)))
        .filter(|(_, d)| *d < 22.)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}
/// Reward part of the action feedback line, in the UI language.
fn reward_summary(zh: bool, heal: i32, damage: i32, mana: i32) -> String {
    if zh {
        format!("生命 +{heal} · 伤害 {damage} · 魔力 +{mana}")
    } else {
        format!("+{heal} HP · {damage} damage · +{mana} MP")
    }
}
/// Skill cost in the UI language, including the HP part of a flesh-casting payment.
fn draw_diamond(x: f32, y: f32, radius: f32, color: Color, filled: bool) {
    let points = [
        vec2(x, y - radius),
        vec2(x + radius, y),
        vec2(x, y + radius),
        vec2(x - radius, y),
    ];
    if filled {
        draw_triangle(points[0], points[1], points[2], color);
        draw_triangle(points[0], points[2], points[3], color);
    } else {
        for i in 0..4 {
            draw_line(
                points[i].x,
                points[i].y,
                points[(i + 1) % 4].x,
                points[(i + 1) % 4].y,
                1.5,
                color,
            );
        }
    }
}
fn draw_breaking_lock(x: f32, y: f32, t: f32, color: Color) {
    let split = ((t - 0.15) / 0.6).clamp(0., 1.);
    if split == 0. {
        draw_lock(x, y, color);
        return;
    }
    let c = Color::new(color.r, color.g, color.b, 1. - split);
    for (direction, offset) in [(-1., 0.), (1., 6.)] {
        let dx = direction * split * 14.;
        let dy = -split * 7. + split * split * 17.;
        draw_rectangle(x + offset + dx, y + 6. + dy, 5., 8., c);
        draw_line(
            x + offset + 2. + dx,
            y + 7. + dy,
            x + offset + 2. + dx,
            y + dy,
            2.,
            c,
        );
    }
}
fn draw_lock(x: f32, y: f32, color: Color) {
    draw_rectangle_lines(x + 2., y, 7., 8., 2., color);
    draw_rectangle(x, y + 6., 11., 8., color);
    draw_rectangle(x + 5., y + 8., 2., 4., Color::from_rgba(25, 33, 46, 255));
}
fn cost_text(zh: bool, mana: i32, blood: i32) -> String {
    match (zh, blood > 0) {
        (true, true) => format!("{mana} 魔力 + {blood} 生命 !"),
        (true, false) => format!("{mana} 魔力"),
        (false, true) => format!("{mana} MP + {blood} HP !"),
        (false, false) => format!("{mana} MP"),
    }
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
    let (window_width, window_height) = if diagnostic("small_window") {
        (512, 360)
    } else {
        ui::WINDOW_SIZES[1]
    };
    let window = Conf {
        window_title: "BattleBlocks".into(),
        window_width,
        window_height,
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
    // Share of the measured width drawn left of `x`: 0 left-aligns, 0.5 centers.
    align: f32,
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
    queue_label(text, x, y, size, color, 0.);
}
fn queue_label(text: &str, x: f32, y: f32, size: f32, color: Color, align: f32) {
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
                    align,
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
            job.align = align;
        }
    });
}
/// Label width in world units. Measure before the frame's first `flush_labels`, so caching
/// a new glyph cannot grow the atlas under text that was already drawn this frame.
fn label_width(text: &str, size: f32) -> f32 {
    if diagnostic("no_text") {
        return 0.;
    }
    let (requested, font_scale, _) = camera_font_scale(size);
    let font_size = font_raster_bucket(requested);
    let font_scale = font_scale * requested.max(1) as f32 / font_size as f32;
    UI_FONT.with(|slot| {
        slot.borrow().as_ref().map_or(0., |ui| {
            measure_text(text, Some(&ui.font), font_size, font_scale).width
        })
    })
}
/// Font size that fits `text` into `width`, never larger than `size`.
fn fit_size(text: &str, size: f32, width: f32) -> f32 {
    (size * width / label_width(text, size).max(1.)).min(size)
}
/// Differently colored pieces of one line from `x`, shrunk together to fit into `room`.
/// Measures like `label_width`, so draw before the frame's first `flush_labels`.
fn label_segments(segments: &[(String, Color)], x: f32, y: f32, size: f32, room: f32) {
    let width: f32 = segments
        .iter()
        .map(|(text, _)| label_width(text, size))
        .sum();
    let size = (size * room / width.max(1.)).min(size);
    let mut x = x;
    for (text, color) in segments {
        label(text, x, y, size, *color);
        x += label_width(text, size);
    }
}
/// Centered on `center_x`. The width is measured when the label is flushed, after the
/// frame's glyphs are cached, so this is safe after an earlier `flush_labels` too.
fn label_centered(text: &str, center_x: f32, y: f32, size: f32, color: Color) {
    queue_label(text, center_x, y, size, color, 0.5);
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
                let x = if job.align > 0. {
                    let width =
                        measure_text(&job.text, Some(font), job.font_size, job.font_scale).width;
                    job.x - width * job.align
                } else {
                    job.x
                };
                draw_text_ex(
                    &job.text,
                    x,
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

fn actor(
    t: &Texture2D,
    w: f32,
    h: f32,
    area: Rect,
    time: f64,
    durations: &[f64],
    indices: &[usize],
) {
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
    // Fit the frame into its slot and stand it on the slot's bottom edge.
    let scale = (area.w / w)
        .min(area.h / h)
        .min(if h <= 96. { 3. } else { 0.55 });
    let size = vec2(w * scale, h * scale);
    draw_texture_ex(
        t,
        area.x + (area.w - size.x) / 2.,
        area.y + area.h - size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
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
    let mut character_page = 0usize;
    let mut enemy_id = (hero_id + 1) % HEROES.len();
    let mut demon = diagnostic_texture(sprites::SPRITES[enemy_id].bytes, "no_actors");
    let mut choosing = bench.is_none() && !startup_commands;
    let audio = audio::Audio::new(!diagnostic("no_music"), !diagnostic("no_sound"))
        .map_err(|error| eprintln!("Audio unavailable: {error}"))
        .ok();
    rng::srand((get_time() * 1e9) as u64);
    let mut b = fresh();
    let mut selected: Option<(usize, usize)> = None;
    let mut move_hint_page = 0usize;
    let mut hints_hover_open = false;
    // Whether the move hints were on screen last frame; clicks act on what was shown.
    let mut hints_shown = false;
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
    let mut console = debug::Console::new(zh);
    let mut game_over_logged = false;
    let mut awakened = [false; 2];
    let mut reward_skill = None;
    let awake_textures: Vec<_> = sprites::AWAKENED
        .iter()
        .map(|s| diagnostic_texture(s.bytes, "no_actors"))
        .collect();
    let mut charge_damage: Option<(i32, bool)> = None;
    let mut presentation = presentation::Presentation::default();
    let mut turn_log = turns::Log::new();
    macro_rules! rules {
        ($side:expr) => {{
            let side: bool = $side;
            content::Rules {
                awake: awakened[side as usize],
                initial_loot: false,
                opponent_hp: Some(if side { hp } else { enemy }),
                opponent: Some((
                    if side { hero_id } else { enemy_id },
                    awakened[(!side) as usize],
                )),
            }
        }};
    }
    macro_rules! damage {
        ($amount:expr, $target:expr) => {
            damage!($amount, $target, "damage")
        };
        ($amount:expr, $target:expr, $cause:expr) => {{
            let side: bool = $target;
            let old = snapshot!();
            let result = content::take_damage(
                if side { enemy_id } else { hero_id },
                if side { &mut enemy } else { &mut hp },
                &mut awakened[side as usize],
                $amount,
            );
            if result.awakened {
                let id = if side { enemy_id } else { hero_id };
                presentation.awakening(
                    side,
                    if side { old.enemy_hp } else { old.hp },
                    content::health_cap(id, false),
                    content::health_cap(id, true),
                    result.restored,
                );
                console.event(
                    "awakening",
                    &format!(
                        "side={} hero={} damage={} restored={} cap={} cause={}",
                        if side { "enemy" } else { "player" },
                        id,
                        result.damage,
                        result.restored,
                        content::health_cap(id, true),
                        $cause
                    ),
                    &old,
                    &snapshot!(),
                );
            }
            result
        }};
    }
    macro_rules! snapshot {
        () => {
            debug::Snapshot {
                awakened,
                reward_skill,
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
                turns: turn_log.clone(),
            }
        };
    }
    macro_rules! restore {
        ($state:expr) => {{
            let state = $state;
            awakened = state.awakened;
            reward_skill = state.reward_skill;
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
            turn_log = state.turns;
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
        console.sync_language(zh);
        console.input();
        let mut command_move = None;
        let mut command_skill = None;
        let mut command_target = None;
        let mut restart_command = false;
        // Process one command per frame so multiple CLI commands retain their order.
        if let Some(line) = console.commands.pop_front() {
            console.say(&format!("> {line}"));
            console.event("command", &line, &snapshot!(), &snapshot!());
            match debug::parse(&line) {
                Err(error) => console.say(error.message(console.zh)),
                Ok(command) => {
                    use debug::Command;
                    let before = snapshot!();
                    match command {
                        Command::Help => console.help(),
                        Command::Status => {
                            let status = if console.zh {
                                format!("状态：我方 {}({hero_id}) 生命 {hp}/{} 魔力 {mp}/{} · 敌方 {}({enemy_id}) 生命 {enemy}/{} 魔力 {enemy_mp}/{} · 行动方 {} · 待结算 {} 格 · 增幅 {extra_bonus} · 三线 {} · AI {} · 暂停 {}",HEROES[hero_id].zh,HEROES[hero_id].hp,content::mana_cap(hero_id, awakened[0]),HEROES[enemy_id].zh,HEROES[enemy_id].hp,content::mana_cap(enemy_id, awakened[1]),if enemy_turn{"敌方"}else{"玩家"},pending.len(),if armed{"是"}else{"否"},["入门","标准","进阶"][difficulty],if paused{"是"}else{"否"})
                            } else {
                                format!("Hero={} Enemy={} HP={}/{} MP={}/{} turn={} pending={} buff={} armed={} AI={} paused={}",hero_id,enemy_id,hp,enemy,mp,enemy_mp,if enemy_turn{"enemy"}else{"player"},pending.len(),extra_bonus,armed,difficulty,paused)
                            };
                            console.say(&status);
                        }
                        Command::Board => {
                            console.say_tr(
                                "Board: one row per line, tile types 0–6, top-left is 1 1",
                                "棋盘：每行 8 个方块类型（0～6），左上角为 1 1",
                            );
                            for (y, row) in b.iter().enumerate() {
                                console.say(&format!("{}: {:?}", y + 1, row));
                            }
                        }
                        Command::Export => console.export(),
                        Command::Undo(player) => {
                            if let Some((state, action)) = console.undo(player) {
                                restore!(state);
                                console.event(
                                    "rollback",
                                    &format!("Restored before action {action}; command: {line}"),
                                    &before,
                                    &snapshot!(),
                                );
                            } else {
                                console.say_tr("No matching action to undo", "没有可撤回的行动");
                            }
                        }
                        Command::Restart => restart_command = true,
                        Command::Move(a, c) => {
                            if choosing
                                || enemy_turn
                                || action_done
                                || presentation.busy()
                                || !pending.is_empty()
                                || hp == 0
                                || enemy == 0
                                || paused
                            {
                                console.say_tr("Move requires a live, unpaused player turn with no pending resolution","只能在玩家回合、未暂停且没有待结算时交换");
                            } else if !board::adjacent(a, c, hero_id) {
                                console.say_tr(
                                    "Move requires adjacent tiles",
                                    "交换的两个方块必须相邻",
                                );
                            } else {
                                command_move = Some((a, c));
                                console.open = false;
                                targeting = None;
                            }
                        }
                        Command::Cast(slot, target) => {
                            let skill = HEROES[hero_id].skills[slot];
                            if choosing
                                || enemy_turn
                                || action_done
                                || presentation.busy()
                                || !pending.is_empty()
                                || hp == 0
                                || enemy == 0
                                || paused
                            {
                                console.say_tr("Cast requires a live, unpaused player turn with no pending resolution","只能在玩家回合、未暂停且没有待结算时施法");
                            } else if content::payment_in(hero_id, hp, mp, skill, rules!(false))
                                .is_none()
                            {
                                console.say_tr("Not enough mana", "魔力不足");
                            } else if (skill == content::Skill::Strike && extra_bonus > 0)
                                || (skill == content::Skill::Landslide && armed)
                            {
                                console.say_tr(
                                    "Preparation skill is already active",
                                    "准备类技能已经生效",
                                );
                            } else {
                                command_skill = Some(skill);
                                command_target = Some(target);
                                console.open = false;
                                targeting = None;
                            }
                        }
                        Command::Ai(level) => {
                            if choosing {
                                difficulty = level;
                                console.say_tr(
                                    "Pre-battle AI difficulty updated",
                                    "已设置开战前的 AI 难度",
                                );
                            } else {
                                console.say_tr("AI difficulty is locked during battle; return to the main menu to choose a new difficulty","战斗中 AI 难度已锁定，回到主菜单才能重新选择");
                            }
                        }
                        Command::Lang(choice) => console.set_language(choice, zh),
                        command => {
                            let allowed =
                                !action_done && pending.is_empty() && !presentation.busy();
                            if !allowed
                                && !matches!(
                                    command,
                                    Command::Hp(..)
                                        | Command::Mp(..)
                                        | Command::Ai(..)
                                        | Command::Pause(..)
                                )
                            {
                                console.say_tr(
                                    "Wait for resolution, or undo before editing board/turn",
                                    "请等待结算完成，或先 undo，再修改棋盘或回合",
                                );
                            } else {
                                console.checkpoint_edit(before.clone());
                                match command {
                                    Command::Hero(side, id) => {
                                        awakened[side as usize] = false;
                                        if side {
                                            enemy_id = id;
                                            enemy = HEROES[id].hp;
                                            enemy_mp = 0;
                                            demon = diagnostic_texture(
                                                sprites::SPRITES[id].bytes,
                                                "no_actors",
                                            );
                                        } else {
                                            hero_id = id;
                                            hp = HEROES[id].hp;
                                            mp = 0;
                                            knight = diagnostic_texture(
                                                sprites::SPRITES[id].bytes,
                                                "no_actors",
                                            );
                                            preview = menu::Preview::new(id, 0);
                                        }
                                        bonuses[side as usize] = 0;
                                        armed_sides[side as usize] = false;
                                        if enemy_turn == side {
                                            extra_bonus = 0;
                                            armed = false;
                                        }
                                    }
                                    Command::Bonus(side, value) => {
                                        bonuses[side as usize] = value;
                                        if enemy_turn == side {
                                            extra_bonus = value;
                                        }
                                    }
                                    Command::Armed(side, value) => {
                                        armed_sides[side as usize] = value;
                                        if enemy_turn == side {
                                            armed = value;
                                        }
                                    }
                                    Command::SetTile(x, y, kind) => b[y][x] = kind,
                                    Command::LoadBoard(board) => b = board,
                                    Command::Shuffle => b = fresh(),
                                    Command::Seed(seed) => rng::srand(seed),
                                    Command::Hp(side, value) => {
                                        if side {
                                            enemy = value
                                                .min(content::health_cap(enemy_id, awakened[1]))
                                        } else {
                                            hp =
                                                value.min(content::health_cap(hero_id, awakened[0]))
                                        }
                                    }
                                    Command::Mp(side, value) => {
                                        if side {
                                            enemy_mp =
                                                value.min(content::mana_cap(enemy_id, awakened[1]))
                                        } else {
                                            mp = value.min(content::mana_cap(hero_id, awakened[0]))
                                        }
                                    }
                                    Command::Turn(side) => {
                                        bonuses[enemy_turn as usize] = extra_bonus;
                                        armed_sides[enemy_turn as usize] = armed;
                                        enemy_turn = side;
                                        extra_bonus = bonuses[side as usize];
                                        armed = armed_sides[side as usize];
                                        enemy_clock = 0.;
                                        turn_log.next(side);
                                    }
                                    Command::Pause(value) => paused = value,
                                    Command::Resolve => {
                                        pending = matches(&b);
                                        action_done = true;
                                        deadline = 0.28;
                                        reward = true;
                                        chain = 1;
                                        console.open = false;
                                    }
                                    _ => unreachable!(),
                                }
                                selected = None;
                                targeting = None;
                                choosing = false;
                                game_over_logged = false;
                                console.event("debug_edit", &line, &before, &snapshot!());
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
        // Buttons on the victory/defeat box, which is drawn only while not paused.
        let mut over_click = false;
        if (hp == 0 || enemy == 0)
            && !paused
            && !choosing
            && !pause_click
            && !was_console_open
            && !console.open
            && !ui_consumed
            && interface.panel.is_none()
            && is_mouse_button_pressed(MouseButton::Left)
        {
            if ui::game_over_rect(0).contains(pointer) {
                over_click = true;
                restart_command = true;
            } else if ui::game_over_rect(1).contains(pointer) {
                over_click = true;
                choosing = true;
                interface.opponent_step = false;
                targeting = None;
                selected = None;
            }
        }
        let block_input =
            was_console_open || console.open || ui_consumed || pause_click || over_click;
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
            awakened = [false; 2];
            reward_skill = None;
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
            turn_log = turns::Log::new();
            console.reset_history();
            console.event("restart", "Restart battle", &before, &snapshot!());
        }
        if choosing {
            if let Some(audio) = &audio {
                audio.playing(false);
            }
            if !interface.opponent_step {
                if clicked(MouseButton::Left) && menu::page_rect(0).contains(pointer) {
                    character_page = character_page.saturating_sub(1);
                }
                if clicked(MouseButton::Left) && menu::page_rect(1).contains(pointer) {
                    character_page = (character_page + 1).min(2);
                }
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
                awakened = [false; 2];
                reward_skill = None;
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
                turn_log = turns::Log::new();
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
                    menu::draw(
                        hero_id,
                        zh,
                        &knight,
                        &thumbs,
                        &tiles,
                        &preview,
                        pointer,
                        character_page,
                        &awake_textures[hero_id],
                    );
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
                let hit = damage!(damage, !enemy_actor);
                let actual = hit.damage;
                turn_log.current.add(0, actual, 0);
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
                turn_log.next(enemy_turn);
                // The last turn has its own row; prompt the new actor afresh.
                note = "Select two adjacent tiles";
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
                let automatic = no_swaps
                    && content::must_refresh_in(&b, id, health, mana, armed, rules!(enemy_turn));
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
                    turn_log.current.actions.push(turns::Action::Reroll);
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
                    let rect = player_skill_rect(i);
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
                cast = strategy::choose_skill_in(
                    &b,
                    enemy_id,
                    enemy,
                    enemy_mp,
                    difficulty,
                    armed,
                    extra_bonus,
                    rules!(true),
                );
                if cast.is_none() {
                    if let Some((a, c)) = strategy::choose_move_in(
                        &b,
                        enemy,
                        enemy_id,
                        enemy_mp,
                        hero_id,
                        difficulty,
                        rules!(true),
                    )
                    .or_else(|| armed.then_some(((0, 0), (1, 0))))
                    {
                        let before = enemy_before.clone();
                        console.checkpoint(before.clone());
                        presentation.swap(&before.board, a, c, armed, enemy_turn, enemy_id);
                        turn_log.current.actions.push(turns::Action::Swap {
                            from: a,
                            to: c,
                            columns: enemy_id == 0 && !armed && a.1 == c.1,
                        });
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
                        turn_log.current.actions.push(turns::Action::Reroll);
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
                        || content::payment_in(hero_id, hp, mp, skill, rules!(false))
                            .is_some_and(|p| p.blood > 0))
                        && content::payment_in(hero_id, hp, mp, skill, rules!(false)).is_some()
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
                let cast_rules = rules!(enemy_turn);
                let id = if enemy_turn { enemy_id } else { hero_id };
                let actor_hp = if enemy_turn { enemy } else { hp };
                let actor_mp = if enemy_turn { enemy_mp } else { mp };
                if let Some(pay) =
                    content::payment_in(id, actor_hp, actor_mp, skill, rules!(enemy_turn))
                {
                    console.checkpoint(before.clone());
                    if enemy_turn {
                        enemy_mp -= pay.mana;
                        damage!(pay.blood, true, "blood_payment");
                    } else {
                        mp -= pay.mana;
                        damage!(pay.blood, false, "blood_payment");
                    }
                    cast_this_frame = true;
                    move_flash = None;
                    last_cast = Some(skill);
                    last_cast_enemy = enemy_turn;
                    cast_time = 1.4;
                    let target = if enemy_turn {
                        strategy::choose_target_in(
                            skill,
                            &b,
                            enemy_id,
                            before.enemy_hp,
                            before.enemy_mp,
                            extra_bonus,
                            cast_rules,
                        )
                    } else {
                        command_target.or(board_target).unwrap_or((3, 3))
                    };
                    reward_skill = Some(skill);
                    skill_flash = content::target_cells(skill, &b, target);
                    let e = content::effect_at_in(
                        skill,
                        &mut b,
                        target,
                        id,
                        actor_hp,
                        actor_mp,
                        rules!(enemy_turn),
                    );
                    let direct_damage = if skill == content::Skill::Charge {
                        charge_damage = Some((e.damage, enemy_turn));
                        presentation.charge(target.0, presentation::CHARGE_TIME);
                        0
                    } else {
                        e.damage
                    };
                    if enemy_turn {
                        enemy = (enemy + e.hp).min(content::health_cap(enemy_id, awakened[1]));

                        enemy_mp = (enemy_mp + e.mp).min(content::mana_cap(enemy_id, awakened[1]));
                    } else {
                        hp = (hp + e.hp).min(content::health_cap(hero_id, awakened[0]));

                        mp = (mp + e.mp).min(content::mana_cap(hero_id, awakened[0]));
                    }
                    let hit = damage!(direct_damage, !enemy_turn);
                    let actual_heal = if enemy_turn {
                        enemy - before.enemy_hp
                    } else {
                        hp - before.hp
                    };
                    let actual_damage = hit.damage;
                    let actual_mana = if enemy_turn {
                        enemy_mp - before.enemy_mp + pay.mana
                    } else {
                        mp - before.mp + pay.mana
                    };
                    turn_log.current.actions.push(turns::Action::Skill(skill));
                    turn_log
                        .current
                        .add(actual_heal, actual_damage, actual_mana);
                    let caster = &HEROES[if enemy_turn { enemy_id } else { hero_id }];
                    feedback = format!(
                        "{} · {}  {}",
                        if zh { caster.zh } else { caster.en },
                        skill.name(zh),
                        reward_summary(zh, actual_heal, actual_damage, actual_mana)
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
                        let (hp_now, mp_now) = if enemy_turn {
                            (enemy, enemy_mp)
                        } else {
                            (hp, mp)
                        };
                        feedback = if zh {
                            format!(
                                "血肉施法 生命 -{} · 生命 {actor_hp} → {hp_now} · 魔力 {actor_mp} → {mp_now}",
                                pay.blood
                            )
                        } else {
                            format!(
                                "Flesh casting -{} HP · HP {actor_hp} → {hp_now} · MP {actor_mp} → {mp_now}",
                                pay.blood
                            )
                        };
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
            // Clicking a listed exchange (a reward-panel row, or a numbered badge on the
            // board while H is held) plays it through the normal player swap below.
            let mut hint_number = None;
            if hints_shown
                && command_move.is_none()
                && pending.is_empty()
                && !enemy_turn
                && !action_done
                && targeting.is_none()
                && !cast_this_frame
                && clicked(MouseButton::Left)
            {
                // Same ranking as the drawn list, so row and badge numbers match.
                let hints: Vec<Move> = content::ranked_moves_in(
                    &b,
                    hero_id,
                    hp,
                    mp,
                    enemy,
                    extra_bonus,
                    armed,
                    rules!(false),
                )
                .into_iter()
                .map(|(m, _)| m)
                .collect();
                if let Some(i) = hint_row_at(pointer, move_hint_page, hints.len()).or_else(|| {
                    is_key_down(KeyCode::H)
                        .then(|| hint_badge_at(pointer, &hints))
                        .flatten()
                }) {
                    command_move = Some(hints[i]);
                    hint_number = Some(i + 1);
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
                        content::clear_rewards_in(
                            id,
                            if enemy_turn { enemy } else { hp },
                            if enemy_turn { enemy_mp } else { mp },
                            &b,
                            &pending,
                            chain.max(1) as usize,
                            extra_bonus,
                            content::Rules {
                                initial_loot: reward_skill == Some(content::Skill::Greedy),
                                ..rules!(enemy_turn)
                            },
                        )
                    } else {
                        (0, 0, 0)
                    };
                    reward_skill = None;
                    if reward {
                        extra_bonus = 0;
                    }
                    for &(x, y) in &pending {
                        b[y][x] = 255;
                    }
                    if enemy_turn {
                        enemy = (enemy + heal).min(content::health_cap(enemy_id, awakened[1]));

                        enemy_mp = (enemy_mp + mana).min(content::mana_cap(enemy_id, awakened[1]));
                    } else {
                        hp = (hp + heal).min(content::health_cap(hero_id, awakened[0]));

                        mp = (mp + mana).min(content::mana_cap(hero_id, awakened[0]));
                    }
                    let hit = damage!(damage, !enemy_turn);
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
                    let actual_damage = hit.damage;
                    let actual_mana = if enemy_turn {
                        enemy_mp - before.enemy_mp
                    } else {
                        mp - before.mp
                    };
                    turn_log
                        .current
                        .add(actual_heal, actual_damage, actual_mana);
                    turn_log.current.chain = turn_log.current.chain.max(chain.max(1));
                    if reward {
                        feedback = format!(
                            "{} · {}",
                            if enemy_turn {
                                tr(zh, "Enemy", "敌方")
                            } else {
                                tr(zh, "You", "我方")
                            },
                            reward_summary(zh, actual_heal, actual_damage, actual_mana)
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
                            turn_log.current.actions.push(turns::Action::Swap {
                                from: a,
                                to: c,
                                columns: hero_id == 0 && !before.armed && a.1 == c.1,
                            });
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
        for enemy_side in [false, true] {
            let panel = side_panel_rect(enemy_side);
            let active_side = enemy_side == enemy_turn && hp > 0 && enemy > 0;
            let accent = if enemy_side { ORANGE } else { SKYBLUE };
            let frame = if active_side {
                accent
            } else {
                Color::new(0.25, 0.28, 0.35, 0.6)
            };
            if active_side {
                draw_rectangle(
                    panel.x,
                    panel.y,
                    panel.w,
                    panel.h,
                    Color::new(accent.r, accent.g, accent.b, 0.07),
                );
            }
            draw_rectangle_lines(
                panel.x,
                panel.y,
                panel.w,
                panel.h,
                if active_side { 3. } else { 1. },
                frame,
            );
            let (id, health, mana) = if enemy_side {
                (enemy_id, enemy, enemy_mp)
            } else {
                (hero_id, hp, mp)
            };
            let x = panel.x + 12.;
            let name = if zh { HEROES[id].zh } else { HEROES[id].en };
            let room = panel.w - 24. - if active_side { 88. } else { 0. };
            label(name, x, 121., fit_size(name, 26., room), WHITE);
            if active_side {
                let tag = Rect::new(panel.x + panel.w - 92., 99., 80., 26.);
                draw_rectangle(
                    tag.x,
                    tag.y,
                    tag.w,
                    tag.h,
                    Color::new(accent.r * 0.3, accent.g * 0.3, accent.b * 0.3, 1.),
                );
                label_centered(
                    tr(zh, "ACTIVE", "行动中"),
                    tag.x + tag.w / 2.,
                    tag.y + 19.,
                    17.,
                    accent,
                );
            }
            let health_color = if enemy_side {
                Color::new(1., 0.62, 0.62, 1.)
            } else {
                Color::new(0.6, 1., 0.6, 1.)
            };
            for (row, (name, value, max, color)) in [
                (
                    tr(zh, "HP", "生命"),
                    health,
                    content::health_cap(id, awakened[enemy_side as usize]),
                    health_color,
                ),
                (
                    tr(zh, "MP", "魔力"),
                    mana,
                    content::mana_cap(id, awakened[enemy_side as usize]),
                    SKYBLUE,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 150. + row as f32 * 38.;
                let (shown, shown_max, extent) = if row == 0 {
                    presentation.health_bar(enemy_side, value, max)
                } else {
                    presentation.mana_bar(enemy_side, value, content::mana_cap(id, false), max)
                };
                let amount = format!("{} / {}", shown.round() as i32, shown_max.round() as i32);
                label(name, x, y, 18., color);
                label(
                    &amount,
                    panel.x + panel.w - 12. - label_width(&amount, 18.),
                    y,
                    18.,
                    color,
                );
                let visual_max = if row == 1 {
                    content::mana_cap(id, true) as f32
                } else {
                    shown_max
                };
                let fill = (shown / visual_max.max(1.)).clamp(0., 1.) * extent;
                draw_rectangle(
                    x,
                    y + 7.,
                    200. * extent,
                    9.,
                    Color::new(0.1, 0.12, 0.17, 1.),
                );
                draw_rectangle(x, y + 7., 200. * fill, 9., color);
                if row == 1 && content::mana_cap(id, false) < content::mana_cap(id, true) {
                    let lock_x = x + 200. * content::mana_cap(id, false) as f32 / visual_max;
                    if !awakened[enemy_side as usize] {
                        draw_rectangle(
                            lock_x,
                            y + 7.,
                            200. - (lock_x - x),
                            9.,
                            Color::new(0.16, 0.19, 0.25, 1.),
                        );
                        draw_lock(lock_x - 5.5, y + 3., SKYBLUE);
                    } else if let Some(t) = presentation.awakening_progress(enemy_side) {
                        draw_breaking_lock(lock_x - 5.5, y + 3., t, SKYBLUE);
                    }
                }
            }
            let show_awake =
                awakened[enemy_side as usize] && presentation.awakened_visible(enemy_side);
            let sprite = if show_awake {
                &sprites::AWAKENED[id]
            } else {
                &sprites::SPRITES[id]
            };
            actor(
                if show_awake {
                    &awake_textures[id]
                } else if enemy_side {
                    &demon
                } else {
                    &knight
                },
                sprite.w,
                sprite.h,
                sprite_rect(enemy_side),
                get_time(),
                sprite.durations,
                sprite.indices,
            );
        }
        // Title centered over the player's sidebar.
        let title = tr(zh, "BATTLE BLOCKS", "方块战斗");
        let sidebar = side_panel_rect(false);
        label_centered(
            title,
            sidebar.x + sidebar.w / 2.,
            57.,
            fit_size(title, 32., sidebar.w - 24.),
            GOLD,
        );
        interface.draw_top(false, muted, paused, pointer, zh);
        // Two-row turn indicator above the board. The current turn, next to the board,
        // stands out: who acts, what was done so far, its running result and the cascade
        // depth. The previous turn sits above it in muted colors.
        let current = current_turn_rect();
        let over = hp == 0 || enemy == 0;
        let turn_color = if over {
            GOLD
        } else if enemy_turn {
            ORANGE
        } else {
            SKYBLUE
        };
        draw_rectangle(
            current.x,
            current.y,
            current.w,
            current.h,
            Color::new(0.07, 0.10, 0.16, 0.98),
        );
        draw_rectangle_lines(current.x, current.y, current.w, current.h, 2., turn_color);
        let chip = Rect::new(current.x + 5., current.y + 5., 104., current.h - 10.);
        draw_rectangle(
            chip.x,
            chip.y,
            chip.w,
            chip.h,
            Color::new(
                turn_color.r * 0.3,
                turn_color.g * 0.3,
                turn_color.b * 0.3,
                1.,
            ),
        );
        label_centered(
            if enemy == 0 {
                tr(zh, "Victory", "胜利")
            } else if hp == 0 {
                tr(zh, "Defeat", "失败")
            } else if enemy_turn {
                tr(zh, "Enemy turn", "敌方回合")
            } else {
                tr(zh, "Your turn", "你的回合")
            },
            chip.x + chip.w / 2.,
            chip.y + 23.,
            18.,
            turn_color,
        );
        let chain_text = format!("{} {chain}", tr(zh, "Chain", "连锁"));
        let chain_x = current.x + current.w - 14. - label_width(&chain_text, 18.);
        label(
            &chain_text,
            chain_x,
            current.y + 28.,
            18.,
            if chain > 1 {
                GOLD
            } else if chain == 1 {
                WHITE
            } else {
                GRAY
            },
        );
        let mut text_x = chip.x + chip.w + 12.;
        if let Some(skill) = turn_log.current.last_skill() {
            icons::draw(skill, text_x, current.y + 10., 22.);
            text_x += 30.;
        }
        let phase = if over {
            tr(zh, "Battle over", "战斗结束").to_string()
        } else if presentation.charging() {
            tr(zh, "Charging", "冲锋中").to_string()
        } else if let Some(swap) = presentation.swapping() {
            if swap {
                tr(zh, "Swapping", "正在交换")
            } else {
                tr(zh, "Tiles falling", "方块掉落")
            }
            .to_string()
        } else if !pending.is_empty() {
            tr(zh, "Clearing matches", "正在消除").to_string()
        } else if action_done {
            tr(zh, "Action resolved", "行动结算完成").to_string()
        } else if enemy_turn {
            tr(zh, "Enemy is choosing", "敌方思考中").to_string()
        } else if let Some(skill) = targeting {
            format!(
                "{} · {}",
                skill.name(zh),
                tr(zh, "click a target", "点击棋盘选择目标")
            )
        } else {
            localized_note(zh, note).to_string()
        };
        // Actions so far, then the running result, or the phase until something changes.
        let actions = turn_log.current.actions_text(zh);
        let mut segments = Vec::new();
        if !actions.is_empty() {
            segments.push((actions, WHITE));
            segments.push((" · ".to_string(), GRAY));
        }
        segments.push(match turn_log.current.results_text(zh) {
            Some(results) => (results, GOLD),
            None => (phase, WHITE),
        });
        label_segments(
            &segments,
            text_x,
            current.y + 28.,
            19.,
            chain_x - 12. - text_x,
        );
        let previous = previous_turn_rect();
        draw_rectangle(
            previous.x,
            previous.y,
            previous.w,
            previous.h,
            Color::new(0.05, 0.06, 0.09, 0.92),
        );
        draw_rectangle_lines(
            previous.x,
            previous.y,
            previous.w,
            previous.h,
            1.,
            Color::new(0.25, 0.28, 0.35, 0.8),
        );
        label_centered(
            tr(zh, "Last turn", "上一回合"),
            chip.x + chip.w / 2.,
            previous.y + 21.,
            15.,
            GRAY,
        );
        let segments = match &turn_log.previous {
            Some(turn) => {
                let mut segments = vec![
                    if turn.enemy {
                        (tr(zh, "Enemy", "敌方").to_string(), ORANGE)
                    } else {
                        (tr(zh, "You", "我方").to_string(), SKYBLUE)
                    },
                    (tr(zh, ": ", "：").to_string(), GRAY),
                ];
                let actions = turn.actions_text(zh);
                if !actions.is_empty() {
                    segments.push((actions, LIGHTGRAY));
                    segments.push((" · ".to_string(), GRAY));
                }
                segments.push((
                    turn.results_text(zh)
                        .unwrap_or_else(|| tr(zh, "no gain", "无收益").to_string()),
                    LIGHTGRAY,
                ));
                if turn.chain > 1 {
                    segments.push((
                        format!(" · {} {}", tr(zh, "Chain", "连锁"), turn.chain),
                        GRAY,
                    ));
                }
                segments
            }
            None => vec![(tr(zh, "None yet", "暂无").to_string(), GRAY)],
        };
        let text_x = chip.x + chip.w + 12.;
        label_segments(
            &segments,
            text_x,
            previous.y + 21.,
            15.,
            previous.x + previous.w - 14. - text_x,
        );
        label(
            &format!(
                "{}{}",
                tr(zh, "AI: ", "AI 难度："),
                [
                    tr(zh, "Beginner", "入门"),
                    tr(zh, "Standard", "标准"),
                    tr(zh, "Advanced", "进阶"),
                ][difficulty]
            ),
            808.,
            452.,
            18.,
            GOLD,
        );
        // Below the board frame; the hint legend uses this line while hints are shown.
        if feedback_time > 0. && targeting.is_none() && !hints_shown {
            label(&feedback, 262., 670., 17., GOLD);
        }
        if !diagnostic("legacy_batches") {
            for y in 0..N {
                for x in 0..N {
                    let pos = vec2(256. + x as f32 * 64., 110. + y as f32 * 64.);
                    procedural_cell(pos.x, pos.y);
                }
            }
        }
        let hints_ready = targeting.is_none()
            && !action_done
            && pending.is_empty()
            && !presentation.busy()
            && !block_input
            && !paused
            && interface.panel.is_none()
            && hp > 0
            && enemy > 0;
        let hints_rect = move_hints_rect();
        // The continuous hover region bridges the entry button and its panel,
        // so a mouse/trackpad can reach rows and pagination without holding H.
        let hints_region = Rect::new(790., 208., 220., 484.);
        hints_hover_open = hints_ready
            && !key(KeyCode::Escape)
            && (hints_rect.contains(pointer)
                || ((hints_hover_open || is_key_down(KeyCode::H))
                    && hints_region.contains(pointer)));
        let show_moves =
            hints_ready && !key(KeyCode::Escape) && (is_key_down(KeyCode::H) || hints_hover_open);
        hints_shown = show_moves;
        // Hints can be played only on the player's own turn; the enemy's are view-only.
        let hints_playable = show_moves && !enemy_turn;
        let hint_actor = if enemy_turn { enemy_id } else { hero_id };
        let player_hover = if !show_moves
            && !enemy_turn
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
                .find(|(i, _)| player_skill_rect(*i).contains(pointer))
                .map(|(_, s)| (*s, false))
        } else {
            None
        };
        let enemy_hover = if !show_moves && !console.open {
            HEROES[enemy_id]
                .skills
                .iter()
                .enumerate()
                .find(|(i, _)| enemy_skill_rect(*i).contains(pointer))
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
        if !show_moves && targeting.is_none() && !enemy_turn && !action_done {
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
        // Hints are derived only from the settled board; drawing never changes
        // selection, RNG, turn state, resources or the action log.
        let hint_health = if enemy_turn { enemy } else { hp };
        let hint_mana = if enemy_turn { enemy_mp } else { mp };
        let hint_opponent = if enemy_turn { hp } else { enemy };
        // Best total reward first; numbers on the board follow the same order.
        let (legal_hints, move_previews): (Vec<Move>, Vec<_>) = if show_moves {
            content::ranked_moves_in(
                &b,
                hint_actor,
                hint_health,
                hint_mana,
                hint_opponent,
                extra_bonus,
                armed,
                rules!(enemy_turn),
            )
            .into_iter()
            .unzip()
        } else {
            (Vec::new(), Vec::new())
        };
        let pages = legal_hints.len().div_ceil(7).max(1);
        let previous_page_rect = Rect::new(798., 590., 52., 24.);
        let next_page_rect = Rect::new(950., 590., 52., 24.);
        if show_moves {
            let wheel = mouse_wheel().1;
            if wheel < 0.
                || key(KeyCode::Right)
                || (clicked(MouseButton::Left) && next_page_rect.contains(pointer))
            {
                move_hint_page = (move_hint_page + 1).min(pages - 1);
            }
            if wheel > 0.
                || key(KeyCode::Left)
                || (clicked(MouseButton::Left) && previous_page_rect.contains(pointer))
            {
                move_hint_page = move_hint_page.saturating_sub(1);
            }
            move_hint_page = move_hint_page.min(pages - 1);
        } else {
            move_hint_page = 0;
        }
        let row_focus = show_moves
            .then(|| hint_row_at(pointer, move_hint_page, legal_hints.len()))
            .flatten();
        let arrow_focus = show_moves
            .then(|| hint_badge_at(pointer, &legal_hints))
            .flatten();
        let focused_hint = row_focus.or(arrow_focus);
        if let Some(i) = arrow_focus {
            move_hint_page = i / 7;
        }
        if show_moves {
            for (i, &(a, c)) in legal_hints.iter().enumerate() {
                presentation::draw_move_hint(
                    a,
                    c,
                    hint_actor == 0 && !armed && a.1 == c.1,
                    i + 1,
                    focused_hint == Some(i),
                );
            }
            if let Some(p) = focused_hint.and_then(|i| move_previews[i].as_ref()) {
                for &(x, y) in &p.cells {
                    draw_rectangle_lines(
                        259. + x as f32 * 64.,
                        113. + y as f32 * 64.,
                        58.,
                        58.,
                        3.,
                        GREEN,
                    );
                }
            }
            if !legal_hints.is_empty() {
                label(
                    if hints_playable {
                        tr(
                            zh,
                            "Blue: tiles · Gold: columns · hold H + click a number",
                            "蓝色：两格交换 / 金色：整列交换 · 按住 H 点编号即可执行",
                        )
                    } else {
                        tr(
                            zh,
                            "Blue: tiles / Gold: whole columns",
                            "蓝色：两格交换 / 金色：整列交换",
                        )
                    },
                    256.,
                    674.,
                    17.,
                    GOLD,
                );
            }
        }
        if targeting.is_none() {
            draw_rectangle(
                hints_rect.x,
                hints_rect.y,
                hints_rect.w,
                hints_rect.h,
                Color::new(0.10, 0.17, 0.23, 1.),
            );
            draw_rectangle_lines(
                hints_rect.x,
                hints_rect.y,
                hints_rect.w,
                hints_rect.h,
                2.,
                if show_moves { GOLD } else { GRAY },
            );
            label(
                tr(zh, "Moves · hold H", "可行移动 · 按住 H"),
                hints_rect.x + 8.,
                hints_rect.y + 19.,
                16.,
                if hints_ready { SKYBLUE } else { GRAY },
            );
            let detail = if show_moves {
                format!(
                    "{} · {} {}",
                    if zh {
                        HEROES[hint_actor].zh
                    } else {
                        HEROES[hint_actor].en
                    },
                    legal_hints.len(),
                    tr(zh, "moves", "种交换")
                )
            } else {
                tr(
                    zh,
                    "Hover to list · click to play",
                    "悬停查看 · 点击条目执行",
                )
                .to_string()
            };
            label(
                &detail,
                hints_rect.x + 8.,
                hints_rect.y + 36.,
                13.,
                if show_moves { GOLD } else { GRAY },
            );
        }

        if !console.open && interface.panel.is_none() && !paused && !show_moves {
            for side in [false, true] {
                let panel = side_panel_rect(side);
                let id = if side { enemy_id } else { hero_id };
                let bar = Rect::new(panel.x + 12., 133., 200., 40.);
                if !awakened[side as usize] {
                    let tx = bar.x + 200. * content::AWAKENING_LINES[id] as f32 / 100.;
                    let cx = bar.x + 200. * content::AWAKENING_CAPS[id] as f32 / 100.;
                    // Independent upper/lower lanes preserve exact values even when equal.
                    draw_line(tx, 153., tx, 157., 1., GOLD);
                    draw_diamond(tx, 152., 5., GOLD, true);
                    draw_line(cx, 166., cx, 170., 1., SKYBLUE);
                    draw_diamond(cx, 171., 4., SKYBLUE, false);
                }
                if bar.contains(pointer) {
                    let x = if side { 548. } else { 256. };
                    let r = Rect::new(
                        x,
                        128.,
                        220.,
                        if awakened[side as usize] { 98. } else { 146. },
                    );
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(25, 33, 46, 255));
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1., SKYBLUE);
                    let text = if awakened[side as usize] {
                        if zh {
                            format!("当前生命上限：{}", content::health_cap(id, true))
                        } else {
                            format!("Current HP cap: {}", content::health_cap(id, true))
                        }
                    } else if zh {
                        format!("觉醒点：{}（{}%）\n觉醒后上限：{}（{}%）\n受伤触线后恢复生命并强化\n{}", content::awakening_line(id), content::AWAKENING_LINES[id], content::health_cap(id,true),content::AWAKENING_CAPS[id],content::awakening_effect(id,zh))
                    } else {
                        format!("Awakening: {} HP ({}%)\nNew HP cap: {} ({}%)\nDamage triggers healing and power\n{}",content::awakening_line(id),content::AWAKENING_LINES[id],content::health_cap(id,true),content::AWAKENING_CAPS[id],content::awakening_effect(id,zh))
                    };
                    for (i, line) in wrap_preview_text(&text, 14.).iter().enumerate() {
                        label(line, x + 10., 150. + i as f32 * 18., 14., GOLD);
                    }
                }
                if content::mana_cap(id, false) < content::mana_cap(id, true)
                    && !awakened[side as usize]
                    && Rect::new(panel.x + 12., 174., 200., 40.).contains(pointer)
                {
                    let x = if side { 548. } else { 256. };
                    draw_rectangle(x, 176., 220., 64., Color::from_rgba(25, 33, 46, 255));
                    label(
                        &if zh {
                            format!("魔力上限锁定：{}", content::mana_cap(id, false))
                        } else {
                            format!("MP cap locked: {}", content::mana_cap(id, false))
                        },
                        x + 10.,
                        199.,
                        16.,
                        SKYBLUE,
                    );
                    label(
                        &if zh {
                            format!("觉醒后解锁至{}魔力", content::mana_cap(id, true))
                        } else {
                            format!("Awakening unlocks {} MP", content::mana_cap(id, true))
                        },
                        x + 10.,
                        223.,
                        15.,
                        GOLD,
                    );
                }
            }
        }
        presentation.draw_awakening(zh);
        if presentation.charging() {
            let id = if last_cast_enemy { enemy_id } else { hero_id };
            let is_awake = awakened[last_cast_enemy as usize];
            let asset = if is_awake {
                &sprites::AWAKENED[id]
            } else {
                &sprites::SPRITES[id]
            };
            presentation.draw_charge(
                if is_awake {
                    &awake_textures[id]
                } else if last_cast_enemy {
                    &demon
                } else {
                    &knight
                },
                asset.w,
                asset.h,
                asset.indices,
            );
        }
        if let Some(skill) = last_cast.filter(|s| cast_time > 0. && *s != content::Skill::Charge) {
            let progress = 1. - cast_time as f32 / 1.4;
            effects::draw(skill, &skill_flash, progress, last_cast_enemy);
        }
        for (i, skill) in HEROES[enemy_id].skills.iter().enumerate() {
            let r = enemy_skill_rect(i);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.12, 0.15, 0.2, 1.));
            if r.contains(pointer) {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., ORANGE);
            }
            icons::draw(*skill, r.x + 5., r.y + 5., 28.);
            let color =
                if content::payment_in(enemy_id, enemy, enemy_mp, *skill, rules!(true)).is_some() {
                    GOLD
                } else {
                    GRAY
                };
            // Cost right-aligned; the name fits into the space left of it.
            let cost = cost_text(zh, content::skill_cost(enemy_id, awakened[1], *skill), 0);
            let cost_width = label_width(&cost, 15.);
            label(&cost, r.x + r.w - 7. - cost_width, r.y + 25., 15., color);
            let name = skill.name(zh);
            label(
                name,
                r.x + 40.,
                r.y + 25.,
                fit_size(name, 17., r.w - 54. - cost_width),
                color,
            );
        }
        for (i, skill) in HEROES[hero_id].skills.iter().enumerate() {
            let rect = player_skill_rect(i);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if content::payment_in(hero_id, hp, mp, *skill, rules!(false)).is_some() {
                    Color::new(0.2, 0.28, 0.35, 1.)
                } else {
                    Color::new(0.1, 0.12, 0.16, 1.)
                },
            );
            if targeting == Some(*skill) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., GOLD);
            } else if player_hover.is_some_and(|(s, _)| s == *skill) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., SKYBLUE);
            }
            icons::draw(*skill, rect.x + 8., rect.y + 8., 26.);
            let title = format!("{}: {}", i + 1, skill.name(zh));
            label(
                &title,
                rect.x + 40.,
                rect.y + 28.,
                fit_size(&title, 21., rect.w - 48.),
                WHITE,
            );
            label(
                &if hero_id == 6 && mp < content::skill_cost(hero_id, awakened[0], *skill) {
                    cost_text(
                        zh,
                        mp,
                        content::blood_cost(
                            hero_id,
                            awakened[0],
                            content::skill_cost(hero_id, awakened[0], *skill) - mp,
                        ),
                    )
                } else {
                    cost_text(zh, content::skill_cost(hero_id, awakened[0], *skill), 0)
                },
                rect.x + 8.,
                rect.y + 50.,
                17.,
                if hero_id == 6 && mp < content::skill_cost(hero_id, awakened[0], *skill) {
                    RED
                } else {
                    SKYBLUE
                },
            );
            for (line, text) in content::skill_description(*skill, awakened[0], zh)
                .lines()
                .enumerate()
            {
                // Shrink a long line to the box width instead of letting it spill out.
                label(
                    text,
                    rect.x + 8.,
                    rect.y + 72. + line as f32 * 18.,
                    fit_size(text, 16., rect.w - 16.),
                    LIGHTGRAY,
                );
            }
        }
        // Each passive has its own identity, icon, state and hit target.
        let mut passive_popup = None;
        for enemy_side in [false, true] {
            let id = if enemy_side { enemy_id } else { hero_id };
            let awake = awakened[enemy_side as usize];
            let area = passive_rect(enemy_side);
            let (health, mana) = if enemy_side {
                (enemy, enemy_mp)
            } else {
                (hp, mp)
            };
            for (i, &passive) in content::passives(id, awake).iter().enumerate() {
                let rect = Rect::new(
                    area.x + (i % 2) as f32 * 106.,
                    area.y + (i / 2) as f32 * 26.,
                    102.,
                    24.,
                );
                let hovered = !show_moves
                    && targeting.is_none()
                    && hover_context.is_none()
                    && !block_input
                    && !paused
                    && interface.panel.is_none()
                    && rect.contains(pointer);
                let status = passive.status(id, health, mana, awake, zh);
                let color = if status.is_some_and(|(_, on)| !on) {
                    GRAY
                } else {
                    GOLD
                };
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::from_rgba(43, 40, 29, 255),
                );
                if hovered {
                    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., GOLD);
                }
                icons::draw_passive(passive, rect.x + 3., rect.y + 3., 18.);
                let name = if matches!(
                    passive,
                    content::Passive::BloodFury | content::Passive::BoilingBlood
                ) {
                    format!(
                        "{}{}",
                        passive.short_name(zh),
                        if status.is_some_and(|(_, on)| on) {
                            "+25%"
                        } else {
                            tr(zh, " · off", "·未触发")
                        }
                    )
                } else {
                    passive.short_name(zh).to_string()
                };
                label(
                    &name,
                    rect.x + 25.,
                    rect.y + 16.,
                    fit_size(&name, 13., 73.),
                    color,
                );
                if matches!(
                    passive,
                    content::Passive::BloodFury | content::Passive::BoilingBlood
                ) && status.is_some_and(|(_, on)| on)
                {
                    draw_circle(rect.x + rect.w - 4., rect.y + 4., 2.5, ORANGE);
                }
                if hovered {
                    passive_popup = Some((passive, id, health, mana, awake));
                }
            }
        }
        if let Some((passive, id, health, mana, awake)) = passive_popup {
            let lines = wrap_preview_text(passive.description(zh), 18.);
            let status = passive.status(id, health, mana, awake, zh);
            let relation = passive.replaces().map(|old| {
                if zh {
                    format!("替代：{}", old.name(zh))
                } else {
                    format!("Replaces: {}", old.name(zh))
                }
            });
            let height = 65.
                + lines.len() as f32 * 26.
                + if status.is_some() { 26. } else { 0. }
                + if relation.is_some() { 24. } else { 0. };
            flush_labels();
            draw_rectangle(270., 225., 485., height, Color::from_rgba(15, 23, 34, 255));
            icons::draw_passive(passive, 286., 239., 28.);
            label(passive.name(zh), 326., 261., 22., GOLD);
            let mut y = 292.;
            for line in &lines {
                label(line, 287., y, 18., WHITE);
                y += 26.;
            }
            if let Some((text, on)) = status {
                label(text, 287., y, 17., if on { ORANGE } else { GRAY });
                y += 26.;
            }
            if let Some(text) = relation {
                label(&text, 287., y, 15., SKYBLUE);
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
        if show_moves {
            flush_labels();
            draw_rectangle(790., 208., 220., 430., Color::new(0.04, 0.07, 0.12, 0.98));
            draw_rectangle_lines(790., 208., 220., 430., 2., SKYBLUE);
            label(tr(zh, "Move rewards", "移动收益"), 800., 232., 20., GOLD);
            label(
                if zh {
                    HEROES[hint_actor].zh
                } else {
                    HEROES[hint_actor].en
                },
                800.,
                252.,
                15.,
                WHITE,
            );
            label(
                tr(
                    zh,
                    "First clear; no refill chains",
                    "首轮收益，不含补充连锁",
                ),
                800.,
                270.,
                13.,
                GRAY,
            );
            for i in move_hint_page * 7..((move_hint_page + 1) * 7).min(legal_hints.len()) {
                let row_y = 278. + (i % 7) as f32 * 44.;
                if focused_hint == Some(i) {
                    draw_rectangle(798., row_y, 204., 43., Color::new(0.16, 0.25, 0.30, 1.));
                    if hints_playable {
                        draw_rectangle_lines(798., row_y, 204., 43., 1., GOLD);
                        label(tr(zh, "click", "点击执行"), 950., row_y + 35., 12., GOLD);
                    }
                }
                let (a, c) = legal_hints[i];
                let column = hint_actor == 0 && !armed && a.1 == c.1;
                label(
                    &format!("#{} {}", i + 1, turns::swap_text(zh, a, c, column)),
                    800.,
                    row_y + 16.,
                    13.,
                    if column { GOLD } else { SKYBLUE },
                );
                if let Some(p) = &move_previews[i] {
                    label(
                        &format!(
                            "{}{}  {}{}  {}{}",
                            tr(zh, "Dmg ", "伤"),
                            p.damage,
                            tr(zh, "HP+", "治"),
                            p.heal,
                            tr(zh, "MP+", "魔"),
                            p.mana_gain
                        ),
                        800.,
                        row_y + 35.,
                        14.,
                        WHITE,
                    );
                }
            }
            if legal_hints.is_empty() {
                label(
                    tr(zh, "No legal exchanges", "当前无可行交换"),
                    800.,
                    305.,
                    17.,
                    GOLD,
                );
            }
            for (rect, enabled, text) in [
                (
                    previous_page_rect,
                    move_hint_page > 0,
                    tr(zh, "< Prev", "上一页"),
                ),
                (
                    next_page_rect,
                    move_hint_page + 1 < pages,
                    tr(zh, "Next >", "下一页"),
                ),
            ] {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.13, 0.20, 0.28, 1.),
                );
                draw_rectangle_lines(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    1.,
                    if enabled { SKYBLUE } else { GRAY },
                );
                label(
                    text,
                    rect.x + 5.,
                    rect.y + 17.,
                    13.,
                    if enabled { WHITE } else { GRAY },
                );
            }
            label(
                &format!("{} / {}", move_hint_page + 1, pages),
                867.,
                607.,
                14.,
                GOLD,
            );
            let focused_preview = focused_hint.and_then(|i| move_previews[i].as_ref());
            let footer = focused_preview
                .map(|p| {
                    format!(
                        "{} {hint_health}→{}  {} {hint_mana}→{}",
                        tr(zh, "HP", "生命"),
                        p.health_after,
                        tr(zh, "MP", "魔力"),
                        p.mana_after
                    )
                })
                .unwrap_or_else(|| {
                    if hints_playable {
                        tr(
                            zh,
                            "Click a row to play · ← → pages",
                            "点击条目即可执行 · ← → 翻页",
                        )
                    } else {
                        tr(zh, "Enemy moves: view only", "敌方的交换，仅供查看")
                    }
                    .to_string()
                });
            label(&footer, 800., 628., 13., LIGHTGRAY);
            if let Some(p) = focused_preview {
                let text = if zh {
                    format!(
                        "敌方生命 → {}{}",
                        p.enemy_health_after,
                        if p.awakens_enemy { " · 将觉醒" } else { "" }
                    )
                } else {
                    format!(
                        "Enemy HP → {}{}",
                        p.enemy_health_after,
                        if p.awakens_enemy { " · Awakening" } else { "" }
                    )
                };
                label(&text, 800., 646., 12., GOLD);
            }
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
                &if id == 6 && mana < content::skill_cost(id, awakened[is_enemy as usize], skill) {
                    cost_text(
                        zh,
                        mana,
                        content::blood_cost(
                            id,
                            awakened[is_enemy as usize],
                            content::skill_cost(id, awakened[is_enemy as usize], skill) - mana,
                        ),
                    )
                } else {
                    format!(
                        "{}{}",
                        tr(zh, "Cost: ", "消耗："),
                        cost_text(
                            zh,
                            content::skill_cost(id, awakened[is_enemy as usize], skill),
                            0
                        )
                    )
                },
                x + 10.,
                266.,
                18.,
                if id == 6 && mana < content::skill_cost(id, awakened[is_enemy as usize], skill) {
                    RED
                } else {
                    SKYBLUE
                },
            );
            let description = content::skill_description(skill, awakened[is_enemy as usize], zh);
            for (line, text) in wrap_preview_text(description, 12.)
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
                content::preview_range_in(
                    skill,
                    &b,
                    id,
                    health,
                    if content::payment_in(id, health, mana, skill, rules!(is_enemy)).is_some() {
                        mana
                    } else {
                        mana.max(content::skill_cost(id, awakened[is_enemy as usize], skill))
                    },
                    opponent,
                    boost,
                    rules!(is_enemy),
                )
            } else {
                (
                    content::preview_rewards_in(
                        skill,
                        &b,
                        exact_target.unwrap_or((3, 3)),
                        id,
                        health,
                        if content::payment_in(id, health, mana, skill, rules!(is_enemy)).is_some()
                        {
                            mana
                        } else {
                            mana.max(content::skill_cost(id, awakened[is_enemy as usize], skill))
                        },
                        opponent,
                        boost,
                        rules!(is_enemy),
                    ),
                    content::preview_rewards_in(
                        skill,
                        &b,
                        exact_target.unwrap_or((3, 3)),
                        id,
                        health,
                        if content::payment_in(id, health, mana, skill, rules!(is_enemy)).is_some()
                        {
                            mana
                        } else {
                            mana.max(content::skill_cost(id, awakened[is_enemy as usize], skill))
                        },
                        opponent,
                        boost,
                        rules!(is_enemy),
                    ),
                )
            };
            let amount = |lo: i32, hi: i32| {
                if lo == hi {
                    lo.to_string()
                } else {
                    // Range over every possible target, e.g. 125～154; not Rust's `..` syntax.
                    format!("{lo}{}{hi}", tr(zh, "–", "～"))
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
            if content::payment_in(id, health, mana, skill, rules!(is_enemy)).is_some() {
                label(
                    &format!(
                        "{} {mana} → {}",
                        tr(zh, "MP", "魔力"),
                        amount(low.mana_after, high.mana_after)
                    ),
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
            if id == 6
                && mana < content::skill_cost(id, awakened[is_enemy as usize], skill)
                && content::payment_in(id, health, mana, skill, rules!(is_enemy)).is_none()
            {
                label(
                    tr(zh, "Not enough HP; must leave 1", "生命不足，支付后须剩1点"),
                    x + 10.,
                    598.,
                    15.,
                    RED,
                );
            }
            if low.blood_cost > 0
                && content::payment_in(id, health, mana, skill, rules!(is_enemy)).is_some()
            {
                label(
                    &if zh {
                        format!("生命 -{} → {} !", low.blood_cost, health - low.blood_cost)
                    } else {
                        format!("-{} HP → {} !", low.blood_cost, health - low.blood_cost)
                    },
                    x + 10.,
                    598.,
                    18.,
                    RED,
                );
                label(
                    &format!(
                        "{} → {}",
                        tr(zh, "After: HP", "结算后生命"),
                        amount(low.health_after, high.health_after)
                    ),
                    x + 10.,
                    620.,
                    16.,
                    RED,
                );
            }
            if high.awakens_enemy {
                let text = if zh {
                    format!(
                        "敌方将觉醒：生命 → {}",
                        amount(low.enemy_health_after, high.enemy_health_after)
                    )
                } else {
                    format!(
                        "Enemy awakens: HP → {}",
                        amount(low.enemy_health_after, high.enemy_health_after)
                    )
                };
                label(&text, x + 10., 570., 13., GOLD);
            } else if high.awakens_self {
                label(
                    tr(zh, "HP payment triggers awakening", "生命支付将触发觉醒"),
                    x + 10.,
                    570.,
                    13.,
                    GOLD,
                );
            } else if exact_target.is_some() {
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
                label_centered(tr(zh, "DEFEAT", "你输了"), 512., 436., 38., GOLD);
            } else {
                label_centered(tr(zh, "VICTORY!", "胜利！"), 512., 345., 40., GOLD);
            }
            ui::draw_game_over_buttons(zh, pointer);
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
    fn hint_clicks_hit_rows_on_the_shown_page_and_board_badges() {
        // Rows are 44 units tall from y=278, seven per page.
        assert_eq!(hint_row_at(vec2(800., 280.), 0, 3), Some(0));
        assert_eq!(
            hint_row_at(vec2(1000., 278. + 44. * 2. + 1.), 1, 20),
            Some(9)
        );
        assert_eq!(hint_row_at(vec2(800., 278. + 44. * 2. + 1.), 0, 2), None);
        assert_eq!(hint_row_at(vec2(797., 280.), 0, 3), None);
        assert_eq!(hint_row_at(vec2(800., 586.), 0, 20), None);
        let hints = [((0, 0), (1, 0)), ((3, 4), (3, 5)), ((3, 5), (3, 6))];
        let badge = presentation::move_hint_anchor((3, 4), (3, 5));
        assert_eq!(hint_badge_at(badge, &hints), Some(1));
        assert_eq!(hint_badge_at(badge + vec2(0., 3.), &hints), Some(1));
        assert_eq!(hint_badge_at(badge + vec2(30., 0.), &hints), None);
        // Off the board nothing is hit, even inside the badge radius.
        assert_eq!(hint_badge_at(vec2(250., 120.), &[((0, 0), (0, 1))]), None);
    }
    #[test]
    fn sidebar_labels_are_short_and_localized() {
        for id in 0..HEROES.len() {
            for awake in [false, true] {
                for &passive in content::passives(id, awake) {
                    for zh in [false, true] {
                        let name = passive.name(zh);
                        assert!(!name.is_empty() && !name.contains(['：', ':', '\n']));
                    }
                }
            }
        }
        assert_eq!(
            reward_summary(true, 3, 12, 0),
            "生命 +3 · 伤害 12 · 魔力 +0"
        );
        assert_eq!(reward_summary(false, 3, 12, 0), "+3 HP · 12 damage · +0 MP");
        assert_eq!(cost_text(true, 25, 50), "25 魔力 + 50 生命 !");
        assert_eq!(cost_text(true, 100, 0), "100 魔力");
        assert_eq!(cost_text(false, 25, 50), "25 MP + 50 HP !");
        assert_eq!(cost_text(false, 100, 0), "100 MP");
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

mod content;
mod menu;
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
const N: usize = 8;
type Board = [[u8; N]; N];
fn matches(b: &Board) -> Vec<(usize, usize)> {
    let mut hit = [[false; N]; N];
    for y in 0..N {
        for x in 0..N {
            if b[y][x] == 255 {
                continue;
            }
            if x + 2 < N && b[y][x] == b[y][x + 1] && b[y][x] == b[y][x + 2] {
                hit[y][x] = true;
                hit[y][x + 1] = true;
                hit[y][x + 2] = true;
            }
            if y + 2 < N && b[y][x] == b[y + 1][x] && b[y][x] == b[y + 2][x] {
                hit[y][x] = true;
                hit[y + 1][x] = true;
                hit[y + 2][x] = true;
            }
        }
    }
    let mut out = Vec::new();
    for y in 0..N {
        for x in 0..N {
            if hit[y][x] {
                out.push((x, y));
            }
        }
    }
    out
}
fn new_board() -> Board {
    let mut b = [[255; N]; N];
    for y in 0..N {
        for x in 0..N {
            loop {
                b[y][x] = rand::gen_range(0, 7);
                if !(x >= 2 && b[y][x] == b[y][x - 1] && b[y][x] == b[y][x - 2])
                    && !(y >= 2 && b[y][x] == b[y - 1][x] && b[y][x] == b[y - 2][x])
                {
                    break;
                }
            }
        }
    }
    b
}
fn swap(b: &mut Board, a: (usize, usize), c: (usize, usize)) {
    let t = b[a.1][a.0];
    b[a.1][a.0] = b[c.1][c.0];
    b[c.1][c.0] = t;
}
fn valid_swap(b: &mut Board, a: (usize, usize), c: (usize, usize)) -> bool {
    if a.0.abs_diff(c.0) + a.1.abs_diff(c.1) != 1 {
        return false;
    }
    swap(b, a, c);
    if matches(b).is_empty() {
        swap(b, a, c);
        false
    } else {
        true
    }
}
fn playable(b: &Board) -> bool {
    for y in 0..N {
        for x in 0..N {
            for c in [(x + 1, y), (x, y + 1)] {
                if c.0 < N && c.1 < N {
                    let mut copy = *b;
                    if valid_swap(&mut copy, (x, y), c) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
fn fresh() -> Board {
    loop {
        let b = new_board();
        if playable(&b) {
            return b;
        }
    }
}
fn drop_board(b: &mut Board) {
    for x in 0..N {
        let mut dest = N;
        for y in (0..N).rev() {
            if b[y][x] != 255 {
                dest -= 1;
                b[dest][x] = b[y][x];
            }
        }
        for y in 0..dest {
            b[y][x] = rand::gen_range(0, 7);
        }
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
    let mut knight = diagnostic_texture(sprites::SPRITES[hero_id].bytes, "no_actors");
    let thumbs = texture(include_bytes!("../assets/hero-thumbnails.png"));
    let mut preview = menu::Preview::new(hero_id, 0);
    let demon = diagnostic_texture(sprites::SPRITES[7].bytes, "no_actors");
    let mut choosing = bench.is_none();
    let audio = audio::Audio::new(!diagnostic("no_music"), !diagnostic("no_sound"))
        .map_err(|error| eprintln!("Audio unavailable: {error}"))
        .ok();
    rand::srand((get_time() * 1e9) as u64);
    let mut b = fresh();
    let mut selected: Option<(usize, usize)> = None;
    let (mut hp, mut enemy, mut mp) = (HEROES[hero_id].hp, 800i32, 0i32);
    let mut pending: Vec<(usize, usize)> = Vec::new();
    let mut deadline = 0.;
    let mut enemy_clock = 0.;
    let mut paused = false;
    let mut muted = false;
    let mut chain = 0;
    let mut note = "Select two adjacent tiles";
    let mut reward = true;
    let mut extra_bonus = 0;
    let mut armed = false;
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
        if is_key_pressed(KeyCode::L)
            || (is_mouse_button_pressed(MouseButton::Left)
                && Rect::new(810., 20., 190., 48.).contains(pointer))
        {
            zh = !zh;
        }
        if is_key_pressed(KeyCode::P) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::M) {
            muted = !muted;
            if let Some(audio) = &audio {
                audio.muted(muted);
            }
        }
        if is_key_pressed(KeyCode::E) {
            choosing = true;
        }
        if is_key_pressed(KeyCode::R) {
            b = fresh();
            hp = HEROES[hero_id].hp;
            enemy = 800;
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
        }
        if choosing {
            if let Some(audio) = &audio {
                audio.playing(false);
            }
            let old_hero = hero_id;
            if is_key_pressed(KeyCode::Down) {
                hero_id = (hero_id + 1) % HEROES.len();
            }
            if is_key_pressed(KeyCode::Up) {
                hero_id = (hero_id + HEROES.len() - 1) % HEROES.len();
            }
            if is_mouse_button_pressed(MouseButton::Left) {
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
                if (is_mouse_button_pressed(MouseButton::Left)
                    && menu::skill_rect(i).contains(pointer))
                    || (i == 0 && is_key_pressed(KeyCode::Key1))
                    || (i == 1 && is_key_pressed(KeyCode::Key2))
                {
                    preview = menu::Preview::new(hero_id, i);
                }
            }
            if is_key_pressed(KeyCode::Enter)
                || (is_mouse_button_pressed(MouseButton::Left)
                    && menu::start_rect().contains(pointer))
            {
                choosing = false;
                b = fresh();
                hp = HEROES[hero_id].hp;
                mp = 0;
                enemy = 800;
                pending.clear();
                selected = None;
                enemy_clock = 0.;
                paused = false;
                armed = false;
                extra_bonus = 0;
                reward = true;
                chain = 0;
                note = "Select two adjacent tiles";
            }
            if choosing {
                clear_background(Color::from_rgba(25, 27, 38, 255));
                set_camera(&cam);
                menu::draw(hero_id, zh, &knight, &thumbs, &tiles, &preview, pointer);
                flush_labels();
                set_default_camera();
                next_frame().await;
                continue;
            }
        }
        if let Some(audio) = &audio {
            audio.playing(!paused && hp > 0 && enemy > 0);
        }
        let active = !paused && hp > 0 && enemy > 0;
        if active {
            enemy_clock += dt;
            if enemy_clock >= 10. && pending.is_empty() {
                hp = (hp - rand::gen_range(20, 120)).max(0);
                enemy = (enemy + 50).min(800);
                enemy_clock = 0.;
            }
            let mut cast = None;
            if pending.is_empty() && hp > 0 {
                for (i, skill) in HEROES[hero_id].skills.iter().enumerate() {
                    let rect = Rect::new(20., 445. + i as f32 * 118., 210., 112.);
                    if (i == 0 && (is_key_pressed(KeyCode::Key1) || is_key_pressed(KeyCode::Space)))
                        || (i == 1 && is_key_pressed(KeyCode::Key2))
                        || (is_mouse_button_pressed(MouseButton::Left) && rect.contains(pointer))
                    {
                        cast = Some(*skill);
                    }
                }
            }
            if let Some(skill) = cast {
                if mp >= skill.cost() {
                    mp -= skill.cost();
                    let e = effect(skill, &mut b);
                    hp = (hp + e.hp).min(HEROES[hero_id].hp);
                    enemy = (enemy - e.damage).max(0);
                    mp = (mp + e.mp).min(HEROES[hero_id].mp);
                    extra_bonus += e.bonus;
                    armed |= e.armed;
                    pending = e.cells;
                    reward = e.reward;
                    if pending.is_empty() {
                        pending = matches(&b);
                        reward = true;
                    }
                    deadline = 0.28;
                    selected = None;
                    note = "Skill cast";
                } else {
                    note = "Not enough mana";
                }
            }
            if !pending.is_empty() && hp > 0 {
                deadline -= dt;
                if deadline <= 0. {
                    let (mut heal, mut damage, mut mana) = (0, 0, 0);
                    let bonus = (pending.len() as i32 - 2).max(1) + extra_bonus;
                    if reward {
                        extra_bonus = 0;
                    }
                    for &(x, y) in &pending {
                        let v = if reward {
                            VALUES[b[y][x] as usize]
                        } else {
                            (0, 0, 0)
                        };
                        heal += v.0;
                        damage += v.1;
                        mana += v.2;
                        b[y][x] = 255;
                    }
                    hp = (hp + heal * bonus).min(HEROES[hero_id].hp);
                    enemy = (enemy - damage * bonus).max(0);
                    mp = (mp + mana * bonus).min(HEROES[hero_id].mp);
                    drop_board(&mut b);
                    pending = matches(&b);
                    reward = true;
                    if !pending.is_empty() {
                        chain += 1;
                        deadline = 0.28;
                    } else if !playable(&b) {
                        b = fresh();
                        note = "No moves: board refreshed";
                    }
                }
            } else if is_mouse_button_pressed(MouseButton::Left) && hp > 0 {
                let (mx, my) = mouse_position();
                let world = cam.screen_to_world(vec2(mx, my));
                let x = (world.x - 256.) / 64.;
                let y = (world.y - 110.) / 64.;
                if x >= 0. && x < 8. && y >= 0. && y < 8. {
                    let c = (x as usize, y as usize);
                    if let Some(a) = selected {
                        let legal = if armed && a.0.abs_diff(c.0) + a.1.abs_diff(c.1) == 1 {
                            swap(&mut b, a, c);
                            for delta in [-1isize, 1] {
                                let p = if a.0 == c.0 {
                                    (a.0 as isize + delta, a.1 as isize)
                                } else {
                                    (a.0 as isize, a.1 as isize + delta)
                                };
                                let q = if a.0 == c.0 {
                                    (c.0 as isize + delta, c.1 as isize)
                                } else {
                                    (c.0 as isize, c.1 as isize + delta)
                                };
                                if p.0 >= 0
                                    && p.1 >= 0
                                    && q.0 >= 0
                                    && q.1 >= 0
                                    && p.0 < N as isize
                                    && p.1 < N as isize
                                    && q.0 < N as isize
                                    && q.1 < N as isize
                                {
                                    swap(
                                        &mut b,
                                        (p.0 as usize, p.1 as usize),
                                        (q.0 as usize, q.1 as usize),
                                    );
                                }
                            }
                            armed = false;
                            true
                        } else {
                            valid_swap(&mut b, a, c)
                        };
                        if legal {
                            pending = matches(&b);
                            reward = true;
                            deadline = 0.28;
                            chain = 1;
                            selected = None;
                            note = "Match!";
                            if !muted {
                                if let Some(audio) = &audio {
                                    audio.crush();
                                }
                            }
                        } else {
                            selected = Some(c);
                            note = "No match: choose another tile";
                        }
                    } else {
                        selected = Some(c);
                    }
                }
            }
        }
        clear_background(Color::from_rgba(25, 27, 38, 255));
        set_camera(&cam);
        procedural_board();
        label(tr(zh, "BATTLE BLOCKS", "方块战斗"), 300., 55., 40., GOLD);
        draw_rectangle(810., 20., 190., 48., Color::new(0.15, 0.18, 0.25, 1.));
        label(tr(zh, "中文 / L", "English / L"), 825., 54., 28., WHITE);
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
            320.,
            435.,
            805.,
            get_time(),
            &[0.13; 4],
            &[0, 1, 2, 3],
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
        label(tr(zh, "DEMON", "大恶魔"), 805., 110., 32., WHITE);
        label(
            &format!("{} {enemy} / 800", tr(zh, "HP", "生命")),
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
        for kind in 0..if !diagnostic("legacy_batches") { 7 } else { 1 } {
            for y in 0..N {
                for x in 0..N {
                    if !diagnostic("legacy_batches") && b[y][x] != kind {
                        continue;
                    }
                    let pos = vec2(256. + x as f32 * 64., 110. + y as f32 * 64.);
                    let color = if pending.contains(&(x, y)) {
                        Color::new(1., 1., 1., 0.35)
                    } else {
                        WHITE
                    };
                    if diagnostic("legacy_batches") {
                        procedural_cell(pos.x, pos.y);
                    }
                    draw_texture_ex(
                        &tiles[b[y][x] as usize],
                        pos.x,
                        pos.y,
                        color,
                        DrawTextureParams {
                            dest_size: Some(vec2(64., 64.)),
                            ..Default::default()
                        },
                    );
                    if selected == Some((x, y)) {
                        draw_rectangle_lines(pos.x, pos.y, 62., 62., 3., GOLD);
                    }
                }
            }
        }
        for (i, skill) in HEROES[hero_id].skills.iter().enumerate() {
            let rect = Rect::new(20., 445. + i as f32 * 118., 210., 112.);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if mp >= skill.cost() {
                    Color::new(0.2, 0.28, 0.35, 1.)
                } else {
                    Color::new(0.1, 0.12, 0.16, 1.)
                },
            );
            label(
                &format!("{}: {}", i + 1, skill.name(zh)),
                rect.x + 8.,
                rect.y + 28.,
                24.,
                WHITE,
            );
            label(
                &format!(
                    "{} {}",
                    skill.cost(),
                    tr(zh, "MP · click / key", "魔力 · 点击 / 按键")
                ),
                rect.x + 8.,
                rect.y + 49.,
                17.,
                SKYBLUE,
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
        {
            label(
                tr(
                    zh,
                    "1 / 2: Skills   E: Hero selection",
                    "1 / 2：技能  E：角色选择",
                ),
                256.,
                669.,
                28.,
                WHITE,
            );
            label(
                tr(
                    zh,
                    "P: Pause   R: Restart   M: Mute",
                    "P：暂停  R：重开  M：静音",
                ),
                256.,
                706.,
                28.,
                WHITE,
            );
        }
        label(
            &format!(
                "{}  {}: {chain}",
                localized_note(zh, note),
                tr(zh, "Chain", "连锁")
            ),
            256.,
            80.,
            26.,
            WHITE,
        );
        if paused || hp == 0 || enemy == 0 {
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

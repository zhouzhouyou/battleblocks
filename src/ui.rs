use super::*;
#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Settings,
    Help,
}
/// Window sizes offered in settings; the game opens in the larger one.
pub const WINDOW_SIZES: [(i32, i32); 2] = [(1024, 720), (1280, 900)];
// Horizontal extent of the volume slider in the first settings row.
const VOLUME_TRACK: (f32, f32) = (462., 662.);
pub struct Ui {
    pub panel: Option<Panel>,
    pub opponent: Option<usize>,
    pub opponent_step: bool,
    fullscreen: bool,
    // Index into WINDOW_SIZES.
    window: usize,
    pub volume: u32,
}
impl Default for Ui {
    fn default() -> Self {
        Self {
            panel: None,
            opponent: None,
            opponent_step: false,
            fullscreen: false,
            window: 1,
            volume: 100,
        }
    }
}
fn frame(r: Rect, hovered: bool) {
    draw_rectangle(
        r.x,
        r.y,
        r.w,
        r.h,
        if hovered {
            Color::from_rgba(61, 76, 98, 255)
        } else {
            Color::from_rgba(39, 47, 64, 255)
        },
    );
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1., GRAY);
}
fn button(r: Rect, text: &str, pointer: Vec2) {
    frame(r, r.contains(pointer));
    label_centered(text, r.x + r.w / 2., r.y + r.h / 2. + 8., 22., WHITE);
}
fn settings_row(i: usize) -> Rect {
    Rect::new(280., 190. + i as f32 * 58., 464., 46.)
}
/// One of the two choices at the right end of a settings toggle row.
fn option_rect(row: usize, i: usize) -> Rect {
    let r = settings_row(row);
    Rect::new(r.x + r.w - 236. + i as f32 * 116., r.y + 7., 112., 32.)
}
pub fn top(i: usize) -> Rect {
    Rect::new(814. + i as f32 * 48., 20., 42., 42.)
}
impl Ui {
    // Returns whether the frame belongs to the interface, including the closing click.
    pub fn input(
        &mut self,
        pointer: Vec2,
        choosing: bool,
        muted: &mut bool,
        paused: &mut bool,
        zh: &mut bool,
    ) -> bool {
        let click = is_mouse_button_pressed(MouseButton::Left);
        let previous = self.panel.is_some();
        if is_key_pressed(KeyCode::Escape) && previous {
            self.panel = None;
            return true;
        }
        let (start, end) = VOLUME_TRACK;
        if self.panel == Some(Panel::Settings)
            && is_mouse_button_down(MouseButton::Left)
            && Rect::new(start - 14., 190., end - start + 28., 46.).contains(pointer)
        {
            self.volume =
                (((pointer.x - start) / (end - start)).clamp(0., 1.) * 100.).round() as u32;
            *muted = false;
        }
        if click {
            if top(0).contains(pointer) {
                self.panel = if self.panel == Some(Panel::Settings) {
                    None
                } else {
                    Some(Panel::Settings)
                };
                return true;
            }
            if top(1).contains(pointer) {
                *muted = !*muted;
                return true;
            }
            if top(2).contains(pointer) && !choosing {
                *paused = !*paused;
                return true;
            }
            if top(3).contains(pointer) {
                self.panel = if self.panel == Some(Panel::Help) {
                    None
                } else {
                    Some(Panel::Help)
                };
                return true;
            }
            if previous && Rect::new(758., 119., 42., 38.).contains(pointer) {
                self.panel = None;
                return true;
            }
            if self.panel == Some(Panel::Settings) {
                // Toggle rows: clicking a choice selects it, clicking elsewhere on the
                // row switches to the other one.
                for row in 1..4 {
                    if !settings_row(row).contains(pointer) {
                        continue;
                    }
                    let current = self.choice(row, *zh);
                    let choice = (0..2)
                        .find(|&i| option_rect(row, i).contains(pointer))
                        .unwrap_or(1 - current);
                    if choice == current {
                        continue;
                    }
                    match row {
                        1 => {
                            self.fullscreen = choice == 1;
                            set_fullscreen(self.fullscreen);
                        }
                        2 => {
                            self.window = choice;
                            let (w, h) = WINDOW_SIZES[choice];
                            request_new_screen_size(w as f32, h as f32);
                        }
                        _ => *zh = choice == 0,
                    }
                }
            }
        }
        previous || self.panel.is_some()
    }
    /// Selected choice of a settings toggle row: fullscreen off/on, window size, 中文/English.
    fn choice(&self, row: usize, zh: bool) -> usize {
        match row {
            1 => self.fullscreen as usize,
            2 => self.window,
            _ => (!zh) as usize,
        }
    }
    pub fn draw_top(&self, choosing: bool, muted: bool, paused: bool, pointer: Vec2, zh: bool) {
        for i in 0..4 {
            let r = top(i);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(39, 47, 64, 255));
            let c = vec2(r.x + 21., r.y + 21.);
            match i {
                0 => {
                    draw_circle_lines(c.x, c.y, 10., 3., WHITE);
                    draw_circle(c.x, c.y, 3., WHITE);
                    for n in 0..8 {
                        let d = vec2(
                            (n as f32 * std::f32::consts::PI / 4.).cos(),
                            (n as f32 * std::f32::consts::PI / 4.).sin(),
                        );
                        draw_line(
                            c.x + d.x * 10.,
                            c.y + d.y * 10.,
                            c.x + d.x * 15.,
                            c.y + d.y * 15.,
                            3.,
                            WHITE,
                        );
                    }
                }
                1 => {
                    let color = if muted { LIGHTGRAY } else { WHITE };
                    let outline = [
                        (-13., -5.),
                        (-8., -5.),
                        (-1., -11.),
                        (-1., 11.),
                        (-8., 5.),
                        (-13., 5.),
                        (-13., -5.),
                    ];
                    for pair in outline.windows(2) {
                        draw_line(
                            c.x + pair[0].0,
                            c.y + pair[0].1,
                            c.x + pair[1].0,
                            c.y + pair[1].1,
                            2.,
                            color,
                        );
                    }
                    if muted {
                        draw_line(c.x + 5., c.y - 9., c.x + 14., c.y + 9., 2., color);
                    } else {
                        for radius in [8., 14.] {
                            for n in 0..16 {
                                let a = -0.78 + n as f32 * 1.56 / 16.;
                                let b = -0.78 + (n + 1) as f32 * 1.56 / 16.;
                                draw_line(
                                    c.x + radius * a.cos(),
                                    c.y + radius * a.sin(),
                                    c.x + radius * b.cos(),
                                    c.y + radius * b.sin(),
                                    2.,
                                    color,
                                );
                            }
                        }
                    }
                }
                2 => {
                    if choosing || paused {
                        draw_triangle(
                            vec2(c.x - 6., c.y - 11.),
                            vec2(c.x - 6., c.y + 11.),
                            vec2(c.x + 11., c.y),
                            if choosing { GRAY } else { WHITE },
                        );
                    } else {
                        draw_rectangle(c.x - 9., c.y - 11., 6., 22., WHITE);
                        draw_rectangle(c.x + 3., c.y - 11., 6., 22., WHITE);
                    }
                }
                _ => label("?", c.x - 8., c.y + 10., 30., GOLD),
            }
            if r.contains(pointer) {
                label(
                    [
                        tr(zh, "Settings", "设置"),
                        tr(zh, "Audio / M", "音频 / M"),
                        tr(zh, "Pause / P", "暂停 / P"),
                        tr(zh, "Tile guide", "方块说明"),
                    ][i],
                    r.x - 45.,
                    84.,
                    18.,
                    WHITE,
                );
            }
        }
    }
    pub fn draw_opponents(
        &self,
        hero: usize,
        zh: bool,
        thumbs: &Texture2D,
        pointer: Vec2,
        difficulty: usize,
    ) {
        label(
            tr(zh, "STEP 2 · CHOOSE OPPONENT", "第二步 · 选择对手"),
            24.,
            58.,
            34.,
            GOLD,
        );
        label(
            &format!(
                "{}：{}",
                tr(zh, "Your hero", "你的角色"),
                if zh { HEROES[hero].zh } else { HEROES[hero].en }
            ),
            26.,
            90.,
            22.,
            LIGHTGRAY,
        );
        // Centered over the column of the back button below.
        let back = back_rect();
        label_centered(
            tr(zh, "AI difficulty", "AI 难度"),
            back.x + back.w / 2.,
            140.,
            24.,
            WHITE,
        );
        for (i, name) in [
            tr(zh, "Beginner", "入门"),
            tr(zh, "Standard", "标准"),
            tr(zh, "Advanced", "进阶"),
        ]
        .iter()
        .enumerate()
        {
            let r = difficulty_rect(i);
            button(r, name, pointer);
            if i == difficulty {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., GOLD);
            }
        }
        for i in 0..8 {
            let r = opponent_rect(i);
            frame(r, r.contains(pointer));
            let selected = if i == 0 {
                self.opponent.is_none()
            } else {
                self.opponent == Some(i - 1)
            };
            if selected {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., GOLD);
            }
            if i == 0 {
                label(
                    tr(zh, "Random opponent", "随机对手"),
                    r.x + 14.,
                    r.y + 32.,
                    24.,
                    GOLD,
                );
                label(
                    tr(zh, "Excludes your hero", "随机时不与己方角色重复"),
                    r.x + 14.,
                    r.y + 65.,
                    17.,
                    LIGHTGRAY,
                );
            } else {
                let h = &HEROES[i - 1];
                draw_texture_ex(
                    thumbs,
                    r.x + 8.,
                    r.y + 12.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(40., 48.)),
                        source: Some(Rect::new((i - 1) as f32 * 40., 0., 40., 48.)),
                        ..Default::default()
                    },
                );
                label(
                    if zh { h.zh } else { h.en },
                    r.x + 58.,
                    r.y + 28.,
                    22.,
                    WHITE,
                );
                label(
                    &format!(
                        "{} {}  {} {}",
                        tr(zh, "HP", "生命"),
                        h.hp,
                        tr(zh, "MP", "魔力"),
                        h.mp
                    ),
                    r.x + 58.,
                    r.y + 51.,
                    17.,
                    LIGHTGRAY,
                );
                label(
                    &format!("{} / {}", h.skills[0].name(zh), h.skills[1].name(zh)),
                    r.x + 8.,
                    r.y + 78.,
                    if zh { 16. } else { 13. },
                    SKYBLUE,
                );
            }
        }
        button(
            back_rect(),
            tr(zh, "Back: choose hero", "上一步：选择角色"),
            pointer,
        );
        button(
            menu::start_rect(),
            tr(zh, "Enter · Start battle", "回车 / 点击 · 开始对战"),
            pointer,
        );
    }
    pub fn draw_panel(&self, zh: bool, muted: bool, tiles: &[Texture2D; 7], pointer: Vec2) {
        let Some(panel) = self.panel else { return };
        flush_labels();
        draw_rectangle(0., 0., 1024., 720., Color::new(0., 0., 0., 0.78));
        draw_rectangle(212., 108., 600., 516., Color::from_rgba(24, 30, 43, 255));
        draw_rectangle_lines(212., 108., 600., 516., 2., SKYBLUE);
        label(
            match panel {
                Panel::Settings => tr(zh, "SETTINGS", "游戏设置"),
                Panel::Help => tr(zh, "TILE GUIDE", "方块说明"),
            },
            236.,
            148.,
            28.,
            GOLD,
        );
        button(Rect::new(758., 119., 42., 38.), "×", pointer);
        match panel {
            Panel::Settings => {
                let names = [
                    if muted {
                        tr(zh, "Volume (muted)", "音量（静音）")
                    } else {
                        tr(zh, "Volume", "音量")
                    },
                    tr(zh, "Fullscreen", "全屏"),
                    tr(zh, "Window", "窗口"),
                    tr(zh, "Language", "语言"),
                ];
                let windows = WINDOW_SIZES.map(|(w, h)| format!("{w} × {h}"));
                let choices = [
                    [tr(zh, "Off", "关闭"), tr(zh, "On", "开启")],
                    [windows[0].as_str(), windows[1].as_str()],
                    ["中文", "English"],
                ];
                for (row, name) in names.iter().enumerate() {
                    let r = settings_row(row);
                    frame(r, row > 0 && r.contains(pointer));
                    label(name, r.x + 12., r.y + r.h / 2. + 8., 22., WHITE);
                    if row == 0 {
                        continue;
                    }
                    let selected = self.choice(row, zh);
                    for (i, text) in choices[row - 1].iter().enumerate() {
                        let o = option_rect(row, i);
                        draw_rectangle(o.x, o.y, o.w, o.h, Color::from_rgba(24, 30, 43, 255));
                        let on = i == selected;
                        draw_rectangle_lines(
                            o.x,
                            o.y,
                            o.w,
                            o.h,
                            if on { 2. } else { 1. },
                            if on {
                                GOLD
                            } else {
                                Color::from_rgba(66, 71, 83, 255)
                            },
                        );
                        label_centered(
                            text,
                            o.x + o.w / 2.,
                            o.y + 22.,
                            18.,
                            if on { WHITE } else { GRAY },
                        );
                    }
                }
                // Slider and percentage share the volume row.
                let (start, end) = VOLUME_TRACK;
                let y = settings_row(0).y + 23.;
                draw_line(start, y, end, y, 5., GRAY);
                let x = start + (end - start) * self.volume as f32 / 100.;
                draw_line(start, y, x, y, 5., SKYBLUE);
                draw_circle(x, y, 8., WHITE);
                label_centered(&format!("{}%", self.volume), 707., y + 7., 20., SKYBLUE);
            }
            Panel::Help => {
                let names = if zh {
                    [
                        "平衡",
                        "治疗",
                        "伤害",
                        "魔力",
                        "治疗＋伤害",
                        "治疗＋魔力",
                        "伤害＋魔力",
                    ]
                } else {
                    [
                        "Balanced",
                        "Healing",
                        "Damage",
                        "Mana",
                        "Heal + damage",
                        "Heal + mana",
                        "Damage + mana",
                    ]
                };
                for i in 0..7 {
                    let y = 174. + i as f32 * 52.;
                    draw_texture_ex(
                        &tiles[i],
                        238.,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(40., 40.)),
                            ..Default::default()
                        },
                    );
                    label(names[i], 294., y + 24., 20., WHITE);
                    let v = VALUES[i];
                    label(
                        &format!(
                            "{} +{}   {} {}   {} +{}",
                            tr(zh, "HP", "生命"),
                            v.0,
                            tr(zh, "DMG", "伤害"),
                            v.1,
                            tr(zh, "MP", "魔力"),
                            v.2
                        ),
                        485.,
                        y + 24.,
                        19.,
                        SKYBLUE,
                    );
                }
                label(
                    tr(
                        zh,
                        "Match 3+ identical tiles. Damage hits your opponent.",
                        "连接三个以上同类方块消除；伤害作用于对手。",
                    ),
                    236.,
                    564.,
                    20.,
                    WHITE,
                );
                label(
                    tr(
                        zh,
                        "Chains: +10% per step, up to 130%. Skills may differ.",
                        "连锁每层＋10%，最高130%；技能按各自说明结算。",
                    ),
                    236.,
                    598.,
                    19.,
                    LIGHTGRAY,
                );
            }
        }
        label(
            tr(zh, "Esc: Close", "Esc：关闭"),
            236.,
            650.,
            18.,
            LIGHTGRAY,
        );
    }
}
pub fn opponent_rect(i: usize) -> Rect {
    Rect::new(
        230. + (i % 2) as f32 * 288.,
        176. + (i / 2) as f32 * 106.,
        276.,
        96.,
    )
}

pub fn pause_rect(i: usize) -> Rect {
    Rect::new(322., 270. + i as f32 * 63., 380., 48.)
}
pub fn draw_pause(zh: bool, pointer: Vec2) {
    flush_labels();
    draw_rectangle(0., 0., 1024., 720., Color::new(0., 0., 0., 0.72));
    draw_rectangle(282., 180., 460., 390., Color::from_rgba(24, 30, 43, 255));
    draw_rectangle_lines(282., 180., 460., 390., 2., SKYBLUE);
    label(tr(zh, "PAUSED", "已暂停"), 322., 232., 34., GOLD);
    for (i, text) in [
        tr(zh, "Continue", "继续游戏"),
        tr(zh, "Abandon & return to main menu", "放弃并回到主菜单"),
        tr(zh, "Restart battle", "重开本局"),
        tr(zh, "Settings menu", "设置菜单"),
    ]
    .iter()
    .enumerate()
    {
        button(pause_rect(i), text, pointer);
    }
}

pub fn back_rect() -> Rect {
    Rect::new(24., 651., 232., 48.)
}

pub fn difficulty_rect(i: usize) -> Rect {
    Rect::new(230. + i as f32 * 194., 112., 180., 40.)
}

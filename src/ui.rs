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
/// Fully visible rows of the opponent grid; further rows scroll.
pub const OPPONENT_VISIBLE_ROWS: usize = 5;
pub struct Ui {
    pub panel: Option<Panel>,
    pub opponent: Option<usize>,
    pub opponent_step: bool,
    // First visible row of each selectable list. Both clamp to the current roster.
    pub hero_scroll: usize,
    pub opponent_scroll: usize,
    help_awakening: bool,
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
            hero_scroll: 0,
            opponent_scroll: 0,
            help_awakening: false,
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
            if self.panel == Some(Panel::Help) {
                if Rect::new(236., 158., 180., 30.).contains(pointer) {
                    self.help_awakening = false;
                    return true;
                }
                if Rect::new(426., 158., 180., 30.).contains(pointer) {
                    self.help_awakening = true;
                    return true;
                }
            }
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
                        tr_key(zh, "ui.screen.draw_top.settings"),
                        tr_key(zh, "ui.screen.draw_top.audio_m"),
                        tr_key(zh, "ui.screen.draw_top.pause_p"),
                        tr_key(zh, "ui.screen.draw_top.tile_guide"),
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
        scroll: usize,
    ) {
        let total = crate::content::ROSTER.len() + 1;
        let rows = opponent_row_count();
        let scroll = clamp_scroll(scroll, rows, OPPONENT_VISIBLE_ROWS);
        label(
            tr_key(zh, "ui.screen.draw_opponents.step_2_choose_opponent"),
            24.,
            58.,
            34.,
            GOLD,
        );
        label(
            &format!(
                "{}：{}",
                tr_key(zh, "ui.screen.draw_opponents.your_hero"),
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
            tr_key(zh, "ui.screen.draw_opponents.ai_difficulty"),
            back.x + back.w / 2.,
            140.,
            24.,
            WHITE,
        );
        for (i, name) in [
            tr_key(zh, "ui.shared.beginner"),
            tr_key(zh, "ui.shared.standard"),
            tr_key(zh, "ui.shared.advanced"),
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
        // Only fully visible grid rows are drawn and clickable, so the last row can
        // never spill over the back/start buttons below the list.
        for row in 0..OPPONENT_VISIBLE_ROWS {
            for column in 0..2 {
                let i = (scroll + row) * 2 + column;
                if i >= total {
                    break;
                }
                let r = opponent_rect(row, column);
                frame(r, r.contains(pointer));
                let selected = if i == 0 {
                    self.opponent.is_none()
                } else {
                    self.opponent == Some(crate::content::ROSTER[i - 1])
                };
                if selected {
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., GOLD);
                }
                if i == 0 {
                    label(
                        tr_key(zh, "ui.screen.draw_opponents.random_opponent"),
                        r.x + 14.,
                        r.y + 32.,
                        24.,
                        GOLD,
                    );
                    label(
                        tr_key(zh, "ui.screen.draw_opponents.excludes_your_hero"),
                        r.x + 14.,
                        r.y + 61.,
                        17.,
                        LIGHTGRAY,
                    );
                } else {
                    let id = crate::content::ROSTER[i - 1];
                    let h = &HEROES[id];
                    draw_texture_ex(
                        thumbs,
                        r.x + 8.,
                        r.y + 12.,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(40., 48.)),
                            source: Some(Rect::new(id as f32 * 40., 0., 40., 48.)),
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
                            tr_key(zh, "ui.shared.hp"),
                            h.hp,
                            tr_key(zh, "ui.shared.mp"),
                            crate::content::mana_cap(id, false)
                        ),
                        r.x + 58.,
                        r.y + 51.,
                        17.,
                        LIGHTGRAY,
                    );
                    label(
                        &format!("{} / {}", h.skills[0].name(zh), h.skills[1].name(zh)),
                        r.x + 8.,
                        r.y + 68.,
                        if zh { 16. } else { 13. },
                        SKYBLUE,
                    );
                }
            }
        }
        draw_scrollbar(opponent_list_rect(), scroll, rows, OPPONENT_VISIBLE_ROWS);
        let focus = (0..OPPONENT_VISIBLE_ROWS)
            .flat_map(|row| (0..2).map(move |column| (row, column)))
            .find_map(|(row, column)| {
                let i = (scroll + row) * 2 + column;
                (i > 0 && i < total && opponent_rect(row, column).contains(pointer))
                    .then(|| crate::content::ROSTER[i - 1])
            })
            .or(self.opponent);
        if let Some(id) = focus {
            label(
                if zh { HEROES[id].zh } else { HEROES[id].en },
                24.,
                218.,
                22.,
                GOLD,
            );
            let line = content::awakening_line(id).to_string();
            let threshold = content::AWAKENING_LINES[id].to_string();
            let cap = content::health_cap(id, true).to_string();
            let cap_percent = content::AWAKENING_CAPS[id].to_string();
            let effect = content::awakening_effect(id, zh);
            let text = crate::format_text(
                zh,
                "hero.opponent_awakening_preview",
                &[&line, &threshold, &cap, &cap_percent, effect],
            );
            for (i, line) in wrap_preview_text(&text, 12.).iter().enumerate() {
                label(line, 24., 251. + i as f32 * 22., 15., LIGHTGRAY);
            }
        }
        button(
            back_rect(),
            tr_key(zh, "ui.screen.draw_opponents.back_choose_hero"),
            pointer,
        );
        button(
            menu::start_rect(),
            tr_key(zh, "ui.screen.draw_opponents.enter_start_battle"),
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
                Panel::Settings => tr_key(zh, "ui.screen.draw_panel.settings"),
                Panel::Help => tr_key(zh, "ui.screen.draw_panel.tile_guide"),
            },
            236.,
            148.,
            28.,
            GOLD,
        );
        button(Rect::new(758., 119., 42., 38.), "×", pointer);
        if panel == Panel::Help {
            for (i, text) in [
                tr_key(zh, "ui.screen.draw_panel.tiles_matching"),
                tr_key(zh, "ui.screen.draw_panel.awakening"),
            ]
            .iter()
            .enumerate()
            {
                let r = Rect::new(236. + i as f32 * 190., 158., 180., 30.);
                frame(r, self.help_awakening == (i == 1));
                label(text, r.x + 10., r.y + 21., 17., WHITE);
            }
            if self.help_awakening {
                label(
                    tr_key(
                        zh,
                        "ui.screen.draw_panel.damage_or_hp_payment_triggers_a_ne",
                    ),
                    236.,
                    214.,
                    16.,
                    GOLD,
                );
                label(
                    tr_key(
                        zh,
                        "ui.screen.draw_panel.the_triggering_hit_cannot_spill_ov",
                    ),
                    236.,
                    242.,
                    16.,
                    LIGHTGRAY,
                );
                label(
                    tr_key(zh, "ui.screen.draw_panel.hero_trigger_new_hp_cap"),
                    236.,
                    278.,
                    17.,
                    SKYBLUE,
                );
                for (id, h) in HEROES.iter().enumerate() {
                    let y = 307. + id as f32 * 40.;
                    label(if zh { h.zh } else { h.en }, 236., y, 16., WHITE);
                    label(
                        &format!(
                            "{} / {}",
                            content::awakening_line(id),
                            content::health_cap(id, true)
                        ),
                        380.,
                        y,
                        16.,
                        GREEN,
                    );
                    let text = content::awakening_effect(id, zh);
                    label(text, 236., y + 18., fit_size(text, 13., 550.), GOLD);
                }
                return;
            }
        }
        match panel {
            Panel::Settings => {
                let names = [
                    if muted {
                        tr_key(zh, "ui.screen.draw_panel.volume_muted")
                    } else {
                        tr_key(zh, "ui.screen.draw_panel.volume")
                    },
                    tr_key(zh, "ui.screen.draw_panel.fullscreen"),
                    tr_key(zh, "ui.screen.draw_panel.window"),
                    tr_key(zh, "ui.screen.draw_panel.language"),
                ];
                let windows = WINDOW_SIZES.map(|(w, h)| format!("{w} × {h}"));
                let choices = [
                    [
                        tr_key(zh, "ui.screen.draw_panel.off"),
                        tr_key(zh, "ui.screen.draw_panel.on"),
                    ],
                    [windows[0].as_str(), windows[1].as_str()],
                    [
                        tr_key(zh, "settings.language_chinese"),
                        tr_key(zh, "settings.language_english"),
                    ],
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
                let names = [
                    tr_key(zh, "ui.screen.draw_help.tile_balanced"),
                    tr_key(zh, "ui.screen.draw_help.tile_healing"),
                    tr_key(zh, "ui.screen.draw_help.tile_damage"),
                    tr_key(zh, "ui.screen.draw_help.tile_mana"),
                    tr_key(zh, "ui.screen.draw_help.tile_heal_damage"),
                    tr_key(zh, "ui.screen.draw_help.tile_heal_mana"),
                    tr_key(zh, "ui.screen.draw_help.tile_damage_mana"),
                ];
                for i in 0..7 {
                    let y = 202. + i as f32 * 48.;
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
                            tr_key(zh, "ui.shared.hp"),
                            v.0,
                            tr_key(zh, "ui.screen.draw_panel.dmg"),
                            v.1,
                            tr_key(zh, "ui.shared.mp"),
                            v.2
                        ),
                        485.,
                        y + 24.,
                        19.,
                        SKYBLUE,
                    );
                }
                label(
                    tr_key(
                        zh,
                        "ui.screen.draw_panel.match_3_identical_tiles_damage_hit",
                    ),
                    236.,
                    564.,
                    20.,
                    WHITE,
                );
                label(
                    tr_key(
                        zh,
                        "ui.screen.draw_panel.chains_10_per_step_up_to_130_skill",
                    ),
                    236.,
                    598.,
                    19.,
                    LIGHTGRAY,
                );
            }
        }
        label(
            tr_key(zh, "ui.screen.draw_panel.esc_close"),
            236.,
            650.,
            18.,
            LIGHTGRAY,
        );
    }
}
/// Largest valid first-row offset for a list showing `visible` rows at a time.
pub fn max_scroll(total: usize, visible: usize) -> usize {
    total.saturating_sub(visible)
}
/// Clamp a stored scroll offset after the roster or the viewport changes.
pub fn clamp_scroll(offset: usize, total: usize, visible: usize) -> usize {
    offset.min(max_scroll(total, visible))
}
/// Smallest offset that keeps `index` inside the visible window.
pub fn scroll_to_show(offset: usize, index: usize, total: usize, visible: usize) -> usize {
    if visible == 0 || index >= total {
        return clamp_scroll(offset, total, visible);
    }
    let next = if index < offset {
        index
    } else if index >= offset + visible {
        index + 1 - visible
    } else {
        offset
    };
    clamp_scroll(next, total, visible)
}
/// Move a scroll offset by whole rows (positive scrolls towards later rows).
pub fn scroll_by(offset: usize, delta: i32, total: usize, visible: usize) -> usize {
    let next = (offset as i64 + delta as i64).max(0) as usize;
    clamp_scroll(next, total, visible)
}
/// Draw a vertical scrollbar immediately right of `area`; invisible when all rows fit.
pub fn draw_scrollbar(area: Rect, offset: usize, total: usize, visible: usize) {
    if total <= visible {
        return;
    }
    let track = Rect::new(area.x + area.w + 4., area.y, 4., area.h);
    draw_rectangle(
        track.x,
        track.y,
        track.w,
        track.h,
        Color::from_rgba(45, 50, 62, 255),
    );
    let thumb_h = (track.h * visible as f32 / total as f32).max(18.);
    let progress =
        offset.min(max_scroll(total, visible)) as f32 / max_scroll(total, visible) as f32;
    draw_rectangle(
        track.x,
        track.y + (track.h - thumb_h) * progress,
        track.w,
        thumb_h,
        Color::from_rgba(132, 143, 166, 255),
    );
}
pub fn opponent_row_count() -> usize {
    (crate::content::ROSTER.len() + 1).div_ceil(2)
}
pub fn opponent_list_rect() -> Rect {
    Rect::new(230., 176., 564., 88. * OPPONENT_VISIBLE_ROWS as f32)
}
/// Grid cell for an on-screen row and column, independent of the scroll offset.
pub fn opponent_rect(row: usize, column: usize) -> Rect {
    Rect::new(
        230. + column as f32 * 288.,
        176. + row as f32 * 88.,
        276.,
        80.,
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
    label(
        tr_key(zh, "ui.screen.draw_pause.paused"),
        322.,
        232.,
        34.,
        GOLD,
    );
    for (i, text) in [
        tr_key(zh, "ui.screen.draw_pause.continue"),
        tr_key(zh, "ui.screen.draw_pause.abandon_return_to_main_menu"),
        tr_key(zh, "ui.screen.draw_pause.restart_battle"),
        tr_key(zh, "ui.screen.draw_pause.settings_menu"),
    ]
    .iter()
    .enumerate()
    {
        button(pause_rect(i), text, pointer);
    }
}

pub fn game_over_rect(i: usize) -> Rect {
    Rect::new(296. + i as f32 * 232., 462., 200., 48.)
}
pub fn draw_game_over_buttons(zh: bool, pointer: Vec2) {
    for (i, text) in [
        tr_key(zh, "ui.screen.draw_game_over_buttons.play_again"),
        tr_key(zh, "ui.screen.draw_game_over_buttons.home"),
    ]
    .iter()
    .enumerate()
    {
        button(game_over_rect(i), text, pointer);
    }
}

pub fn back_rect() -> Rect {
    Rect::new(24., 651., 232., 48.)
}

pub fn difficulty_rect(i: usize) -> Rect {
    Rect::new(230. + i as f32 * 194., 112., 180., 40.)
}

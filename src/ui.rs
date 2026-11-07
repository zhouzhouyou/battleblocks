use super::*;
mod help;
#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Settings,
    Help,
}
/// Window sizes offered in settings; the game opens in the larger one.
pub const WINDOW_SIZES: [(i32, i32); 2] = [(1024, 720), (1280, 900)];
// Shared slider geometry for master, music, effects and voice.
const VOLUME_TRACK: (f32, f32) = (506., 662.);
/// Modal panel frame. Every panel row is laid out inside it with this much padding,
/// so content can neither touch the border nor spill past it.
pub const PANEL: Rect = Rect::new(212., 108., 600., 516.);
const PANEL_PAD: f32 = 24.;
/// Left edge and usable width of the panel content area.
const PANEL_LEFT: f32 = PANEL.x + PANEL_PAD;
const PANEL_INNER: f32 = PANEL.w - 2. * PANEL_PAD;
/// Tabs of the help panel and the gap between them.
pub const HELP_TAB_COUNT: usize = 5;
const SETTINGS_TAB_COUNT: usize = 4;
const TAB_GAP: f32 = 10.;
const TAB_TOP: f32 = 158.;
const TAB_HEIGHT: f32 = 30.;
/// Tabs share the content width, so the strip always ends on the panel's inner edge
/// no matter how many tabs exist. Input and drawing use this one geometry.
pub fn help_tab_rect(i: usize) -> Rect {
    panel_tab_rect(i, HELP_TAB_COUNT)
}
fn panel_tab_rect(i: usize, count: usize) -> Rect {
    let count = count.max(1) as f32;
    let w = (PANEL_INNER - TAB_GAP * (count - 1.)) / count;
    Rect::new(
        PANEL_LEFT + i as f32 * (w + TAB_GAP),
        TAB_TOP,
        w,
        TAB_HEIGHT,
    )
}
/// Fully visible rows of the opponent grid; further rows scroll.
pub const OPPONENT_VISIBLE_ROWS: usize = 4;
pub struct Ui {
    pub panel: Option<Panel>,
    pub opponent: Option<usize>,
    pub opponent_step: bool,
    pub hero_group: content::HeroGroup,
    pub opponent_group: content::HeroGroup,
    // First visible row of each selectable list. Both clamp to the current roster.
    pub hero_scroll: usize,
    pub opponent_scroll: usize,
    help_tab: usize,
    settings_tab: usize,
    fullscreen: bool,
    // Index into WINDOW_SIZES.
    window: usize,
    pub volume: u32,
    channel_volume: [u32; 3],
    channel_enabled: [bool; 3],
    pub ai_experiment: strategy::AiExperiment,
}
impl Default for Ui {
    fn default() -> Self {
        Self {
            panel: None,
            opponent: None,
            opponent_step: false,
            hero_group: content::HeroGroup::New,
            opponent_group: content::HeroGroup::New,
            hero_scroll: 0,
            opponent_scroll: 0,
            help_tab: 0,
            settings_tab: 0,
            fullscreen: false,
            window: 1,
            volume: 100,
            channel_volume: [100; 3],
            channel_enabled: [true; 3],
            ai_experiment: strategy::AiExperiment::default(),
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
    outline(r.x, r.y, r.w, r.h, 1., GRAY);
}
fn button(r: Rect, text: &str, pointer: Vec2) {
    frame(r, r.contains(pointer));
    label_centered(text, r.x + r.w / 2., r.y + r.h / 2. + 8., 22., WHITE);
}
/// Selection tabs share geometry between drawing and hit testing on both pages.
pub fn group_tab_rect(opponents: bool, index: usize) -> Rect {
    if opponents {
        Rect::new(230. + index as f32 * 288., 164., 276., 44.)
    } else {
        Rect::new(24. + index as f32 * 120., 112., 112., 44.)
    }
}
pub fn draw_group_tabs(opponents: bool, selected: content::HeroGroup, zh: bool, pointer: Vec2) {
    for (i, group) in content::HeroGroup::ALL.into_iter().enumerate() {
        let r = group_tab_rect(opponents, i);
        let key = match group {
            content::HeroGroup::New => "ui.selection.new",
            content::HeroGroup::Legacy => "ui.selection.legacy",
        };
        let text = format!("{} ({})", tr_key(zh, key), group.roster().len());
        frame(r, r.contains(pointer));
        if group == selected {
            outline(r.x, r.y, r.w, r.h, 2., GOLD);
        }
        label_centered(
            &text,
            r.x + r.w / 2.,
            r.y + 29.,
            fit_size(&text, 18., r.w - 12.),
            if group == selected { GOLD } else { WHITE },
        );
    }
}
fn settings_row(i: usize) -> Rect {
    Rect::new(280., 208. + i as f32 * 58., 464., 46.)
}
/// One of the two choices at the right end of a settings toggle row.
fn option_rect(row: usize, i: usize) -> Rect {
    let r = settings_row(row);
    Rect::new(r.x + r.w - 236. + i as f32 * 116., r.y + 7., 112., 32.)
}
fn sound_toggle_rect(row: usize) -> Rect {
    let r = settings_row(row);
    Rect::new(444., r.y + 7., 42., 32.)
}
fn experiment_toggle_rect(row: usize) -> Rect {
    let r = settings_row(row);
    Rect::new(r.x + r.w - 54., r.y + 7., 42., 32.)
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
        if self.panel == Some(Panel::Settings)
            && self.settings_tab == 2
            && is_mouse_button_down(MouseButton::Left)
        {
            self.drag_volume(pointer);
        }
        if click {
            if self.panel == Some(Panel::Settings) {
                for i in 0..SETTINGS_TAB_COUNT {
                    if panel_tab_rect(i, SETTINGS_TAB_COUNT).contains(pointer) {
                        self.settings_tab = i;
                        return true;
                    }
                }
                if self.settings_tab == 2 {
                    for row in 0..4 {
                        if sound_toggle_rect(row).contains(pointer) {
                            if row == 0 {
                                *muted = !*muted;
                            } else {
                                self.channel_enabled[row - 1] = !self.channel_enabled[row - 1];
                            }
                            return true;
                        }
                    }
                }
                if self.settings_tab == 3 {
                    for row in 0..4 {
                        if settings_row(row).contains(pointer) {
                            match row {
                                0 => {
                                    self.ai_experiment.opponent_skills =
                                        !self.ai_experiment.opponent_skills
                                }
                                1 => {
                                    self.ai_experiment.opponent_reply =
                                        !self.ai_experiment.opponent_reply
                                }
                                2 => self.ai_experiment.follow_up = !self.ai_experiment.follow_up,
                                _ => self.ai_experiment.portals = !self.ai_experiment.portals,
                            }
                            return true;
                        }
                    }
                }
            }
            if self.panel == Some(Panel::Help) {
                for i in 0..HELP_TAB_COUNT {
                    if help_tab_rect(i).contains(pointer) {
                        self.help_tab = i;
                        return true;
                    }
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
            if self.panel == Some(Panel::Settings) && self.settings_tab < 2 {
                // Toggle rows: clicking a choice selects it, clicking elsewhere on the
                // row switches to the other one.
                for row in 0..(if self.settings_tab == 0 { 1 } else { 2 }) {
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
                    match (self.settings_tab, row) {
                        (1, 0) => {
                            self.fullscreen = choice == 1;
                            set_fullscreen(self.fullscreen);
                        }
                        (1, 1) => {
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
    /// Effective channel gains retain each slider value when its switch is off.
    pub fn sound_levels(&self) -> [u32; 3] {
        std::array::from_fn(|i| {
            if self.channel_enabled[i] {
                self.channel_volume[i]
            } else {
                0
            }
        })
    }
    fn drag_volume(&mut self, pointer: Vec2) {
        let (start, end) = VOLUME_TRACK;
        for row in 0..4 {
            let r = settings_row(row);
            if Rect::new(start - 10., r.y, end - start + 20., r.h).contains(pointer) {
                let value =
                    (((pointer.x - start) / (end - start)).clamp(0., 1.) * 100.).round() as u32;
                if row == 0 {
                    self.volume = value;
                } else {
                    self.channel_volume[row - 1] = value;
                }
            }
        }
    }
    fn choice(&self, row: usize, zh: bool) -> usize {
        match (self.settings_tab, row) {
            (1, 0) => self.fullscreen as usize,
            (1, 1) => self.window,
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
        let roster = self.opponent_group.roster();
        let total = roster.len() + 1;
        let rows = opponent_row_count(roster.len());
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
        let depth_labels = [
            tr_key(zh, "ui.screen.draw_opponents.depth_beginner"),
            tr_key(zh, "ui.screen.draw_opponents.depth_standard"),
            tr_key(zh, "ui.screen.draw_opponents.depth_advanced"),
        ];
        for (i, name) in [
            tr_key(zh, "ui.shared.beginner"),
            tr_key(zh, "ui.shared.standard"),
            tr_key(zh, "ui.shared.advanced"),
        ]
        .iter()
        .enumerate()
        {
            let r = difficulty_rect(i);
            frame(r, r.contains(pointer));
            if i == difficulty {
                outline(r.x, r.y, r.w, r.h, 2., GOLD);
            }
            label_centered(name, r.x + r.w / 2., r.y + 23., 19., WHITE);
            let depth = depth_labels[i];
            label_centered(
                depth,
                r.x + r.w / 2.,
                r.y + 36.,
                fit_size(depth, 11., r.w - 16.),
                LIGHTGRAY,
            );
        }
        draw_group_tabs(true, self.opponent_group, zh, pointer);
        label(
            tr_key(zh, "ui.selection.group_hint"),
            230.,
            616.,
            18.,
            LIGHTGRAY,
        );
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
                    self.opponent == Some(roster[i - 1])
                };
                if selected {
                    outline(r.x, r.y, r.w, r.h, 2., GOLD);
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
                        tr_key(zh, "ui.selection.random_in_group"),
                        r.x + 14.,
                        r.y + 61.,
                        fit_size(tr_key(zh, "ui.selection.random_in_group"), 17., r.w - 28.),
                        LIGHTGRAY,
                    );
                } else {
                    let id = roster[i - 1];
                    let h = &HEROES[id];
                    draw_texture_ex(
                        thumbs,
                        r.x + 8.,
                        r.y + 12.,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(40., 48.)),
                            source: Some(Rect::new(
                                crate::sprites::thumbnail_index(id) as f32 * 40.,
                                0.,
                                40.,
                                48.,
                            )),
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
                    let skills = h
                        .skills
                        .iter()
                        .map(|skill| skill.name(zh))
                        .collect::<Vec<_>>()
                        .join(" / ");
                    label(
                        &skills,
                        r.x + 8.,
                        r.y + 68.,
                        fit_size(&skills, if zh { 16. } else { 13. }, r.w - 16.),
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
                    .then(|| roster[i - 1])
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
        draw_rectangle(
            PANEL.x,
            PANEL.y,
            PANEL.w,
            PANEL.h,
            Color::from_rgba(24, 30, 43, 255),
        );
        outline(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 2., SKYBLUE);
        label(
            match panel {
                Panel::Settings => tr_key(zh, "ui.screen.draw_panel.settings"),
                Panel::Help => tr_key(zh, "ui.screen.draw_panel.tile_guide"),
            },
            PANEL_LEFT,
            148.,
            28.,
            GOLD,
        );
        button(Rect::new(758., 119., 42., 38.), "×", pointer);
        if panel == Panel::Help {
            for (i, text) in [
                tr_key(zh, "ui.screen.draw_panel.tiles_matching"),
                tr_key(zh, "ui.screen.draw_panel.awakening"),
                tr_key(zh, "ui.screen.draw_panel.fatigue_tab"),
                tr_key(zh, "help.gravity.tab"),
                tr_key(zh, "help.portals.tab"),
            ]
            .iter()
            .enumerate()
            {
                let r = help_tab_rect(i);
                frame(r, self.help_tab == i);
                label_centered(
                    text,
                    r.x + r.w / 2.,
                    r.y + 21.,
                    fit_size(text, 17., r.w - 12.),
                    WHITE,
                );
            }
            if self.help_tab >= 3 {
                help::draw(self.help_tab, zh, tiles);
                return;
            }
            if self.help_tab >= 1 {
                // Awakening explains the mechanic; endgame pressure explains the ramp.
                // Every hero's own numbers live on its awakening page.
                let lines = if self.help_tab == 1 {
                    [
                        (
                            tr_key(
                                zh,
                                "ui.screen.draw_panel.damage_or_hp_payment_triggers_a_ne",
                            ),
                            GOLD,
                        ),
                        (
                            tr_key(
                                zh,
                                "ui.screen.draw_panel.the_triggering_hit_cannot_spill_ov",
                            ),
                            LIGHTGRAY,
                        ),
                        (
                            tr_key(zh, "ui.screen.draw_panel.awakening_numbers"),
                            SKYBLUE,
                        ),
                    ]
                } else {
                    [
                        (tr_key(zh, "ui.screen.draw_panel.fatigue"), GOLD),
                        (tr_key(zh, "ui.screen.draw_panel.fatigue_limits"), LIGHTGRAY),
                        (tr_key(zh, "ui.screen.draw_panel.fatigue_strip"), SKYBLUE),
                    ]
                };
                for (i, (text, color)) in lines.iter().enumerate() {
                    label(
                        text,
                        PANEL_LEFT,
                        224. + i as f32 * 38.,
                        fit_size(text, 17., PANEL_INNER),
                        *color,
                    );
                }
                return;
            }
        }
        match panel {
            Panel::Settings => {
                for (i, key) in [
                    "settings.general",
                    "settings.display",
                    "settings.sound",
                    "settings.experiment",
                ]
                .iter()
                .enumerate()
                {
                    let r = panel_tab_rect(i, SETTINGS_TAB_COUNT);
                    frame(r, self.settings_tab == i);
                    label(tr_key(zh, key), r.x + 10., r.y + 21., 17., WHITE);
                }
                if self.settings_tab == 3 {
                    for (row, (key, on, note)) in [
                        (
                            "settings.experiment.opponent_skills",
                            self.ai_experiment.opponent_skills,
                            "settings.experiment.opponent_skills_note",
                        ),
                        (
                            "settings.experiment.opponent_reply",
                            self.ai_experiment.opponent_reply,
                            "settings.experiment.opponent_reply_note",
                        ),
                        (
                            "settings.experiment.follow_up",
                            self.ai_experiment.follow_up,
                            "settings.experiment.follow_up_note",
                        ),
                        (
                            "settings.experiment.portals",
                            self.ai_experiment.portals,
                            "settings.experiment.portals_note",
                        ),
                    ]
                    .iter()
                    .enumerate()
                    {
                        let r = settings_row(row);
                        frame(r, r.contains(pointer));
                        let name = tr_key(zh, key);
                        label(name, r.x + 12., r.y + 20., fit_size(name, 19., 380.), WHITE);
                        let note = tr_key(zh, note);
                        label(
                            note,
                            r.x + 12.,
                            r.y + 38.,
                            fit_size(note, 13., 380.),
                            LIGHTGRAY,
                        );
                        let toggle = experiment_toggle_rect(row);
                        frame(toggle, *on);
                        let text = tr_key(
                            zh,
                            if *on {
                                "ui.screen.draw_panel.on"
                            } else {
                                "ui.screen.draw_panel.off"
                            },
                        );
                        label_centered(
                            text,
                            toggle.x + toggle.w / 2.,
                            toggle.y + 21.,
                            14.,
                            if *on { GOLD } else { GRAY },
                        );
                    }
                    let note = tr_key(zh, "settings.experiment.note");
                    label(
                        note,
                        PANEL_LEFT,
                        470.,
                        fit_size(note, 16., PANEL_INNER),
                        SKYBLUE,
                    );
                } else if self.settings_tab == 2 {
                    for (row, key) in [
                        "settings.master_volume",
                        "settings.music",
                        "settings.effects",
                        "settings.voice",
                    ]
                    .iter()
                    .enumerate()
                    {
                        let r = settings_row(row);
                        frame(r, false);
                        let name = tr_key(zh, key);
                        label(name, r.x + 12., r.y + 30., fit_size(name, 20., 144.), WHITE);
                        let enabled = if row == 0 {
                            !muted
                        } else {
                            self.channel_enabled[row - 1]
                        };
                        let toggle = sound_toggle_rect(row);
                        frame(toggle, enabled);
                        let text = tr_key(
                            zh,
                            if enabled {
                                "ui.screen.draw_panel.on"
                            } else {
                                "ui.screen.draw_panel.off"
                            },
                        );
                        label_centered(
                            text,
                            toggle.x + toggle.w / 2.,
                            toggle.y + 21.,
                            14.,
                            if enabled { GOLD } else { GRAY },
                        );
                        let value = if row == 0 {
                            self.volume
                        } else {
                            self.channel_volume[row - 1]
                        };
                        let (start, end) = VOLUME_TRACK;
                        let y = r.y + 23.;
                        let x = start + (end - start) * value as f32 / 100.;
                        draw_line(start, y, end, y, 5., GRAY);
                        draw_line(start, y, x, y, 5., if enabled { SKYBLUE } else { GRAY });
                        draw_circle(x, y, 8., WHITE);
                        label_centered(
                            &format_text(zh, "settings.percent", &[&value.to_string()]),
                            707.,
                            y + 7.,
                            20.,
                            SKYBLUE,
                        );
                    }
                } else {
                    let windows = WINDOW_SIZES.map(|(w, h)| {
                        format_text(
                            zh,
                            "settings.window_size",
                            &[&w.to_string(), &h.to_string()],
                        )
                    });
                    let entries = if self.settings_tab == 0 {
                        vec![(
                            "ui.screen.draw_panel.language",
                            [
                                tr_key(zh, "settings.language_chinese"),
                                tr_key(zh, "settings.language_english"),
                            ],
                        )]
                    } else {
                        vec![
                            (
                                "ui.screen.draw_panel.fullscreen",
                                [
                                    tr_key(zh, "ui.screen.draw_panel.off"),
                                    tr_key(zh, "ui.screen.draw_panel.on"),
                                ],
                            ),
                            (
                                "ui.screen.draw_panel.window",
                                [windows[0].as_str(), windows[1].as_str()],
                            ),
                        ]
                    };
                    for (row, (key, choices)) in entries.iter().enumerate() {
                        let r = settings_row(row);
                        frame(r, r.contains(pointer));
                        label(tr_key(zh, key), r.x + 12., r.y + 31., 22., WHITE);
                        let selected = self.choice(row, zh);
                        for (i, text) in choices.iter().enumerate() {
                            let o = option_rect(row, i);
                            draw_rectangle(o.x, o.y, o.w, o.h, Color::from_rgba(24, 30, 43, 255));
                            let on = i == selected;
                            outline(
                                o.x,
                                o.y,
                                o.w,
                                o.h,
                                if on { 2. } else { 1. },
                                if on { GOLD } else { GRAY },
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
                }
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
                // Rules that apply to the whole board, wrapped into the panel width.
                for (text, y, size, color) in [
                    (
                        tr_key(
                            zh,
                            "ui.screen.draw_panel.match_3_identical_tiles_damage_hit",
                        ),
                        556.,
                        20.,
                        WHITE,
                    ),
                    (
                        tr_key(
                            zh,
                            "ui.screen.draw_panel.chains_10_per_step_up_to_130_skill",
                        ),
                        584.,
                        19.,
                        LIGHTGRAY,
                    ),
                ] {
                    label(
                        text,
                        PANEL_LEFT,
                        y,
                        fit_size(text, size, PANEL_INNER),
                        color,
                    );
                }
            }
        }
        // The hint is part of the panel; it must never fall outside the frame.
        label(
            tr_key(zh, "ui.screen.draw_panel.esc_close"),
            PANEL_LEFT,
            PANEL.y + PANEL.h - 12.,
            16.,
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
pub fn opponent_row_count(roster_len: usize) -> usize {
    (roster_len + 1).div_ceil(2)
}
pub fn opponent_list_rect() -> Rect {
    Rect::new(230., 220., 564., 88. * OPPONENT_VISIBLE_ROWS as f32)
}
/// Grid cell for an on-screen row and column, independent of the scroll offset.
pub fn opponent_rect(row: usize, column: usize) -> Rect {
    Rect::new(
        230. + column as f32 * 288.,
        220. + row as f32 * 88.,
        276.,
        80.,
    )
}

pub fn pause_rect(i: usize) -> Rect {
    Rect::new(322., 270. + i as f32 * 63., 380., 48.)
}
// Between the panel title and the first button; the battle HUD leaves no row for it.
pub fn pause_difficulty_rect() -> Rect {
    Rect::new(322., 242., 380., 20.)
}
pub fn draw_pause(zh: bool, difficulty: usize, pointer: Vec2) {
    flush_labels();
    draw_rectangle(0., 0., 1024., 720., Color::new(0., 0., 0., 0.72));
    draw_rectangle(282., 180., 460., 390., Color::from_rgba(24, 30, 43, 255));
    outline(282., 180., 460., 390., 2., SKYBLUE);
    label(
        tr_key(zh, "ui.screen.draw_pause.paused"),
        322.,
        232.,
        34.,
        GOLD,
    );
    let level = [
        tr_key(zh, "ui.shared.beginner"),
        tr_key(zh, "ui.shared.standard"),
        tr_key(zh, "ui.shared.advanced"),
    ][difficulty.min(2)];
    let depth = [
        tr_key(zh, "ui.screen.draw_opponents.depth_beginner"),
        tr_key(zh, "ui.screen.draw_opponents.depth_standard"),
        tr_key(zh, "ui.screen.draw_opponents.depth_advanced"),
    ][difficulty.min(2)];
    let difficulty_text = format_text(zh, "ui.screen.draw_pause.ai_difficulty", &[level, depth]);
    let difficulty_rect = pause_difficulty_rect();
    label(
        &difficulty_text,
        difficulty_rect.x,
        difficulty_rect.bottom() - 4.,
        fit_size(&difficulty_text, 16., difficulty_rect.w),
        LIGHTGRAY,
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

#[cfg(test)]
mod sound_settings_tests {
    use super::*;
    #[test]
    fn settings_tabs_fill_the_panel_independently_of_help_tabs() {
        let first = panel_tab_rect(0, SETTINGS_TAB_COUNT);
        let last = panel_tab_rect(SETTINGS_TAB_COUNT - 1, SETTINGS_TAB_COUNT);
        assert_eq!(first.x, PANEL_LEFT);
        assert!((last.x + last.w - PANEL_LEFT - PANEL_INNER).abs() < 0.001);
        assert!(first.w > help_tab_rect(0).w);
    }
    #[test]
    fn experiment_rows_and_toggles_stay_inside_the_settings_panel() {
        assert_eq!(SETTINGS_TAB_COUNT, 4);
        let defaults = strategy::AiExperiment::default();
        assert!(
            defaults.opponent_skills
                && defaults.opponent_reply
                && defaults.follow_up
                && defaults.portals
        );
        for row in 0..4 {
            let r = settings_row(row);
            assert!(r.x >= PANEL.x && r.right() <= PANEL.right());
            assert!(r.y >= PANEL.y && r.bottom() <= PANEL.bottom());
            let toggle = experiment_toggle_rect(row);
            assert!(toggle.x >= r.x && toggle.right() <= r.right());
            assert!(toggle.y >= r.y && toggle.bottom() <= r.bottom());
        }
    }

    #[test]
    fn category_switches_preserve_slider_values_and_other_channels() {
        let mut ui = Ui::default();
        assert_eq!(ui.sound_levels(), [100; 3]);
        ui.channel_volume = [25, 40, 60];
        ui.channel_enabled[1] = false;
        assert_eq!(ui.sound_levels(), [25, 0, 60]);
        ui.volume = 0;
        ui.volume = 100;
        assert_eq!(ui.sound_levels(), [25, 0, 60]);
        ui.channel_enabled[1] = true;
        assert_eq!(ui.sound_levels(), [25, 40, 60]);
    }
    #[test]
    fn sound_slider_hit_boxes_do_not_toggle_or_drag_neighbours() {
        let mut ui = Ui::default();
        let row = settings_row(2);
        ui.drag_volume(vec2(VOLUME_TRACK.0, row.y + 20.));
        assert_eq!(ui.channel_volume, [100, 0, 100]);
        ui.drag_volume(vec2(VOLUME_TRACK.1, row.y + 20.));
        assert_eq!(ui.channel_volume, [100; 3]);
        ui.drag_volume(vec2(sound_toggle_rect(1).x + 5., settings_row(1).y + 20.));
        assert_eq!(ui.channel_volume, [100; 3]);
        for row in 0..4 {
            assert!(settings_row(row).y + settings_row(row).h < PANEL.y + PANEL.h);
            assert!(sound_toggle_rect(row).x + sound_toggle_rect(row).w < VOLUME_TRACK.0 - 10.);
        }
    }
}

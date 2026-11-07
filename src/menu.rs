use super::*;
pub struct Preview {
    pub skill: usize,
    pub board: Board,
    pub changed: Board,
    pub cells: Vec<(usize, usize)>,
    pub since: f64,
    /// False for skills that leave every tile in place; those preview the hero instead.
    pub affects_tiles: bool,
    /// Silhouette after the cast (e.g. the nun's frenzy form turns her dark).
    pub dark: bool,
    /// Silhouette the menu shows before the cast, i.e. the current awakening form.
    pub base_dark: bool,
    /// Status the cast puts on the hero, shown with the character preview.
    pub status: Option<content::StatusId>,
    /// The cast lifts the hero off the ground (the jump skill).
    pub hop: bool,
}
impl Preview {
    /// `awakened` selects the skill list the menu is showing: the awakening page can
    /// replace an active, so the preview must follow the page instead of always
    /// reading the awakened list.
    pub fn new(hero: usize, skill: usize, awakened: bool) -> Self {
        Self::with_time(hero, skill, awakened, get_time())
    }
    pub fn new_static(hero: usize, skill: usize, awakened: bool) -> Self {
        Self::with_time(hero, skill, awakened, 0.)
    }
    fn with_time(hero: usize, skill: usize, awakened: bool, since: f64) -> Self {
        let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        let mut changed = board;
        let effect = effect(content::skills(hero, awakened)[skill], &mut changed);
        let mut cells = effect.cells;
        for y in 0..N {
            for x in 0..N {
                if changed[y][x] != board[y][x] && !cells.contains(&(x, y)) {
                    cells.push((x, y));
                }
            }
        }
        if content::skills(hero, awakened)[skill] == content::Skill::Landslide {
            cells = vec![(2, 3), (3, 3), (2, 2), (3, 2), (2, 4), (3, 4)];
            for y in 2..=4 {
                swap(&mut changed, (2, y), (3, y));
            }
        }
        if content::skills(hero, awakened)[skill] == content::Skill::Strike {
            cells = vec![(2, 3), (3, 3), (4, 3)];
            for x in 2..=4 {
                changed[3][x] = 0;
            }
        }
        // The skill's own cast rules decide both the resulting status and whether the
        // hero changes form, so the preview never grows a per-skill branch.
        let cast = content::skills(hero, awakened)[skill];
        let mut state = crate::role_state::RoleState::default();
        state.on_cast(cast, &board, (2, 3));
        let status = state.statuses.0.iter().flatten().map(|s| s.id).next();
        // Appearance before and after the cast. `awakened` is the menu page's state and
        // statuses can flip it either way: frenzy darkens, returning to reason clears it.
        let base_dark = sprites::shadow_form(hero, awakened, content::StatusSet::default());
        let dark = sprites::shadow_form(hero, awakened, state.statuses);
        let hop = content::skill_profile(cast, content::Rules::default()).hop;
        let affects_tiles = !cells.is_empty() || changed != board;
        Self {
            skill,
            board,
            changed,
            cells,
            since,
            affects_tiles,
            dark,
            base_dark,
            status,
            hop,
        }
    }
}
/// Final page is the character file; pages 0..2 are resources, passives and awakening.
pub const CHARACTER_PAGE_COUNT: usize = 4;
/// Rows of the hero list that fit without covering the key hints below it.
pub const HERO_VISIBLE_ROWS: usize = 8;
pub const HERO_ROW_STEP: f32 = 53.;
/// On-screen row rectangle. Map it back to a roster entry through the scroll offset.
pub fn hero_rect(row: usize) -> Rect {
    Rect::new(24., 164. + row as f32 * HERO_ROW_STEP, 232., 48.)
}
pub fn hero_list_rect() -> Rect {
    Rect::new(24., 164., 232., HERO_ROW_STEP * HERO_VISIBLE_ROWS as f32)
}
const SKILL_X: f32 = 548.;
const SKILL_Y: f32 = 126.;
const SKILL_W: f32 = 452.;
const SKILL_MIN_H: f32 = 88.;
const SKILL_GAP: f32 = 6.;
const SKILL_BOTTOM: f32 = 414.;

fn skill_description_lines(skill: content::Skill, awake: bool, zh: bool) -> Vec<String> {
    wrap_label_text(
        content::skill_description(skill, awake, zh),
        16.,
        SKILL_W - 28.,
    )
}

fn skill_max_lines(rect: Rect) -> usize {
    (((rect.h - 49. - 5.) / 17.).floor().max(0.) as usize) + 1
}

/// Lay skill cards out from their wrapped description height. Most cards keep
/// the original 88px; a card with four or more lines grows instead of spilling.
pub fn skill_rects(hero: usize, awake: bool, zh: bool) -> Vec<Rect> {
    let mut rects = Vec::new();
    let mut y = SKILL_Y;
    for skill in content::skills(hero, awake) {
        let lines = skill_description_lines(*skill, awake, zh).len();
        let height = if lines > 3 {
            (49. + lines as f32 * 17. + SKILL_GAP).max(SKILL_MIN_H)
        } else {
            SKILL_MIN_H
        };
        rects.push(Rect::new(SKILL_X, y, SKILL_W, height));
        y += height + SKILL_GAP;
    }
    // Future multi-skill descriptions may not all fit. Equalize rather than
    // overlapping; the draw pass truncates to each card's visible line count.
    if !rects.is_empty() && y - SKILL_GAP > SKILL_BOTTOM {
        let count = rects.len();
        let equal =
            ((SKILL_BOTTOM - SKILL_Y - SKILL_GAP * (count - 1) as f32) / count as f32).max(70.);
        let mut y = SKILL_Y;
        for rect in &mut rects {
            rect.y = y;
            rect.h = equal;
            y += equal + SKILL_GAP;
        }
    }
    rects
}
pub fn start_rect() -> Rect {
    Rect::new(284., 651., 716., 48.)
}
fn panel(rect: Rect, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
    outline(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.,
        Color::from_rgba(66, 71, 83, 255),
    );
}
fn sprite(texture: &Texture2D, asset: &SpriteAsset, area: Rect) {
    sprite_frames(texture, asset, area, get_time(), asset.idle());
}
/// Draw one frame of an arbitrary atlas row (e.g. the hero's cast row).
fn sprite_frames(
    texture: &Texture2D,
    asset: &SpriteAsset,
    area: Rect,
    time: f64,
    clip: sprites::AnimationClip,
) {
    let frame = clip.frame(time);
    let new_role = asset.anchor.is_some();
    let scale = if new_role {
        crate::role_scale(area) * 96. / asset.w
    } else {
        (area.w / asset.w).min(area.h / asset.h)
    };
    let size = vec2(asset.w * scale, asset.h * scale);
    let (size, rotation) =
        crate::presentation::idle_pose(size, get_time(), clip.indices.len() == 1);
    draw_texture_ex(
        texture,
        area.x + (area.w - size.x) / 2.,
        if let Some(anchor) = asset.anchor {
            area.y + area.h - anchor[1] * scale
        } else {
            area.y + area.h - size.y
        },
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation,
            pivot: Some(vec2(area.x + area.w / 2., area.y + area.h)),
            source: Some(Rect::new(
                (frame % (texture.width() / asset.w) as usize) as f32 * asset.w,
                (frame / (texture.width() / asset.w) as usize) as f32 * asset.h,
                asset.w,
                asset.h,
            )),
            ..Default::default()
        },
    );
}
/// Preview for a skill that leaves the board alone: the hero casts and, when the
/// skill changes the hero's form, switches silhouette after the cast point.
fn draw_skill_actor(
    hero: usize,
    preview: &Preview,
    after: bool,
    texture: &Texture2D,
    awakened: &Texture2D,
) {
    let area = Rect::new(558., 428., 200., 184.);
    let center = area.x + area.w / 2.;
    let dark = if after {
        preview.dark
    } else {
        preview.base_dark
    };
    let t = (((get_time() - preview.since) % 3.) / 3. - 0.45) / 0.55;
    let t = t.clamp(0., 1.) as f32;
    // The jump skill lifts the hero; its shadow and the cast ring stay on the ground.
    let lift = if after && preview.hop {
        (std::f32::consts::PI * (t * 1.6).min(1.)).sin() * 30.
    } else {
        0.
    };
    draw_ellipse(
        center,
        area.y + area.h,
        74. - lift * 0.5,
        11. - lift * 0.08,
        0.,
        Color::from_rgba(20, 23, 31, 255),
    );
    if after {
        // Cast burst: a ring expanding from the feet, red while the form turns hostile.
        let fade = 1. - t;
        let color = if preview.dark {
            Color::new(0.9, 0.3, 0.35, 0.7 * fade)
        } else {
            Color::new(0.45, 0.72, 1., 0.6 * fade)
        };
        draw_circle(
            center,
            area.y + area.h - 54.,
            46. + t * 10.,
            Color::new(color.r, color.g, color.b, color.a * 0.28),
        );
        draw_circle_lines(center, area.y + area.h - 54., 30. + t * 26., 2., color);
    }
    let (texture, asset) = if dark {
        (awakened, &sprites::AWAKENED[hero])
    } else {
        (texture, &sprites::SPRITES[hero])
    };
    let area = Rect::new(area.x, area.y - lift, area.w, area.h);
    // After the cast point the hero performs the skill: play its cast row (the same
    // frames the battle uses) instead of standing in the idle loop.
    let selected_skill = content::skills(hero, preview.base_dark)[preview.skill];
    let actor_clip = sprites::cast_clip(hero, dark, Some(selected_skill));
    match actor_clip.filter(|_| after) {
        Some(clip) => sprite_frames(
            texture,
            asset,
            area,
            // One cast gesture, then hold its last key for the rest of the phase.
            t as f64
                * if asset.legacy_awake() {
                    clip.duration()
                } else {
                    1.65
                },
            clip,
        ),
        None => sprite(texture, asset, area),
    }
}
pub fn draw(
    hero: usize,
    group: content::HeroGroup,
    zh: bool,
    texture: &Texture2D,
    thumbs: &Texture2D,
    tiles: &[Texture2D; 7],
    preview: &Preview,
    pointer: Vec2,
    page: usize,
    scroll: usize,
    awakened_texture: &Texture2D,
) {
    let mut resource_hint = None;
    let dim = Color::from_rgba(29, 33, 43, 255);
    let card = Color::from_rgba(35, 40, 52, 255);
    label(
        tr_key(zh, "ui.menu.draw.step_1_choose_your_hero"),
        24.,
        58.,
        34.,
        GOLD,
    );
    label(
        tr_key(zh, "ui.menu.draw.preview_your_hero_and_skills_befor"),
        26.,
        88.,
        19.,
        LIGHTGRAY,
    );
    ui::draw_group_tabs(false, group, zh, pointer);
    label(
        tr_key(zh, "ui.selection.group_hint"),
        26.,
        624.,
        18.,
        LIGHTGRAY,
    );
    let roster = group.roster();
    let scroll = ui::clamp_scroll(scroll, roster.len(), HERO_VISIBLE_ROWS);
    let visible_rows = HERO_VISIBLE_ROWS.min(roster.len().saturating_sub(scroll));
    for row in 0..visible_rows {
        let i = roster[scroll + row];
        let h = &HEROES[i];
        let r = hero_rect(row);
        panel(
            r,
            if i == hero || r.contains(pointer) {
                Color::from_rgba(48, 53, 64, 255)
            } else {
                dim
            },
        );
        if i == hero {
            draw_rectangle(r.x, r.y, 4., r.h, GOLD);
        }
        draw_texture_ex(
            thumbs,
            r.x + 12.,
            r.y + 8.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(32., 38.)),
                source: Some(Rect::new(
                    crate::sprites::thumbnail_index(i) as f32 * 40.,
                    0.,
                    40.,
                    48.,
                )),
                ..Default::default()
            },
        );
        label(
            if zh { h.zh } else { h.en },
            r.x + 67.,
            r.y + 23.,
            fit_size(if zh { h.zh } else { h.en }, 20., r.w - 75.),
            if i == hero { GOLD } else { WHITE },
        );
        label(
            &format!(
                "{} {}   {} {}",
                tr_key(zh, "ui.shared.hp"),
                h.hp,
                tr_key(zh, "ui.shared.mp"),
                content::mana_cap(i, false)
            ),
            r.x + 67.,
            r.y + 44.,
            14.,
            LIGHTGRAY,
        );
    }
    ui::draw_scrollbar(hero_list_rect(), scroll, roster.len(), HERO_VISIBLE_ROWS);
    label(
        tr_key(zh, "ui.menu.draw.select_hero"),
        26.,
        662.,
        18.,
        LIGHTGRAY,
    );
    label(
        tr_key(zh, "ui.menu.draw.click_a_skill_to_preview"),
        26.,
        688.,
        18.,
        LIGHTGRAY,
    );
    let h = &HEROES[hero];
    panel(Rect::new(284., 112., 244., 518.), card);
    let name = if zh { h.zh } else { h.en };
    label(name, 302., 151., fit_size(name, 34., 208.), GOLD);
    label(
        tr_key(zh, "ui.menu.draw.animated_preview"),
        302.,
        181.,
        16.,
        LIGHTGRAY,
    );
    let detail_page = page != 0;
    let portrait_height = if detail_page { 140. } else { 190. };
    let shadow_y = if detail_page { 333. } else { 383. };
    draw_ellipse(
        406.,
        shadow_y,
        82.,
        12.,
        0.,
        Color::from_rgba(20, 23, 31, 255),
    );
    let portrait_asset = if page == 2 {
        &sprites::AWAKENED[hero]
    } else {
        &sprites::SPRITES[hero]
    };
    let portrait_texture = if page == 2 { awakened_texture } else { texture };
    let portrait_area = Rect::new(291., 198., 230., portrait_height);
    if portrait_asset.legacy_awake() {
        let cycle = (get_time() - preview.since).max(0.) % 3.;
        let skill = content::skills(hero, true)[preview.skill];
        let cast = portrait_asset.actor_cast(skill).unwrap().clip;
        let (clip, elapsed) = if (1.35..2.75).contains(&cycle) {
            (cast, cycle - 1.35)
        } else {
            (
                portrait_asset.idle(),
                if cycle >= 2.75 {
                    cycle - 2.75
                } else {
                    cycle + 0.25
                },
            )
        };
        sprite_frames(
            portrait_texture,
            portrait_asset,
            portrait_area,
            elapsed,
            clip,
        );
    } else {
        sprite(portrait_texture, portrait_asset, portrait_area);
    }
    if page == 0 {
        label(
            tr_key(zh, "ui.menu.draw.max_health"),
            304.,
            412.,
            17.,
            LIGHTGRAY,
        );
        label(&h.hp.to_string(), 304., 439., 28., GREEN);
        draw_rectangle(304., 449., 204., 9., GREEN);
        let trigger_x = 304. + 204. * content::AWAKENING_LINES[hero] as f32 / 100.;
        let cap_x = 304. + 204. * content::AWAKENING_CAPS[hero] as f32 / 100.;
        crate::draw_diamond(trigger_x, 444., 5., GOLD, true);
        draw_line(cap_x, 458., cap_x, 463., 1., SKYBLUE);
        crate::draw_diamond(cap_x, 464., 4., SKYBLUE, false);
        crate::draw_diamond(309., 480., 4., GOLD, true);
        crate::draw_diamond(309., 502., 4., SKYBLUE, false);
        let awakening_line = content::awakening_line(hero).to_string();
        let awakening_percent = content::AWAKENING_LINES[hero].to_string();
        let trigger = crate::format_text(
            zh,
            "hero.hp_awaken_trigger",
            &[&awakening_line, &awakening_percent],
        );
        let hp_cap = content::health_cap(hero, true).to_string();
        let hp_cap_percent = content::AWAKENING_CAPS[hero].to_string();
        let recovered = crate::format_text(zh, "hero.hp_new_cap", &[&hp_cap, &hp_cap_percent]);
        label(&trigger, 320., 485., fit_size(&trigger, 15., 190.), GOLD);
        label(
            &recovered,
            320.,
            507.,
            fit_size(&recovered, 15., 190.),
            SKYBLUE,
        );
        label(
            tr_key(zh, "ui.menu.draw.max_mana"),
            304.,
            540.,
            17.,
            LIGHTGRAY,
        );
        let mana = content::mana_cap(hero, false);
        label(&mana.to_string(), 304., 566., 28., SKYBLUE);
        draw_rectangle(304., 578., 204., 9., SKYBLUE);
        if content::mana_cap(hero, false) < content::mana_cap(hero, true) {
            let unlocked = 204. * mana as f32 / content::mana_cap(hero, true) as f32;
            crate::draw_lock(304. + unlocked - 5.5, 574., SKYBLUE);
            let new_mana = content::mana_cap(hero, true).to_string();
            let text = crate::format_text(zh, "hero.mana_unlocked", &[&new_mana]);
            label(&text, 508. - label_width(&text, 14.), 566., 14., LIGHTGRAY);
        }
    } else if page == 1 {
        label(
            tr_key(zh, "ui.menu.draw.passive_skill"),
            304.,
            373.,
            22.,
            GOLD,
        );
        draw_passive_entries(content::passives(hero, false), zh, false);
    } else if page == 2 {
        label(tr_key(zh, "ui.menu.draw.awakening"), 304., 373., 22., GOLD);
        let changes = content::passives(hero, true)
            .iter()
            .copied()
            .filter(|p| !content::passives(hero, false).contains(p))
            .collect::<Vec<_>>();
        draw_passive_entries(&changes, zh, true);
    } else {
        label(tr_key(zh, "ui.menu.draw.lore_intro"), 304., 373., 22., GOLD);
        if let Some(lore) = content::role_lore(hero) {
            let (size, step, limit, max) = if zh {
                (13., 17., 15.4, 12)
            } else {
                (12., 15., 16.8, 14)
            };
            let lines = wrap_preview_text(lore.intro(zh), limit);
            for (i, text) in lines.iter().take(max).enumerate() {
                label(text, 304., 401. + i as f32 * step, size, LIGHTGRAY);
            }
        }
    }
    if page == 0 {
        let old_mana = content::mana_cap(hero, false);
        let new_mana = content::mana_cap(hero, true);
        let hint = if Rect::new(304., 438., 204., 35.).contains(pointer) {
            let line = content::awakening_line(hero).to_string();
            let threshold = content::AWAKENING_LINES[hero].to_string();
            let cap = content::health_cap(hero, true).to_string();
            let cap_percent = content::AWAKENING_CAPS[hero].to_string();
            Some(crate::format_text(
                zh,
                "hero.awakening_tooltip",
                &[&line, &threshold, &cap, &cap_percent],
            ))
        } else if Rect::new(304., 568., 204., 28.).contains(pointer) {
            Some(if old_mana < new_mana {
                let old = old_mana.to_string();
                let new = new_mana.to_string();
                crate::format_text(zh, "hero.mana_cap_tooltip", &[&old, &new])
            } else {
                let old = old_mana.to_string();
                crate::format_text(zh, "hero.mana_cap_basic", &[&old])
            })
        } else {
            None
        };
        if let Some(text) = hint {
            // Draw after the preview so the resource tooltip stays above it.
            resource_hint = Some(text);
        }
    }
    for (i, text) in [tr_key(zh, "ui.shared.prev"), tr_key(zh, "ui.shared.next")]
        .iter()
        .enumerate()
    {
        let r = page_rect(i);
        panel(r, dim);
        label(
            text,
            r.x + 7.,
            r.y + 16.,
            14.,
            if (i == 0 && page == 0) || (i == 1 && page == CHARACTER_PAGE_COUNT - 1) {
                GRAY
            } else {
                SKYBLUE
            },
        );
    }
    label(
        &format!("{} / {}", page + 1, CHARACTER_PAGE_COUNT),
        388.,
        623.,
        14.,
        WHITE,
    );
    if page == CHARACTER_PAGE_COUNT - 1 {
        draw_lore(hero, zh);
    } else {
        let visible_skills = content::skills(hero, page == 2);
        let skill_rects = skill_rects(hero, page == 2, zh);
        for (i, (skill, r)) in visible_skills.iter().zip(skill_rects).enumerate() {
            panel(
                r,
                if i == preview.skill {
                    Color::from_rgba(44, 48, 59, 255)
                } else {
                    dim
                },
            );
            if i == preview.skill {
                outline(r.x, r.y, r.w, r.h, 2., GOLD);
            }
            icons::draw(*skill, r.x + 12., r.y + 8., 28.);
            let details = format!(
                "{} {} · {} {}",
                content::skill_cost(hero, page == 2, *skill),
                tr_key(zh, "ui.shared.mp"),
                tr_key(zh, "ui.shared.cd"),
                skill.cooldown()
            );
            let details_size = fit_size(&details, 18., r.w * 0.46);
            let details_width = label_width(&details, details_size);
            let details_x = r.x + r.w - 14. - details_width;
            let title = format!("{}  {}", i + 1, skill.name(zh));
            label(
                &title,
                r.x + 50.,
                r.y + 28.,
                fit_size(&title, 21., (details_x - (r.x + 58.)).max(1.)),
                WHITE,
            );
            label(&details, details_x, r.y + 27., details_size, SKYBLUE);
            let description = skill_description_lines(*skill, page == 2, zh);
            for (line, text) in description.iter().take(skill_max_lines(r)).enumerate() {
                label(
                    text,
                    r.x + 14.,
                    r.y + 49. + line as f32 * 17.,
                    fit_size(text, 16., r.w - 28.),
                    LIGHTGRAY,
                );
            }
        }
        label(
            tr_key(zh, "ui.menu.draw.skills_click_to_preview"),
            550.,
            112.,
            18.,
            LIGHTGRAY,
        );
        let skill = visible_skills[preview.skill.min(visible_skills.len() - 1)];
        let phase = ((get_time() - preview.since) % 3.0) / 3.;
        let after = phase > 0.45;
        panel(Rect::new(548., 422., 452., 208.), dim);
        if preview.affects_tiles {
            let clear = matches!(
                skill,
                content::Skill::Charge
                    | content::Skill::Guard
                    | content::Skill::Lightning
                    | content::Skill::Meteor
                    | content::Skill::Greedy
                    | content::Skill::Imp
                    | content::Skill::Chort
                    | content::Skill::Rush
                    | content::Skill::Hammer
                    | content::Skill::Entomb
                    | content::Skill::EmberBlast
            );
            for y in 0..N {
                for x in 0..N {
                    let px = 565. + x as f32 * 23.;
                    let py = 434. + y as f32 * 23.;
                    draw_rectangle(px, py, 22., 22., Color::from_rgba(77, 69, 64, 255));
                    let affected = preview.cells.contains(&(x, y));
                    let board = if after {
                        &preview.changed
                    } else {
                        &preview.board
                    };
                    draw_texture_ex(
                        &tiles[board[y][x] as usize],
                        px,
                        py,
                        if after && clear && affected {
                            Color::new(1., 1., 1., 0.15)
                        } else {
                            WHITE
                        },
                        DrawTextureParams {
                            dest_size: Some(vec2(22., 22.)),
                            ..Default::default()
                        },
                    );
                    if affected {
                        outline(px, py, 22., 22., 1.5, if after { SKYBLUE } else { GOLD });
                    }
                }
            }
        } else {
            // The board would not change, so preview the hero acting instead of tiles.
            draw_skill_actor(hero, preview, after, texture, awakened_texture);
        }
        label(
            tr_key(zh, "ui.menu.draw.effect_preview"),
            766.,
            454.,
            21.,
            GOLD,
        );
        label(skill.name(zh), 766., 486., 20., WHITE);
        label(
            if after {
                match skill {
                    content::Skill::Guard => tr_key(zh, "ui.menu.draw.hp_20_max"),
                    content::Skill::Landslide => tr_key(zh, "ui.menu.draw.next_swap"),
                    content::Skill::Strike => tr_key(zh, "ui.menu.draw.next_clear_100"),
                    _ => tr_key(zh, "ui.menu.draw.after_casting"),
                }
            } else {
                tr_key(zh, "ui.menu.draw.before_casting")
            },
            766.,
            521.,
            21.,
            SKYBLUE,
        );
        let hint = if !preview.affects_tiles {
            match preview.status {
                // A form change names the new form; other casts name their status.
                Some(status) if preview.dark != preview.base_dark => format!(
                    "{}：{}",
                    tr_key(zh, "ui.menu.draw.character_form"),
                    status.name(zh)
                ),
                Some(status) => status.name(zh).to_string(),
                None => tr_key(zh, "ui.menu.draw.cast_animation").to_string(),
            }
        } else {
            tr_key(zh, "ui.menu.draw.outlined_affected_tiles").to_string()
        };
        for (i, line) in wrap_preview_text(&hint, 17.5).iter().enumerate() {
            label(line, 766., 559. + i as f32 * 19., 16., LIGHTGRAY);
        }
    }
    let r = start_rect();
    panel(
        r,
        if r.contains(pointer) {
            Color::from_rgba(69, 91, 57, 255)
        } else {
            Color::from_rgba(51, 70, 45, 255)
        },
    );
    label_centered(
        tr_key(zh, "ui.menu.draw.next_choose_opponent"),
        r.x + r.w / 2.,
        682.,
        25.,
        WHITE,
    );
    if let Some(text) = resource_hint {
        let lines = text
            .lines()
            .map(|line| (line, fit_size(line, 17., 422.)))
            .collect::<Vec<_>>();
        crate::flush_labels();
        panel(
            Rect::new(548., 422., 452., 24. + lines.len() as f32 * 24.),
            Color::from_rgba(25, 33, 46, 255),
        );
        for (i, (line, size)) in lines.iter().enumerate() {
            label(line, 562., 450. + i as f32 * 24., *size, LIGHTGRAY);
        }
    }
}

fn draw_lore(hero: usize, zh: bool) {
    let Some(lore) = content::role_lore(hero) else {
        return;
    };
    let card = Color::from_rgba(35, 40, 52, 255);
    panel(Rect::new(548., 112., 452., 518.), card);
    label(
        tr_key(zh, "ui.menu.draw.lore_file"),
        566.,
        138.,
        16.,
        LIGHTGRAY,
    );
    label(
        lore.title(zh),
        566.,
        172.,
        fit_size(lore.title(zh), 27., 416.),
        GOLD,
    );
    label(
        tr_key(zh, "ui.menu.draw.lore_traits"),
        566.,
        206.,
        14.,
        SKYBLUE,
    );
    let mut y = 228.;
    for i in 0..8 {
        let Some(text) = lore.trait_text(i, zh) else {
            break;
        };
        for part in wrap_preview_text(&format!("· {text}"), 28.6) {
            if y > 300. {
                break;
            }
            label(&part, 566., y, 14., LIGHTGRAY);
            y += 17.;
        }
    }
    label(tr_key(zh, "ui.menu.draw.lore_lines"), 566., 312., 18., GOLD);
    let lines: Vec<_> = lore.lines(zh).collect();
    let split = lines.len().div_ceil(2);
    for (column, chunk) in lines.chunks(split.max(1)).enumerate() {
        let x = if column == 0 { 566. } else { 786. };
        let mut line_y = 338.;
        for text in chunk {
            let quoted = format!("“{text}”");
            for part in wrap_preview_text(&quoted, 15.4) {
                if line_y > 612. {
                    break;
                }
                label(&part, x, line_y, 13., LIGHTGRAY);
                line_y += 16.;
            }
            line_y += 3.;
        }
    }
}

fn draw_passive_entries(items: &[content::Passive], zh: bool, changes: bool) {
    let mut y = 391.;
    for &passive in items {
        let lines = wrap_label_text(passive.description(zh), if zh { 14. } else { 13. }, 220.);
        icons::draw_passive(passive, 304., y, 22.);
        label(
            passive.name(zh),
            334.,
            y + 17.,
            fit_size(passive.name(zh), 16., 124.),
            GOLD,
        );
        if changes {
            let kind = if passive.replaces().is_some() {
                tr_key(zh, "ui.menu.draw_passive_entries.replaces")
            } else {
                tr_key(zh, "ui.menu.draw_passive_entries.new")
            };
            label(kind, 477., y + 16., 12., SKYBLUE);
        }
        for (line, text) in lines.iter().enumerate() {
            label(
                text,
                304.,
                y + 43. + line as f32 * 20.,
                if zh { 14. } else { 13. },
                LIGHTGRAY,
            );
        }
        // Advance by actual content height, leaving a clear gap before the next icon/title.
        y += 43. + lines.len().saturating_sub(1) as f32 * 20. + 22.;
    }
}
pub fn page_rect(i: usize) -> Rect {
    Rect::new(if i == 0 { 302. } else { 436. }, 607., 76., 22.)
}

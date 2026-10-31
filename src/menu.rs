use super::*;
pub struct Preview {
    pub skill: usize,
    pub board: Board,
    pub changed: Board,
    pub cells: Vec<(usize, usize)>,
    pub since: f64,
}
impl Preview {
    pub fn new(hero: usize, skill: usize) -> Self {
        Self::with_time(hero, skill, get_time())
    }
    pub fn new_static(hero: usize, skill: usize) -> Self {
        Self::with_time(hero, skill, 0.)
    }
    fn with_time(hero: usize, skill: usize, since: f64) -> Self {
        let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        let mut changed = board;
        let effect = effect(content::skills(hero, true)[skill], &mut changed);
        let mut cells = effect.cells;
        for y in 0..N {
            for x in 0..N {
                if changed[y][x] != board[y][x] && !cells.contains(&(x, y)) {
                    cells.push((x, y));
                }
            }
        }
        if content::skills(hero, true)[skill] == content::Skill::Landslide {
            cells = vec![(2, 3), (3, 3), (2, 2), (3, 2), (2, 4), (3, 4)];
            for y in 2..=4 {
                swap(&mut changed, (2, y), (3, y));
            }
        }
        if content::skills(hero, true)[skill] == content::Skill::Strike {
            cells = vec![(2, 3), (3, 3), (4, 3)];
            for x in 2..=4 {
                changed[3][x] = 0;
            }
        }
        Self {
            skill,
            board,
            changed,
            cells,
            since,
        }
    }
}
/// Rows of the hero list that fit without covering the key hints below it.
pub const HERO_VISIBLE_ROWS: usize = 9;
pub const HERO_ROW_STEP: f32 = 53.;
/// On-screen row rectangle. Map it back to a roster entry through the scroll offset.
pub fn hero_rect(row: usize) -> Rect {
    Rect::new(24., 112. + row as f32 * HERO_ROW_STEP, 232., 48.)
}
pub fn hero_list_rect() -> Rect {
    Rect::new(24., 112., 232., HERO_ROW_STEP * HERO_VISIBLE_ROWS as f32)
}
pub fn skill_rect(i: usize) -> Rect {
    Rect::new(548., 126. + i as f32 * 94., 452., 88.)
}
pub fn start_rect() -> Rect {
    Rect::new(284., 651., 716., 48.)
}
fn panel(rect: Rect, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.,
        Color::from_rgba(66, 71, 83, 255),
    );
}
fn sprite(texture: &Texture2D, asset: &SpriteAsset, area: Rect) {
    let time = get_time() % asset.durations.iter().sum::<f64>();
    let mut end = 0.;
    let mut frame = 0;
    for (i, d) in asset.durations.iter().enumerate() {
        end += d;
        if time < end {
            frame = i;
            break;
        }
    }
    let new_role = asset.w == 384. && asset.h == 384.;
    let scale = if new_role {
        crate::role_scale(area) * 96. / asset.w
    } else {
        (area.w / asset.w).min(area.h / asset.h)
    };
    let size = vec2(asset.w * scale, asset.h * scale);
    let (size, rotation) =
        crate::presentation::idle_pose(size, get_time(), asset.indices.len() == 1);
    draw_texture_ex(
        texture,
        area.x + (area.w - size.x) / 2.,
        if new_role {
            area.y + area.h - 84. * (asset.w / 96.) * scale
        } else {
            area.y + area.h - size.y
        },
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation,
            pivot: Some(vec2(area.x + area.w / 2., area.y + area.h)),
            source: Some(Rect::new(
                (asset.indices[frame] % (texture.width() / asset.w) as usize) as f32 * asset.w,
                (asset.indices[frame] / (texture.width() / asset.w) as usize) as f32 * asset.h,
                asset.w,
                asset.h,
            )),
            ..Default::default()
        },
    );
}
pub fn draw(
    hero: usize,
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
    let scroll = ui::clamp_scroll(scroll, content::ROSTER.len(), HERO_VISIBLE_ROWS);
    let visible_rows = HERO_VISIBLE_ROWS.min(content::ROSTER.len().saturating_sub(scroll));
    for row in 0..visible_rows {
        let i = content::ROSTER[scroll + row];
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
                source: Some(Rect::new(i as f32 * 40., 0., 40., 48.)),
                ..Default::default()
            },
        );
        label(
            if zh { h.zh } else { h.en },
            r.x + 67.,
            r.y + 23.,
            20.,
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
    ui::draw_scrollbar(
        hero_list_rect(),
        scroll,
        content::ROSTER.len(),
        HERO_VISIBLE_ROWS,
    );
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
    label(if zh { h.zh } else { h.en }, 302., 151., 34., GOLD);
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
    sprite(
        if page == 2 { awakened_texture } else { texture },
        if page == 2 {
            &sprites::AWAKENED[hero]
        } else {
            &sprites::SPRITES[hero]
        },
        Rect::new(291., 198., 230., portrait_height),
    );
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
    } else {
        label(tr_key(zh, "ui.menu.draw.awakening"), 304., 373., 22., GOLD);
        let changes = content::passives(hero, true)
            .iter()
            .copied()
            .filter(|p| !content::passives(hero, false).contains(p))
            .collect::<Vec<_>>();
        draw_passive_entries(&changes, zh, true);
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
            if (i == 0 && page == 0) || (i == 1 && page == 2) {
                GRAY
            } else {
                SKYBLUE
            },
        );
    }
    label(&format!("{} / 3", page + 1), 388., 623., 14., WHITE);
    let visible_skills = content::skills(hero, page == 2);
    for (i, skill) in visible_skills.iter().enumerate() {
        let r = skill_rect(i);
        panel(
            r,
            if i == preview.skill {
                Color::from_rgba(44, 48, 59, 255)
            } else {
                dim
            },
        );
        if i == preview.skill {
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., GOLD);
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
        for (line, text) in content::skill_description(*skill, page == 2, zh)
            .lines()
            .enumerate()
        {
            label(
                text,
                r.x + 14.,
                r.y + 49. + line as f32 * 17.,
                fit_size(text, 16., r.w - 28.),
                LIGHTGRAY,
            );
        }
    }
    if h.skills.len() == 1 {
        label(
            tr_key(zh, "ui.menu.draw.one_distinctive_skill"),
            564.,
            306.,
            23.,
            GOLD,
        );
        label(
            tr_key(zh, "ui.menu.draw.each_hero_plays_differently"),
            564.,
            341.,
            19.,
            LIGHTGRAY,
        );
        label(
            tr_key(zh, "ui.menu.draw.preview_the_effect_below"),
            564.,
            369.,
            19.,
            LIGHTGRAY,
        );
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
                draw_rectangle_lines(px, py, 22., 22., 1.5, if after { SKYBLUE } else { GOLD });
            }
        }
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
    label(
        tr_key(zh, "ui.menu.draw.outlined_affected_tiles"),
        766.,
        559.,
        16.,
        LIGHTGRAY,
    );
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

fn draw_passive_entries(items: &[content::Passive], zh: bool, changes: bool) {
    let mut y = 391.;
    for &passive in items {
        let lines = wrap_preview_text(passive.description(zh), if zh { 14.5 } else { 16. });
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

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
        let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        let mut changed = board;
        let effect = effect(HEROES[hero].skills[skill], &mut changed);
        let mut cells = effect.cells;
        for y in 0..N {
            for x in 0..N {
                if changed[y][x] != board[y][x] && !cells.contains(&(x, y)) {
                    cells.push((x, y));
                }
            }
        }
        if HEROES[hero].skills[skill] == content::Skill::Landslide {
            cells = vec![(2, 3), (3, 3), (2, 2), (3, 2), (2, 4), (3, 4)];
            for y in 2..=4 {
                swap(&mut changed, (2, y), (3, y));
            }
        }
        if HEROES[hero].skills[skill] == content::Skill::Strike {
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
            since: get_time(),
        }
    }
}
pub fn hero_rect(i: usize) -> Rect {
    Rect::new(24., 112. + i as f32 * 73., 232., 64.)
}
pub fn skill_rect(i: usize) -> Rect {
    Rect::new(548., 126. + i as f32 * 141., 452., 129.)
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
    let scale = (area.w / asset.w).min(area.h / asset.h);
    let size = vec2(asset.w * scale, asset.h * scale);
    draw_texture_ex(
        texture,
        area.x + (area.w - size.x) / 2.,
        area.y + area.h - size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(Rect::new(
                asset.indices[frame] as f32 * asset.w,
                0.,
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
) {
    let dim = Color::from_rgba(29, 33, 43, 255);
    let card = Color::from_rgba(35, 40, 52, 255);
    label(
        tr(zh, "STEP 1 · CHOOSE YOUR HERO", "第一步 · 选择角色"),
        24.,
        58.,
        34.,
        GOLD,
    );
    label(
        tr(
            zh,
            "Preview your hero and skills before battle",
            "查看角色与技能，准备回合制对战",
        ),
        26.,
        88.,
        19.,
        LIGHTGRAY,
    );
    for (i, h) in HEROES.iter().enumerate() {
        let r = hero_rect(i);
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
                dest_size: Some(vec2(40., 48.)),
                source: Some(Rect::new(i as f32 * 40., 0., 40., 48.)),
                ..Default::default()
            },
        );
        label(
            if zh { h.zh } else { h.en },
            r.x + 67.,
            r.y + 27.,
            23.,
            if i == hero { GOLD } else { WHITE },
        );
        label(
            &format!(
                "{} {}   {} {}",
                tr(zh, "HP", "生命"),
                h.hp,
                tr(zh, "MP", "魔力"),
                h.mp
            ),
            r.x + 67.,
            r.y + 51.,
            16.,
            LIGHTGRAY,
        );
    }
    label(
        tr(zh, "↑ / ↓: Select hero", "↑ / ↓：选择角色"),
        26.,
        662.,
        18.,
        LIGHTGRAY,
    );
    label(
        tr(zh, "Click a skill to preview", "点击技能可预览效果"),
        26.,
        688.,
        18.,
        LIGHTGRAY,
    );
    let h = &HEROES[hero];
    panel(Rect::new(284., 112., 244., 518.), card);
    label(if zh { h.zh } else { h.en }, 302., 151., 34., GOLD);
    label(
        tr(zh, "ANIMATED PREVIEW", "角色动画预览"),
        302.,
        181.,
        16.,
        LIGHTGRAY,
    );
    draw_ellipse(406., 403., 82., 12., 0., Color::from_rgba(20, 23, 31, 255));
    sprite(
        texture,
        &sprites::SPRITES[hero],
        Rect::new(301., 198., 210., 202.),
    );
    label(tr(zh, "MAX HEALTH", "生命上限"), 304., 432., 17., LIGHTGRAY);
    label(&h.hp.to_string(), 304., 459., 28., GREEN);
    draw_rectangle(304., 469., 204., 5., Color::from_rgba(48, 55, 61, 255));
    draw_rectangle(304., 469., 204. * h.hp as f32 / 800., 5., GREEN);
    label(tr(zh, "MAX MANA", "魔力上限"), 304., 495., 17., LIGHTGRAY);
    label(&h.mp.to_string(), 304., 521., 28., SKYBLUE);
    draw_rectangle(304., 531., 204., 5., Color::from_rgba(48, 55, 61, 255));
    draw_rectangle(304., 531., 204. * h.mp as f32 / 800., 5., SKYBLUE);
    label(
        tr(zh, "PASSIVE SKILL", "被动技能"),
        304.,
        557.,
        17.,
        LIGHTGRAY,
    );
    for (i, line) in content::passive(hero, zh).lines().enumerate() {
        label(
            line,
            304.,
            577. + i as f32 * 18.,
            fit_size(line, 13., 214.),
            GOLD,
        );
    }
    for (i, skill) in h.skills.iter().enumerate() {
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
        label(
            &format!("{}  {}", i + 1, skill.name(zh)),
            r.x + 50.,
            r.y + 28.,
            25.,
            WHITE,
        );
        label(
            &format!("{} {}", skill.cost(), tr(zh, "MP", "魔力")),
            r.x + 332.,
            r.y + 27.,
            18.,
            SKYBLUE,
        );
        for (line, text) in skill.description(zh).lines().enumerate() {
            label(
                text,
                r.x + 14.,
                r.y + 55. + line as f32 * 23.,
                19.,
                LIGHTGRAY,
            );
        }
    }
    if h.skills.len() == 1 {
        label(
            tr(zh, "ONE DISTINCTIVE SKILL", "专属技能"),
            564.,
            306.,
            23.,
            GOLD,
        );
        label(
            tr(
                zh,
                "Each hero plays differently.",
                "每个角色都有不同的战斗方式。",
            ),
            564.,
            341.,
            19.,
            LIGHTGRAY,
        );
        label(
            tr(
                zh,
                "Preview the effect below.",
                "在下方预览技能影响的方块。",
            ),
            564.,
            369.,
            19.,
            LIGHTGRAY,
        );
    }
    label(
        tr(zh, "SKILLS · click to preview", "技能 · 点击查看效果"),
        550.,
        112.,
        18.,
        LIGHTGRAY,
    );
    let skill = h.skills[preview.skill];
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
        tr(zh, "EFFECT PREVIEW", "技能效果预览"),
        766.,
        454.,
        21.,
        GOLD,
    );
    label(skill.name(zh), 766., 486., 20., WHITE);
    label(
        if after {
            match skill {
                content::Skill::Guard => tr(zh, "HP +20% max", "回复生命上限20%"),
                content::Skill::Landslide => tr(zh, "Next swap", "下次交换"),
                content::Skill::Strike => tr(zh, "Next clear: +100%", "下次消除：收益+100%"),
                _ => tr(zh, "After casting", "释放后"),
            }
        } else {
            tr(zh, "Before casting", "释放前")
        },
        766.,
        521.,
        21.,
        SKYBLUE,
    );
    label(
        tr(zh, "Outlined: affected tiles", "描边：受影响的方块"),
        766.,
        559.,
        16.,
        LIGHTGRAY,
    );
    label(
        tr(zh, "Demo · no mana spent", "演示不消耗魔力"),
        766.,
        594.,
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
        tr(zh, "NEXT · CHOOSE OPPONENT", "下一步：选择对手"),
        r.x + r.w / 2.,
        682.,
        25.,
        WHITE,
    );
}

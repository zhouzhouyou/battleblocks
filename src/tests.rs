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
fn every_character_webp_decodes_with_its_original_atlas_dimensions() {
    for asset in sprites::SPRITES.iter().chain(sprites::AWAKENED) {
        let decoded = image::load_from_memory(asset.bytes).expect("runtime image decoder");
        let frames = asset.indices.iter().max().unwrap() + 1;
        assert_eq!(decoded.width() % asset.w as u32, 0);
        assert_eq!(decoded.height() % asset.h as u32, 0);
        assert!(
            decoded.width() / asset.w as u32 * (decoded.height() / asset.h as u32) >= frames as u32
        );
        assert!(decoded.to_rgba8().pixels().any(|pixel| pixel[3] > 0));
    }
}
#[test]
fn new_role_atlases_have_all_animation_frames_and_fixed_binary_palettes() {
    for id in [content::KNIGHT, content::DWARF, content::PRIEST] {
        let image = image::load_from_memory(sprites::SPRITES[id].bytes)
            .unwrap()
            .to_rgba8();
        assert_eq!(image.dimensions(), (384 * 4, 384 * 4));
        // Visible idle rows must share the same display height across roles;
        // transparent canvas padding must not shrink the priest or dwarf.
        for row in 0..2 {
            let max_height = (0..4)
                .map(|column| {
                    let occupied: Vec<_> = (0..384)
                        .filter(|&y| {
                            (0..384)
                                .any(|x| image.get_pixel(column * 384 + x, row * 384 + y)[3] > 0)
                        })
                        .collect();
                    occupied.last().unwrap() - occupied.first().unwrap() + 1
                })
                .max()
                .unwrap();
            assert_eq!(max_height, 320, "role {id} idle row {row}");
        }
        for frame in 0..16 {
            let mut colors = std::collections::HashSet::new();
            for y in frame / 4 * 384..(frame / 4 + 1) * 384 {
                for x in frame % 4 * 384..(frame % 4 + 1) * 384 {
                    let pixel = image.get_pixel(x, y);
                    assert!(pixel[3] == 0 || pixel[3] == 255);
                    if pixel[3] > 0 {
                        colors.insert([pixel[0], pixel[1], pixel[2]]);
                    }
                }
            }
            assert!(!colors.is_empty() && colors.len() <= 64);
        }
    }
    let book = image::load_from_memory(sprites::BOOK_BYTES).unwrap();
    assert_eq!((book.width(), book.height()), (1536, 384));
}
#[test]
fn global_skills_cast_directly_even_when_paid_with_health() {
    use content::Skill;
    for awake in [false, true] {
        let rules = content::Rules {
            awake,
            ..Default::default()
        };
        for mana in [0, 80, 100] {
            assert!(content::payment_in(6, 600, mana, Skill::Corrupt, rules).is_some());
            assert!(!skill_requires_board_click(Skill::Corrupt));
            assert!(skill_requires_board_click(Skill::Drain));
        }
    }
    assert!(!skill_requires_board_click(Skill::Guard));
    assert!(!skill_requires_board_click(Skill::Imp));
    assert!(skill_requires_board_click(Skill::Charge));
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
#[test]
fn selection_scroll_keeps_rows_inside_their_viewport() {
    let roster = content::ROSTER.len();
    let hero_area = menu::hero_list_rect();
    let last_hero = menu::hero_rect(menu::HERO_VISIBLE_ROWS - 1);
    assert!(last_hero.y + last_hero.h <= hero_area.y + hero_area.h);
    // The hero hints start at y=662; the last visible row must stay above them.
    assert!(hero_area.y + hero_area.h < 650.);

    let opponent_area = ui::opponent_list_rect();
    let last_opponent = ui::opponent_rect(ui::OPPONENT_VISIBLE_ROWS - 1, 0);
    assert!(last_opponent.y + last_opponent.h <= opponent_area.y + opponent_area.h);
    // The back/start buttons begin at y=651; the grid must stop before them.
    assert!(opponent_area.y + opponent_area.h < ui::back_rect().y);

    assert_eq!(
        ui::scroll_to_show(0, roster - 1, roster, menu::HERO_VISIBLE_ROWS),
        ui::max_scroll(roster, menu::HERO_VISIBLE_ROWS)
    );
    assert_eq!(ui::scroll_to_show(0, 0, roster, menu::HERO_VISIBLE_ROWS), 0);
    let opponent_rows = ui::opponent_row_count();
    assert_eq!(
        ui::scroll_to_show(
            0,
            opponent_rows - 1,
            opponent_rows,
            ui::OPPONENT_VISIBLE_ROWS
        ),
        ui::max_scroll(opponent_rows, ui::OPPONENT_VISIBLE_ROWS)
    );
    assert_eq!(
        ui::scroll_by(0, 1, opponent_rows, ui::OPPONENT_VISIBLE_ROWS),
        1
    );
    assert_eq!(
        ui::scroll_by(1, 1, opponent_rows, ui::OPPONENT_VISIBLE_ROWS),
        1
    );
    assert_eq!(
        ui::scroll_by(0, -1, opponent_rows, ui::OPPONENT_VISIBLE_ROWS),
        0
    );
    assert_eq!(ui::clamp_scroll(9, 3, 5), 0);
}

#[test]
fn character_file_is_the_fourth_and_final_selection_page() {
    assert_eq!(menu::CHARACTER_PAGE_COUNT, 4);
    assert_eq!(menu::page_rect(0).y, menu::page_rect(1).y);
    assert!(menu::page_rect(1).x > menu::page_rect(0).x);
}

#[test]
fn english_wrapping_keeps_words_whole_while_cjk_still_wraps_by_character() {
    let source = "The quick brown fox jumps over the lazy dog";
    let rows = wrap_preview_text(source, 10.);
    assert!(rows.len() > 1);
    assert_eq!(
        rows.join(" ").split_whitespace().collect::<Vec<_>>(),
        source.split_whitespace().collect::<Vec<_>>()
    );
    assert!(rows.iter().all(|row| !row.contains("  ")));
    let cjk = "矮人把每一笔仇恨写进书里，把每一个答案抡进锤子里。";
    let rows = wrap_preview_text(cjk, 8.);
    assert!(rows.len() > 1);
    assert_eq!(rows.concat(), cjk);
}

#[test]
fn headless_autoplay_finishes_and_replays_the_same_seed() {
    let mut session = app::Session::new_headless();
    session.prepare_headless_batch();
    let first = session.headless_batch_match(content::DWARF, content::KNIGHT, 1234, 1, 10_000);
    let second = session.headless_batch_match_at_dt(
        content::DWARF,
        content::KNIGHT,
        1234,
        1,
        10_000,
        1. / 60.,
    );
    assert_eq!(first.winner, second.winner);
    assert_eq!(first.rounds, second.rounds);
    assert_eq!((first.hp, first.enemy_hp), (second.hp, second.enemy_hp));
    assert!(first.winner.is_some(), "{first:?}");
    assert!(first.steps < 10_000, "{first:?}");
    assert!(first.rounds >= 1, "{first:?}");
}
#[test]
fn help_tabs_share_the_panel_width_without_leaving_it() {
    let panel = ui::PANEL;
    let inset = ui::help_tab_rect(0).x - panel.x;
    let inner_right = panel.x + panel.w - inset;
    for i in 0..ui::HELP_TAB_COUNT {
        let r = ui::help_tab_rect(i);
        assert!(r.x >= panel.x, "tab {i} starts before the panel");
        assert!(
            r.x + r.w <= inner_right,
            "tab {i} runs past the panel content"
        );
        if i > 0 {
            let previous = ui::help_tab_rect(i - 1);
            assert!(
                previous.x + previous.w < r.x,
                "tab {i} overlaps the previous tab"
            );
        }
    }
}
#[test]
fn skill_preview_shows_the_hero_when_no_tile_changes() {
    // The nun's frenzy never touches the board, so the preview shows her new form.
    let frenzy = menu::Preview::new_static(content::NUN, 0, false);
    assert!(!frenzy.affects_tiles);
    assert!(frenzy.dark);
    assert_eq!(frenzy.status, Some(content::StatusId::Frenzy));
    // Single-target heals and preparations also leave the board alone.
    let heal = menu::Preview::new_static(content::NUN, 1, false);
    assert!(!heal.affects_tiles);
    assert!(!heal.dark);
    // Board skills keep the tile grid.
    let charge = menu::Preview::new_static(content::KNIGHT, 0, false);
    assert!(charge.affects_tiles);
    let miracle = menu::Preview::new_static(content::KNIGHT, 1, false);
    assert!(!miracle.affects_tiles);
    // The jump skill lifts the hero in the character preview; other casts do not.
    let jump = menu::Preview::new_static(content::DWARF, 0, false);
    assert!(jump.hop && !jump.affects_tiles);
    assert!(!miracle.hop && !charge.hop);
}

#[test]
fn awakening_page_previews_the_awakened_active() {
    // The nun's awakening swaps frenzy for returning to reason.
    assert_eq!(
        content::skills(content::NUN, false)[0],
        content::Skill::Frenzy
    );
    assert_eq!(
        content::skills(content::NUN, true)[0],
        content::Skill::Rational
    );
    let base = menu::Preview::new_static(content::NUN, 0, false);
    let awakened = menu::Preview::new_static(content::NUN, 0, true);
    assert_eq!(base.status, Some(content::StatusId::Frenzy));
    assert_eq!(awakened.status, Some(content::StatusId::Rational));
    assert!(base.dark && !awakened.dark);
}

use super::*;
fn board() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
#[test]
fn autoplay_roster_keeps_new_knight_and_unlocks_third_slots() {
    assert!(!DEFAULT_AUTOPLAY_ROSTER.contains(&0));
    assert!(
        DEFAULT_AUTOPLAY_ROSTER.contains(&KNIGHT)
            && DEFAULT_AUTOPLAY_ROSTER.contains(&DWARF)
            && DEFAULT_AUTOPLAY_ROSTER.contains(&PRIEST)
    );
    assert_eq!(skills(KNIGHT, false), &[Skill::Rush, Skill::Miracle]);
    assert_eq!(
        skills(KNIGHT, true),
        &[Skill::Rush, Skill::Miracle, Skill::Hammer]
    );
    assert_eq!(
        skills(PRIEST, true),
        &[Skill::Entomb, Skill::Resurrect, Skill::Copy]
    );
}
#[test]
fn strength_and_horizontal_awakened_bonus_are_exact_and_do_not_leak_to_cascades() {
    let mut b = board();
    b[0][0] = 2;
    b[0][1] = 4;
    b[0][2] = 6;
    let cells = [(0, 0), (1, 0), (2, 0)]; // base 12 + 7 + 7 = 26
    assert_eq!(
        clear_rewards_in(KNIGHT, 600, 0, &b, &cells, 1, 0, Rules::default()).1,
        32
    );
    assert_eq!(
        clear_rewards_in(DWARF, 600, 0, &b, &cells, 1, 0, Rules::default()).1,
        32
    );
    let awake = Rules {
        awake: true,
        horizontal: true,
        ..Rules::default()
    };
    assert_eq!(
        clear_rewards_in(KNIGHT, 300, 0, &b, &cells, 1, 0, awake).1,
        37
    );
    assert_eq!(
        clear_rewards_in(KNIGHT, 300, 0, &b, &cells, 2, 0, awake).1,
        35
    ); // 110% cascade, no extra horizontal bonus
}
#[test]
fn dwarf_reach_tracks_current_health_and_jump_and_preserves_rejected_targets() {
    let b = board();
    let original = b;
    let cap = health_cap(DWARF, true);
    assert_eq!(
        reachable_rows(
            DWARF,
            cap,
            Rules {
                awake: true,
                ..Default::default()
            }
        ),
        4
    );
    for (percent, rows) in [(80, 5), (60, 6), (40, 7)] {
        let rules = Rules {
            awake: true,
            ..Default::default()
        };
        assert_eq!(reachable_rows(DWARF, cap * percent / 100, rules), rows);
        assert_eq!(
            reachable_rows(
                DWARF,
                cap * percent / 100,
                Rules {
                    statuses: StatusSet::single(
                        StatusId::ExtraReach,
                        Skill::Jump,
                        StatusDuration::OwnerTurnEnd(1)
                    ),
                    ..rules
                }
            ),
            rows + 1
        );
    }
    assert!(!legal_target(
        DWARF,
        700,
        Skill::Forge,
        &b,
        (1, 3),
        Rules::default()
    ));
    assert!(legal_target(
        DWARF,
        700,
        Skill::Forge,
        &b,
        (1, 3),
        Rules {
            statuses: StatusSet::single(
                StatusId::ExtraReach,
                Skill::Jump,
                StatusDuration::OwnerTurnEnd(1)
            ),
            ..Default::default()
        }
    ));
    assert!(!legal_swap_region(
        DWARF,
        700,
        (1, 3),
        (1, 4),
        Rules::default()
    ));
    assert!(legal_swap_region(
        DWARF,
        700,
        (1, 3),
        (1, 4),
        Rules {
            statuses: StatusSet::single(
                StatusId::ExtraReach,
                Skill::Jump,
                StatusDuration::OwnerTurnEnd(1)
            ),
            ..Default::default()
        }
    ));
    let rejected = preview_rewards_in(
        Skill::Forge,
        &b,
        (1, 2),
        DWARF,
        700,
        100,
        500,
        0,
        Rules::default(),
    );
    assert_eq!(
        (rejected.damage, rejected.mana_after, rejected.health_after),
        (0, 100, 700)
    );
    assert_eq!(b, original);
}
#[test]
fn dwarf_swap_immunity_tracks_each_cleared_tiles_row_and_preserves_other_rewards() {
    let b = [[0; N]; N];
    let top = [(0, 0), (1, 0), (2, 0)];
    let bottom = [(0, 5), (1, 5), (2, 5)];
    let rules = Rules {
        opponent: Some((DWARF, false)),
        swap_clear: true,
        ..Default::default()
    };
    let settle = |hero, cells: &[(usize, usize)], rules| {
        settle_batch_in(
            hero,
            500,
            0,
            500,
            clear_batch_in(hero, 500, 0, &b, cells, 1, 0, rules),
            rules,
        )
    };
    assert_eq!(
        clear_rewards_in(KNIGHT, 500, 0, &b, &top, 1, 0, rules),
        (6, 18, 12)
    );
    let top_hit = settle(KNIGHT, &top, rules);
    assert_eq!(top_hit.damage, 0);
    assert_eq!((top_hit.actor_hp, top_hit.actor_mp), (506, 12));
    assert!(top_hit
        .triggers
        .iter()
        .any(|t| t.message == Some("Dwarf.evade")));
    let imp = settle(4, &top, rules);
    assert_eq!((imp.damage, imp.actor_mp), (0, 18));
    assert_eq!(settle(KNIGHT, &bottom, rules).damage, 18);
    assert_eq!(settle(KNIGHT, &[(0, 0), (1, 5), (2, 6)], rules).damage, 12);
    assert_eq!(
        settle(
            KNIGHT,
            &top,
            Rules {
                swap_clear: false,
                ..rules
            }
        )
        .damage,
        18
    );
    assert_eq!(
        settle(
            KNIGHT,
            &top,
            Rules {
                opponent: Some((PRIEST, false)),
                ..rules
            }
        )
        .damage,
        18
    );
}

#[test]
fn copied_skill_uses_native_cost_and_end_turn_rule_but_slot_cooldown_is_one() {
    let b = board();
    let rules = Rules {
        copied_skill: Some(Skill::Resurrect),
        pockets: [Some(2), None],
        ..Default::default()
    };
    assert_eq!(effective_skill(Skill::Copy, rules), Skill::Resurrect);
    assert_eq!(
        payment_in(PRIEST, 400, 25, Skill::Copy, rules),
        Some(Payment { mana: 25, blood: 0 })
    );
    assert!(!effective_skill(Skill::Copy, rules).ends_turn());
    assert!(legal_target(PRIEST, 400, Skill::Copy, &b, (0, 0), rules));
    let mut cd = Cooldowns::default();
    cd.cast(0, Skill::Copy);
    assert_eq!(cd.remaining(0, Skill::Copy), 1);
    assert_eq!(cd.remaining(0, Skill::Resurrect), 0);
    let costly = Rules {
        copied_skill: Some(Skill::Meteor),
        ..rules
    };
    assert!(payment_in(PRIEST, 400, 159, Skill::Copy, costly).is_none());
    assert_eq!(
        payment_in(PRIEST, 400, 160, Skill::Copy, costly)
            .unwrap()
            .mana,
        160
    );
    assert!(effective_skill(Skill::Copy, costly).ends_turn());
}
#[test]
fn priest_previews_are_pure_and_obey_capacity_costs_and_resource_caps() {
    let mut b = board();
    b[2][0] = 1;
    b[2][1] = 1;
    b[2][2] = 3;
    let original = b;
    let rng = crate::rng::state();
    let rules = Rules {
        pockets: [Some(1), None],
        ..Default::default()
    };
    let p = preview_rewards_in(
        Skill::Resurrect,
        &b,
        (2, 2),
        PRIEST,
        649,
        160,
        700,
        0,
        rules,
    );
    assert_eq!(p.health_after, 650);
    assert_eq!(p.mana_after, 135);
    assert!(p.heal <= 1);
    assert!(!legal_target(
        PRIEST,
        600,
        Skill::Entomb,
        &b,
        (0, 0),
        Rules {
            pockets: [Some(1), Some(2)],
            ..rules
        }
    ));
    assert!(!legal_target(
        PRIEST,
        600,
        Skill::Resurrect,
        &b,
        (0, 0),
        Rules::default()
    ));
    assert!(payment_in(PRIEST, 600, 54, Skill::Entomb, rules).is_none());
    let e = effect_at_in(
        Skill::Entomb,
        &mut b.clone(),
        (0, 0),
        PRIEST,
        600,
        35,
        rules,
    );
    assert_eq!((e.hp, e.damage, e.mp, e.reward), (0, 0, 0, false));
    assert_eq!(e.cells, [(0, 0)]);
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), rng);
}
#[test]
fn hammer_is_normal_elimination_plus_eighty_and_preview_never_draws_randomness() {
    let b = [[2; N]; N];
    let rng = crate::rng::state();
    let e = effect_at_in(
        Skill::Hammer,
        &mut b.clone(),
        (6, 6),
        KNIGHT,
        300,
        150,
        Rules {
            awake: true,
            ..Default::default()
        },
    );
    assert_eq!(e.damage, 80);
    assert!(e.reward);
    assert_eq!(e.cells.len(), 4);
    let p = preview_rewards_in(
        Skill::Hammer,
        &b,
        (6, 6),
        KNIGHT,
        300,
        150,
        700,
        0,
        Rules {
            awake: true,
            ..Default::default()
        },
    );
    assert_eq!(p.damage, 80 + 4 * (12 + 2));
    assert_eq!(p.mana_after, 70);
    let _ = preview_range_in(
        Skill::Hammer,
        &b,
        KNIGHT,
        300,
        150,
        700,
        0,
        Rules {
            awake: true,
            ..Default::default()
        },
    );
    assert_eq!(crate::rng::state(), rng);
}

use super::*;
fn board() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
#[test]
fn public_roster_replaces_old_knight_and_unlocks_third_slots() {
    assert!(!ROSTER.contains(&0));
    assert!(ROSTER.contains(&KNIGHT) && ROSTER.contains(&DWARF) && ROSTER.contains(&PRIEST));
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
        35
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
        38
    );
    assert_eq!(
        clear_rewards_in(KNIGHT, 300, 0, &b, &cells, 2, 0, awake).1,
        38
    ); // 110% cascade, no extra +10%
    assert_eq!(
        clear_rewards_in(
            KNIGHT,
            300,
            0,
            &b,
            &cells,
            1,
            0,
            Rules {
                miracle: true,
                ..awake
            }
        )
        .1,
        31
    ); // 90%, additive
    assert_eq!(
        clear_rewards_in(
            KNIGHT,
            300,
            0,
            &b,
            &cells,
            2,
            0,
            Rules {
                miracle: true,
                ..awake
            }
        )
        .1,
        31
    ); // cascade 110 - 20
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
                    jumped: true,
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
            jumped: true,
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
            jumped: true,
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
fn upper_half_swap_immunity_depends_on_target_and_not_other_rewards() {
    let b = [[0; N]; N];
    let cells = [(0, 0), (1, 0), (2, 0)];
    let blocked = Rules {
        opponent: Some((DWARF, false)),
        upper_swap: true,
        ..Default::default()
    };
    let (h, d, m) = clear_rewards_in(KNIGHT, 500, 0, &b, &cells, 1, 0, blocked);
    assert_eq!((h, d, m), (6, 0, 12));
    assert_eq!(
        clear_rewards_in(KNIGHT, 500, 0, &b, &cells, 2, 0, blocked).1,
        0
    );
    assert_eq!(
        clear_rewards_in(
            KNIGHT,
            500,
            0,
            &b,
            &cells,
            1,
            0,
            Rules {
                upper_swap: false,
                ..blocked
            }
        )
        .1,
        21
    );
    assert_eq!(
        clear_rewards_in(
            KNIGHT,
            500,
            0,
            &b,
            &cells,
            1,
            0,
            Rules {
                opponent: Some((PRIEST, false)),
                ..blocked
            }
        )
        .1,
        21
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
        payment_in(PRIEST, 400, 0, Skill::Copy, rules),
        Some(Payment { mana: 0, blood: 0 })
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
        180,
        700,
        0,
        rules,
    );
    assert_eq!(p.health_after, 650);
    assert_eq!(p.mana_after, 180);
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
    assert!(payment_in(PRIEST, 600, 34, Skill::Entomb, rules).is_none());
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
    assert_eq!(p.damage, 80 + 4 * (12 + 3));
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

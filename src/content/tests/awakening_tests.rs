use super::*;
fn rules(awake: bool, enemy: usize, other_awake: bool) -> Rules {
    Rules {
        awake,
        opponent: Some((enemy, other_awake)),
        ..Rules::default()
    }
}
fn board() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
#[test]
fn every_role_triggers_exactly_at_its_line_and_survives_a_lethal_hit() {
    for hero in 0..7 {
        let threshold = awakening_line(hero);
        let mut hp = threshold + 2;
        let mut awake = false;
        assert!(!take_damage(hero, &mut hp, &mut awake, 1).awakened);
        assert!(take_damage(hero, &mut hp, &mut awake, 1).awakened);
        assert_eq!(hp, health_cap(hero, true));
        assert!(awake);
        let hit = take_damage(hero, &mut hp, &mut awake, 10_000);
        assert!(!hit.awakened);
        assert_eq!(hp, 0);
        hp = HEROES[hero].hp;
        awake = false;
        assert!(take_damage(hero, &mut hp, &mut awake, 10_000).awakened);
        assert_eq!(hp, health_cap(hero, true));
        assert_eq!(take_damage(hero, &mut hp, &mut awake, 80).damage, 80);
        assert_eq!(hp, health_cap(hero, true) - 80);
    }
    let mut hp = 0;
    let mut awake = false;
    assert!(!take_damage(0, &mut hp, &mut awake, 10).awakened);
    hp = 300;
    assert!(!take_damage(0, &mut hp, &mut awake, 0).awakened);
}
#[test]
fn imp_strength_moves_from_blast_to_rewarded_clears_in_both_phases() {
    let b = board();
    let original = b;
    let seed = crate::rng::state();
    let cells = vec![(0, 0)]; // balanced tile: base 2 HP / 4 damage / 4 MP
    let count = b.iter().flatten().filter(|&&t| t == 0).count() as i32;
    for (awake, blast, bonus) in [(false, 12, 4), (true, 14, 6)] {
        let r = rules(awake, 0, true);
        assert_eq!(
            clear_rewards_in(4, 300, 0, &b, &cells, 1, 0, r),
            (2, 4 + bonus, 6)
        );
        let e = effect_at_in(Skill::Imp, &mut b.clone(), (0, 0), 4, 300, 60, r);
        assert_eq!((e.damage, e.mp), (count * blast, count * 4));
        assert!(!e.reward); // direct blast does not also receive clear/passive rewards
        assert!(payment_in(4, 300, 59, Skill::Imp, r).is_none());
        let p = preview_rewards_in(Skill::Imp, &b, (0, 0), 4, 300, 60, 385, 0, r);
        assert_eq!(p.damage, count * blast);
        assert_eq!(p.mana_after, count * 4);
        let p = preview_rewards_in(Skill::Imp, &[[0; N]; N], (0, 0), 4, 300, 120, 385, 0, r);
        assert_eq!(p.mana_after, 120);
        assert!(skill_description(Skill::Imp, awake, true).contains(&format!("{blast}伤害")));
    }
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn tuned_enemy_caps_match_real_hits_for_every_target_and_phase() {
    let b = board();
    let original = b;
    let seed = crate::rng::state();
    for enemy in [2, 4, 5, 6] {
        for awake in [false, true] {
            let hp = if awake {
                health_cap(enemy, true)
            } else {
                awakening_line(enemy) + 1
            };
            for target in [(0, 0), (3, 0), (6, 0)] {
                let mut copy = b;
                let e = effect_at_in(
                    Skill::Charge,
                    &mut copy,
                    target,
                    0,
                    700,
                    100,
                    Rules::default(),
                );
                let (mut actual_hp, mut actual_awake) = (hp, awake);
                take_damage(enemy, &mut actual_hp, &mut actual_awake, e.damage);
                let preview = preview_rewards_in(
                    Skill::Charge,
                    &b,
                    target,
                    0,
                    700,
                    100,
                    hp,
                    0,
                    rules(false, enemy, awake),
                );
                assert_eq!(preview.enemy_health_after, actual_hp);
                assert_eq!(preview.awakens_enemy, !awake && actual_awake);
                assert_eq!(preview.mana_after, 0);
            }
        }
    }
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn phase_health_changes_reach_actual_caps_without_changing_costs_or_preview_state() {
    let mut b = board();
    b[0][..3].copy_from_slice(&[1, 0, 1]);
    b[1][1] = 1;
    let original = b;
    let seed = crate::rng::state();
    for hero in [2, 4, 5, 6] {
        let cap = health_cap(hero, true);
        let r = rules(true, 0, true);
        let p = preview_move_in(&b, hero, cap - 1, 0, 385, 0, false, (1, 0), (1, 1), r).unwrap();
        let mut actual = b;
        assert!(crate::board::hero_swap(&mut actual, (1, 0), (1, 1), hero));
        let (heal, damage, mana) =
            clear_rewards_in(hero, cap - 1, 0, &actual, &crate::matches(&actual), 1, 0, r);
        assert_eq!(p.health_after, (cap - 1 + heal).min(cap));
        assert_eq!(
            (p.heal, p.damage, p.mana_after),
            (1, damage, mana.min(mana_cap(hero, true)))
        );
        for &skill in HEROES[hero].skills {
            let cost = skill_cost(hero, true, skill);
            assert_eq!(
                payment_in(hero, cap, cost, skill, r),
                Some(Payment {
                    mana: cost,
                    blood: 0
                })
            );
        }
    }
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn same_role_has_independent_flags_and_reset_is_a_plain_snapshot_restore() {
    let mut hp = [400, 400];
    let mut flags = [false, false];
    let original = (hp, flags);
    take_damage(0, &mut hp[1], &mut flags[1], 1000);
    assert_eq!(flags, [false, true]);
    assert_eq!(hp, [400, 385]);
    (hp, flags) = original;
    assert_eq!(hp, [400, 400]);
    assert_eq!(flags, [false, false]);
}
#[test]
fn awakened_buffs_costs_and_caps_use_the_actual_shared_rules() {
    let cells = vec![(0, 0), (1, 0), (2, 0)];
    let r = rules(true, 2, true);
    let b = [[2; N]; N];
    assert_eq!(clear_rewards_in(1, 200, 0, &b, &cells, 1, 1, r).1, 75); // (200+25)% then elf -7%
    assert_eq!(
        clear_rewards_in(4, 200, 0, &[[0; N]; N], &cells, 1, 0, r).1,
        30
    );
    assert_eq!(clear_rewards_in(5, 270, 0, &b, &cells, 1, 0, r).1, 45);
    assert_eq!(clear_rewards_in(5, 301, 0, &b, &cells, 1, 0, r).1, 36);
    assert_eq!(
        clear_rewards_in(3, health_cap(3, true) - 1, 0, &[[1; N]; N], &cells, 1, 0, r).2,
        8
    );
    assert_eq!(skill_cost(0, true, Skill::Guard), 80);
    assert_eq!(skill_cost(2, true, Skill::Meteor), 140);
    assert!(payment_in(2, 200, 139, Skill::Meteor, r).is_none());
    assert_eq!(payment_in(2, 200, 140, Skill::Meteor, r).unwrap().mana, 140);
    let p = preview_rewards_in(Skill::Guard, &board(), (0, 0), 0, 380, 80, 500, 0, r);
    assert_eq!((p.health_after, p.heal, p.mana_after), (385, 5, 0));
    assert_eq!(
        effect_at_in(Skill::Charge, &mut b.clone(), (0, 0), 0, 385, 100, r).damage,
        315
    );
}
#[test]
fn awakened_guard_heals_its_new_cap_and_preserves_tile_consumption() {
    let r = rules(true, 0, true);
    let b = board();
    let p = preview_rewards_in(Skill::Guard, &b, (0, 0), 0, 200, 80, 385, 0, r);
    assert_eq!((p.health_after, p.heal, p.mana_after), (277, 77, 0));
    let e = effect_at_in(Skill::Guard, &mut b.clone(), (0, 0), 0, 200, 80, r);
    assert_eq!(e.hp, 77);
    assert!(!e.reward);
    assert!(e.cells.iter().all(|&(x, y)| b[y][x] == 1));
    assert!(payment_in(0, 200, 79, Skill::Guard, r).is_none());
}
#[test]
fn loot_boost_applies_only_to_its_first_clear_and_mana_still_caps() {
    let r = rules(true, 0, true);
    let b = [[2; N]; N];
    let p = preview_rewards_in(Skill::Greedy, &b, (0, 0), 3, 300, 500, 700, 0, r);
    assert_eq!(p.damage, 93); // 24*1.3 floor + 48*1.3 floor
    let cells = vec![(0, 0), (1, 0), (0, 1), (1, 1)];
    assert_eq!(clear_rewards_in(3, 300, 0, &b, &cells, 1, 0, r).1, 48);
    assert_eq!(
        clear_rewards_in(
            3,
            300,
            0,
            &b,
            &cells,
            1,
            0,
            Rules {
                initial_loot: true,
                ..r
            }
        )
        .1,
        62
    );
    let p = preview_rewards_in(Skill::Meteor, &[[3; N]; N], (0, 0), 2, 200, 400, 700, 0, r);
    assert_eq!(p.mana_after, 400);
    assert_eq!(p.mana_gain, 140);
}
#[test]
fn wizard_full_mana_heals_normally_and_checks_phase_cap_before_clear() {
    let b = [[1; N]; N];
    let cells = vec![(0, 0), (1, 0), (2, 0)];
    for (awake, cap) in [(false, 250), (true, 400)] {
        let r = rules(awake, 0, true);
        assert_eq!(
            clear_rewards_in(2, 200, cap, &b, &cells, 1, 0, r),
            (18, 0, 0)
        );
        assert_eq!(
            clear_rewards_in(2, 200, cap - 1, &b, &cells, 1, 0, r),
            (9, 0, 9)
        );
        // Filling MP in this wave still uses the below-cap rule for that wave.
        let mut swap = board();
        swap[0][..3].copy_from_slice(&[1, 0, 1]);
        swap[1][1] = 1;
        let original = swap;
        let seed = crate::rng::state();
        let full = preview_move_in(&swap, 2, 200, cap, 700, 0, false, (1, 0), (1, 1), r).unwrap();
        let low =
            preview_move_in(&swap, 2, 200, cap - 1, 700, 0, false, (1, 0), (1, 1), r).unwrap();
        assert_eq!((full.heal, full.mana_gain), (18, 0));
        assert_eq!((low.heal, low.mana_gain), (9, 1));
        assert_eq!(swap, original);
        assert_eq!(crate::rng::state(), seed);
    }
}
#[test]
fn reduced_reserves_cover_each_roles_largest_skill() {
    assert_eq!(
        HEROES.iter().map(|h| h.mp).collect::<Vec<_>>(),
        [200, 120, 400, 160, 120, 160, 100, 200, 130, 160, 160, 180]
    );
    for hero in 0..HEROES.len() {
        for awake in [false, true] {
            for &skill in HEROES[hero].skills {
                assert!(skill_cost(hero, awake, skill) <= mana_cap(hero, awake));
            }
        }
    }
}
#[test]
fn wizard_reserve_caps_skill_and_swap_rewards_in_both_phases() {
    let b = [[3; N]; N];
    let original = b;
    let seed = crate::rng::state();
    for (awake, cap) in [(false, 250), (true, 400)] {
        assert_eq!(mana_cap(2, awake), cap);
        let r = rules(awake, 0, true);
        assert!(payment_in(2, 200, 59, Skill::Lightning, r).is_none());
        assert_eq!(
            payment_in(2, 200, 60, Skill::Lightning, r),
            Some(Payment { mana: 60, blood: 0 })
        );
        let p = preview_rewards_in(Skill::Lightning, &b, (0, 0), 2, 250, cap - 10, 700, 0, r);
        assert_eq!(p.mana_after, cap);
        assert_eq!(p.mana_gain, 10 + Skill::Lightning.cost());
        let mut swapped = board();
        swapped[0][..3].copy_from_slice(&[3, 0, 3]);
        swapped[1][1] = 3;
        let p =
            preview_move_in(&swapped, 2, 250, cap - 1, 700, 0, false, (1, 0), (1, 1), r).unwrap();
        assert_eq!((p.mana_gain, p.mana_after), (1, cap));
    }
    for hero in [0, 1, 3, 4, 5, 6] {
        assert_eq!(mana_cap(hero, false), HEROES[hero].mp);
    }
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn awakened_flesh_casting_rounds_up_and_requires_survival() {
    let r = rules(true, 0, true);
    for (mp, blood) in [(0, 25), (1, 25), (25, 13), (47, 2), (49, 1), (50, 0)] {
        let pay = payment_in(6, 100, mp, Skill::Drain, r).unwrap();
        assert_eq!(
            pay,
            Payment {
                mana: mp.min(50),
                blood
            }
        );
        if blood > 0 {
            assert!(payment_in(6, blood, mp, Skill::Drain, r).is_none());
            assert!(payment_in(6, blood + 1, mp, Skill::Drain, r).is_some());
        }
    }
    assert_eq!(
        payment_in(6, 101, 0, Skill::Drain, rules(false, 0, true))
            .unwrap()
            .blood,
        100
    );
    assert!(payment_in(0, 100, 0, Skill::Drain, r).is_none());
}
#[test]
fn awakened_flesh_preview_matches_payment_effect_targets_and_caps() {
    let b = board();
    let original = b;
    let seed = crate::rng::state();
    let r = rules(true, 0, true);
    let cap = health_cap(6, true);
    let mut damage = Vec::new();
    for target in [(0, 0), (1, 0), (2, 0)] {
        let p = preview_rewards_in(Skill::Drain, &b, target, 6, cap, 25, 700, 0, r);
        let mut copy = b;
        let e = effect_at_in(Skill::Drain, &mut copy, target, 6, cap, 25, r);
        assert_eq!(p.blood_cost, 13);
        assert_eq!(p.health_after, (cap - 13 + e.hp).min(cap));
        assert_eq!(p.mana_after, 0);
        assert_eq!(p.damage, e.damage);
        assert!(!p.awakens_self);
        damage.push(p.damage);
    }
    assert!(damage.windows(2).any(|w| w[0] != w[1]));
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn payment_can_awaken_before_healing_and_boost_that_same_skill() {
    let b = board();
    let original = b;
    let seed = crate::rng::state();
    let r = rules(false, 0, false);
    let p = preview_rewards_in(Skill::Drain, &b, (0, 0), 6, 250, 25, 700, 0, r);
    let e = effect_at_in(
        Skill::Drain,
        &mut b.clone(),
        (0, 0),
        6,
        250,
        25,
        Rules { awake: true, ..r },
    );
    assert!(p.awakens_self);
    assert_eq!(p.blood_cost, 50);
    assert_eq!(p.health_after, health_cap(6, true));
    assert_eq!(p.damage, e.damage);
    assert_eq!(p.mana_after, 0);
    assert_eq!(payment_in(6, 100, 0, Skill::Drain, r), None); // still must be payable
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn previews_simulate_opponent_awakening_and_subsequent_damage_without_mutation() {
    let b = board();
    let original = b;
    let seed = crate::rng::state();
    let r = rules(true, 0, false);
    let p = preview_rewards_in(Skill::Charge, &b, (0, 0), 0, 385, 300, 330, 0, r);
    assert!(p.awakens_enemy);
    assert_eq!(p.enemy_health_after, 385);
    let p2 = preview_rewards_in(
        Skill::Charge,
        &b,
        (0, 0),
        0,
        385,
        300,
        330,
        0,
        rules(true, 0, true),
    );
    assert!(!p2.awakens_enemy);
    assert!(p2.enemy_health_after < 330);
    let (low, high) = preview_range_in(Skill::Charge, &b, 0, 385, 300, 700, 0, r);
    for y in 0..N {
        for x in 0..N {
            let p = preview_rewards_in(Skill::Charge, &b, (x, y), 0, 385, 300, 700, 0, r);
            assert!((low.damage..=high.damage).contains(&p.damage));
            assert!(
                (low.enemy_health_after..=high.enemy_health_after).contains(&p.enemy_health_after)
            );
        }
    }
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}

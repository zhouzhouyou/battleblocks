use super::*;
fn board() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
#[test]
fn might_is_shared_and_column_swaps_carry_the_knight_identity() {
    assert_eq!(passives(DWARF, false), &[Passive::Might, Passive::Short]);
    assert_eq!(passives(KNIGHT, false), &[Passive::Might, Passive::Clumsy]);
    let cells = [(0, 0), (1, 0), (2, 0)];
    let b = [[2; N]; N];
    // Damage tile 12; Might +2 for both. The knight's edge is the column-swap percentage.
    assert_eq!(
        clear_rewards_in(DWARF, 600, 0, &b, &cells, 1, 0, Rules::default()).1,
        42
    );
    assert_eq!(
        clear_rewards_in(KNIGHT, 600, 0, &b, &cells, 1, 0, Rules::default()).1,
        42
    );
    let horizontal = Rules {
        horizontal: true,
        ..Rules::default()
    };
    assert_eq!(
        clear_rewards_in(KNIGHT, 600, 0, &b, &cells, 1, 0, horizontal).1,
        45
    );
    let awake = Rules {
        awake: true,
        horizontal: true,
        ..Rules::default()
    };
    assert_eq!(
        clear_rewards_in(KNIGHT, 600, 0, &b, &cells, 1, 0, awake).1,
        49
    );
}
#[test]
fn charge_destroys_two_columns_and_triggers_only_attack_and_mana() {
    let b = [[2; N]; N];
    let mut copy = b;
    let e = effect_at_in(
        Skill::Rush,
        &mut copy,
        (0, 0),
        KNIGHT,
        700,
        100,
        Rules::default(),
    );
    assert_eq!(e.cells.len(), 2 * N);
    assert!(!e.reward);
    assert_eq!(e.hp, 0);
    assert_eq!(e.mp, 0);
    assert_eq!(e.damage, 2 * N as i32 * (12 + 2));
    let batch = e.reward_batch(Skill::Rush);
    assert_eq!(batch.total(Resource::Healing, EventSide::Actor), 0);
    assert_eq!(batch.total(Resource::Mana, EventSide::Actor), 0);
    assert_eq!(batch.total(Resource::Damage, EventSide::Opponent), e.damage);
    assert_eq!((Skill::Rush.cost(), Skill::Rush.cooldown()), (100, 3));
    assert!(Skill::Rush.ends_turn());

    // Healing tiles provide neither healing nor mana through the charge.
    let mut heal = [[1; N]; N];
    let e = effect_at_in(
        Skill::Rush,
        &mut heal,
        (0, 0),
        KNIGHT,
        700,
        100,
        Rules::default(),
    );
    assert_eq!((e.hp, e.damage, e.mp), (0, 0, 0));
    assert_eq!(e.reward_batch(Skill::Rush).events.len(), 0);

    // Mana tiles are converted, matching the skill text.
    let mut mana = [[3; N]; N];
    let e = effect_at_in(
        Skill::Rush,
        &mut mana,
        (0, 0),
        KNIGHT,
        700,
        100,
        Rules::default(),
    );
    assert_eq!((e.hp, e.damage, e.mp), (0, 0, 2 * N as i32 * 12));
    assert_eq!(b, [[2; N]; N]);
}
#[test]
fn miracle_no_longer_penalises_the_turn_and_forces_one_move() {
    let b = board();
    let mut state = crate::role_state::RoleState::default();
    state.on_cast(Skill::Miracle, &b, (0, 0));
    assert!(state.force_armed);
    assert!(state.statuses.0.iter().all(Option::is_none));
    assert_eq!((Skill::Miracle.cost(), Skill::Miracle.cooldown()), (40, 2));
    assert!(!Skill::Miracle.ends_turn());
    let before = clear_rewards_in(KNIGHT, 600, 0, &b, &[(0, 0)], 1, 0, Rules::default());
    let after = clear_rewards_in(
        KNIGHT,
        600,
        0,
        &b,
        &[(0, 0)],
        1,
        0,
        Rules {
            statuses: state.statuses,
            ..Rules::default()
        },
    );
    assert_eq!(before, after);
    let mut moved = b;
    assert!(state.complete_force(&mut moved, (0, 0), (1, 0), KNIGHT));
    assert!(!state.force_armed);
}
#[test]
fn no_cooldown_repeat_casts_raise_mana_cost_and_reset_each_turn() {
    let mut rules = Rules::default();
    assert_eq!(skill_cost_in(PRIEST, Skill::Resurrect, rules), 25);
    rules.repeat_casts[Skill::Resurrect.index()] = 1;
    assert_eq!(skill_cost_in(PRIEST, Skill::Resurrect, rules), 40);
    assert!(payment_in(PRIEST, 650, 39, Skill::Resurrect, rules).is_none());
    assert_eq!(
        payment_in(PRIEST, 650, 40, Skill::Resurrect, rules)
            .unwrap()
            .mana,
        40
    );
    rules.repeat_casts[Skill::Resurrect.index()] = 2;
    assert_eq!(skill_cost_in(PRIEST, Skill::Resurrect, rules), 55);
    // End-turn skills cannot be spammed, so their repeat counter must not leak.
    rules.repeat_casts[Skill::Forge.index()] = 2;
    assert_eq!(skill_cost_in(DWARF, Skill::Forge, rules), 80);
    let mut state = crate::role_state::RoleState::default();
    state.record_cast(Skill::Resurrect);
    assert_eq!(state.repeat_casts[Skill::Resurrect.index()], 1);
    state.start_turn();
    assert_eq!(state.repeat_casts, [0; Skill::ALL.len()]);
}
#[test]
fn fatigue_starts_after_twelve_rounds_and_shortens_grindy_games() {
    assert_eq!(fatigue_scales(12), (100, 100));
    assert_eq!(fatigue_scales(13), (112, 90));
    assert_eq!(fatigue_scales(30), (300, 30));
    assert_eq!(fatigue_scales(100), (300, 30));
    let rules = Rules {
        round: 13,
        opponent: Some((KNIGHT, false)),
        ..Rules::default()
    };
    let settled = settle_rewards_in(PRIEST, 500, 0, 700, (10, 100, 5), 0, rules);
    assert_eq!(settled.actor_hp, 509);
    assert_eq!(settled.actor_mp, 5);
    assert_eq!(settled.damage, 112);
}
#[test]
fn every_hero_has_a_character_file_and_the_four_have_full_voice_lines() {
    for hero in 0..HEROES.len() {
        let lore = role_lore(hero).expect("role lore");
        assert!(!lore.title(true).is_empty() && !lore.title(false).is_empty());
        assert!(!lore.intro(true).is_empty() && !lore.intro(false).is_empty());
    }
    for hero in [DWARF, KNIGHT, PRIEST, NUN] {
        assert!(role_lore(hero).unwrap().lines(true).count() >= 6);
        assert!(role_lore(hero).unwrap().lines(false).count() >= 6);
    }
    assert!(role_lore(NUN)
        .unwrap()
        .lines(true)
        .any(|line| line.contains("愿圣光抚平你的伤痛")));
    assert!(role_lore(DWARF)
        .unwrap()
        .lines(true)
        .any(|line| line.contains("该死") || line.contains("够不到")));
}

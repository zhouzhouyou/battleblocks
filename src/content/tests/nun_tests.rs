use super::*;

fn rules(awake: bool, mode: TurnMode) -> Rules {
    Rules {
        awake,
        turn_mode: mode,
        opponent: Some((KNIGHT, false)),
        ..Rules::default()
    }
}

#[test]
fn friendship_uses_raw_healing_even_when_full_and_frenzy_changes_only_opponent_healing() {
    let ordinary = settle_rewards_in(
        NUN,
        650,
        0,
        500,
        (18, 0, 0),
        0,
        rules(false, TurnMode::Normal),
    );
    assert_eq!(
        (ordinary.actor_hp, ordinary.opponent_hp, ordinary.damage),
        (650, 518, 0)
    );
    let frenzy = settle_rewards_in(
        NUN,
        650,
        0,
        500,
        (18, 0, 0),
        0,
        rules(false, TurnMode::Frenzy),
    );
    assert_eq!(
        (frenzy.actor_hp, frenzy.opponent_hp, frenzy.damage),
        (650, 500, 18)
    );
    let capped = settle_rewards_in(
        NUN,
        650,
        0,
        700,
        (18, 0, 0),
        0,
        rules(false, TurnMode::Frenzy),
    );
    assert_eq!(capped.damage, 18);
}

#[test]
fn two_nuns_do_not_retrigger_friendship_from_received_healing() {
    let rules = Rules {
        opponent: Some((NUN, false)),
        ..rules(false, TurnMode::Normal)
    };
    let result = settle_rewards_in(NUN, 600, 0, 600, (20, 0, 0), 0, rules);
    assert_eq!(
        (result.actor_hp, result.opponent_hp, result.damage),
        (620, 620, 0)
    );
}

#[test]
fn frenzy_changes_rewards_without_arming_board_or_adding_legal_moves() {
    let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    let ordinary = rules(false, TurnMode::Normal);
    let frenzy = rules(false, TurnMode::Frenzy);
    assert!(
        !effect_at_in(
            Skill::Frenzy,
            &mut board.clone(),
            (0, 0),
            NUN,
            500,
            50,
            ordinary
        )
        .armed
    );
    assert!(
        !effect_at_in(
            Skill::Rational,
            &mut board.clone(),
            (0, 0),
            NUN,
            500,
            50,
            ordinary
        )
        .armed
    );
    assert_eq!(
        move_options_in(&board, NUN, 500, ordinary),
        move_options_in(&board, NUN, 500, frenzy)
    );
    let listed = ranked_moves_in(&board, NUN, 500, 50, 600, 0, false, frenzy);
    assert_eq!(
        listed.len(),
        move_options_in(&board, NUN, 500, frenzy).len()
    );
}

#[test]
fn healing_prayer_counts_each_side_maximum_and_friendship_adds_a_second_enemy_heal() {
    let mut board = [[0; N]; N];
    let e = effect_at_in(
        Skill::Healing,
        &mut board,
        (0, 0),
        NUN,
        400,
        100,
        rules(false, TurnMode::Normal),
    );
    assert_eq!((e.hp, e.opponent_heal), (65, 70));
    let normal = settle_rewards_in(
        NUN,
        400,
        40,
        400,
        (e.hp, 0, 0),
        e.opponent_heal,
        rules(false, TurnMode::Normal),
    );
    assert_eq!(
        (normal.actor_hp, normal.opponent_hp, normal.damage),
        (465, 535, 0)
    );
    let frenzy = settle_rewards_in(
        NUN,
        400,
        40,
        400,
        (e.hp, 0, 0),
        e.opponent_heal,
        rules(false, TurnMode::Frenzy),
    );
    assert_eq!(
        (frenzy.actor_hp, frenzy.opponent_hp, frenzy.damage),
        (465, 400, 135)
    );
    assert_eq!(Skill::Healing.cost(), 60);
    assert_eq!(Skill::Healing.cooldown(), 3);
    assert!(Skill::Healing.ends_turn());
}

#[test]
fn awakening_replaces_friendship_and_rational_changes_all_attack_into_both_sides_healing() {
    let awake = rules(true, TurnMode::Normal);
    let madness = settle_rewards_in(NUN, 200, 0, 400, (35, 0, 0), 70, awake);
    assert_eq!(
        (madness.actor_hp, madness.opponent_hp, madness.damage),
        (200, 400, 105)
    );
    let rational = settle_rewards_in(
        NUN,
        200,
        0,
        400,
        (0, 40, 0),
        0,
        rules(true, TurnMode::Rational),
    );
    assert_eq!(
        (rational.actor_hp, rational.opponent_hp, rational.damage),
        (240, 440, 0)
    );
    assert_eq!(skills(NUN, false), &[Skill::Frenzy, Skill::Healing]);
    assert_eq!(skills(NUN, true), &[Skill::Rational, Skill::Healing]);
    assert_eq!(
        passives(NUN, true),
        &[Passive::Madness, Passive::AutoRecovery]
    );
    assert!(!Skill::Frenzy.ends_turn() && !Skill::Rational.ends_turn());
    assert_eq!((Skill::Frenzy.cost(), Skill::Rational.cost()), (30, 30));
}

#[test]
fn automatic_recovery_restores_mana_to_cap_and_converts_its_ten_healing() {
    let normal = settle_rewards_in(
        NUN,
        640,
        175,
        500,
        (10, 0, 10),
        0,
        rules(false, TurnMode::Normal),
    );
    assert_eq!(
        (normal.actor_hp, normal.actor_mp, normal.opponent_hp),
        (650, 180, 510)
    );
    let frenzy = settle_rewards_in(
        NUN,
        640,
        175,
        500,
        (10, 0, 10),
        0,
        rules(false, TurnMode::Frenzy),
    );
    assert_eq!(
        (frenzy.actor_hp, frenzy.actor_mp, frenzy.damage),
        (650, 180, 10)
    );
    let awake = settle_rewards_in(
        NUN,
        300,
        175,
        500,
        (10, 0, 10),
        0,
        rules(true, TurnMode::Normal),
    );
    assert_eq!(
        (awake.actor_hp, awake.actor_mp, awake.damage),
        (300, 180, 10)
    );
}

#[test]
fn preview_is_pure_and_matches_direct_healing_and_conversion() {
    let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    let original = board;
    let seed = crate::rng::state();
    let normal = preview_rewards_in(
        Skill::Healing,
        &board,
        (0, 0),
        NUN,
        400,
        100,
        500,
        0,
        rules(false, TurnMode::Normal),
    );
    assert_eq!(
        (
            normal.health_after,
            normal.enemy_health_after,
            normal.mana_after
        ),
        (465, 635, 40)
    );
    let frenzy = preview_rewards_in(
        Skill::Healing,
        &board,
        (0, 0),
        NUN,
        400,
        100,
        500,
        0,
        rules(false, TurnMode::Frenzy),
    );
    assert_eq!(
        (
            frenzy.health_after,
            frenzy.damage,
            frenzy.enemy_health_after
        ),
        (465, 135, 365)
    );
    assert_eq!(board, original);
    assert_eq!(crate::rng::state(), seed);
}

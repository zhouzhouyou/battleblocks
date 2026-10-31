use super::*;

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Frenzy,
    Rational,
}
impl Mode {
    fn statuses(self) -> StatusSet {
        let mut s = StatusSet::default();
        match self {
            Self::Frenzy => s.insert(
                StatusId::Frenzy,
                Skill::Frenzy,
                StatusDuration::OwnerTurnEnd(1),
            ),
            Self::Rational => s.insert(
                StatusId::Rational,
                Skill::Rational,
                StatusDuration::OwnerTurnEnd(1),
            ),
            _ => {}
        }
        s
    }
}
fn rules(awake: bool, mode: Mode) -> Rules {
    Rules {
        awake,
        statuses: mode.statuses(),
        opponent: Some((KNIGHT, false)),
        ..Rules::default()
    }
}

#[test]
fn friendship_uses_raw_healing_even_when_full_and_frenzy_changes_only_opponent_healing() {
    let ordinary = settle_rewards_in(NUN, 650, 0, 500, (18, 0, 0), 0, rules(false, Mode::Normal));
    assert_eq!(
        (ordinary.actor_hp, ordinary.opponent_hp, ordinary.damage),
        (650, 518, 0)
    );
    let frenzy = settle_rewards_in(NUN, 650, 0, 500, (18, 0, 0), 0, rules(false, Mode::Frenzy));
    assert_eq!(
        (frenzy.actor_hp, frenzy.opponent_hp, frenzy.damage),
        (650, 500, 18)
    );
    let capped = settle_rewards_in(NUN, 650, 0, 700, (18, 0, 0), 0, rules(false, Mode::Frenzy));
    assert_eq!(capped.damage, 18);
}

#[test]
fn two_nuns_do_not_retrigger_friendship_from_received_healing() {
    let rules = Rules {
        opponent: Some((NUN, false)),
        ..rules(false, Mode::Normal)
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
    let ordinary = rules(false, Mode::Normal);
    let frenzy = rules(false, Mode::Frenzy);
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
fn ordinary_match_healing_runs_through_friendship_settlement() {
    let mut board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    board[0][..4].copy_from_slice(&[1, 0, 1, 1]);
    assert!(crate::matches(&board).is_empty());
    assert!(crate::board::hero_swap(&mut board, (0, 0), (1, 0), NUN));
    let cells = crate::matches(&board);
    assert_eq!(cells.len(), 3);
    let rules = rules(false, Mode::Normal);
    let (heal, damage, mana) = clear_rewards_in(NUN, 650, 0, &board, &cells, 1, 0, rules);
    assert_eq!((heal, damage, mana), (18, 0, 0));
    let settled = settle_rewards_in(NUN, 650, 0, 400, (heal, damage, mana), 0, rules);
    assert_eq!((settled.actor_hp, settled.opponent_hp), (650, 418));

    let mut original = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    original[0][..4].copy_from_slice(&[1, 0, 1, 1]);
    let preview =
        preview_move_in(&original, NUN, 650, 0, 400, 0, false, (0, 0), (1, 0), rules).unwrap();
    assert_eq!((preview.heal, preview.enemy_health_after), (0, 418));
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
        rules(false, Mode::Normal),
    );
    assert_eq!((e.hp, e.opponent_heal), (65, 70));
    let normal = settle_rewards_in(
        NUN,
        400,
        40,
        400,
        (e.hp, 0, 0),
        e.opponent_heal,
        rules(false, Mode::Normal),
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
        rules(false, Mode::Frenzy),
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
fn awakening_keeps_friendship_and_reason_only_disables_madness() {
    let awake = rules(true, Mode::Normal);
    let madness = settle_rewards_in(NUN, 200, 0, 400, (35, 0, 0), 70, awake);
    assert_eq!(
        (madness.actor_hp, madness.opponent_hp, madness.damage),
        (200, 400, 140)
    );
    let rational = settle_rewards_in(NUN, 200, 0, 400, (0, 40, 0), 0, rules(true, Mode::Rational));
    assert_eq!(
        (rational.actor_hp, rational.opponent_hp, rational.damage),
        (200, 400, 40)
    );
    assert_eq!(skills(NUN, false), &[Skill::Frenzy, Skill::Healing]);
    assert_eq!(skills(NUN, true), &[Skill::Rational, Skill::Healing]);
    assert_eq!(
        passives(NUN, true),
        &[Passive::FriendMilk, Passive::Madness, Passive::AutoRecovery]
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
        rules(false, Mode::Normal),
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
        rules(false, Mode::Frenzy),
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
        rules(true, Mode::Normal),
    );
    assert_eq!(
        (awake.actor_hp, awake.actor_mp, awake.damage),
        (300, 180, 20)
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
        rules(false, Mode::Normal),
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
        rules(false, Mode::Frenzy),
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

#[test]
fn conversion_preserves_causal_chain_but_changes_immediate_source_for_dwarf() {
    let mut board = [[255; N]; N];
    board[0][0] = 1;
    let source = EventSource::Tile {
        position: (0, 0),
        tile: 1,
        chain: 1,
        from_swap: true,
    };
    let make_rules = |awake, mode| Rules {
        opponent: Some((DWARF, false)),
        swap_clear: true,
        ..rules(awake, mode)
    };
    let normal = settle_batch_in(
        NUN,
        400,
        0,
        400,
        clear_batch_in(
            NUN,
            400,
            0,
            &board,
            &[(0, 0)],
            1,
            0,
            make_rules(false, Mode::Normal),
        ),
        make_rules(false, Mode::Normal),
    );
    let gift = normal
        .events
        .iter()
        .find(|e| e.recipient == EventSide::Opponent)
        .unwrap();
    assert_eq!(
        gift.causes.iter().map(|c| c.source).collect::<Vec<_>>(),
        vec![source, EventSource::Passive(Passive::FriendMilk)]
    );
    let frenzy_rules = make_rules(false, Mode::Frenzy);
    let frenzy = settle_batch_in(
        NUN,
        400,
        0,
        400,
        clear_batch_in(NUN, 400, 0, &board, &[(0, 0)], 1, 0, frenzy_rules),
        frenzy_rules,
    );
    assert_eq!((frenzy.actor_hp, frenzy.damage), (406, 6));
    let hit = frenzy
        .events
        .iter()
        .find(|e| e.resource == Resource::Damage)
        .unwrap();
    assert_eq!(
        hit.causes.iter().map(|c| c.source).collect::<Vec<_>>(),
        vec![
            source,
            EventSource::Passive(Passive::FriendMilk),
            EventSource::Status(StatusId::Frenzy)
        ]
    );
    assert!(!frenzy
        .triggers
        .iter()
        .any(|t| t.ability == AbilityId::Passive(Passive::Short)));
    let awake_rules = make_rules(true, Mode::Normal);
    let awake = settle_batch_in(
        NUN,
        200,
        0,
        400,
        clear_batch_in(NUN, 200, 0, &board, &[(0, 0)], 1, 0, awake_rules),
        awake_rules,
    );
    assert_eq!((awake.actor_hp, awake.damage), (200, 12));
    let paths: Vec<_> = awake
        .events
        .iter()
        .filter(|e| e.resource == Resource::Damage)
        .map(|e| e.causes.iter().map(|c| c.source).collect::<Vec<_>>())
        .collect();
    assert!(paths.contains(&vec![source, EventSource::Passive(Passive::Madness)]));
    assert!(paths.contains(&vec![
        source,
        EventSource::Passive(Passive::FriendMilk),
        EventSource::Passive(Passive::Madness)
    ]));
    let reason_rules = make_rules(true, Mode::Rational);
    let reason = settle_batch_in(
        NUN,
        200,
        0,
        400,
        clear_batch_in(NUN, 200, 0, &board, &[(0, 0)], 1, 0, reason_rules),
        reason_rules,
    );
    assert_eq!(
        (reason.actor_hp, reason.opponent_hp, reason.damage),
        (206, 406, 0)
    );
}

#[test]
fn automatic_recovery_converted_by_madness_reports_madness_as_immediate_source() {
    let rules = Rules {
        awake: true,
        opponent: Some((KNIGHT, false)),
        ..Rules::default()
    };
    let settled = settle_batch_in(
        NUN,
        400,
        0,
        400,
        turn_end_rewards(NUN, 400, 0, rules),
        rules,
    );

    assert_eq!(settled.damage, 20);
    let damage: Vec<_> = settled
        .events
        .iter()
        .filter(|event| event.resource == Resource::Damage)
        .filter(|event| event.recipient == EventSide::Opponent)
        .collect();
    assert_eq!(damage.len(), 2);
    assert!(damage
        .iter()
        .all(|event| event.current_source() == EventSource::Passive(Passive::Madness)));
    assert_eq!(
        settled.source_for(Resource::Damage, EventSide::Opponent),
        Some(EventSource::Passive(Passive::Madness))
    );
    assert_eq!(EventSource::Passive(Passive::Madness).name(true), "癫狂");
}

use super::*;

fn board() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}

#[test]
fn fire_elf_identity_and_loadout_replace_the_wizard_slot() {
    assert!(ROSTER.contains(&FIRE_ELF) && !ROSTER.contains(&2));
    assert_eq!(HEROES[FIRE_ELF].hp, 620);
    assert_eq!(HEROES[FIRE_ELF].mp, 180);
    assert_eq!(skills(FIRE_ELF, false), &[Skill::EmberBlast]);
    assert_eq!(
        passives(FIRE_ELF, false),
        &[Passive::ManaAffinity, Passive::SleepRecovery]
    );
    assert_eq!(Passive::SoundSleep.replaces(), Some(Passive::SleepRecovery));
    assert_eq!(
        passives(FIRE_ELF, true),
        &[Passive::ManaAffinity, Passive::SoundSleep]
    );
    assert_eq!(
        skill_cost_in(FIRE_ELF, Skill::EmberBlast, Rules::default()),
        180
    );
}

#[test]
fn ember_blast_requires_full_mana_then_spends_everything() {
    let base = Rules::default();
    assert!(payment_in(FIRE_ELF, 620, 179, Skill::EmberBlast, base).is_none());
    assert_eq!(
        payment_in(FIRE_ELF, 620, 180, Skill::EmberBlast, base)
            .unwrap()
            .mana,
        180
    );

    let awake = Rules {
        awake: true,
        ..Rules::default()
    };
    assert!(payment_in(FIRE_ELF, 620, 143, Skill::EmberBlast, awake).is_none());
    assert_eq!(
        payment_in(FIRE_ELF, 620, 144, Skill::EmberBlast, awake)
            .unwrap()
            .mana,
        144
    );
    assert_eq!(
        payment_in(FIRE_ELF, 620, 180, Skill::EmberBlast, awake)
            .unwrap()
            .mana,
        180
    );
}

#[test]
fn ember_blast_is_one_full_board_destroy_and_scales_with_mana() {
    let mut b = [[3; N]; N];
    let base = effect_at_in(
        Skill::EmberBlast,
        &mut b,
        (0, 0),
        FIRE_ELF,
        620,
        180,
        Rules::default(),
    );
    assert_eq!(base.cells.len(), N * N);
    assert_eq!(base.damage, 223);
    assert!(!base.reward);
    assert_eq!(base.mp, 0);

    let early = effect_at_in(
        Skill::EmberBlast,
        &mut b,
        (0, 0),
        FIRE_ELF,
        620,
        144,
        Rules {
            awake: true,
            ..Rules::default()
        },
    );
    assert_eq!(early.damage, 183);

    let full_awake = effect_at_in(
        Skill::EmberBlast,
        &mut b,
        (0, 0),
        FIRE_ELF,
        620,
        180,
        Rules {
            awake: true,
            ..Rules::default()
        },
    );
    assert_eq!(full_awake.damage, 256);
}

#[test]
fn mana_affinity_converts_two_non_mana_tiles_without_creating_a_match() {
    let mut board = [[2; N]; N];
    for y in 0..N {
        for x in 0..N {
            board[y][x] = if (x + y) % 2 == 0 { 2 } else { 1 };
        }
    }
    assert!(crate::matches(&board).is_empty());
    let triggers = turn_start_effects(FIRE_ELF, 620, 0, &mut board, Rules::default());
    assert!(!triggers.is_empty());
    let converted: Vec<_> = (0..N)
        .flat_map(|y| (0..N).map(move |x| (x, y)))
        .filter(|&(x, y)| board[y][x] == 3)
        .collect();
    assert_eq!(converted.len(), 2);
    assert!(crate::matches(&board).is_empty());
}

#[test]
fn mana_affinity_does_not_touch_mana_tiles_or_force_a_match() {
    let mut board = [[5; N]; N];
    for y in 0..N {
        for x in 0..N {
            board[y][x] = if (x + y) % 2 == 0 { 5 } else { 6 };
        }
    }
    board[0][0] = 3;
    board[0][1] = 3;
    board[0][2] = 2;
    assert!(crate::matches(&board).is_empty());
    let original = board;
    let triggers = turn_start_effects(FIRE_ELF, 620, 0, &mut board, Rules::default());
    assert!(triggers.is_empty());
    assert_eq!(board, original);
    assert!(crate::matches(&board).is_empty());
}

#[test]
fn sleep_recovers_at_turn_end_and_sound_sleep_improves_it() {
    let statuses = StatusSet::single(StatusId::Sleep, Skill::EmberBlast, StatusDuration::SkipTurn);
    let base = turn_end_rewards(
        FIRE_ELF,
        620,
        0,
        Rules {
            statuses,
            ..Rules::default()
        },
    );
    assert_eq!(base.total(Resource::Healing, EventSide::Actor), 49);
    assert_eq!(base.total(Resource::Mana, EventSide::Actor), 27);

    let awake = turn_end_rewards(
        FIRE_ELF,
        341,
        0,
        Rules {
            awake: true,
            statuses,
            ..Rules::default()
        },
    );
    assert_eq!(awake.total(Resource::Healing, EventSide::Actor), 40);
    assert_eq!(awake.total(Resource::Mana, EventSide::Actor), 36);
}

#[test]
fn sound_sleep_reduces_incoming_damage_while_asleep() {
    let statuses = StatusSet::single(StatusId::Sleep, Skill::EmberBlast, StatusDuration::SkipTurn);
    let mut reward = RewardEvent {
        resource: Resource::Damage,
        recipient: EventSide::Actor,
        amount: 100,
        causes: vec![EventCause {
            source: EventSource::Unspecified,
            by: EventSide::Opponent,
        }],
    };
    let ctx = EventContext::new(
        FIRE_ELF,
        341,
        0,
        Rules {
            awake: true,
            statuses,
            ..Rules::default()
        },
    );
    dispatch_rules(
        &ctx,
        &mut RuleEvent::Incoming {
            reward: &mut reward,
        },
        &mut Vec::new(),
    );
    assert_eq!(reward.amount, 70);

    let base_ctx = EventContext::new(
        FIRE_ELF,
        620,
        0,
        Rules {
            statuses,
            ..Rules::default()
        },
    );
    let mut base_reward = reward.clone();
    base_reward.amount = 100;
    dispatch_rules(
        &base_ctx,
        &mut RuleEvent::Incoming {
            reward: &mut base_reward,
        },
        &mut Vec::new(),
    );
    assert_eq!(base_reward.amount, 100);
}

#[test]
fn sleep_is_inserted_after_the_cast_turn_and_consumed_after_the_skip() {
    let mut state = crate::role_state::RoleState::default();
    state.on_cast(Skill::EmberBlast, &board(), (0, 0));
    assert!(state.sleep_pending);
    state.end_turn();
    assert!(state.statuses.contains(StatusId::Sleep));
    assert!(!state.sleep_pending);
    state.end_turn();
    assert!(!state.statuses.contains(StatusId::Sleep));
}

#[test]
fn copied_ember_blast_uses_the_casters_cap_and_makes_the_caster_sleep() {
    // The priest's copy slot only exists in the awakened loadout.
    let awake = Rules {
        awake: true,
        copied_skill: Some(Skill::EmberBlast),
        ..Rules::default()
    };
    assert!(payment_in(PRIEST, 650, 127, Skill::Copy, awake).is_none());
    assert_eq!(
        payment_in(PRIEST, 650, 128, Skill::Copy, awake)
            .unwrap()
            .mana,
        128
    );
    assert_eq!(
        payment_in(PRIEST, 650, 160, Skill::Copy, awake)
            .unwrap()
            .mana,
        160
    );
    let effect = effect_at_in(Skill::Copy, &mut board(), (0, 0), PRIEST, 650, 128, awake);
    assert_eq!(effect.damage, 165);
    assert_eq!(effect.cells.len(), N * N);
    let mut state = crate::role_state::RoleState::default();
    state.on_cast(Skill::EmberBlast, &board(), (0, 0));
    assert!(state.sleep_pending);
}

#[test]
fn copied_ember_blast_does_not_hijack_the_priests_other_skill_payments() {
    let rules = Rules {
        awake: true,
        copied_skill: Some(Skill::EmberBlast),
        ..Rules::default()
    };
    // Resurrect costs 25, far below Ember Blast's 80% threshold (128).
    assert!(payment_in(PRIEST, 650, 24, Skill::Resurrect, rules).is_none());
    assert_eq!(
        payment_in(PRIEST, 650, 25, Skill::Resurrect, rules)
            .unwrap()
            .mana,
        25
    );
    assert!(payment_in(PRIEST, 650, 54, Skill::Entomb, rules).is_none());
    assert_eq!(
        payment_in(PRIEST, 650, 55, Skill::Entomb, rules)
            .unwrap()
            .mana,
        55
    );
}

#[test]
fn copied_ember_blast_does_not_grant_the_fire_elfs_sleep_recovery() {
    let statuses = StatusSet::single(StatusId::Sleep, Skill::EmberBlast, StatusDuration::SkipTurn);
    let rules = Rules {
        awake: true,
        copied_skill: Some(Skill::EmberBlast),
        statuses,
        ..Rules::default()
    };
    let batch = turn_end_rewards(PRIEST, 650, 0, rules);
    assert_eq!(batch.total(Resource::Healing, EventSide::Actor), 0);
    assert_eq!(batch.total(Resource::Mana, EventSide::Actor), 0);
}

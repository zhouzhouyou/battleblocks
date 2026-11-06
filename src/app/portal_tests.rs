use super::*;
fn clean() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
fn session() -> Session {
    let mut s = Session::new_headless();
    s.hero_id = content::PORTAL_CUBE;
    s.enemy_id = content::PORTAL_CUBE;
    s.choosing = false;
    s.b = clean();
    s.hp = 500;
    s.enemy = 500;
    s.mp = 100;
    s.enemy_mp = 100;
    s.paused = false;
    s.deadline = 0.;
    s
}
fn portals() -> board::PortalPair {
    board::PortalPair {
        a: board::PortalEdge::at((1, 2), board::Gravity::Down),
        b: board::PortalEdge::at((4, 4), board::Gravity::Down),
        owner: false,
    }
}
#[test]
fn cast_commits_only_valid_pairs_and_undo_restores_board_geometry_and_cost() {
    let mut s = session();
    let before = s.snapshot();
    let frame = Frame::headless(1.);
    s.target_anchor = Some((1, 2));
    let mut frame = frame;
    frame.command_skill = Some(content::Skill::PortalPair);
    frame.command_target = Some((1, 4));
    assert!(!s.cast_skill(&frame, Some(content::Skill::PortalPair), false));
    assert_eq!(s.mp, 100);
    assert_eq!(s.board_mechanics, before.board_mechanics);
    frame.command_target = Some((4, 4));
    assert!(s.cast_skill(&frame, Some(content::Skill::PortalPair), false));
    assert_eq!(s.mp, 45);
    assert_eq!(s.board_mechanics.portals[0], Some(portals()));
    assert!(!s.action_done);
    assert_eq!(s.b, before.board);
    assert!(s.targeting.is_none() && s.target_anchor.is_none());
    assert_eq!(s.cooldowns.remaining(0, content::Skill::PortalPair), 1);
    s.restore(before.clone());
    assert_eq!(s.mp, before.mp);
    assert_eq!(s.board_mechanics, before.board_mechanics);
    assert_eq!(s.cooldowns, before.cooldowns);
}
#[test]
fn real_refill_rewards_both_owners_on_either_turn_and_rollback_replays_rng() {
    for side in [false, true] {
        let mut s = session();
        s.enemy_turn = side;
        s.board_mechanics.place(portals()).unwrap();
        s.b[4][4] = 0;
        s.pending = vec![(1, 3)];
        s.reward = false;
        s.chain = 1;
        let before = s.snapshot();
        s.resolve_pending(1.);
        let after = s.snapshot();
        assert_eq!(s.b[3][1], 0);
        assert_eq!((s.mp, s.enemy_mp), (101, 101));
        assert_eq!((s.hp, s.enemy), (500, 500));
        assert_eq!(
            (
                s.turn_log.current.heal,
                s.turn_log.current.damage,
                s.turn_log.current.mana
            ),
            (1, 1, 1)
        );
        s.restore(before);
        s.resolve_pending(1.);
        assert_eq!(s.b, after.board);
        assert_eq!(rng::state(), after.rng_state);
        assert_eq!((s.mp, s.enemy_mp), (after.mp, after.enemy_mp));
    }
}
#[test]
fn turn_boundaries_keep_gates_for_opponent_then_expire_and_rescan_matches() {
    let mut s = session();
    s.board_mechanics.place(portals()).unwrap();
    s.board_mechanics.set_gravity(board::Gravity::Up, false);
    s.action_done = true;
    s.advance_turn(&Frame::headless(1.));
    assert!(s.enemy_turn);
    assert!(s.board_mechanics.portals[0].is_some());
    assert_eq!(s.board_mechanics.gravity(), board::Gravity::Up);
    // Removing a gate reconnects three tiles: that match must resolve, not be lost.
    s.b[1][1] = 2;
    s.b[2][1] = 2;
    s.b[3][1] = 2;
    assert!(!s.board_mechanics.matches(&s.b).contains(&(1, 2)));
    s.action_done = true;
    s.deadline = 0.;
    s.advance_turn(&Frame::headless(1.));
    assert!(!s.enemy_turn);
    assert_eq!(s.board_mechanics, Default::default());
    assert!(s.pending.contains(&(1, 2)));
    assert_eq!(s.chain, 1);
}
#[test]
fn enemy_cast_pair_belongs_to_enemy_and_invalid_gravity_does_not_spend() {
    let mut s = session();
    s.enemy_turn = true;
    let mut frame = Frame::headless(1.);
    frame.command_target = Some((7, 7));
    assert!(s.cast_skill(&frame, Some(content::Skill::PortalPair), false));
    assert!(s.board_mechanics.portals[1].is_some());
    assert!(s.board_mechanics.portals[0].is_none());
    assert!(s.board_mechanics.portals[1].unwrap().owner);
    s.enemy_turn = false;
    assert!(!s.cast_skill(&frame, Some(content::Skill::GravityShift), false));
    assert_eq!(s.mp, 100);
}
#[test]
fn portal_matches_finish_with_identical_seeded_results_at_different_frame_speeds() {
    let mut s = session();
    for level in [1, 2] {
        let a = s.headless_batch_match_at_dt(
            content::PORTAL_CUBE,
            content::KNIGHT,
            30300 + level as u64,
            level,
            50000,
            1.,
        );
        let b = s.headless_batch_match_at_dt(
            content::PORTAL_CUBE,
            content::KNIGHT,
            30300 + level as u64,
            level,
            50000,
            1. / 60.,
        );
        assert!(a.winner.is_some(), "portal match failed to terminate");
        assert_eq!(
            (a.winner, a.rounds, a.hp, a.enemy_hp),
            (b.winner, b.rounds, b.hp, b.enemy_hp)
        );
    }
}

#[test]
fn placing_a_pair_queues_remote_matches_immediately_and_preview_equals_real_first_wave() {
    for side in [false, true] {
        let mut s = session();
        s.enemy_turn = side;
        for (x, y) in [(1, 1), (1, 2), (4, 5)] {
            s.b[y][x] = 6;
        }
        assert!(board::matches(&s.b).is_empty());
        let before = s.snapshot();
        let rules = content::Rules {
            target_anchor: Some((1, 2)),
            ..s.rules(side)
        };
        let preview = content::preview_rewards_in(
            content::Skill::PortalPair,
            &s.b,
            (4, 4),
            content::PORTAL_CUBE,
            500,
            100,
            500,
            0,
            rules,
        );
        assert_eq!(rng::state(), before.rng_state);
        // GUI player input uses the selected anchor; enemy command defaults to the
        // opposite line, so exercise the same explicit effect through player cast.
        s.enemy_turn = false;
        s.target_anchor = Some((1, 2));
        let mut frame = Frame::headless(1.);
        frame.command_target = Some((4, 4));
        frame.command_skill = Some(content::Skill::PortalPair);
        assert!(s.cast_skill(&frame, Some(content::Skill::PortalPair), false));
        assert_eq!(s.pending, vec![(1, 1), (1, 2), (4, 5)]);
        assert_eq!(s.mp, 45);
        assert_eq!(s.chain, 1);
        assert!(!s.action_done);
        s.resolve_pending(1.);
        assert_eq!(
            (
                preview.health_after,
                preview.mana_after,
                preview.enemy_health_after
            ),
            (s.hp, s.mp, s.enemy)
        );
        let after = s.snapshot();
        s.restore(before.clone());
        assert_eq!(s.b, before.board);
        assert_eq!(s.board_mechanics, before.board_mechanics);
        // Same commands after rollback reproduce resources, board and RNG.
        s.enemy_turn = false;
        s.target_anchor = Some((1, 2));
        assert!(s.cast_skill(&frame, Some(content::Skill::PortalPair), false));
        s.resolve_pending(1.);
        assert_eq!(
            (s.b, s.hp, s.mp, s.enemy, rng::state()),
            (
                after.board,
                after.hp,
                after.mp,
                after.enemy_hp,
                after.rng_state
            )
        );
    }
}

#[test]
fn existing_portals_work_for_ordinary_roles_and_ai_values_the_same_remote_clear() {
    let mut s = session();
    s.hero_id = content::PRIEST;
    s.board_mechanics.place(portals()).unwrap();
    for (x, y) in [(1, 1), (1, 2), (5, 5)] {
        s.b[y][x] = 6;
    }
    s.b[5][4] = 2;
    assert!(s.board_mechanics.matches(&s.b).is_empty());
    let rules = s.rules(false);
    let before = s.snapshot();
    let p = content::preview_move_in(
        &s.b,
        s.hero_id,
        s.hp,
        s.mp,
        s.enemy,
        0,
        false,
        (4, 5),
        (5, 5),
        rules,
    )
    .unwrap();
    assert_eq!(p.cells, vec![(1, 1), (1, 2), (4, 5)]);
    let mut copy = s.b;
    assert!(board::hero_swap_in(
        &mut copy,
        (4, 5),
        (5, 5),
        s.hero_id,
        s.board_mechanics
    ));
    let score = strategy::score(
        &copy,
        s.hero_id,
        s.hp,
        s.mp,
        content::Rules {
            swap_clear: true,
            horizontal: true,
            ..rules
        },
    );
    assert!(
        (score
            - ((p.damage - p.opponent_heal) as f32
                + p.heal as f32 * 0.8
                + p.mana_gain as f32 * 0.65))
            .abs()
            < 0.001
    );
    assert!(
        content::ranked_moves_in(&s.b, s.hero_id, s.hp, s.mp, s.enemy, 0, false, rules)
            .iter()
            .any(|(m, _)| *m == ((4, 5), (5, 5)))
    );
    assert_eq!(rng::state(), before.rng_state);
    s.b = copy;
    s.pending = p.cells.clone();
    s.reward = true;
    s.chain = 1;
    s.new_role_states[0].horizontal = true;
    s.new_role_states[0].swap_clear = true;
    s.resolve_pending(1.);
    assert_eq!(
        (s.hp, s.mp, s.enemy),
        (p.health_after, p.mana_after, p.enemy_health_after)
    );
}

#[test]
fn known_transit_preview_matches_actual_caps_defenders_and_both_side_orders() {
    for side in [false, true] {
        for opponent in [content::PORTAL_CUBE, content::DWARF, content::FIRE_ELF] {
            let mut s = session();
            s.enemy_turn = side;
            if side {
                s.hero_id = opponent;
            } else {
                s.enemy_id = opponent;
            }
            s.hp = content::health_cap(s.hero_id, false) - 1;
            s.enemy = content::health_cap(s.enemy_id, false) - 1;
            s.mp = content::mana_cap(s.hero_id, false) - 1;
            s.enemy_mp = content::mana_cap(s.enemy_id, false) - 1;
            s.board_mechanics.place(portals()).unwrap();
            s.b[4][4] = 0;
            s.pending = vec![(1, 3)];
            s.reward = false;
            s.chain = 1;
            let before = s.snapshot();
            let (hp, mp, enemy_hp) = if side {
                (s.enemy, s.enemy_mp, s.hp)
            } else {
                (s.hp, s.mp, s.enemy)
            };
            let p = content::preview_clear_in(
                &s.b,
                &s.pending,
                content::PORTAL_CUBE,
                hp,
                mp,
                enemy_hp,
                1,
                0,
                false,
                s.rules(side),
            );
            assert_eq!(s.snapshot().rng_state, before.rng_state);
            assert_eq!(s.b, before.board);
            assert_eq!(s.rules(side).statuses, Default::default());
            s.resolve_pending(1.);
            let actual = if side {
                (s.enemy, s.enemy_mp, s.hp)
            } else {
                (s.hp, s.mp, s.enemy)
            };
            assert_eq!(
                (p.health_after, p.mana_after, p.enemy_health_after),
                actual,
                "actor_enemy={side}, opponent={opponent}"
            );
        }
    }
}

#[test]
fn enemy_portal_cast_preserves_the_planned_anchor_in_validation_and_effects() {
    let mut s = session();
    s.enemy_turn = true;
    s.target_anchor = Some((1, 2));
    let mut frame = Frame::headless(1.);
    frame.command_skill = Some(content::Skill::PortalPair);
    frame.command_target = Some((4, 4));
    assert!(s.cast_skill(&frame, Some(content::Skill::PortalPair), false));
    let mut expected = portals();
    expected.owner = true;
    assert_eq!(s.board_mechanics.portals[1], Some(expected));
    assert_eq!(s.enemy_mp, 45);
    assert_eq!(s.cooldowns.remaining(1, content::Skill::PortalPair), 1);
    assert!(!s.action_done);
}

#[test]
fn portal_and_copying_priest_finish_the_previously_stalled_seed() {
    let mut s = session();
    let result = s.headless_batch_match(
        content::PORTAL_CUBE,
        content::PRIEST,
        5541476115080839947,
        1,
        20000,
    );
    assert!(
        result.winner.is_some(),
        "stalled at round {}",
        result.rounds
    );
}

#[test]
fn portal_mirrors_finish_with_both_sides_using_the_same_two_gate_limit() {
    let mut s = session();
    for (level, seed) in [(1, 361201), (2, 361202)] {
        let result = s.headless_batch_match(
            content::PORTAL_CUBE,
            content::PORTAL_CUBE,
            seed,
            level,
            20000,
        );
        assert!(
            result.winner.is_some(),
            "mirror stalled: level={level}, seed={seed}"
        );
        assert_eq!(s.board_mechanics.portals.len(), 2);
        for (side, pair) in s.board_mechanics.portals.iter().enumerate() {
            if let Some(pair) = pair {
                assert_eq!(pair.owner, side == 1);
            }
        }
    }
}

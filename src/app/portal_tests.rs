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
    assert_eq!(s.mp, 65);
    assert_eq!(s.board_mechanics.portals[0], Some(portals()));
    assert!(!s.action_done);
    assert_eq!(s.b, before.board);
    assert!(s.targeting.is_none() && s.target_anchor.is_none());
    assert_eq!(s.cooldowns.remaining(0, content::Skill::PortalPair), 2);
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

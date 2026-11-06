use super::*;
use crate::board::{self, BoardMechanics, Gravity, PortalEdge, PortalPair, TileFall};
fn board() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
fn transit(kind: u8, count: usize) -> TileFall {
    TileFall {
        from: (0, 0),
        to: (4, 5),
        kind,
        portals: vec![((0, 1), (4, 5)); count],
    }
}
#[test]
fn transit_gains_each_base_resource_once_per_crossing_without_clear_modifiers() {
    for tile in 0..7 {
        let rules = Rules {
            awake: true,
            swap_clear: true,
            horizontal: true,
            statuses: StatusSet::single(
                StatusId::DoubleRewards,
                Skill::Strike,
                StatusDuration::UntilClear,
            ),
            ..Default::default()
        };
        let batch = portal_transit_batch(PORTAL_CUBE, 300, 0, &[transit(tile, 2)], rules);
        let v = VALUES[tile as usize];
        assert_eq!(
            batch.totals(),
            (
                2 * (v.0 > 0) as i32,
                2 * (v.1 > 0) as i32,
                2 * (v.2 > 0) as i32
            )
        );
        assert_eq!(batch.triggers.len(), 1);
        assert!(batch.events.iter().all(|e| e.current_source()
            == EventSource::Passive(Passive::PortalYield)
            && matches!(e.causes[0].source,EventSource::Portal{tile:t,..} if t==tile)));
    }
    assert!(
        portal_transit_batch(KNIGHT, 300, 0, &[transit(0, 1)], Rules::default())
            .events
            .is_empty()
    );
    assert!(
        portal_transit_batch(PORTAL_CUBE, 300, 0, &[transit(0, 0)], Rules::default())
            .events
            .is_empty()
    );
}
#[test]
fn transit_settlement_obeys_caps_and_defenders_without_immunity_from_old_position() {
    let rules = Rules {
        opponent: Some((DWARF, false)),
        ..Default::default()
    };
    let batch = portal_transit_batch(PORTAL_CUBE, 619, 159, &[transit(0, 3)], rules);
    let settled = settle_batch_in(PORTAL_CUBE, 619, 159, 600, batch, rules);
    assert_eq!(
        (settled.actor_hp, settled.actor_mp, settled.damage),
        (620, 160, 3)
    );
    let rules = Rules {
        opponent: Some((FIRE_ELF, true)),
        opponent_statuses: StatusSet::single(
            StatusId::Sleep,
            Skill::EmberBlast,
            StatusDuration::SkipTurn,
        ),
        ..Default::default()
    };
    let batch = portal_transit_batch(PORTAL_CUBE, 300, 0, &[transit(2, 10)], rules);
    let s = settle_batch_in(PORTAL_CUBE, 300, 0, 300, batch, rules);
    // Incoming damage truncates each 1-point event under the existing sleep rule.
    assert_eq!(s.damage, 0);
}
#[test]
fn setup_skills_validate_before_cost_and_previews_leave_state_and_rng_untouched() {
    let mut b = board();
    let before = b;
    let seed = crate::rng::state();
    let rules = Rules {
        target_anchor: Some((1, 2)),
        ..Default::default()
    };
    assert!(!legal_target(
        PORTAL_CUBE,
        620,
        Skill::PortalPair,
        &b,
        (1, 4),
        rules
    ));
    assert!(legal_target(
        PORTAL_CUBE,
        620,
        Skill::PortalPair,
        &b,
        (4, 4),
        rules
    ));
    assert!(payment_in(PORTAL_CUBE, 620, 34, Skill::PortalPair, rules).is_none());
    assert_eq!(
        payment_in(PORTAL_CUBE, 620, 35, Skill::PortalPair, rules)
            .unwrap()
            .mana,
        35
    );
    let p = preview_rewards_in(
        Skill::PortalPair,
        &b,
        (4, 4),
        PORTAL_CUBE,
        620,
        100,
        700,
        0,
        rules,
    );
    assert_eq!((p.heal, p.damage, p.mana_after), (0, 0, 65));
    let invalid = preview_rewards_in(
        Skill::PortalPair,
        &b,
        (1, 4),
        PORTAL_CUBE,
        620,
        100,
        700,
        0,
        rules,
    );
    assert_eq!(invalid.mana_after, 100);
    let e = effect_at_in(
        Skill::PortalPair,
        &mut b,
        (4, 4),
        PORTAL_CUBE,
        620,
        100,
        rules,
    );
    let mut m = rules.board_mechanics;
    e.board_change.unwrap().apply(&mut m, false).unwrap();
    assert!(m.portals[0].is_some());
    assert_eq!(b, before);
    assert_eq!(crate::rng::state(), seed);
    assert_eq!(rules.board_mechanics, BoardMechanics::default());
    assert!(!Skill::PortalPair.ends_turn());
    assert!(!Skill::GravityShift.ends_turn());
}
#[test]
fn changing_gravity_uses_quadrants_and_removes_incompatible_portals() {
    let mut m = BoardMechanics::default();
    m.place(PortalPair {
        a: PortalEdge::at((1, 2), Gravity::Down),
        b: PortalEdge::at((4, 4), Gravity::Down),
        owner: false,
    })
    .unwrap();
    for (target, g, keeps) in [
        ((0, 0), Gravity::Up, true),
        ((7, 0), Gravity::Right, false),
        ((0, 7), Gravity::Left, false),
    ] {
        let mut b = board();
        let rules = Rules {
            board_mechanics: m,
            ..Default::default()
        };
        assert!(legal_target(
            PORTAL_CUBE,
            620,
            Skill::GravityShift,
            &b,
            target,
            rules
        ));
        let e = effect_at_in(
            Skill::GravityShift,
            &mut b,
            target,
            PORTAL_CUBE,
            620,
            100,
            rules,
        );
        let mut next = m;
        e.board_change.unwrap().apply(&mut next, false).unwrap();
        assert_eq!(next.gravity(), g);
        assert_eq!(next.portals[0].is_some(), keeps);
        assert_eq!(b, board());
    }
}
#[test]
fn ai_and_hints_share_blocked_matching_and_never_preview_an_illegal_exchange() {
    let mut b = board();
    b[0][0] = 2;
    b[0][1] = 1;
    b[0][2] = 2;
    b[0][3] = 2;
    b[1][1] = 2;
    let mut m = BoardMechanics::default();
    m.place(PortalPair {
        a: PortalEdge::at((1, 0), Gravity::Down),
        b: PortalEdge::at((4, 4), Gravity::Down),
        owner: false,
    })
    .unwrap();
    let rules = Rules {
        board_mechanics: m,
        ..Default::default()
    };
    assert!(preview_move_in(
        &b,
        PORTAL_CUBE,
        620,
        0,
        700,
        0,
        false,
        (1, 0),
        (1, 1),
        rules
    )
    .is_none());
    for level in 0..3 {
        if let Some((a, c)) =
            crate::strategy::choose_move_in(&b, 620, PORTAL_CUBE, 0, KNIGHT, level, rules)
        {
            assert!(board::hero_swap_in(&mut b.clone(), a, c, PORTAL_CUBE, m));
        }
    }
    for ((a, c), p) in ranked_moves_in(&b, PORTAL_CUBE, 620, 0, 700, 0, false, rules) {
        assert!(!m.blocks(a, c));
        assert!(p.is_some());
    }
}

#[test]
fn only_the_gravity_setup_skill_advertises_gravity_control() {
    assert!(skill_profile(Skill::GravityShift, Rules::default()).gravity_change);
    assert!(!skill_profile(Skill::PortalPair, Rules::default()).gravity_change);
    assert!(!skill_profile(Skill::Hammer, Rules::default()).gravity_change);
}

#[test]
fn portal_pair_defaults_to_the_opposite_gravity_line_inside_the_board() {
    for (gravity, target, partner) in [
        (Gravity::Down, (1, 2), (5, 2)),
        (Gravity::Up, (1, 2), (5, 2)),
        (Gravity::Left, (1, 2), (1, 6)),
        (Gravity::Right, (1, 2), (1, 6)),
    ] {
        let mut mechanics = BoardMechanics::default();
        mechanics.set_gravity(gravity, false);
        let pair = mechanics.portal_pair(target, None, false);
        assert_eq!(pair.a.cell, partner, "{gravity:?}");
        assert_eq!(pair.b.cell, target);
        assert_eq!(pair.a.horizontal, gravity.vertical());
        assert_eq!(pair.b.horizontal, gravity.vertical());
        assert!(!pair.owner);
    }
}

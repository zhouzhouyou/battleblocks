use super::*;
use content::{projectile::Aim, Skill};
fn session() -> Session {
    let mut s = Session::new_simulation();
    s.prepare_headless_batch();
    s.hero_id = content::SLINGSHOT_USER;
    s.enemy_id = 8;
    rng::srand(819);
    s.reset_battle();
    s.mp = 160;
    s.new_role_states[0].turn_start_done = true;
    s
}
#[test]
fn projectile_launch_locks_board_pays_once_and_undo_restores_rng() {
    for dt in [1. / 144., 1. / 30., 10.] {
        let mut s = session();
        let before = s.snapshot();
        let mut frame = Frame::headless(dt);
        frame.command_skill = Some(Skill::HeavySlingshot);
        frame.command_target = Some((7, 3));
        assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
        assert_eq!(s.mp, 105);
        assert_eq!(s.b, before.board);
        assert_eq!(s.enemy, before.enemy_hp);
        assert!(s.presentation.busy());
        assert!(s.charge_damage.as_ref().unwrap().projectile.is_some());
        let flying = s.snapshot();
        s.restore(flying.clone());
        assert!(s.presentation.busy());
        assert_eq!(s.charge_damage, flying.charge_damage);
        s.restore(before.clone());
        assert_eq!(s.b, before.board);
        assert_eq!(s.mp, 160);
        assert_eq!(rng::state(), before.rng_state);
        assert!(s.charge_damage.is_none());
    }
}
#[test]
fn projectile_real_action_matches_preview_and_timestep() {
    let mut results = vec![];
    for dt in [1. / 144., 1. / 30., 10.] {
        let mut s = session();
        let rules = content::Rules {
            projectile_aim: Some(Aim::default()),
            ..s.rules(false)
        };
        let p = content::preview_rewards_in(
            Skill::LightSlingshot,
            &s.b,
            (3, 3),
            content::SLINGSHOT_USER,
            s.hp,
            s.mp,
            s.enemy,
            0,
            rules,
        );
        let initial = s.snapshot();
        s.projectile_aim = Some(Aim::default());
        let mut frame = Frame::headless(dt);
        frame.command_skill = Some(Skill::LightSlingshot);
        frame.command_aim = Some(Aim::default());
        assert!(s.cast_skill(&frame, Some(Skill::LightSlingshot), false));
        while s.presentation.busy() {
            s.presentation.tick(dt);
        }
        s.action_done = false;
        s.advance_turn(&frame);
        assert_eq!(s.hp, p.health_after);
        assert_eq!(s.mp, p.mana_after);
        assert_eq!(s.enemy, p.enemy_health_after);
        while !s.pending.is_empty() || s.presentation.busy() {
            s.presentation.tick(dt);
            if !s.pending.is_empty() {
                s.resolve_pending(dt);
            }
        }
        results.push((s.b, s.hp, s.mp, s.enemy, rng::state()));
        s.restore(initial);
    }
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}
#[test]
fn projectile_monte_carlo_preserves_session_and_live_rng() {
    let mut s = session();
    let before = s.snapshot();
    let key = planning::plan_key(before.clone());
    s.projectile_estimate_for_test(Skill::HeavySlingshot, Aim::default());
    assert_eq!(planning::plan_key(s.snapshot()), key);
    assert_eq!(rng::state(), before.rng_state);
}
#[test]
fn projectile_probability_is_rolled_only_in_actual_session_and_replays() {
    let mut s = session();
    s.hp = 600;
    s.mp = 0;
    let before = s.snapshot();
    let mut batch = content::RewardBatch::gains(
        (20, 30, 40),
        0,
        content::EventSource::Skill(Skill::HeavySlingshot),
    );
    for event in &mut batch.events {
        event.probability = 50;
    }
    s.apply_batch_rewards(batch.clone());
    let result = (s.hp, s.mp, s.enemy, rng::state());
    assert!([600, 620].contains(&s.hp));
    assert!([0, 40].contains(&s.mp));
    s.restore(before);
    s.apply_batch_rewards(batch);
    assert_eq!((s.hp, s.mp, s.enemy, rng::state()), result);
}

#[test]
fn projectile_insufficient_mana_is_transactional_and_targets_use_resource_passives() {
    let mut s = session();
    s.mp = 54;
    let before = planning::plan_key(s.snapshot());
    let mut frame = Frame::headless(1.);
    frame.command_skill = Some(Skill::HeavySlingshot);
    assert!(!s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
    assert_eq!(planning::plan_key(s.snapshot()), before);
    let board = [[1; N]; N];
    let rules = content::Rules {
        projectile_aim: Some(Aim::default()),
        ..Default::default()
    };
    let p = content::preview_rewards_in(
        Skill::LightSlingshot,
        &board,
        (3, 3),
        content::NUN,
        500,
        100,
        500,
        0,
        rules,
    );
    assert!(
        p.opponent_heal > 0,
        "resource-event passives respond to projectile healing"
    );
}

#[test]
fn projectile_sweep_is_deterministic_and_reports_both_upgrade_phases() {
    let before = rng::state();
    let first = Session::projectile_sweep(1, 491);
    let second = Session::projectile_sweep(1, 491);
    assert_eq!(first, second);
    assert_eq!(first.0.lines().count(), 65);
    assert_eq!(first.1.lines().count(), 5);
    assert_eq!(rng::state(), before);
    assert!(first.0.contains("true,HeavySlingshot"));
    assert_eq!(
        debug::parse("shoot 2 35.5 75").unwrap(),
        debug::Command::Shoot(
            1,
            Aim {
                angle_mdeg: 35500,
                power_permille: 750
            }
        )
    );
    assert!(debug::parse("shoot 2 NaN 100").is_err());
}

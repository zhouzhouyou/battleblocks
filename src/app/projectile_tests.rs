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
fn projectile_blast_aftermath_and_rollback_keep_the_same_presentation_clock() {
    let mut s = session();
    s.awakened[0] = true;
    s.projectile_aim = Some(Aim {
        ammo: content::projectile::Ammo::Explosive,
        ..Default::default()
    });
    let mut frame = Frame::headless(0.01);
    frame.command_skill = Some(Skill::HeavySlingshot);
    frame.command_aim = Some(Aim {
        ammo: content::projectile::Ammo::Explosive,
        ..Default::default()
    });
    assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
    let flight = s
        .charge_damage
        .as_ref()
        .unwrap()
        .projectile
        .as_ref()
        .unwrap();
    assert!(flight.explosion_radius.is_some());
    let duration = flight.duration;
    assert!(
        (s.presentation.remaining() - duration - effects::PROJECTILE_BLAST_DURATION as f64).abs()
            < 0.001
    );
    s.presentation.tick(duration + 0.4);
    let snapshot = s.snapshot();
    assert!(s.presentation.busy());
    let remaining = s.presentation.remaining();
    s.restore(snapshot);
    assert!((s.presentation.remaining() - remaining).abs() < 0.001);
    s.presentation.tick(remaining + 0.001);
    assert!(!s.presentation.busy());
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
        assert_eq!(s.mp, 90);
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
    s.mp = 69;
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
    assert_eq!(first.0.lines().count(), 81);
    assert_eq!(first.1.lines().count(), 6);
    assert_eq!(rng::state(), before);
    assert!(first.0.contains("true,HeavySlingshot"));
    assert_eq!(
        debug::parse("shoot 2 35.5 75").unwrap(),
        debug::Command::Shoot(
            1,
            Aim {
                angle_mdeg: 35500,
                power_permille: 750,
                ..Default::default()
            }
        )
    );
    assert!(debug::parse("shoot 2 NaN 100").is_err());
}

#[test]
fn projectile_awakening_increases_force_and_rejects_locked_blasts() {
    use content::projectile::Ammo;
    let mut s = session();
    let normal = content::skill_profile(Skill::HeavySlingshot, s.rules(false))
        .projectile
        .unwrap();
    s.awakened[0] = true;
    let awake = content::skill_profile(Skill::HeavySlingshot, s.rules(false))
        .projectile
        .unwrap();
    for angle in (5000..=80000).step_by(5000) {
        for power in [0, 250, 750, 1000] {
            let aim = Aim {
                angle_mdeg: angle,
                power_permille: power,
                ammo: Ammo::Solid,
            };
            let ordinary = normal.solve(&s.b, aim);
            let powered = awake.solve(&s.b, aim);
            assert!((powered.velocity.x / ordinary.velocity.x - 1.15).abs() < 1e-9);
            assert!((powered.velocity.y / ordinary.velocity.y - 1.15).abs() < 1e-9);
        }
    }
    let blast = awake.solve(
        &s.b,
        Aim {
            ammo: Ammo::Explosive,
            ..Default::default()
        },
    );
    assert_eq!(blast.ammo, Ammo::Explosive);
    assert!(blast.explosion_radius.is_some());
    assert_eq!(content::skills(content::SLINGSHOT_USER, true).len(), 2);
    s.awakened[0] = false;
    let before = s.snapshot();
    let mut frame = Frame::headless(0.01);
    frame.command_skill = Some(Skill::HeavySlingshot);
    frame.command_aim = Some(Aim {
        ammo: Ammo::Explosive,
        ..Default::default()
    });
    assert!(!s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
    assert_eq!(s.mp, before.mp);
    assert_eq!(s.b, before.board);
    assert_eq!(
        debug::parse("shoot 2 35 100 explosive").unwrap(),
        debug::Command::Shoot(1, frame.command_aim.unwrap())
    );
    assert!(debug::parse("shoot 2 35 100 stones").is_err());
}

#[test]
fn projectile_rewards_respect_payment_caps_opponents_and_pure_preview() {
    use content::projectile::Ammo;
    for awake in [false, true] {
        for enemy in [content::DWARF, content::NUN] {
            let mut s = session();
            s.enemy_id = enemy;
            s.enemy = content::health_cap(enemy, false);
            s.awakened[0] = awake;
            s.hp = content::health_cap(content::SLINGSHOT_USER, awake);
            s.b = [[2; N]; N];
            let aim = Aim {
                ammo: if awake { Ammo::Explosive } else { Ammo::Solid },
                ..Default::default()
            };
            let rules = content::Rules {
                projectile_aim: Some(aim),
                ..s.rules(false)
            };
            let before = s.snapshot();
            let rng_before = rng::state();
            let preview = content::preview_rewards_in(
                Skill::HeavySlingshot,
                &s.b,
                (3, 3),
                s.hero_id,
                s.hp,
                s.mp,
                s.enemy,
                0,
                rules,
            );
            assert_eq!(preview.mana_after, 160 - 70);
            assert_eq!(s.snapshot().json(), before.json());
            assert_eq!(rng::state(), rng_before);
            let mut frame = Frame::headless(0.01);
            frame.command_skill = Some(Skill::HeavySlingshot);
            frame.command_aim = Some(aim);
            assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
            assert_eq!(s.mp, 90);
            s.presentation.tick(10.);
            s.advance_turn(&frame);
            assert_eq!(s.mp, preview.mana_after);
            assert_eq!(s.enemy, preview.enemy_health_after);
            s.restore(before);
            s.b = [[3; N]; N];
            let preview = content::preview_rewards_in(
                Skill::HeavySlingshot,
                &s.b,
                (3, 3),
                s.hero_id,
                s.hp,
                s.mp,
                s.enemy,
                0,
                rules,
            );
            assert_eq!(preview.mana_after, content::mana_cap(s.hero_id, awake));
            s.mp = 69;
            assert!(!s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
            assert_eq!(s.mp, 69);
        }
    }
}

#[test]
fn projectile_plans_search_both_ammunition_types_and_preserve_the_selected_shot() {
    use super::planning::{PlanAction, SearchPolicy};
    use content::projectile::Ammo;
    let mut s = session();
    s.awakened[0] = true;
    s.hp = content::health_cap(s.hero_id, true);
    let policy = SearchPolicy {
        targets: 1,
        ..SearchPolicy::for_level(1)
    };
    let before = s.snapshot();
    let candidates = s.plan_candidates(policy);
    for ammo in [Ammo::Solid, Ammo::Explosive] {
        let action = candidates.iter().copied().find(|action| {
            matches!(action, PlanAction::Shoot(Skill::HeavySlingshot, aim) if aim.ammo == ammo)
        }).expect("heavy slingshot must search each available ammunition type");
        s.restore(before.clone());
        s.projectile_aim = Some(Aim {
            ammo: if ammo == Ammo::Solid {
                Ammo::Explosive
            } else {
                Ammo::Solid
            },
            ..Default::default()
        });
        let mut frame = Frame::headless(0.01);
        action.inject(&mut s, &mut frame);
        assert_eq!(frame.command_aim.unwrap().ammo, ammo);
        assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
        assert_eq!(
            s.charge_damage
                .as_ref()
                .unwrap()
                .projectile
                .as_ref()
                .unwrap()
                .ammo,
            ammo
        );
    }
}

#[test]
fn projectile_balanced_costs_require_full_payment_for_each_ammunition_and_phase() {
    use content::projectile::Ammo;
    for awake in [false, true] {
        for (skill, cost, ammo) in [
            (Skill::LightSlingshot, 30, Ammo::Solid),
            (Skill::HeavySlingshot, 70, Ammo::Solid),
            (Skill::HeavySlingshot, 70, Ammo::Explosive),
        ] {
            if ammo == Ammo::Explosive && !awake {
                continue;
            }
            let mut s = session();
            s.awakened[0] = awake;
            s.hp = content::health_cap(s.hero_id, awake);
            s.b = [[2; N]; N];
            s.mp = cost - 1;
            let mut frame = Frame::headless(0.01);
            frame.command_skill = Some(skill);
            frame.command_aim = Some(Aim {
                ammo,
                ..Default::default()
            });
            assert_eq!(
                content::skill_cost_in(s.hero_id, skill, s.rules(false)),
                cost
            );
            let before = s.snapshot();
            assert!(!s.cast_skill(&frame, Some(skill), false));
            assert_eq!(s.snapshot().json(), before.json());
            s.mp = cost;
            let before = s.snapshot();
            let preview = content::preview_rewards_in(
                skill,
                &s.b,
                (3, 3),
                s.hero_id,
                s.hp,
                s.mp,
                s.enemy,
                0,
                content::Rules {
                    projectile_aim: frame.command_aim,
                    ..s.rules(false)
                },
            );
            assert_eq!(s.snapshot().json(), before.json());
            assert_eq!(rng::state(), before.rng_state);
            assert!(s.cast_skill(&frame, Some(skill), false));
            assert_eq!(s.mp, 0);
            s.presentation.tick(10.);
            s.advance_turn(&frame);
            assert_eq!(s.mp, preview.mana_after);
            assert_eq!(s.enemy, preview.enemy_health_after);
        }
    }
}

#[test]
fn projectile_overdrive_preserves_effect_ratios_and_heavy_bombs_reach_the_last_column() {
    use content::projectile::{Ammo, End};
    let s = session();
    for skill in [Skill::LightSlingshot, Skill::HeavySlingshot] {
        let normal = content::skill_profile(skill, s.rules(false))
            .projectile
            .unwrap();
        let awake = content::skill_profile(
            skill,
            content::Rules {
                awake: true,
                ..s.rules(false)
            },
        )
        .projectile
        .unwrap();
        assert_eq!(normal.direct_reward.percent, awake.direct_reward.percent);
        assert_eq!(
            normal.direct_reward.probability,
            awake.direct_reward.probability
        );
        assert_eq!(skill.cooldown(), 2);
    }
    let config = content::skill_profile(
        Skill::HeavySlingshot,
        content::Rules {
            awake: true,
            ..s.rules(false)
        },
    )
    .projectile
    .unwrap();
    let solid = config.solve(&s.b, Aim::default());
    let blast = config.solve(
        &s.b,
        Aim {
            ammo: Ammo::Explosive,
            ..Default::default()
        },
    );
    assert!((solid.velocity.x / blast.velocity.x - 1.75_f64.sqrt()).abs() < 1e-9);
    let farthest = (5000..=80000)
        .step_by(100)
        .map(|angle_mdeg| {
            config.solve(
                &s.b,
                Aim {
                    ammo: Ammo::Explosive,
                    angle_mdeg,
                    power_permille: 1000,
                },
            )
        })
        .max_by(|a, b| {
            a.position(a.duration)
                .x
                .total_cmp(&b.position(b.duration).x)
        })
        .unwrap();
    let landing = farthest.position(farthest.duration);
    assert!(matches!(farthest.end, End::Landing | End::Fuse));
    assert!(landing.x > 6. && landing.x < 6.3, "burst {landing:?}");
    assert!(farthest.explosion_cells.iter().any(|cell| cell.0 == 7));
    assert!(farthest.explosion_radius.is_some());
}

#[test]
fn projectile_enemy_launch_and_snapshot_keep_right_to_left_physics() {
    use content::projectile::Ammo;
    for ammo in [Ammo::Solid, Ammo::Explosive] {
        let mut s = session();
        s.enemy_id = content::SLINGSHOT_USER;
        s.enemy_turn = true;
        s.enemy_mp = 160;
        s.awakened[1] = true;
        s.enemy = content::health_cap(s.enemy_id, true);
        let aim = Aim {
            ammo,
            ..Default::default()
        };
        let config = content::skill_profile(Skill::HeavySlingshot, s.rules(true))
            .projectile
            .unwrap();
        let expected = config.solve(&s.b, aim);
        assert!(expected.origin.x > 8.);
        assert!(expected.velocity.x < 0.);
        let mut frame = Frame::headless(0.01);
        frame.command_skill = Some(Skill::HeavySlingshot);
        frame.command_aim = Some(aim);
        let before = s.snapshot();
        assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
        assert_eq!(s.enemy_mp, 90);
        assert_eq!(
            s.charge_damage
                .as_ref()
                .unwrap()
                .projectile
                .as_ref()
                .unwrap(),
            &expected
        );
        let flying = s.snapshot();
        s.restore(flying);
        assert_eq!(
            s.charge_damage
                .as_ref()
                .unwrap()
                .projectile
                .as_ref()
                .unwrap(),
            &expected
        );
        assert_eq!(s.cooldowns.remaining(1, Skill::HeavySlingshot), 2);
        s.cooldowns.start_turn(1);
        assert_eq!(s.cooldowns.remaining(1, Skill::HeavySlingshot), 1);
        s.cooldowns.start_turn(1);
        assert_eq!(s.cooldowns.remaining(1, Skill::HeavySlingshot), 0);
        s.restore(before);
        assert_eq!(s.enemy_mp, 160);
        assert_eq!(s.cooldowns.remaining(1, Skill::HeavySlingshot), 0);
    }
}

#[test]
fn projectile_advanced_hints_choose_ammunition_by_the_board() {
    use content::projectile::Ammo;
    use planning::PlanAction;
    let mut selected = std::collections::HashSet::new();
    for seed in 490049..490065 {
        let mut s = session();
        rng::srand(seed);
        s.reset_battle();
        s.awakened[0] = true;
        s.hp = content::health_cap(s.hero_id, true);
        s.mp = 160;
        s.new_role_states[0].turn_start_done = true;
        let before = s.snapshot();
        let plans = s.ranked_ai_plans(false);
        let action = plans[0].steps[0].action;
        if let PlanAction::Shoot(Skill::HeavySlingshot, aim) = action {
            selected.insert(aim.ammo as usize);
        }
        assert_eq!(s.snapshot().json(), before.json());
        assert_eq!(rng::state(), before.rng_state);
    }
    assert!(selected.contains(&(Ammo::Explosive as usize)));
    assert!(selected.contains(&(Ammo::Solid as usize)));
}

#[test]
fn projectile_fixed_airburst_ends_flight_early_and_matches_preview_after_rollback() {
    use content::projectile::{Ammo, End};
    let mut s = session();
    s.awakened[0] = true;
    s.hp = content::health_cap(s.hero_id, true);
    let aim = Aim {
        ammo: Ammo::Explosive,
        angle_mdeg: 60000,
        power_permille: 1000,
    };
    let config = content::skill_profile(Skill::HeavySlingshot, s.rules(false))
        .projectile
        .unwrap();
    let blast = config.solve(&s.b, aim);
    let solid = config.solve(
        &s.b,
        Aim {
            ammo: Ammo::Solid,
            ..aim
        },
    );
    assert_eq!(config.explosion.unwrap().fuse, Some(0.8));
    assert_eq!(blast.end, End::Fuse);
    assert_eq!(blast.duration, 0.8);
    assert!(blast.position(blast.duration).y < 6.);
    assert!(solid.duration > blast.duration);
    let rules = content::Rules {
        projectile_aim: Some(aim),
        ..s.rules(false)
    };
    let before = s.snapshot();
    let preview = content::preview_rewards_in(
        Skill::HeavySlingshot,
        &s.b,
        (3, 3),
        s.hero_id,
        s.hp,
        s.mp,
        s.enemy,
        0,
        rules,
    );
    assert_eq!(s.snapshot().json(), before.json());
    assert_eq!(rng::state(), before.rng_state);
    let mut frame = Frame::headless(0.01);
    frame.command_skill = Some(Skill::HeavySlingshot);
    frame.command_aim = Some(aim);
    assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
    assert_eq!(s.mp, 90);
    assert_eq!(s.b, before.board);
    assert_eq!(
        s.charge_damage
            .as_ref()
            .unwrap()
            .projectile
            .as_ref()
            .unwrap(),
        &blast
    );
    s.presentation.tick(0.4);
    let flying = s.snapshot();
    s.restore(flying.clone());
    assert_eq!(s.snapshot().json(), flying.json());
    s.presentation.tick(10.);
    s.advance_turn(&frame);
    assert_eq!(s.mp, preview.mana_after);
    assert_eq!(s.enemy, preview.enemy_health_after);
}

#[test]
fn projectile_gravity_shift_matches_preview_payment_and_rollback_for_both_sides() {
    use content::projectile::Ammo;
    for direction in [
        board::Gravity::Down,
        board::Gravity::Up,
        board::Gravity::Left,
        board::Gravity::Right,
    ] {
        for enemy in [false, true] {
            for ammo in [Ammo::Solid, Ammo::Explosive] {
                let mut s = session();
                s.enemy_id = content::SLINGSHOT_USER;
                s.enemy_turn = enemy;
                s.enemy_mp = 160;
                s.awakened = [true; 2];
                s.hp = content::health_cap(s.hero_id, true);
                s.enemy = content::health_cap(s.enemy_id, true);
                s.b = [[2; N]; N];
                s.board_mechanics.set_gravity(direction, !enemy);
                let aim = Aim {
                    ammo,
                    ..Default::default()
                };
                let rules = content::Rules {
                    projectile_aim: Some(aim),
                    ..s.rules(enemy)
                };
                let config = content::skill_profile(Skill::HeavySlingshot, rules)
                    .projectile
                    .unwrap();
                assert_eq!(config.gravity_direction, direction);
                let expected = config.solve(&s.b, aim);
                let before = s.snapshot();
                let preview = content::preview_rewards_in(
                    Skill::HeavySlingshot,
                    &s.b,
                    (3, 3),
                    content::SLINGSHOT_USER,
                    s.hp,
                    160,
                    s.enemy,
                    0,
                    rules,
                );
                assert_eq!(s.snapshot().json(), before.json());
                assert_eq!(rng::state(), before.rng_state);
                let mut frame = Frame::headless(0.01);
                frame.command_skill = Some(Skill::HeavySlingshot);
                frame.command_aim = Some(aim);
                assert!(s.cast_skill(&frame, Some(Skill::HeavySlingshot), false));
                assert_eq!(if enemy { s.enemy_mp } else { s.mp }, 90);
                assert_eq!(
                    s.charge_damage
                        .as_ref()
                        .unwrap()
                        .projectile
                        .as_ref()
                        .unwrap(),
                    &expected
                );
                let flying = s.snapshot();
                s.restore(flying.clone());
                assert_eq!(s.snapshot().json(), flying.json());
                s.presentation.tick(10.);
                s.advance_turn(&frame);
                assert_eq!(
                    if enemy { s.hp } else { s.enemy },
                    preview.enemy_health_after
                );
                assert_eq!(if enemy { s.enemy_mp } else { s.mp }, preview.mana_after);
                s.restore(before.clone());
                assert_eq!(s.snapshot().json(), before.json());
                assert_eq!(rng::state(), before.rng_state);
            }
        }
    }
}

use super::*;
use crate::board;
use projectile::*;
fn config(skill: Skill, rules: Rules) -> ProjectileConfig {
    static CONFIG: std::sync::OnceLock<Vec<(Skill, bool, ProjectileConfig)>> =
        std::sync::OnceLock::new();
    let entries = CONFIG.get_or_init(|| {
        include_str!("../../../config/projectiles.txt")
            .lines()
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|line| {
                let f: Vec<_> = line.split('|').collect();
                assert_eq!(f.len(), 17);
                let skill = *Skill::ALL
                    .iter()
                    .find(|s| format!("{s:?}") == f[0])
                    .expect("projectile skill");
                let number = |i: usize| f[i].parse::<f64>().expect("projectile parameter");
                let rule = |i: usize| RewardRule {
                    percent: f[i]
                        .split(',')
                        .map(|v| v.parse().unwrap())
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                    probability: f[i + 1]
                        .split(',')
                        .map(|v| v.parse().unwrap())
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                };
                for index in [2, 3, 6, 7, 10, 12, 13, 16] {
                    assert!(
                        number(index).is_finite() && number(index) >= 0.,
                        "finite nonnegative projectile parameter"
                    );
                }
                assert_eq!(number(13).fract(), 0., "integer penetration count");
                let radius = number(10);
                assert!(
                    skill != Skill::LightSlingshot || radius == 0.,
                    "light slingshot cannot explode"
                );
                let config = ProjectileConfig {
                    from_right: false,
                    min_speed: number(2),
                    max_speed: number(3),
                    min_angle: f[4].parse().unwrap(),
                    max_angle: f[5].parse().unwrap(),
                    gravity: number(6),
                    gravity_direction: board::Gravity::Down,
                    radius: number(7),
                    direct_reward: rule(8),
                    explosion: (radius > 0.).then(|| Explosion {
                        radius,
                        mass: number(16),
                        landing: f[11].parse().unwrap(),
                        fuse: (number(12) > 0.).then(|| number(12)),
                        penetrations: (number(13) > 0.).then(|| f[13].parse().unwrap()),
                        reward: rule(14),
                    }),
                };
                assert!(
                    number(16) >= 1.
                        && config.min_speed > 0.
                        && config.max_speed >= config.min_speed
                        && config.gravity > 0.
                        && config.radius > 0.
                        && config.radius < 0.8
                );
                assert!(
                    config.min_angle >= 0
                        && config.max_angle < 90000
                        && config.min_angle <= config.max_angle
                );
                for rule in [
                    config.direct_reward,
                    config
                        .explosion
                        .map(|e| e.reward)
                        .unwrap_or(config.direct_reward),
                ] {
                    assert!(
                        rule.percent.iter().all(|v| *v >= 0)
                            && rule.probability.iter().all(|v| *v <= 100)
                    );
                }
                (skill, f[1].parse().unwrap(), config)
            })
            .collect()
    });
    let config = entries
        .iter()
        .find(|(s, a, _)| *s == skill && *a == rules.awake)
        .expect("projectile phase configuration")
        .2;
    ProjectileConfig {
        from_right: rules.launch_from_right,
        gravity_direction: rules.board_mechanics.gravity(),
        ..config
    }
}
fn rewards(
    board: &Board,
    cells: &[(usize, usize)],
    rule: RewardRule,
    skill: Skill,
    explosion: bool,
) -> RewardBatch {
    let mut batch = RewardBatch::default();
    // Allocate aggregate truncation back to the source cells, as for ordinary clears.
    let mut cumulative = [0; 3];
    for &(x, y) in cells {
        let v = VALUES[board[y][x] as usize];
        for (i, (resource, recipient, value)) in [
            (Resource::Healing, EventSide::Actor, v.0),
            (Resource::Damage, EventSide::Opponent, v.1),
            (Resource::Mana, EventSide::Actor, v.2),
        ]
        .into_iter()
        .enumerate()
        {
            let before = cumulative[i] * rule.percent[i] / 100;
            cumulative[i] += value;
            let amount = cumulative[i] * rule.percent[i] / 100 - before;
            let mut part = RewardBatch::default();
            part.push(
                resource,
                recipient,
                amount,
                EventSource::Projectile {
                    skill,
                    position: (x, y),
                    tile: board[y][x],
                    explosion,
                },
            );
            for event in &mut part.events {
                event.probability = rule.probability[i];
            }
            batch.events.extend(part.events);
        }
    }
    batch
}
pub fn handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    _: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Active(skill) = id else {
        return;
    };
    match event {
        RuleEvent::Profile { value } => {
            value.projectile = Some(config(skill, ctx.rules));
        }
        RuleEvent::Targeted { value, .. } => **value = true,
        RuleEvent::LegalTarget { allowed, .. } => {
            **allowed = ctx.rules.projectile_aim.is_none_or(|aim| {
                aim.ammo != Ammo::Explosive || config(skill, ctx.rules).explosion.is_some()
            });
        }
        RuleEvent::NoSwapUsable { value, .. } => **value = Some(true),
        RuleEvent::Cast {
            board,
            target,
            effect,
            ..
        } => {
            let config = config(skill, ctx.rules);
            let flight = config.solve(
                board,
                ctx.rules
                    .projectile_aim
                    .unwrap_or_else(|| config.aim_for_target(*target)),
            );
            let direct: Vec<_> = flight.hits.iter().map(|h| h.cell).collect();
            let mut batch = rewards(board, &direct, config.direct_reward, skill, false);
            if let Some(explosion) = config.explosion {
                batch.events.extend(
                    rewards(
                        board,
                        &flight.explosion_cells,
                        explosion.reward,
                        skill,
                        true,
                    )
                    .events,
                );
            }
            let totals = batch.totals();
            effect.hp = totals.0;
            effect.damage = totals.1;
            effect.mp = totals.2;
            effect.cells = flight.cells();
            effect.events = Some(batch);
        }
        _ => {}
    }
}

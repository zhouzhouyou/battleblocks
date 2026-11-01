use super::*;
pub(super) fn effect(
    skill: Skill,
    board: &Board,
    target: (usize, usize),
    ctx: &EventContext,
) -> Effect {
    match skill {
        Skill::Rush => {
            let x = target.0.min(N - 2);
            let cells: Vec<_> = (0..N).flat_map(|y| [(x, y), (x + 1, y)]).collect();
            let mut effect = Effect {
                cells,
                reward: false,
                ..Default::default()
            };
            let mut batch = RewardBatch::default();
            for &(cx, cy) in &effect.cells {
                let tile = board[cy][cx];
                if tile >= 7 {
                    continue;
                }
                let value = VALUES[tile as usize];
                if value.1 == 0 && value.2 == 0 {
                    continue;
                }
                let source = EventSource::Tile {
                    position: (cx, cy),
                    tile,
                    chain: 0,
                    from_swap: false,
                };
                let mut gains = (0, value.1, value.2);
                let mut triggers = Vec::new();
                dispatch_rules(
                    ctx,
                    &mut RuleEvent::TileReward {
                        source,
                        gains: &mut gains,
                    },
                    &mut triggers,
                );
                effect.triggers.extend(triggers);
                batch
                    .events
                    .extend(RewardBatch::gains(gains, 0, source).events);
                effect.damage += gains.1;
                effect.mp += gains.2;
            }
            effect.events = Some(batch);
            effect
        }
        Skill::Hammer => {
            let (x, y) = (target.0.min(N - 2), target.1.min(N - 2));
            Effect {
                cells: vec![(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)],
                damage: 80,
                reward: true,
                ..Default::default()
            }
        }
        Skill::Miracle => Effect::default(),
        _ => unreachable!("not a knight skill"),
    }
}

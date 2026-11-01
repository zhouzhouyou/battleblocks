use super::*;
pub(super) fn effect(skill: Skill, target: (usize, usize)) -> Effect {
    if skill == Skill::Entomb {
        Effect {
            cells: vec![target],
            ..Default::default()
        }
    } else {
        Effect::default()
    }
}
pub(super) fn insert(b: &mut Board, target: (usize, usize), tile: u8) {
    // Insertion pushes toward the bottom even if a future turn changes gravity.
    for y in (target.1 + 1..N).rev() {
        b[y][target.0] = b[y - 1][target.0];
    }
    b[target.1][target.0] = tile;
}

pub(super) fn cast(
    skill: Skill,
    b: &mut Board,
    target: (usize, usize),
    ctx: &EventContext,
) -> Effect {
    if skill == Skill::Resurrect {
        if let Some(tile) = ctx
            .rules
            .pockets
            .get(ctx.rules.pocket_slot)
            .copied()
            .flatten()
        {
            insert(b, target, tile);
            let mut batch = RewardBatch::default();
            batch.push(
                Resource::Damage,
                EventSide::Opponent,
                6,
                EventSource::Skill(skill),
            );
            return Effect {
                damage: 6,
                events: Some(batch),
                ..Default::default()
            };
        }
        Effect::default()
    } else {
        effect(skill, target)
    }
}

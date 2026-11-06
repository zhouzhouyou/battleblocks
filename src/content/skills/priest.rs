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
            crate::board::insert_tile(b, target, tile, ctx.rules.board_mechanics);
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

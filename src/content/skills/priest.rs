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
pub(super) fn insert(
    b: &mut Board,
    target: (usize, usize),
    tile: u8,
    gravity: crate::board::Gravity,
) {
    // Insertion pushes toward the settling side of the current gravity;
    // the tile squeezed out at that edge is destroyed.
    use crate::board::Gravity;
    let (x, y) = target;
    match gravity {
        Gravity::Down => {
            for row in (y + 1..N).rev() {
                b[row][x] = b[row - 1][x];
            }
        }
        Gravity::Up => {
            for row in 0..y {
                b[row][x] = b[row + 1][x];
            }
        }
        Gravity::Right => {
            for column in (x + 1..N).rev() {
                b[y][column] = b[y][column - 1];
            }
        }
        Gravity::Left => {
            for column in 0..x {
                b[y][column] = b[y][column + 1];
            }
        }
    }
    b[y][x] = tile;
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
            insert(b, target, tile, ctx.rules.board_mechanics.gravity());
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

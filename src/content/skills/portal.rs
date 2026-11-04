//! Skill targeting translates player choices into board topology edits.
use super::*;
use crate::board::{BoardMechanics, Gravity, PortalEdge, PortalPair};
/// Target a tile's bottom/right edge. With no first target (AI/compact console cast),
/// use a deterministic partner on the opposite gravity line at the same depth.
pub fn pair(rules: Rules, target: (usize, usize)) -> PortalPair {
    let first = rules.target_anchor.unwrap_or_else(|| {
        if rules.board_mechanics.gravity().vertical() {
            ((target.0 + N / 2) % N, target.1)
        } else {
            (target.0, (target.1 + N / 2) % N)
        }
    });
    let g = rules.board_mechanics.gravity();
    PortalPair {
        a: PortalEdge::at(first, g),
        b: PortalEdge::at(target, g),
        owner: rules.actor_enemy,
    }
}
pub fn direction(target: (usize, usize)) -> Gravity {
    let (x, y) = target;
    if y < N / 2 {
        if x < N / 2 {
            Gravity::Up
        } else {
            Gravity::Right
        }
    } else if x < N / 2 {
        Gravity::Left
    } else {
        Gravity::Down
    }
}
pub fn mechanics(skill: Skill, rules: Rules, target: (usize, usize)) -> Option<BoardMechanics> {
    let mut next = rules.board_mechanics;
    match skill {
        Skill::PortalPair => {
            next.place(pair(rules, target)).ok()?;
        }
        Skill::GravityShift => {
            next.set_gravity(direction(target), rules.actor_enemy);
        }
        _ => return None,
    }
    Some(next)
}

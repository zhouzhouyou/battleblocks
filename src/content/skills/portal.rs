//! Skill targeting translates player choices into board topology edits.
use super::*;
use crate::board::{BoardMechanics, Gravity, PortalPair};
/// Pair construction, default partner selection and edge orientation are board
/// topology; the skill only supplies the chosen target and anchor.
pub fn pair(rules: Rules, target: (usize, usize)) -> PortalPair {
    rules
        .board_mechanics
        .portal_pair(target, rules.target_anchor, rules.actor_enemy)
}
pub fn mechanics(skill: Skill, rules: Rules, target: (usize, usize)) -> Option<BoardMechanics> {
    let mut next = rules.board_mechanics;
    match skill {
        Skill::PortalPair => {
            next.place(pair(rules, target)).ok()?;
        }
        Skill::GravityShift => {
            next.set_gravity(Gravity::from_quadrant(target), rules.actor_enemy);
        }
        _ => return None,
    }
    Some(next)
}

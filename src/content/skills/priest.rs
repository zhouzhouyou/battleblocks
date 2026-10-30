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

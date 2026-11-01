use super::*;
pub(super) fn effect(skill: Skill, b: &mut Board, target: (usize, usize)) -> Effect {
    if skill == Skill::Forge {
        let (x, y) = (target.0.min(N - 2), target.1.min(N - 2));
        for row in &mut b[y..y + 2] {
            for tile in &mut row[x..x + 2] {
                *tile = 2;
            }
        }
    }
    Effect::default()
}

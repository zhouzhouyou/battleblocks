use super::*;
pub(super) fn effect(skill: Skill, target: (usize, usize)) -> Effect {
    match skill {
        Skill::Rush => {
            let x = target.0.min(N - 2);
            Effect {
                cells: (0..N).flat_map(|y| [(x, y), (x + 1, y)]).collect(),
                reward: true,
                ..Default::default()
            }
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

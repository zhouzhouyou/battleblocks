use super::*;

pub(super) fn effect(skill: Skill) -> Effect {
    match skill {
        Skill::Frenzy | Skill::Rational | Skill::Healing => Effect::default(),
        _ => unreachable!(),
    }
}

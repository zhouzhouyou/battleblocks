use super::*;

pub(super) fn effect(skill: Skill) -> Effect {
    match skill {
        Skill::Frenzy | Skill::Rational | Skill::Healing => Effect::default(),
        _ => unreachable!(),
    }
}

pub(super) fn cast(skill: Skill, ctx: &EventContext) -> Effect {
    let mut e = effect(skill);
    if skill == Skill::Healing {
        e.hp = health_cap(ctx.hero, ctx.rules.awake) * 7 / 100;
        e.opponent_heal = ctx
            .rules
            .opponent
            .map(|(id, a)| health_cap(id, a) * 7 / 100)
            .unwrap_or(0);
    }
    e
}

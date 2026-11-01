use super::*;

/// The single active is a full-board destroy with one damage settlement.
/// It deliberately does not trigger tile effects: the board clear is tempo,
/// not a second mana cycle.
pub(super) fn cast(skill: Skill, ctx: &EventContext) -> Effect {
    let mut effect = Effect::default();
    if skill != Skill::EmberBlast {
        return effect;
    }
    effect.cells = (0..N).flat_map(|y| (0..N).map(move |x| (x, y))).collect();
    effect.reward = false;
    let mut damage = ctx.mp * 110 / 100 + 25;
    if ctx.rules.awake && ctx.mp >= mana_cap(ctx.hero, true) {
        damage = damage * 115 / 100;
    }
    effect.damage = damage;
    effect
}

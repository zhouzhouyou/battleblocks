use super::*;

pub const AWAKENING_LINES: [i32; 14] = [45, 35, 40, 50, 37, 35, 30, 45, 40, 40, 40, 40, 40, 50];
pub const AWAKENING_CAPS: [i32; 14] = [55, 45, 50, 55, 50, 47, 44, 55, 60, 55, 55, 55, 55, 55];
pub fn awakening_line(hero: usize) -> i32 {
    HEROES[hero].hp * AWAKENING_LINES[hero] / 100
}
pub fn health_cap(hero: usize, awake: bool) -> i32 {
    if awake {
        HEROES[hero].hp * AWAKENING_CAPS[hero] / 100
    } else {
        HEROES[hero].hp
    }
}
/// Wizard's reserve unlocks on awakening; other roles keep their base cap.
pub fn mana_cap(hero: usize, awake: bool) -> i32 {
    let mut value = HEROES[hero].mp;
    dispatch_rules(
        &EventContext::new(
            hero,
            0,
            0,
            Rules {
                awake,
                ..Default::default()
            },
        ),
        &mut RuleEvent::ManaCap { value: &mut value },
        &mut Vec::new(),
    );
    value
}
pub fn skill_cost(hero: usize, awake: bool, skill: Skill) -> i32 {
    let mut value = skill.cost();
    dispatch_rules(
        &EventContext::new(
            hero,
            0,
            0,
            Rules {
                awake,
                ..Default::default()
            },
        ),
        &mut RuleEvent::SkillCost {
            skill,
            value: &mut value,
        },
        &mut Vec::new(),
    );
    value
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageResult {
    pub damage: i32,
    pub awakened: bool,
    pub restored: i32,
}
/// One damage event. The triggering hit cannot spill into the second life bar.
pub fn take_damage(hero: usize, hp: &mut i32, awake: &mut bool, damage: i32) -> DamageResult {
    if damage <= 0 || *hp <= 0 {
        return DamageResult::default();
    }
    let before = *hp;
    let hit = before.saturating_sub(damage).max(0);
    let triggered = !*awake && hit <= awakening_line(hero);
    *hp = if triggered {
        health_cap(hero, true)
    } else {
        hit
    };
    *awake |= triggered;
    DamageResult {
        damage: before - hit,
        awakened: triggered,
        restored: if triggered { *hp - hit } else { 0 },
    }
}
pub(super) fn preview_hit(rules: Rules, hp: &mut i32, damage: i32) -> (i32, Rules) {
    let mut next = rules;
    if let Some((hero, mut awake)) = rules.opponent {
        let hit = take_damage(hero, hp, &mut awake, damage);
        next.opponent = Some((hero, awake));
        (hit.damage, next)
    } else {
        let actual = damage.max(0).min(*hp);
        *hp -= actual;
        (actual, next)
    }
}
/// Persistent indicators for conditional passives; phase itself has no badge.
#[cfg(test)]
pub fn passive_status(
    hero: usize,
    hp: i32,
    mp: i32,
    awake: bool,
    zh: bool,
) -> Option<(&'static str, bool)> {
    passives(hero, awake)
        .iter()
        .find_map(|p| p.summary_status(hero, hp, mp, awake, zh))
}

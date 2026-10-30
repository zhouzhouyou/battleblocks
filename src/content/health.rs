use super::*;

pub const AWAKENING_LINES: [i32; 10] = [45, 35, 40, 50, 37, 35, 30, 45, 40, 40];
pub const AWAKENING_CAPS: [i32; 10] = [55, 45, 50, 55, 50, 47, 44, 55, 60, 55];
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
    if has_passive(hero, awake, Passive::ManaSeal) {
        250
    } else {
        HEROES[hero].mp
    }
}
pub fn skill_cost(hero: usize, awake: bool, skill: Skill) -> i32 {
    if has_passive(hero, awake, Passive::GuardEconomy) && skill == Skill::Guard {
        80
    } else if has_passive(hero, awake, Passive::MeteorEconomy) && skill == Skill::Meteor {
        140
    } else {
        skill.cost()
    }
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
pub fn passive_status(
    hero: usize,
    hp: i32,
    mp: i32,
    awake: bool,
    zh: bool,
) -> Option<(&'static str, bool)> {
    if hero == 5 {
        let passive = if has_passive(hero, awake, Passive::BoilingBlood) {
            Passive::BoilingBlood
        } else {
            Passive::BloodFury
        };
        return passive.status(hero, hp, mp, awake, zh);
    }
    if hero == 6 && hp > 0 {
        let missing = HEROES[hero]
            .skills
            .iter()
            .copied()
            .filter(|&s| mp < skill_cost(hero, awake, s))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            let active = missing.into_iter().any(|s| {
                payment_in(
                    hero,
                    hp,
                    mp,
                    s,
                    Rules {
                        awake,
                        ..Rules::default()
                    },
                )
                .is_some()
            });
            return Some((
                if active {
                    crate::tr_key(zh, "health.blood_ready")
                } else {
                    crate::tr_key(zh, "health.blood_too_low")
                },
                active,
            ));
        }
    }
    None
}

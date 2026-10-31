use super::*;
#[path = "skills/dwarf.rs"]
mod dwarf;
#[path = "skills/knight.rs"]
mod knight;
#[path = "skills/legacy.rs"]
mod legacy;
#[path = "skills/nun.rs"]
mod nun;
#[path = "skills/priest.rs"]
mod priest;

pub const VALUES: [(i32, i32, i32); 7] = [
    (2, 4, 4),
    (6, 0, 0),
    (0, 12, 0),
    (0, 0, 12),
    (3, 7, 0),
    (3, 0, 7),
    (0, 7, 7),
];
#[derive(Default)]
pub struct Effect {
    pub cells: Vec<(usize, usize)>,
    pub hp: i32,
    pub opponent_heal: i32,
    pub damage: i32,
    pub mp: i32,
    pub reward: bool,
    pub bonus: i32,
    pub armed: bool,
}
pub fn effect(skill: Skill, b: &mut Board) -> Effect {
    effect_at(skill, b, (3, 3))
}
pub fn targeted(skill: Skill) -> bool {
    matches!(
        skill,
        Skill::Charge
            | Skill::Nature
            | Skill::Greedy
            | Skill::Chort
            | Skill::Balance
            | Skill::War
            | Skill::Drain
            | Skill::Rush
            | Skill::Forge
            | Skill::Entomb
            | Skill::Resurrect
            | Skill::Copy
    )
}
#[cfg(test)]
pub fn target_cells(skill: Skill, b: &Board, target: (usize, usize)) -> Vec<(usize, usize)> {
    target_cells_in(skill, b, target, 0, HEROES[0].hp, 0, Rules::default())
}
pub fn target_cells_in(
    skill: Skill,
    b: &Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    rules: Rules,
) -> Vec<(usize, usize)> {
    let mut copy = *b;
    let e = effect_at_in(skill, &mut copy, target, hero, hp, mp, rules);
    let transformed_only = e.cells.is_empty();
    let mut cells = e.cells;
    for y in 0..N {
        for x in 0..N {
            if b[y][x] != copy[y][x] && !cells.contains(&(x, y)) {
                cells.push((x, y));
            }
        }
    }
    if transformed_only {
        for cell in crate::matches(&copy) {
            if !cells.contains(&cell) {
                cells.push(cell);
            }
        }
    }
    cells
}
pub fn trigger_tiles(
    board: &Board,
    cells: &[(usize, usize)],
    healing_percent: i32,
    damage_percent: i32,
    mana_percent: i32,
) -> (i32, i32, i32) {
    let (mut h, mut d, mut m) = (0, 0, 0);
    for &(x, y) in cells {
        if board[y][x] < 7 {
            let v = VALUES[board[y][x] as usize];
            h += v.0;
            d += v.1;
            m += v.2;
        }
    }
    (
        h * healing_percent / 100,
        d * damage_percent / 100,
        m * mana_percent / 100,
    )
}
pub fn effect_at(skill: Skill, b: &mut Board, target: (usize, usize)) -> Effect {
    match skill {
        Skill::Rush | Skill::Miracle | Skill::Hammer => knight::effect(skill, target),
        Skill::Jump | Skill::Forge => dwarf::effect(skill, b, target),
        Skill::Entomb | Skill::Resurrect | Skill::Copy => priest::effect(skill, target),
        Skill::Frenzy | Skill::Healing | Skill::Rational => nun::effect(skill),
        _ => legacy::effect(skill, b, target),
    }
}

pub fn effect_at_in(
    skill: Skill,
    b: &mut Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    rules: Rules,
) -> Effect {
    let skill = effective_skill(skill, rules);
    if skill == Skill::Resurrect {
        if let Some(tile) = rules.pockets.get(rules.pocket_slot).copied().flatten() {
            priest::insert(b, target, tile);
        }
        return Effect::default();
    }
    let mut e = effect_at(skill, b, target);
    if skill == Skill::Guard {
        e.hp = health_cap(hero, rules.awake) / 5;
    }
    if skill == Skill::Healing {
        e.hp = health_cap(hero, rules.awake) / 10;
        e.opponent_heal = rules
            .opponent
            .map(|(id, awake)| health_cap(id, awake) / 10)
            .unwrap_or(0);
    }
    if rules.awake {
        if hero == 0 && skill == Skill::Guard {
            e.hp = health_cap(hero, true) / 5;
        }
        if has_passive(hero, rules.awake, Passive::BlastMastery) && skill == Skill::Imp {
            e.damage += e.cells.len() as i32 * 2;
        }
        if (has_passive(hero, rules.awake, Passive::ChargeMastery) && skill == Skill::Charge)
            || (has_passive(hero, rules.awake, Passive::LootMastery) && skill == Skill::Greedy)
        {
            e.damage = e.damage * if hero == 0 { 125 } else { 130 } / 100;
        }
        if has_passive(hero, rules.awake, Passive::BloodSacrifice)
            && payment_in(hero, hp, mp, skill, rules).is_some_and(|p| p.blood > 0)
        {
            e.damage = e.damage * 120 / 100;
        }
    }
    e
}

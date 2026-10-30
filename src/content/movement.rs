use super::*;

pub fn reachable_rows(hero: usize, hp: i32, rules: Rules) -> usize {
    if hero != DWARF {
        return N;
    }
    let levels = if rules.awake {
        [80, 60, 40]
            .iter()
            .filter(|&&v| hp * 100 <= health_cap(hero, true) * v)
            .count()
    } else {
        0
    };
    (4 + levels + usize::from(rules.jumped)).min(N)
}
pub fn targeted_in(skill: Skill, rules: Rules) -> bool {
    targeted(effective_skill(skill, rules))
}
pub fn move_options_in(
    board: &Board,
    hero: usize,
    hp: i32,
    rules: Rules,
) -> Vec<((usize, usize), (usize, usize))> {
    crate::board::hero_options(board, hero)
        .into_iter()
        .filter(|&(a, b)| legal_swap_region(hero, hp, a, b, rules))
        .collect()
}
pub fn legal_swap_region(
    hero: usize,
    hp: i32,
    a: (usize, usize),
    b: (usize, usize),
    rules: Rules,
) -> bool {
    hero != DWARF
        || (a.1 >= N - reachable_rows(hero, hp, rules)
            && b.1 >= N - reachable_rows(hero, hp, rules))
}
pub fn legal_target(
    hero: usize,
    hp: i32,
    skill: Skill,
    board: &Board,
    target: (usize, usize),
    rules: Rules,
) -> bool {
    if target.0 >= N || target.1 >= N {
        return false;
    }
    let effective = effective_skill(skill, rules);
    if effective == Skill::Copy {
        return false;
    }
    if effective == Skill::Jump && rules.jumped {
        return false;
    }
    if effective == Skill::Entomb
        && (rules.pockets.iter().all(Option::is_some) || board[target.1][target.0] >= 7)
    {
        return false;
    }
    if effective == Skill::Resurrect
        && rules
            .pockets
            .get(rules.pocket_slot)
            .copied()
            .flatten()
            .is_none()
    {
        return false;
    }
    if hero == DWARF && targeted(effective) {
        let cells = match effective {
            Skill::Forge
            | Skill::Nature
            | Skill::Greedy
            | Skill::Balance
            | Skill::War
            | Skill::Drain => {
                let (x, y) = (target.0.min(N - 2), target.1.min(N - 2));
                vec![(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)]
            }
            Skill::Entomb | Skill::Resurrect => vec![target],
            _ => effect_at(effective, &mut board.clone(), target).cells,
        };
        if cells
            .iter()
            .any(|&(_, y)| y < N - reachable_rows(hero, hp, rules))
        {
            return false;
        }
    }
    true
}
/// Actual actions that can continue a no-swap turn. Preparations alone do not unlock a dead board.
#[cfg(test)]
pub fn usable_skills(board: &Board, hero: usize, hp: i32, mp: i32) -> Vec<Skill> {
    usable_skills_in(board, hero, hp, mp, Rules::default())
}
pub fn usable_skills_in(board: &Board, hero: usize, hp: i32, mp: i32, rules: Rules) -> Vec<Skill> {
    skills(hero, rules.awake)
        .iter()
        .copied()
        .filter(|&skill| {
            if !legal_target(hero, hp, skill, board, (0, 0), rules)
                && !targeted(effective_skill(skill, rules))
            {
                return false;
            }
            let Some(pay) = payment_in(hero, hp, mp, skill, rules) else {
                return false;
            };
            if effective_skill(skill, rules) == Skill::Jump && hero == DWARF && !rules.jumped {
                return !move_options_in(
                    board,
                    hero,
                    hp,
                    Rules {
                        jumped: true,
                        ..rules
                    },
                )
                .is_empty();
            }
            (0..N).any(|y| {
                (0..N).any(|x| {
                    if !legal_target(hero, hp, skill, board, (x, y), rules) {
                        return false;
                    }
                    if effective_skill(skill, rules) == Skill::Miracle {
                        return true;
                    }
                    let mut copy = *board;
                    let e = effect_at_in(skill, &mut copy, (x, y), hero, hp, mp, rules);
                    !e.cells.is_empty()
                        || copy != *board
                        || e.armed
                        || e.damage > 0
                        || (e.hp > pay.blood && hp - pay.blood < health_cap(hero, rules.awake))
                        || (mp - pay.mana + e.mp).min(mana_cap(hero, rules.awake)) > mp
                })
            })
        })
        .collect()
}
#[cfg(test)]
pub fn must_refresh(board: &Board, hero: usize, hp: i32, mp: i32, armed: bool) -> bool {
    must_refresh_in(board, hero, hp, mp, armed, Rules::default())
}
#[allow(dead_code)] // Retained for the headless legacy balance probe.
pub fn must_refresh_in(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    armed: bool,
    rules: Rules,
) -> bool {
    !armed
        && crate::board::hero_options(board, hero).is_empty()
        && usable_skills_in(board, hero, hp, mp, rules).is_empty()
}

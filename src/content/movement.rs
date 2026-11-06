use super::*;

/// Reach is measured from the downstream edge of the current gravity, so a
/// "bottom four rows" passive becomes the settling four rows/columns.
pub fn reachable_depth(hero: usize, hp: i32, rules: Rules) -> usize {
    let mut depth = N;
    dispatch_rules(
        &EventContext::new(hero, hp, 0, rules),
        &mut RuleEvent::Reach { depth: &mut depth },
        &mut Vec::new(),
    );
    depth.min(N)
}
pub fn reachable_cell(hero: usize, hp: i32, cell: (usize, usize), rules: Rules) -> bool {
    rules.board_mechanics.gravity().depth(cell) < reachable_depth(hero, hp, rules)
}
pub fn movement_profile(hero: usize) -> MovementProfile {
    let mut profile = MovementProfile::default();
    dispatch_rules(
        &EventContext::new(hero, 0, 0, Rules::default()),
        &mut RuleEvent::Movement {
            profile: &mut profile,
        },
        &mut Vec::new(),
    );
    profile
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
    crate::board::hero_options_in(board, hero, rules.board_mechanics)
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
    let depth = reachable_depth(hero, hp, rules);
    let gravity = rules.board_mechanics.gravity();
    !rules.board_mechanics.blocks(a, b) && gravity.depth(a) < depth && gravity.depth(b) < depth
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
    if !ability_enabled(
        &EventContext::new(hero, hp, 0, rules),
        AbilityId::Active(effective),
    ) {
        return false;
    }
    let mut allowed = true;
    dispatch_active(
        effective,
        &EventContext::new(hero, hp, 0, rules),
        &mut RuleEvent::LegalTarget {
            skill: effective,
            board,
            target,
            allowed: &mut allowed,
        },
        &mut Vec::new(),
    );
    if !allowed {
        return false;
    }
    let depth = reachable_depth(hero, hp, rules);
    if depth < N && targeted(effective) {
        let mut copy = *board;
        let e = effect_at_in(effective, &mut copy, target, hero, hp, 0, rules);
        let mut cells = e.cells;
        for y in 0..N {
            for x in 0..N {
                if board[y][x] != copy[y][x] {
                    cells.push((x, y));
                }
            }
        }
        let gravity = rules.board_mechanics.gravity();
        if cells.iter().any(|&cell| gravity.depth(cell) >= depth) {
            return false;
        }
    }
    true
}
/// Uniformly pick a target inside the reachable settling region. Used by
/// skills whose target is chosen at random (Hammer) so it follows gravity.
pub fn random_reachable_target(hero: usize, hp: i32, rules: Rules) -> (usize, usize) {
    let depth = crate::rng::gen_range(0, reachable_depth(hero, hp, rules).max(1));
    let along = crate::rng::gen_range(0, N - 1);
    rules.board_mechanics.gravity().cell_at_depth(depth, along)
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
            let mut override_use = None;
            dispatch_active(
                effective_skill(skill, rules),
                &EventContext::new(hero, hp, mp, rules),
                &mut RuleEvent::NoSwapUsable {
                    board,
                    value: &mut override_use,
                },
                &mut Vec::new(),
            );
            if let Some(value) = override_use {
                return value;
            }
            (0..N).any(|y| {
                (0..N).any(|x| {
                    if !legal_target(hero, hp, skill, board, (x, y), rules) {
                        return false;
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
        && crate::board::hero_options_in(board, hero, rules.board_mechanics).is_empty()
        && usable_skills_in(board, hero, hp, mp, rules).is_empty()
}

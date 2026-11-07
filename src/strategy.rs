use crate::content::Rules;
use crate::{content::Skill, Board, N};
type Move = ((usize, usize), (usize, usize));

/// Experimental AI evaluation switches. They only change what the search
/// prices; the canonical rules always settle actions normally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AiExperiment {
    /// Include the opponent's skills when pricing their best reply.
    pub opponent_skills: bool,
    /// Look one opponent turn ahead and subtract their best reply.
    pub opponent_reply: bool,
    /// After that reply, also price the owner's following turn.
    pub follow_up: bool,
    /// Actively consider portal and gravity-changing skills.
    pub portals: bool,
}

impl Default for AiExperiment {
    fn default() -> Self {
        Self {
            opponent_skills: true,
            opponent_reply: true,
            follow_up: true,
            portals: true,
        }
    }
}

/// Apply a candidate to a private board, with the same exchange and reward context everywhere.
fn after_move(
    b: &Board,
    hero: usize,
    a: (usize, usize),
    c: (usize, usize),
    rules: Rules,
) -> (Board, Rules) {
    let mut copy = *b;
    let _ = crate::board::exchange_in(&mut copy, a, c, hero, false, rules.board_mechanics);
    (
        copy,
        Rules {
            horizontal: a.1 == c.1,
            swap_clear: true,
            ..rules
        },
    )
}

pub(crate) fn best_move_score_in(b: &Board, hero: usize, hp: i32, mp: i32, rules: Rules) -> f32 {
    choose_move_in(b, hp, hero, mp, hero, 1, rules)
        .map(|(a, c)| {
            let (next, next_rules) = after_move(b, hero, a, c, rules);
            score(&next, hero, hp, mp, next_rules)
        })
        .unwrap_or(0.)
}
/// Shared AI resource weights for single actions and multi-step hints.
pub(crate) fn reward_value(damage: i32, health: i32, mana: i32) -> f32 {
    damage as f32 + health as f32 * 0.8 + mana as f32 * 0.65
}
pub(crate) fn score(b: &Board, hero: usize, hp: i32, mp: i32, rules: Rules) -> f32 {
    let p = crate::content::preview_clear_in(
        b,
        &rules.board_mechanics.matches(b),
        hero,
        hp,
        mp,
        rules.opponent_hp.unwrap_or(10000),
        1,
        0,
        true,
        rules,
    );
    reward_value(p.damage - p.opponent_heal, p.heal, p.mana_gain)
}
#[cfg(test)]
pub fn choose_move(
    b: &Board,
    hp: i32,
    hero: usize,
    mp: i32,
    opponent: usize,
    level: usize,
) -> Option<Move> {
    choose_move_in(b, hp, hero, mp, opponent, level, Rules::default())
}
pub fn choose_move_in(
    b: &Board,
    hp: i32,
    hero: usize,
    mp: i32,
    opponent: usize,
    level: usize,
    rules: Rules,
) -> Option<Move> {
    let moves = crate::content::move_options_in(b, hero, hp, rules);
    if moves.is_empty() {
        return None;
    }
    if level == 0 {
        return Some(moves[crate::rng::gen_range(0, moves.len())]);
    }
    ranked_move_options_in(b, hp, hero, mp, opponent, level, rules)
        .first()
        .map(|(m, _)| *m)
}

/// All exchange candidates scored by the same policy used for an AI move.
pub(crate) fn ranked_move_options_in(
    b: &Board,
    hp: i32,
    hero: usize,
    mp: i32,
    opponent: usize,
    level: usize,
    rules: Rules,
) -> Vec<(Move, f32)> {
    let moves = crate::content::move_options_in(b, hero, hp, rules);
    let mut ranked: Vec<_> = moves
        .into_iter()
        .map(|(a, c)| {
            let (mut copy, move_rules) = after_move(b, hero, a, c, rules);
            let mut value = score(&copy, hero, hp, mp, move_rules);
            if level == 2 {
                for (x, y) in rules.board_mechanics.matches(&copy) {
                    copy[y][x] = 255;
                }
                let threat = crate::board::hero_options_in(&copy, opponent, rules.board_mechanics)
                    .into_iter()
                    .map(|(p, q)| {
                        let mut reply = copy;
                        crate::board::hero_swap_in(
                            &mut reply,
                            p,
                            q,
                            opponent,
                            rules.board_mechanics,
                        );
                        score(
                            &reply,
                            opponent,
                            crate::content::health_cap(
                                opponent,
                                rules.opponent.is_some_and(|(_, a)| a),
                            ),
                            0,
                            Rules {
                                board_mechanics: rules.board_mechanics,
                                awake: rules.opponent.is_some_and(|(_, a)| a),
                                ..Rules::default()
                            },
                        )
                    })
                    .fold(0., f32::max);
                value -= threat * 0.35;
            }
            ((a, c), value)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked
}
#[cfg(test)]
pub fn choose_skill(
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    level: usize,
    armed: bool,
    bonus: i32,
) -> Option<Skill> {
    choose_skill_in(b, id, hp, mp, level, armed, bonus, Rules::default())
}
#[cfg(test)]
pub fn choose_skill_in(
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    level: usize,
    armed: bool,
    bonus: i32,
    rules: Rules,
) -> Option<Skill> {
    choose_skill_available_in(b, id, hp, mp, level, armed, bonus, rules, &[])
}

/// Exclude skills that cannot currently be used, before evaluating AI choices.
pub fn choose_skill_available_in(
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    level: usize,
    armed: bool,
    bonus: i32,
    rules: Rules,
    unavailable: &[Skill],
) -> Option<Skill> {
    if level == 0 {
        let affordable: Vec<_> = crate::content::skills(id, rules.awake)
            .iter()
            .copied()
            .filter(|s| !unavailable.contains(s))
            .filter(|s| {
                (0..N).any(|y| {
                    (0..N).any(|x| crate::content::legal_target(id, hp, *s, b, (x, y), rules))
                })
            })
            .filter(|s| {
                crate::content::payment_in(id, hp, mp, *s, rules).is_some()
                    && !(crate::content::skill_profile(*s, rules).healing_only
                        && hp == crate::content::health_cap(id, rules.awake))
            })
            .filter(|s| {
                !((crate::content::skill_profile(*s, rules).requires_unarmed && armed)
                    || (crate::content::skill_profile(*s, rules).requires_no_bonus && bonus > 0)
                    || (crate::content::skill_profile(*s, rules).healing_only
                        && hp == crate::content::health_cap(id, rules.awake)))
            })
            .collect();
        return if affordable.is_empty() || crate::rng::gen_range(0, 4) != 0 {
            None
        } else {
            Some(affordable[crate::rng::gen_range(0, affordable.len())])
        };
    }
    let swap_value = best_move_score_in(b, id, hp, mp, rules);
    let utility = |s| {
        let target = choose_target_in(s, b, id, hp, mp, bonus, rules);
        let mut copy = *b;
        if let Some(value) = crate::content::ability_ai_utility(
            s,
            &crate::content::EventContext::new(id, hp, mp, rules),
            b,
            bonus,
            swap_value,
        ) {
            return value;
        }
        let e = crate::content::effect_at_in(s, &mut copy, target, id, hp, mp, rules);
        let p = crate::content::preview_rewards_in(
            s,
            b,
            target,
            id,
            hp,
            mp,
            rules.opponent_hp.unwrap_or(10000),
            bonus,
            rules,
        );
        if e.bonus > 0 {
            return (swap_value - crate::content::skill_cost(id, rules.awake, s) as f32 * 0.5)
                as i32;
        }
        ((p.damage - p.opponent_heal) as f32
            + p.heal as f32 * 0.8
            + (p.mana_after - mp) as f32 * 0.5
            + e.cells.len() as f32 * 0.7) as i32
    };
    let skill = crate::content::skills(id, rules.awake)
        .iter()
        .copied()
        .filter(|s| !unavailable.contains(s))
        .filter(|s| {
            (0..N)
                .any(|y| (0..N).any(|x| crate::content::legal_target(id, hp, *s, b, (x, y), rules)))
        })
        .filter(|s| {
            crate::content::payment_in(id, hp, mp, *s, rules).is_some()
                && !(crate::content::skill_profile(*s, rules).healing_only
                    && hp == crate::content::health_cap(id, rules.awake))
        })
        .filter(|s| {
            !((crate::content::skill_profile(*s, rules).requires_unarmed && armed)
                || (crate::content::skill_profile(*s, rules).requires_no_bonus && bonus > 0))
        })
        .max_by_key(|&s| utility(s))?;
    let threshold = if crate::content::skill_profile(skill, rules).zero_ai_threshold {
        0
    } else {
        swap_value as i32
    };
    (utility(skill) > 0 && utility(skill) >= threshold).then_some(skill)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ai_only_chooses_legal_moves() {
        crate::rng::srand(17);
        for _ in 0..20 {
            let b = crate::fresh();
            for level in 0..3 {
                let (a, c) = choose_move(&b, 300, 0, 0, 0, level).unwrap();
                assert!(crate::board::hero_swap(&mut b.clone(), a, c, 0));
            }
        }
    }
    #[test]
    fn ai_cannot_spend_unavailable_mana() {
        assert!(choose_skill(&crate::fresh(), 0, 700, 0, 2, false, 0).is_none());
    }
}
pub fn choose_target_in(
    skill: Skill,
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    bonus: i32,
    rules: Rules,
) -> (usize, usize) {
    (0..N)
        .flat_map(|y| (0..N).map(move |x| (x, y)))
        .filter(|&target| crate::content::legal_target(id, hp, skill, b, target, rules))
        .max_by_key(|&target| {
            let p = crate::content::preview_rewards_in(
                skill,
                b,
                target,
                id,
                hp,
                mp,
                rules.opponent_hp.unwrap_or(10000),
                bonus,
                rules,
            );
            (p.damage - p.opponent_heal) * 100 + p.heal * 80 + (p.mana_after - mp) * 65
        })
        .unwrap_or((3, 3))
}

pub fn choose_force_move_in(
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    bonus: i32,
    rules: Rules,
) -> Option<Move> {
    let mut best: Option<(Move, f32)> = None;
    for y in 0..N {
        for x in 0..N {
            for (dx, dy) in [(1, 0), (0, 1)] {
                let (a, c) = ((x, y), (x + dx, y + dy));
                if c.0 >= N || c.1 >= N || !crate::content::legal_swap_region(id, hp, a, c, rules) {
                    continue;
                }
                let mut next = *b;
                let mut state = crate::role_state::RoleState {
                    force_armed: true,
                    statuses: rules.statuses,
                    ..Default::default()
                };
                if !state.complete_force_in(&mut next, a, c, id, rules.board_mechanics) {
                    continue;
                }
                let next_rules = Rules {
                    statuses: state.statuses,
                    ..rules
                };
                let value = best_move_score_in(&next, id, hp, mp, next_rules) + bonus as f32;
                if best.as_ref().is_none_or(|(_, v)| value > *v) {
                    best = Some(((a, c), value));
                }
            }
        }
    }
    best.map(|(m, _)| m)
}

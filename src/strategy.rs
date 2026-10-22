use crate::{
    content::{Skill, HEROES},
    matches, Board, N,
};
type Move = ((usize, usize), (usize, usize));
fn score(b: &Board, hero: usize, hp: i32, mp: i32) -> f32 {
    let (h, d, m) = crate::content::clear_rewards(hero, hp, mp, b, &matches(b), 1, 0);
    d as f32 + h.min(HEROES[hero].hp - hp) as f32 * 0.8 + m.min(HEROES[hero].mp - mp) as f32 * 0.65
}
pub fn choose_move(
    b: &Board,
    hp: i32,
    hero: usize,
    mp: i32,
    opponent: usize,
    level: usize,
) -> Option<Move> {
    let moves = crate::board::hero_options(b, hero);
    if moves.is_empty() {
        return None;
    }
    if level == 0 {
        return Some(moves[crate::rng::gen_range(0, moves.len())]);
    }
    moves
        .into_iter()
        .map(|(a, c)| {
            let mut copy = *b;
            crate::board::hero_swap(&mut copy, a, c, hero);
            let mut value = score(&copy, hero, hp, mp);
            if level == 2 {
                for (x, y) in matches(&copy) {
                    copy[y][x] = 255;
                }
                let threat = crate::board::hero_options(&copy, opponent)
                    .into_iter()
                    .map(|(p, q)| {
                        let mut reply = copy;
                        crate::board::hero_swap(&mut reply, p, q, opponent);
                        score(&reply, opponent, HEROES[opponent].hp, 0)
                    })
                    .fold(0., f32::max);
                value -= threat * 0.35;
            }
            ((a, c), value)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(m, _)| m)
}
pub fn choose_skill(
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    level: usize,
    armed: bool,
    bonus: i32,
) -> Option<Skill> {
    if level == 0 {
        let affordable: Vec<_> = HEROES[id]
            .skills
            .iter()
            .copied()
            .filter(|s| {
                crate::content::payment(id, hp, mp, *s).is_some()
                    && !(matches!(s, Skill::Guard | Skill::Blood) && hp == HEROES[id].hp)
            })
            .filter(|s| {
                !((*s == Skill::Landslide && armed)
                    || (*s == Skill::Strike && bonus > 0)
                    || (matches!(s, Skill::Blood | Skill::Guard) && hp == HEROES[id].hp))
            })
            .collect();
        return if affordable.is_empty() || crate::rng::gen_range(0, 4) != 0 {
            None
        } else {
            Some(affordable[crate::rng::gen_range(0, affordable.len())])
        };
    }
    let swap_value = choose_move(b, hp, id, mp, id, 1)
        .map(|(a, c)| {
            let mut copy = *b;
            crate::board::hero_swap(&mut copy, a, c, id);
            score(&copy, id, hp, mp)
        })
        .unwrap_or(0.);
    let utility = |s| {
        let target = choose_target(s, b, id, hp, mp, bonus);
        let mut copy = *b;
        let e = crate::content::effect_at(s, &mut copy, target);
        let p = crate::content::preview_rewards(s, b, target, id, hp, mp, 10000, bonus);
        if e.bonus > 0 {
            return (swap_value - s.cost() as f32 * 0.5) as i32;
        }
        (p.damage as f32
            + p.heal as f32 * 0.8
            + (p.mana_after - mp) as f32 * 0.5
            + e.cells.len() as f32 * 0.7) as i32
    };
    let skill = HEROES[id]
        .skills
        .iter()
        .copied()
        .filter(|s| {
            crate::content::payment(id, hp, mp, *s).is_some()
                && !(matches!(s, Skill::Guard | Skill::Blood) && hp == HEROES[id].hp)
        })
        .filter(|s| {
            !((matches!(s, Skill::Landslide) && armed) || (matches!(s, Skill::Strike) && bonus > 0))
        })
        .max_by_key(|&s| utility(s))?;
    let threshold = if skill == Skill::Strike {
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
pub fn choose_target(
    skill: Skill,
    b: &Board,
    id: usize,
    hp: i32,
    mp: i32,
    bonus: i32,
) -> (usize, usize) {
    (0..N)
        .flat_map(|y| (0..N).map(move |x| (x, y)))
        .max_by_key(|&target| {
            let p = crate::content::preview_rewards(skill, b, target, id, hp, mp, 10000, bonus);
            p.damage * 100 + p.heal * 80 + (p.mana_after - mp) * 50
        })
        .unwrap_or((3, 3))
}

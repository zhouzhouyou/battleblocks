use crate::{
    content::{Skill, HEROES, VALUES},
    matches, valid_swap, Board, N,
};
type Move = ((usize, usize), (usize, usize));
fn options(b: &Board) -> Vec<Move> {
    let mut out = Vec::new();
    for y in 0..N {
        for x in 0..N {
            for c in [(x + 1, y), (x, y + 1)] {
                if c.0 < N && c.1 < N && valid_swap(&mut b.clone(), (x, y), c) {
                    out.push(((x, y), c));
                }
            }
        }
    }
    out
}
fn score(b: &Board, hp: i32, max_hp: i32, mp: i32, max_mp: i32) -> f32 {
    let cells = matches(b);
    let multiplier = 1;
    let (mut h, mut d, mut m) = (0, 0, 0);
    for (x, y) in cells {
        if b[y][x] == 255 {
            continue;
        }
        let v = VALUES[b[y][x] as usize];
        h += v.0;
        d += v.1;
        m += v.2;
    }
    (d * multiplier) as f32
        + ((h * multiplier).min(max_hp - hp)) as f32 * 0.8
        + ((m * multiplier).min(max_mp - mp)) as f32 * 0.65
}
pub fn choose_move(
    b: &Board,
    hp: i32,
    max_hp: i32,
    mp: i32,
    max_mp: i32,
    level: usize,
) -> Option<Move> {
    let moves = options(b);
    if moves.is_empty() {
        return None;
    }
    if level == 0 {
        return Some(moves[crate::rng::gen_range(0, moves.len())]);
    }
    moves.into_iter().max_by(|&(a, c), &(d, e)| {
        let evaluate = |a, c| {
            let mut copy = *b;
            valid_swap(&mut copy, a, c);
            let mut value = score(&copy, hp, max_hp, mp, max_mp);
            if level == 2 {
                // Penalize opportunities in untouched cells; no knowledge of random refill.
                for (x, y) in matches(&copy) {
                    copy[y][x] = 255;
                }
                let threat = options(&copy)
                    .iter()
                    .map(|&(p, q)| {
                        let mut reply = copy;
                        valid_swap(&mut reply, p, q);
                        score(&reply, max_hp, max_hp, 0, max_mp)
                    })
                    .fold(0., f32::max);
                value -= threat * 0.35;
            }
            value
        };
        evaluate(a, c).total_cmp(&evaluate(d, e))
    })
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
                s.cost() <= mp && !(matches!(s, Skill::Guard | Skill::Blood) && hp == HEROES[id].hp)
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
    let swap_value = choose_move(b, hp, HEROES[id].hp, mp, HEROES[id].mp, 1)
        .map(|(a, c)| {
            let mut copy = *b;
            valid_swap(&mut copy, a, c);
            score(&copy, hp, HEROES[id].hp, mp, HEROES[id].mp)
        })
        .unwrap_or(0.);
    let utility = |s| {
        let target = choose_target(s, b, hp, HEROES[id].hp);
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
            s.cost() <= mp && !(matches!(s, Skill::Guard | Skill::Blood) && hp == HEROES[id].hp)
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
                let (a, c) = choose_move(&b, 300, 700, 0, 300, level).unwrap();
                assert!(valid_swap(&mut b.clone(), a, c));
            }
        }
    }
    #[test]
    fn ai_cannot_spend_unavailable_mana() {
        assert!(choose_skill(&crate::fresh(), 0, 700, 0, 2, false, 0).is_none());
    }
}
pub fn choose_target(skill: Skill, b: &Board, hp: i32, max_hp: i32) -> (usize, usize) {
    (0..N)
        .flat_map(|y| (0..N).map(move |x| (x, y)))
        .max_by_key(|&target| {
            let mut copy = *b;
            let e = crate::content::effect_at(skill, &mut copy, target);
            let rewards = e
                .cells
                .iter()
                .map(|&(x, y)| {
                    let v = VALUES[b[y][x] as usize];
                    v.1 + v.2 / 2 + if hp < max_hp { v.0 } else { 0 }
                })
                .sum::<i32>();
            e.damage
                + e.hp.min(max_hp - hp)
                + e.mp / 2
                + if e.reward { rewards } else { 0 }
                + matches(&copy).len() as i32 * 10
        })
        .unwrap_or((3, 3))
}

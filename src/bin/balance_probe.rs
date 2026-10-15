#![allow(dead_code)]
#[path = "../board.rs"]
mod board;
#[path = "../content.rs"]
mod content;
#[path = "../rng.rs"]
mod rng;
#[path = "../strategy.rs"]
mod strategy;
#[cfg(test)]
use board::fresh;
use board::{matches, valid_swap, Board, N};
use content::{HEROES, VALUES};
fn game(ids: [usize; 2], seed: u64) -> (Option<usize>, usize, i32) {
    rng::srand(seed);
    let mut b = board::fresh();
    let mut hp = [HEROES[ids[0]].hp, HEROES[ids[1]].hp];
    let mut mp = [0, 0];
    let mut buffs = [0, 0];
    let mut armed = [false, false];
    let mut turn = 0;
    let mut burst = 0;
    for action in 0..240 {
        let other = 1 - turn;
        let before = hp[other];
        let mut cells;
        let mut reward;
        if let Some(skill) = strategy::choose_skill(
            &b,
            ids[turn],
            hp[turn],
            mp[turn],
            1,
            armed[turn],
            buffs[turn],
        ) {
            mp[turn] -= skill.cost();
            let target = strategy::choose_target(skill, &b, hp[turn], HEROES[ids[turn]].hp);
            let e = content::effect_at(skill, &mut b, target);
            hp[turn] = (hp[turn] + e.hp).min(HEROES[ids[turn]].hp);
            hp[other] = (hp[other] - e.damage).max(0);
            mp[turn] = (mp[turn] + e.mp).min(HEROES[ids[turn]].mp);
            buffs[turn] += e.bonus;
            armed[turn] |= e.armed;
            cells = e.cells;
            reward = e.reward;
            if cells.is_empty() {
                cells = matches(&b);
                reward = true;
            }
            if e.armed || e.bonus > 0 {
                continue;
            }
        } else if let Some((a, c)) = strategy::choose_move(
            &b,
            hp[turn],
            HEROES[ids[turn]].hp,
            mp[turn],
            HEROES[ids[turn]].mp,
            1,
        ) {
            if armed[turn] {
                board::parallel_swap(&mut b, a, c);
                armed[turn] = false;
            } else {
                valid_swap(&mut b, a, c);
            }
            cells = matches(&b);
            reward = true;
        } else {
            b = board::fresh();
            continue;
        }
        let mut chain = 0;
        while !cells.is_empty() && chain < 64 {
            let multiplier = content::reward_scale(chain + 1, buffs[turn]);
            let (mut h, mut d, mut m) = (0, 0, 0);
            for &(x, y) in &cells {
                if reward {
                    let v = VALUES[b[y][x] as usize];
                    h += v.0;
                    d += v.1;
                    m += v.2;
                }
                b[y][x] = 255;
            }
            hp[turn] = (hp[turn] + h * multiplier / 100).min(HEROES[ids[turn]].hp);
            hp[other] = (hp[other] - d * multiplier / 100).max(0);
            mp[turn] = (mp[turn] + m * multiplier / 100).min(HEROES[ids[turn]].mp);
            if reward {
                buffs[turn] = 0;
            }
            board::drop_board(&mut b);
            cells = matches(&b);
            reward = true;
            chain += 1;
        }
        burst = burst.max(before - hp[other]);
        if hp[other] == 0 {
            return (Some(turn), action + 1, burst);
        }
        if !board::playable(&b) {
            b = board::fresh();
        }
        turn = other;
    }
    (None, 240, burst)
}
fn main() {
    let samples: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);
    let seed_start: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10000);
    let mut wins = [0; 7];
    let mut games = [0; 7];
    let mut draws = 0;
    let mut total = 0;
    let mut lengths = Vec::new();
    let mut bursts = Vec::new();
    let mut pairs = [[0; 7]; 7];
    let mut starts = 0;
    for a in 0..7 {
        for b in a + 1..7 {
            for seed in 0..samples {
                for ids in [[a, b], [b, a]] {
                    let (winner, length, burst) = game(ids, seed_start + seed as u64);
                    games[a] += 1;
                    games[b] += 1;
                    total += 1;
                    lengths.push(length);
                    bursts.push(burst);
                    if let Some(side) = winner {
                        wins[ids[side]] += 1;
                        pairs[ids[side]][ids[1 - side]] += 1;
                        if side == 0 {
                            starts += 1;
                        }
                    } else {
                        draws += 1;
                    }
                }
            }
        }
    }
    println!("hero,wins,games,win_percent");
    for i in 0..7 {
        println!(
            "{},{},{},{:.1}",
            HEROES[i].en,
            wins[i],
            games[i],
            wins[i] as f64 / games[i] as f64 * 100.
        );
    }
    lengths.sort();
    bursts.sort();
    println!(
        "games={total},draws={draws},median_actions={},p90_burst={},first_side_win={:.1}%",
        lengths[lengths.len() / 2],
        bursts[bursts.len() * 9 / 10],
        starts as f64 / total as f64 * 100.
    );
    println!("pairwise_win_counts={pairs:?}");
}

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
use board::{matches, Board, N};
use content::HEROES;
#[derive(Clone, Copy, Default, Debug)]
struct Metrics {
    swaps: u64,
    skills: u64,
    blood_casts: u64,
    blood_paid: i64,
    damage: i64,
    healing: i64,
    mana: i64,
    mana_waste: i64,
}
fn game(ids: [usize; 2], seed: u64, metrics: &mut [Metrics; 7]) -> (Option<usize>, usize, i32) {
    rng::srand(seed);
    let mut b = board::fresh();
    let mut hp = [HEROES[ids[0]].hp, HEROES[ids[1]].hp];
    let mut mp = [0, 0];
    let mut buffs = [0, 0];
    let mut armed = [false, false];
    let mut awakened = [false; 2];
    macro_rules! rules {
        ($side:expr) => {{
            let side = $side;
            content::Rules {
                awake: awakened[side],
                opponent: Some((ids[1 - side], awakened[1 - side])),
                opponent_hp: Some(hp[1 - side]),
                ..content::Rules::default()
            }
        }};
    }
    let mut turn = 0;
    let mut burst = 0;
    for action in 0..240 {
        let other = 1 - turn;
        let before = hp[other];
        let mut cells;
        let mut reward;
        let mut loot = false;
        if let Some(skill) = strategy::choose_skill_in(
            &b,
            ids[turn],
            hp[turn],
            mp[turn],
            probe_level(),
            armed[turn],
            buffs[turn],
            rules!(turn),
        ) {
            let target = strategy::choose_target_in(
                skill,
                &b,
                ids[turn],
                hp[turn],
                mp[turn],
                buffs[turn],
                rules!(turn),
            );
            metrics[ids[turn]].skills += 1;
            let pay =
                content::payment_in(ids[turn], hp[turn], mp[turn], skill, rules!(turn)).unwrap();
            let cost_hp = hp[turn];
            let cost_mp = mp[turn];
            mp[turn] -= pay.mana;
            content::take_damage(ids[turn], &mut hp[turn], &mut awakened[turn], pay.blood);
            if pay.blood > 0 {
                metrics[ids[turn]].blood_casts += 1;
                metrics[ids[turn]].blood_paid += pay.blood as i64;
            }
            let e = content::effect_at_in(
                skill,
                &mut b,
                target,
                ids[turn],
                cost_hp,
                cost_mp,
                rules!(turn),
            );
            record(
                &mut metrics[ids[turn]],
                hp[turn],
                mp[turn],
                hp[other],
                ids[turn],
                e.hp,
                e.damage,
                e.mp,
                awakened[turn],
            );
            hp[turn] = (hp[turn] + e.hp).min(content::health_cap(ids[turn], awakened[turn]));
            content::take_damage(ids[other], &mut hp[other], &mut awakened[other], e.damage);
            mp[turn] = (mp[turn] + e.mp).min(content::mana_cap(ids[turn], awakened[turn]));
            buffs[turn] += e.bonus;
            armed[turn] |= e.armed;
            loot = skill == content::Skill::Greedy;
            cells = e.cells;
            reward = e.reward;
            if cells.is_empty() {
                cells = matches(&b);
                reward = true;
            }
            if e.armed || e.bonus > 0 {
                continue;
            }
        } else if let Some((a, c)) = strategy::choose_move_in(
            &b,
            hp[turn],
            ids[turn],
            mp[turn],
            ids[other],
            probe_level(),
            rules!(turn),
        )
        .or_else(|| armed[turn].then_some(((0, 0), (1, 0))))
        {
            metrics[ids[turn]].swaps += 1;
            if armed[turn] {
                board::parallel_swap(&mut b, a, c);
                armed[turn] = false;
            } else {
                board::hero_swap(&mut b, a, c, ids[turn]);
            }
            cells = matches(&b);
            reward = true;
        } else {
            b = board::fresh_for(ids[turn]);
            continue;
        }
        let mut chain = 0;
        while !cells.is_empty() && chain < 64 && hp[turn] > 0 && hp[other] > 0 {
            let (h, d, m) = if reward {
                content::clear_rewards_in(
                    ids[turn],
                    hp[turn],
                    mp[turn],
                    &b,
                    &cells,
                    chain + 1,
                    buffs[turn],
                    content::Rules {
                        initial_loot: loot && chain == 0,
                        ..rules!(turn)
                    },
                )
            } else {
                (0, 0, 0)
            };
            for &(x, y) in &cells {
                b[y][x] = 255;
            }
            record(
                &mut metrics[ids[turn]],
                hp[turn],
                mp[turn],
                hp[other],
                ids[turn],
                h,
                d,
                m,
                awakened[turn],
            );
            hp[turn] = (hp[turn] + h).min(content::health_cap(ids[turn], awakened[turn]));
            content::take_damage(ids[other], &mut hp[other], &mut awakened[other], d);
            mp[turn] = (mp[turn] + m).min(content::mana_cap(ids[turn], awakened[turn]));
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
        if content::must_refresh_in(
            &b,
            ids[other],
            hp[other],
            mp[other],
            armed[other],
            rules!(other),
        ) {
            b = board::fresh_for(ids[other]);
        }
        turn = other;
    }
    (None, 240, burst)
}
fn record(
    out: &mut Metrics,
    hp: i32,
    mp: i32,
    enemy: i32,
    id: usize,
    h: i32,
    d: i32,
    m: i32,
    awake: bool,
) {
    out.healing += h.min(content::health_cap(id, awake) - hp) as i64;
    out.damage += d.min(enemy) as i64;
    out.mana += m.min(content::mana_cap(id, awake) - mp) as i64;
    out.mana_waste += (mp + m - content::mana_cap(id, awake)).max(0) as i64;
}
fn probe_level() -> usize {
    std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .min(2)
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
    assert!(samples > 0, "samples must be positive");
    let mut metrics = [Metrics::default(); 7];
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
                    let (winner, length, burst) = game(ids, seed_start + seed as u64, &mut metrics);
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
        "games={total},draws={draws},median_actions={},p90_burst={},first_side_win={:.1}%,p10_actions={},p90_actions={},max_actions={}",
        lengths[lengths.len() / 2],
        bursts[bursts.len() * 9 / 10],
        starts as f64 / total as f64 * 100.,
        lengths[lengths.len()/10],
        lengths[lengths.len()*9/10],
        lengths[lengths.len()-1]
    );
    println!("pairwise_win_counts={pairs:?}");
    println!("hero,swaps,skills,blood_casts,blood_paid,actual_damage,actual_healing,actual_mana,wasted_mana");
    for (id, m) in metrics.iter().enumerate() {
        println!(
            "{},{},{},{},{},{},{},{},{}",
            HEROES[id].en,
            m.swaps,
            m.skills,
            m.blood_casts,
            m.blood_paid,
            m.damage,
            m.healing,
            m.mana,
            m.mana_waste
        );
    }
}

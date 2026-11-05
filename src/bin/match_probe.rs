//! Full-state AI-vs-AI screening for the playable roster.
//! This uses the same board, content, role-state, preview and AI modules as the game.
#![allow(dead_code)]
#[path = "../board.rs"]
mod board;
#[path = "../content.rs"]
mod content;
#[path = "../rng.rs"]
mod rng;
#[path = "../role_state.rs"]
mod role_state;
#[path = "../strategy.rs"]
mod strategy;

#[cfg(test)]
use board::fresh;
use board::{matches, Board, N};
use content::{RewardBatch, Rules, Skill, HEROES};
use role_state::RoleState;

fn tr_key(zh: bool, key: &'static str) -> &'static str {
    static TEXT: std::sync::LazyLock<std::collections::HashMap<&'static str, [&'static str; 2]>> =
        std::sync::LazyLock::new(|| {
            include_str!("../../config/localization.tsv")
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| {
                    let mut fields = line.splitn(3, '\t');
                    (
                        fields.next().unwrap(),
                        [fields.next().unwrap(), fields.next().unwrap()],
                    )
                })
                .collect()
        });
    TEXT[key][zh as usize]
}

const MAX_ACTIONS: usize = 240;

#[derive(Clone)]
struct Side {
    id: usize,
    hp: i32,
    mp: i32,
    awake: bool,
    cooldowns: content::Cooldowns,
    role: RoleState,
    bonus: i32,
    armed: bool,
}

#[derive(Default, Clone, Copy)]
struct Metrics {
    swaps: u64,
    skills: u64,
}

fn rules(sides: &[Side; 2], side: usize, round: u32) -> Rules {
    let me = &sides[side];
    let other = &sides[1 - side];
    Rules {
        awake: me.awake,
        opponent_hp: Some(other.hp),
        opponent_mp: other.mp,
        opponent_statuses: other.role.statuses,
        opponent: Some((other.id, other.awake)),
        horizontal: me.role.horizontal,
        swap_clear: me.role.swap_clear,
        pockets: me.role.pockets,
        pocket_slot: me.role.pocket_slot,
        copied_skill: me.role.copied_skill.and_then(Skill::from_index),
        statuses: me.role.statuses,
        repeat_casts: me.role.repeat_casts,
        round,
        ..Rules::default()
    }
}

fn refresh_copy(sides: &mut [Side; 2], side: usize, round: u32) {
    let other = 1 - side;
    let actor_rules = rules(sides, side, round);
    let other_rules = rules(sides, other, round);
    let options: Vec<_> = content::skills(sides[other].id, sides[other].awake)
        .iter()
        .copied()
        .map(|s| content::effective_skill(s, other_rules))
        .filter(|s| !content::skill_profile(*s, other_rules).copy_slot)
        .collect();
    let mut commands = Vec::new();
    content::dispatch_rules(
        &content::EventContext::new(sides[side].id, sides[side].hp, sides[side].mp, actor_rules),
        &mut content::RuleEvent::RefreshAbilities {
            options: &options,
            commands: &mut commands,
        },
        &mut Vec::new(),
    );
    for command in commands {
        if let content::StateCommand::CopySkill { options } = command {
            sides[side].role.copied_skill = if options.is_empty() {
                None
            } else {
                Some(options[rng::gen_range(0, options.len())].index())
            };
        }
    }
}

fn begin_turn(sides: &mut [Side; 2], side: usize, round: u32) {
    sides[side].cooldowns.start_turn(side);
    sides[side].role.start_turn();
    refresh_copy(sides, side, round);
}

fn apply_batch(sides: &mut [Side; 2], side: usize, batch: RewardBatch, round: u32) -> i32 {
    let rules = rules(sides, side, round);
    let id = sides[side].id;
    let (hp, mp) = (sides[side].hp, sides[side].mp);
    let opponent_hp = sides[1 - side].hp;
    let settled = content::settle_batch_in(id, hp, mp, opponent_hp, batch, rules);
    sides[side].hp = settled.actor_hp;
    sides[side].mp = settled.actor_mp;
    sides[1 - side].hp = settled.opponent_hp;
    let mut dealt = 0;
    if settled.damage > 0 {
        let other = 1 - side;
        let other_id = sides[other].id;
        let hit = content::take_damage(
            other_id,
            &mut sides[other].hp,
            &mut sides[other].awake,
            settled.damage,
        );
        dealt = hit.damage;
        if hit.awakened {
            refresh_copy(sides, other, round);
        }
    }
    dealt
}

fn clear_cells(
    board: &mut Board,
    sides: &mut [Side; 2],
    side: usize,
    cells: &mut Vec<(usize, usize)>,
    reward: bool,
    round: u32,
    _metrics: &mut [Metrics],
) -> i32 {
    let id = sides[side].id;
    let mut chain = 0usize;
    let mut dealt = 0;
    let mut rewarded = reward;
    while !cells.is_empty() && chain < 64 && sides[0].hp > 0 && sides[1].hp > 0 {
        let rules = rules(sides, side, round);
        let batch = if rewarded {
            content::clear_batch_in(
                id,
                sides[side].hp,
                sides[side].mp,
                board,
                cells,
                chain + 1,
                sides[side].bonus,
                rules,
            )
        } else {
            RewardBatch::default()
        };
        if rewarded && chain == 0 {
            sides[side].role.direct_tiles.extend(
                cells
                    .iter()
                    .filter_map(|&(x, y)| (board[y][x] < 7).then_some(board[y][x])),
            );
        }
        let mut batch = batch;
        batch.triggers.clear();
        dealt += apply_batch(sides, side, batch, round);
        if rewarded {
            sides[side].bonus = 0;
            sides[side]
                .role
                .statuses
                .consume(content::StatusDuration::UntilClear);
        }
        for &(x, y) in cells.iter() {
            board[y][x] = 255;
        }
        board::drop_board(board);
        *cells = matches(board);
        rewarded = true;
        chain += 1;
    }
    dealt
}

fn cast_skill(
    board: &mut Board,
    sides: &mut [Side; 2],
    side: usize,
    skill: Skill,
    round: u32,
    metrics: &mut [Metrics],
) -> bool {
    let id = sides[side].id;
    let cast_rules = rules(sides, side, round);
    let effective = content::effective_skill(skill, cast_rules);
    let profile = content::skill_profile(effective, cast_rules);
    let target = if profile.random_target {
        content::random_reachable_target(id, sides[side].hp, cast_rules)
    } else {
        strategy::choose_target_in(
            skill,
            board,
            id,
            sides[side].hp,
            sides[side].mp,
            sides[side].bonus,
            cast_rules,
        )
    };
    let Some(pay) = content::payment_in(id, sides[side].hp, sides[side].mp, skill, cast_rules)
    else {
        return false;
    };
    if !content::legal_target(id, sides[side].hp, skill, board, target, cast_rules) {
        return false;
    }
    let (actor_hp, actor_mp) = (sides[side].hp, sides[side].mp);
    sides[side].mp -= pay.mana;
    if pay.blood > 0 {
        let hit = content::take_damage(id, &mut sides[side].hp, &mut sides[side].awake, pay.blood);
        if hit.awakened {
            refresh_copy(sides, side, round);
        }
    }
    sides[side].cooldowns.cast(side, skill);
    sides[side].role.record_cast(skill);
    metrics[id].skills += 1;
    let before_board = *board;
    let e = content::effect_at_in(skill, board, target, id, actor_hp, actor_mp, cast_rules);
    sides[side].role.on_cast(effective, &before_board, target);
    let mut batch = e.reward_batch(effective);
    if profile.delayed_damage {
        let mut delayed = RewardBatch::default();
        let mut immediate = Vec::new();
        for event in std::mem::take(&mut batch.events) {
            if event.resource == content::Resource::Damage
                && event.recipient == content::EventSide::Opponent
            {
                delayed.events.push(event);
            } else {
                immediate.push(event);
            }
        }
        delayed.triggers = std::mem::take(&mut batch.triggers);
        batch.events = immediate;
        apply_batch(sides, side, batch, round);
        apply_batch(sides, side, delayed, round);
    } else {
        apply_batch(sides, side, batch, round);
    }
    sides[side].bonus += e.bonus;
    sides[side].armed |= e.armed;
    let mut cells = e.cells;
    let mut reward = e.reward;
    if cells.is_empty() {
        cells = matches(board);
        reward = true;
    }
    clear_cells(board, sides, side, &mut cells, reward, round, metrics);
    effective.ends_turn()
}

fn make_move(
    board: &mut Board,
    sides: &mut [Side; 2],
    side: usize,
    a: (usize, usize),
    c: (usize, usize),
    round: u32,
    metrics: &mut [Metrics],
) {
    let id = sides[side].id;
    if sides[side].armed {
        board::parallel_swap(board, a, c);
        sides[side].armed = false;
        sides[side]
            .role
            .statuses
            .consume(content::StatusDuration::UntilSwap);
    } else {
        board::hero_swap(board, a, c, id);
    }
    metrics[id].swaps += 1;
    sides[side].role.record_swap(a, c);
    let mut cells = matches(board);
    clear_cells(board, sides, side, &mut cells, true, round, metrics);
}

fn end_turn(
    board: &mut Board,
    sides: &mut [Side; 2],
    side: usize,
    round: u32,
    metrics: &mut [Metrics],
) -> bool {
    let id = sides[side].id;
    let turn_rules = rules(sides, side, round);
    let ending = content::turn_end_rewards(id, sides[side].hp, sides[side].mp, turn_rules);
    apply_batch(sides, side, ending, round);
    if sides[0].hp == 0 || sides[1].hp == 0 {
        return false;
    }
    let cleared = sides[side].role.direct_tiles.clone();
    let receiver = 1 - side;
    let receiver_rules = rules(sides, receiver, round);
    let mut commands = Vec::new();
    content::dispatch_rules(
        &content::EventContext::new(
            sides[receiver].id,
            sides[receiver].hp,
            sides[receiver].mp,
            receiver_rules,
        ),
        &mut content::RuleEvent::OpponentTurnEnd {
            cleared: &cleared,
            state: &sides[receiver].role,
            commands: &mut commands,
        },
        &mut Vec::new(),
    );
    for command in commands {
        if let content::StateCommand::FillPocket { slot, options } = command {
            if !options.is_empty() {
                sides[receiver].role.pockets[slot] =
                    Some(options[rng::gen_range(0, options.len())]);
                sides[receiver].role.select_occupied_pocket();
            }
        }
    }
    sides[side].role.end_turn();
    let next_round = if side == 1 { round + 1 } else { round };
    begin_turn(sides, receiver, next_round);
    let _ = board;
    let _ = metrics;
    true
}

fn game(
    ids: [usize; 2],
    seed: u64,
    level: usize,
    metrics: &mut [Metrics],
) -> (Option<usize>, usize, u32) {
    rng::srand(seed);
    let mut board = board::fresh();
    let mut sides = [
        Side {
            id: ids[0],
            hp: HEROES[ids[0]].hp,
            mp: 0,
            awake: false,
            cooldowns: content::Cooldowns::default(),
            role: RoleState::default(),
            bonus: 0,
            armed: false,
        },
        Side {
            id: ids[1],
            hp: HEROES[ids[1]].hp,
            mp: 0,
            awake: false,
            cooldowns: content::Cooldowns::default(),
            role: RoleState::default(),
            bonus: 0,
            armed: false,
        },
    ];
    let mut side = 0usize;
    let mut round = 1u32;
    begin_turn(&mut sides, side, round);
    for action in 0..MAX_ACTIONS {
        if sides[0].hp == 0 {
            return (Some(1), action, round);
        }
        if sides[1].hp == 0 {
            return (Some(0), action, round);
        }
        let id = sides[side].id;
        if sides[side].role.force_armed {
            let rules = rules(&sides, side, round);
            if let Some((a, c)) = strategy::choose_force_move_in(
                &board,
                id,
                sides[side].hp,
                sides[side].mp,
                sides[side].bonus,
                rules,
            ) {
                if sides[side].role.complete_force(&mut board, a, c, id) {
                    metrics[id].swaps += 1;
                    continue;
                }
            }
            sides[side].role.force_armed = false;
        }
        let rules = rules(&sides, side, round);
        let unavailable: Vec<_> = content::skills(id, sides[side].awake)
            .iter()
            .copied()
            .filter(|skill| sides[side].cooldowns.remaining(side, *skill) > 0)
            .collect();
        let cast = strategy::choose_skill_available_in(
            &board,
            id,
            sides[side].hp,
            sides[side].mp,
            level,
            sides[side].armed,
            sides[side].bonus,
            rules,
            &unavailable,
        );
        let ended;
        if let Some(skill) = cast {
            ended = cast_skill(&mut board, &mut sides, side, skill, round, metrics);
        } else if let Some((a, c)) = strategy::choose_move_in(
            &board,
            sides[side].hp,
            id,
            sides[side].mp,
            sides[1 - side].id,
            level,
            rules,
        ) {
            make_move(&mut board, &mut sides, side, a, c, round, metrics);
            ended = true;
        } else {
            board = board::fresh_for(id);
            continue;
        }
        if sides[0].hp == 0 {
            return (Some(1), action + 1, round);
        }
        if sides[1].hp == 0 {
            return (Some(0), action + 1, round);
        }
        if ended {
            if !end_turn(&mut board, &mut sides, side, round, metrics) {
                if sides[0].hp == 0 {
                    return (Some(1), action + 1, round);
                }
                return (Some(0), action + 1, round);
            }
            if side == 1 {
                round += 1;
            }
            side = 1 - side;
        }
    }
    (None, MAX_ACTIONS, round)
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
    let level: usize = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .min(2);
    let roster: Vec<usize> = std::env::args()
        .nth(4)
        .map(|text| {
            text.split(',')
                .filter_map(|s| s.parse::<usize>().ok())
                .collect()
        })
        .unwrap_or_else(|| (0..7).collect());
    if roster.contains(&content::FIRE_ELF) {
        eprintln!(
            "match_probe does not model Sleeping turns; use `--headless --autoplay` for the fire elf"
        );
        std::process::exit(2);
    }
    assert!(samples > 0 && roster.len() >= 2);
    let count = roster.len();
    let mut wins = vec![0u64; count];
    let mut games = vec![0u64; count];
    let mut pairs = vec![vec![0u64; count]; count];
    let mut metrics = vec![Metrics::default(); HEROES.len()];
    let mut draws = 0;
    let mut lengths = Vec::new();
    let mut rounds = Vec::new();
    let mut starts = 0;
    for a in 0..count {
        for b in (a + 1)..count {
            for seed in 0..samples {
                for order in [[a, b], [b, a]] {
                    let ids = [roster[order[0]], roster[order[1]]];
                    let (winner, length, round) =
                        game(ids, seed_start + seed as u64, level, &mut metrics);
                    games[order[0]] += 1;
                    games[order[1]] += 1;
                    lengths.push(length);
                    rounds.push(round);
                    if let Some(side) = winner {
                        wins[order[side]] += 1;
                        pairs[order[side]][order[1 - side]] += 1;
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
    for i in 0..count {
        println!(
            "{},{},{},{:.1}",
            HEROES[roster[i]].en,
            wins[i],
            games[i],
            wins[i] as f64 / games[i].max(1) as f64 * 100.
        );
    }
    lengths.sort();
    rounds.sort();
    let total: u64 = games.iter().sum::<u64>() / 2;
    println!(
        "games={total},draws={draws},median_actions={},p10_actions={},p90_actions={},max_actions={},median_rounds={},p90_rounds={},max_rounds={},first_side_win={:.1}%",
        lengths[lengths.len() / 2],
        lengths[lengths.len() / 10],
        lengths[lengths.len() * 9 / 10],
        lengths[lengths.len() - 1],
        rounds[rounds.len() / 2],
        rounds[rounds.len() * 9 / 10],
        rounds[rounds.len() - 1],
        starts as f64 / (total - draws).max(1) as f64 * 100.
    );
    println!("pairwise_win_counts={pairs:?}");
    println!("hero,swaps,skills");
    for (i, id) in roster.iter().enumerate() {
        println!(
            "{},{},{}",
            HEROES[*id].en, metrics[*id].swaps, metrics[*id].skills
        );
        let _ = i;
    }
}

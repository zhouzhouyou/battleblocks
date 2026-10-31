use super::*;

/// First clear 100%; later cascades +10% per wave, capped at 130%.
/// Each preparation bonus adds 100%.
pub fn reward_scale(chain: usize, bonus: i32) -> i32 {
    100 + chain.saturating_sub(1).min(3) as i32 * 10 + bonus.max(0) * 100
}

/// Stateless passives: same calculation for game, previews and AI.
#[cfg(test)]
pub fn clear_rewards(
    hero: usize,
    hp: i32,
    mp: i32,
    board: &Board,
    cells: &[(usize, usize)],
    chain: usize,
    bonus: i32,
) -> (i32, i32, i32) {
    clear_rewards_in(hero, hp, mp, board, cells, chain, bonus, Rules::default())
}
pub fn clear_rewards_in(
    hero: usize,
    hp: i32,
    mp: i32,
    board: &Board,
    cells: &[(usize, usize)],
    chain: usize,
    bonus: i32,
    rules: Rules,
) -> (i32, i32, i32) {
    let (mut h, mut d, mut m) = (0, 0, 0);
    for &(x, y) in cells {
        if board[y][x] == 255 {
            continue;
        }
        let (a, b, c) = VALUES[board[y][x] as usize];
        h += a;
        d += b;
        if b > 0 {
            d += if hero == KNIGHT {
                3
            } else if hero == DWARF {
                2
            } else {
                0
            };
        }
        m += c;
        if board[y][x] == 0 {
            if has_passive(hero, rules.awake, Passive::BalancedAffinity) {
                d += 4;
            }
            if has_passive(hero, rules.awake, Passive::BalancedResonance) {
                d += 6;
            }
            if has_passive(hero, rules.awake, Passive::ElementReturn) {
                m += 2;
            }
        }
    }
    let scale = reward_scale(chain, bonus) - if rules.miracle { 20 } else { 0 }
        + if hero == KNIGHT && rules.awake && rules.horizontal && chain == 1 {
            10
        } else {
            0
        };
    h = h * scale / 100;
    d = d
        * (scale
            + if has_passive(hero, rules.awake, Passive::ElvenPower) {
                25
            } else {
                0
            })
        / 100;
    m = m * scale / 100;
    if has_passive(hero, rules.awake, Passive::BattleTraining) {
        h = h * 106 / 100;
        d = d * 106 / 100;
        m = m * 106 / 100;
    }
    if has_passive(hero, rules.awake, Passive::ForestPact) {
        h = h * 93 / 100;
        d = d * 93 / 100;
        m = m * 93 / 100;
    }
    if has_passive(hero, rules.awake, Passive::ArcaneConversion) && mp < mana_cap(hero, rules.awake)
    {
        let converted = h / 2;
        h -= converted;
        m += converted;
    }
    if has_passive(hero, rules.awake, Passive::Recycling) {
        m += (hp + h - health_cap(hero, rules.awake)).max(0) / 2;
    }
    if (has_passive(hero, rules.awake, Passive::BloodFury)
        || has_passive(hero, rules.awake, Passive::BoilingBlood))
        && hp * 100
            <= HEROES[hero].hp
                * if has_passive(hero, rules.awake, Passive::BoilingBlood) {
                    50
                } else {
                    35
                }
    {
        d = d * 125 / 100;
    }
    if has_passive(hero, rules.awake, Passive::LootMastery) && rules.initial_loot {
        d = d * 130 / 100;
    }
    if rules.upper_swap && rules.opponent.is_some_and(|(id, _)| id == DWARF) {
        d = 0;
    }
    (h, d, m)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Payment {
    pub mana: i32,
    pub blood: i32,
}
#[cfg(test)]
pub fn payment(hero: usize, hp: i32, mp: i32, skill: Skill) -> Option<Payment> {
    payment_in(hero, hp, mp, skill, Rules::default())
}
pub fn blood_cost(hero: usize, awake: bool, missing: i32) -> i32 {
    let missing = missing.max(0);
    if has_passive(hero, awake, Passive::FleshConversion) {
        (missing + 1) / 2
    } else {
        missing * 2
    }
}
pub fn payment_in(hero: usize, hp: i32, mp: i32, skill: Skill, rules: Rules) -> Option<Payment> {
    let missing = (skill_cost_in(hero, skill, rules) - mp).max(0);
    let blood = blood_cost(hero, rules.awake, missing);
    if missing == 0 || (hero == 6 && hp > blood) {
        Some(Payment {
            mana: skill_cost_in(hero, skill, rules).min(mp),
            blood,
        })
    } else {
        None
    }
}

/// Apply one source event once, before either side's health cap. Converted
/// results are terminal values: they cannot trigger another conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardSettlement {
    pub actor_hp: i32,
    pub actor_mp: i32,
    pub opponent_hp: i32,
    pub damage: i32,
}
pub fn settle_rewards_in(
    hero: usize,
    hp: i32,
    mp: i32,
    opponent_hp: i32,
    gains: (i32, i32, i32),
    direct_opponent_heal: i32,
    rules: Rules,
) -> RewardSettlement {
    let (mut heal, mut damage, mana) = gains;
    let mut opponent_heal = direct_opponent_heal;
    if hero == NUN && !rules.awake {
        opponent_heal += heal;
    }
    if rules.turn_mode == TurnMode::Rational {
        heal += damage;
        opponent_heal += damage;
        damage = 0;
    } else if hero == NUN && rules.awake {
        damage += heal + opponent_heal;
        heal = 0;
        opponent_heal = 0;
    } else if rules.turn_mode == TurnMode::Frenzy {
        damage += opponent_heal;
        opponent_heal = 0;
    }
    let opponent_cap = rules
        .opponent
        .map(|(id, awake)| health_cap(id, awake))
        .unwrap_or(i32::MAX);
    RewardSettlement {
        actor_hp: (hp + heal).min(health_cap(hero, rules.awake)),
        actor_mp: (mp + mana).min(mana_cap(hero, rules.awake)),
        opponent_hp: (opponent_hp + opponent_heal).min(opponent_cap),
        damage,
    }
}

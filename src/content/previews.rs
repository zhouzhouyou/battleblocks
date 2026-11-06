use super::*;

/// Exact rewards before the first random refill, including conversion matches, cost and caps.
#[derive(Clone, Copy)]
pub struct SkillPreview {
    pub opponent_heal: i32,
    pub enemy_health_after: i32,
    pub awakens_enemy: bool,
    pub awakens_self: bool,
    pub heal: i32,
    pub damage: i32,
    pub mana_gain: i32,
    pub mana_after: i32,
    pub blood_cost: i32,
    pub health_after: i32,
}
#[cfg(test)]
pub fn preview_rewards(
    skill: Skill,
    board: &Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
) -> SkillPreview {
    preview_rewards_in(
        skill,
        board,
        target,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        Rules::default(),
    )
}
pub fn preview_rewards_in(
    skill: Skill,
    board: &Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    rules: Rules,
) -> SkillPreview {
    if !legal_target(hero, hp, skill, board, target, rules) {
        return SkillPreview {
            opponent_heal: 0,
            enemy_health_after: enemy_hp,
            awakens_enemy: false,
            awakens_self: false,
            heal: 0,
            damage: 0,
            mana_gain: 0,
            mana_after: mp,
            blood_cost: 0,
            health_after: hp,
        };
    }
    let mut copy = *board;
    let pay = payment_in(hero, hp, mp, skill, rules).unwrap_or(Payment {
        mana: skill_cost_in(hero, skill, rules).min(mp),
        blood: 0,
    });
    let mut health = hp;
    let mut actor_awake = rules.awake;
    let paid_hit = if rules.opponent.is_some() {
        take_damage(hero, &mut health, &mut actor_awake, pay.blood)
    } else {
        health -= pay.blood;
        DamageResult::default()
    };
    let mut rules = Rules {
        awake: actor_awake,
        ..rules
    };
    let e = effect_at_in(skill, &mut copy, target, hero, hp, mp, rules);
    if let Some(change) = e.board_change {
        let _ = change.apply(&mut rules.board_mechanics, rules.actor_enemy);
    }
    let mut state = crate::role_state::RoleState {
        statuses: rules.statuses,
        ..Default::default()
    };
    state.on_cast(effective_skill(skill, rules), board, target);
    rules.statuses = state.statuses;
    let settled = settle_batch_in(
        hero,
        health,
        mp - pay.mana,
        enemy_hp,
        e.reward_batch(effective_skill(skill, rules)),
        rules,
    );
    health = settled.actor_hp;
    let mut opponent = settled.opponent_hp;
    let mut opponent_heal = opponent - enemy_hp;
    let (mut damage, mut hit_rules) = preview_hit(rules, &mut opponent, settled.damage);
    let spent = mp - pay.mana;
    let mut mana = settled.actor_mp;
    let (cells, rewarded) = if e.cells.is_empty() {
        (rules.board_mechanics.matches(&copy), true)
    } else {
        (e.cells.clone(), e.reward)
    };
    let mut clear_awakens_self = false;
    if health > 0 && opponent > 0 && !cells.is_empty() {
        let clear_rules = Rules {
            initial_loot: skill_profile(skill, rules).initial_loot && e.reward,
            ..hit_rules
        };
        let clear = preview_clear_in(
            &copy,
            &cells,
            hero,
            health,
            mana,
            opponent,
            1,
            bonus + e.bonus,
            rewarded,
            clear_rules,
        );
        health = clear.health_after;
        opponent = clear.enemy_health_after;
        damage += clear.damage;
        opponent_heal += clear.opponent_heal;
        mana = clear.mana_after;
        clear_awakens_self = clear.awakens_self;
        if clear.awakens_enemy {
            hit_rules.opponent = hit_rules.opponent.map(|(id, _)| (id, true));
        }
    }
    SkillPreview {
        opponent_heal,
        awakens_self: paid_hit.awakened || clear_awakens_self,
        heal: health - hp,
        damage,
        enemy_health_after: opponent,
        awakens_enemy: rules.opponent.is_some_and(|(_, a)| !a)
            && hit_rules.opponent.is_some_and(|(_, a)| a),
        mana_gain: mana - spent,
        mana_after: mana,
        blood_cost: pay.blood,
        health_after: health,
    }
}
#[cfg(test)]
pub fn preview_range(
    skill: Skill,
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
) -> (SkillPreview, SkillPreview) {
    preview_range_in(
        skill,
        board,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        Rules::default(),
    )
}
pub fn preview_range_in(
    skill: Skill,
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    rules: Rules,
) -> (SkillPreview, SkillPreview) {
    let first = (0..N)
        .flat_map(|y| (0..N).map(move |x| (x, y)))
        .find(|&t| legal_target(hero, hp, skill, board, t, rules))
        .unwrap_or((0, 0));
    let mut low = preview_rewards_in(skill, board, first, hero, hp, mp, enemy_hp, bonus, rules);
    let mut high = low;
    for y in 0..N {
        for x in 0..N {
            if (x, y) == first || !legal_target(hero, hp, skill, board, (x, y), rules) {
                continue;
            }
            let p = preview_rewards_in(skill, board, (x, y), hero, hp, mp, enemy_hp, bonus, rules);
            low.enemy_health_after = low.enemy_health_after.min(p.enemy_health_after);
            high.enemy_health_after = high.enemy_health_after.max(p.enemy_health_after);
            low.awakens_enemy &= p.awakens_enemy;
            high.awakens_enemy |= p.awakens_enemy;
            low.awakens_self &= p.awakens_self;
            high.awakens_self |= p.awakens_self;
            low.health_after = low.health_after.min(p.health_after);
            high.health_after = high.health_after.max(p.health_after);
            low.opponent_heal = low.opponent_heal.min(p.opponent_heal);
            high.opponent_heal = high.opponent_heal.max(p.opponent_heal);
            low.heal = low.heal.min(p.heal);
            high.heal = high.heal.max(p.heal);
            low.damage = low.damage.min(p.damage);
            high.damage = high.damage.max(p.damage);
            low.mana_gain = low.mana_gain.min(p.mana_gain);
            high.mana_gain = high.mana_gain.max(p.mana_gain);
            low.mana_after = low.mana_after.min(p.mana_after);
            high.mana_after = high.mana_after.max(p.mana_after);
        }
    }
    (low, high)
}

pub struct MovePreview {
    pub opponent_heal: i32,
    pub awakens_self: bool,
    pub enemy_health_after: i32,
    pub awakens_enemy: bool,
    pub heal: i32,
    pub damage: i32,
    pub mana_gain: i32,
    pub health_after: i32,
    pub mana_after: i32,
    pub cells: Vec<(usize, usize)>,
}
/// Exact first-wave rewards for an exchange, before any random refill.
#[cfg(test)]
pub fn preview_move(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
    a: (usize, usize),
    c: (usize, usize),
) -> Option<MovePreview> {
    preview_move_in(
        board,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        armed,
        a,
        c,
        Rules::default(),
    )
}
pub fn preview_move_in(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
    a: (usize, usize),
    c: (usize, usize),
    rules: Rules,
) -> Option<MovePreview> {
    let mut copy = *board;
    crate::board::exchange_in(&mut copy, a, c, hero, armed, rules.board_mechanics)?;
    if !reachable_cell(hero, hp, a, rules) || !reachable_cell(hero, hp, c, rules) {
        return None;
    }
    let rules = Rules {
        horizontal: a.1 == c.1,
        swap_clear: true,
        ..rules
    };
    let cells = rules.board_mechanics.matches(&copy);
    Some(preview_clear_in(
        &copy, &cells, hero, hp, mp, enemy_hp, 1, bonus, true, rules,
    ))
}
/// First-wave rewards plus guaranteed survivor crossings; generated tiles are unknown.
/// Uses the same compaction, per-owner event dispatch and side ordering as Session.
pub fn preview_clear_in(
    board: &Board,
    cells: &[(usize, usize)],
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    chain: usize,
    bonus: i32,
    rewarded: bool,
    mut rules: Rules,
) -> MovePreview {
    let opponent_was_awake = rules.opponent.is_some_and(|(_, a)| a);
    let batch = if rewarded {
        clear_batch_in(hero, hp, mp, board, cells, chain, bonus, rules)
    } else {
        RewardBatch::default()
    };
    let settled = settle_batch_in(hero, hp, mp, enemy_hp, batch, rules);
    let mut health = settled.actor_hp;
    let mut mana = settled.actor_mp;
    let mut opponent = settled.opponent_hp;
    let mut opponent_heal = opponent - enemy_hp;
    let (mut damage, next) = preview_hit(rules, &mut opponent, settled.damage);
    let mut awakens_self = false;
    let mut opponent_mana = rules.opponent_mp;
    rules = next;
    if rewarded {
        rules.statuses.consume(StatusDuration::UntilClear);
    }
    if rules.board_mechanics.portals.iter().any(Option::is_some) {
        let mut copy = *board;
        for &(x, y) in cells {
            copy[y][x] = 255;
        }
        let falls = crate::board::compact_board_in(&mut copy, rules.board_mechanics)
            .expect("validated board topology");
        for side in [false, true] {
            rules.opponent_hp = Some(opponent);
            rules.opponent_mp = opponent_mana;
            if side == rules.actor_enemy {
                let batch = portal_transit_batch(hero, health, mana, &falls, rules);
                let settled = settle_batch_in(hero, health, mana, opponent, batch, rules);
                health = settled.actor_hp;
                mana = settled.actor_mp;
                opponent_heal += settled.opponent_hp - opponent;
                opponent = settled.opponent_hp;
                let (actual, next) = preview_hit(rules, &mut opponent, settled.damage);
                damage += actual;
                rules = next;
            } else if let Some((id, awake)) = rules.opponent {
                let other = Rules {
                    actor_enemy: side,
                    awake,
                    opponent: Some((hero, rules.awake)),
                    opponent_hp: Some(health),
                    opponent_mp: mana,
                    statuses: rules.opponent_statuses,
                    opponent_statuses: rules.statuses,
                    ..rules
                };
                let batch = portal_transit_batch(id, opponent, opponent_mana, &falls, other);
                let settled = settle_batch_in(id, opponent, opponent_mana, health, batch, other);
                opponent_heal += settled.actor_hp - opponent;
                opponent = settled.actor_hp;
                opponent_mana = settled.actor_mp;
                health = settled.opponent_hp;
                let hit = take_damage(hero, &mut health, &mut rules.awake, settled.damage);
                awakens_self |= hit.awakened;
            }
        }
    }
    MovePreview {
        opponent_heal,
        awakens_self,
        heal: health - hp,
        damage,
        enemy_health_after: opponent,
        awakens_enemy: rules.opponent.is_some()
            && !opponent_was_awake
            && rules.opponent.is_some_and(|(_, a)| a),
        mana_gain: mana - mp,
        health_after: health,
        mana_after: mana,
        cells: cells.to_vec(),
    }
}
/// Every listed exchange with its preview, best first: by heal + damage + mana, then by
/// heal, damage and mana, each from high to low. Equal rewards keep the board order.
#[cfg(test)]
pub fn ranked_moves(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
) -> Vec<(((usize, usize), (usize, usize)), Option<MovePreview>)> {
    ranked_moves_in(
        board,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        armed,
        Rules::default(),
    )
}
pub fn ranked_moves_in(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
    rules: Rules,
) -> Vec<(((usize, usize), (usize, usize)), Option<MovePreview>)> {
    let mut moves: Vec<_> = crate::board::move_hints_in(board, hero, armed, rules.board_mechanics)
        .into_iter()
        .map(|(a, c)| {
            let p = preview_move_in(board, hero, hp, mp, enemy_hp, bonus, armed, a, c, rules);
            ((a, c), p)
        })
        .collect();
    moves.retain(|(_, p)| p.is_some());
    moves.sort_by_key(|(_, p)| {
        std::cmp::Reverse(p.as_ref().map(|p| {
            (
                p.heal + p.damage + p.mana_gain,
                p.heal,
                p.damage,
                p.mana_gain,
            )
        }))
    });
    moves
}

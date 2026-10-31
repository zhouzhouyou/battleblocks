use super::*;

/// Exact rewards before the first random refill, including conversion matches, cost and caps.
pub struct SkillPreview {
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
    rules.turn_mode = match effective_skill(skill, rules) {
        Skill::Frenzy => TurnMode::Frenzy,
        Skill::Rational => TurnMode::Rational,
        _ => rules.turn_mode,
    };
    let settled = settle_rewards_in(
        hero,
        health,
        mp - pay.mana,
        enemy_hp,
        (e.hp, e.damage, e.mp),
        e.opponent_heal,
        rules,
    );
    health = settled.actor_hp;
    let mut opponent = settled.opponent_hp;
    let (mut damage, mut hit_rules) = preview_hit(rules, &mut opponent, settled.damage);
    let spent = mp - pay.mana;
    let mut mana = settled.actor_mp;
    let (cells, rewarded) = if e.cells.is_empty() {
        (crate::matches(&copy), true)
    } else {
        (e.cells.clone(), e.reward)
    };
    if health > 0 && opponent > 0 && rewarded && !cells.is_empty() {
        let clear_rules = Rules {
            initial_loot: skill == Skill::Greedy && e.reward,
            ..rules
        };
        let (h, d, m) = clear_rewards_in(
            hero,
            health,
            mana,
            &copy,
            &cells,
            1,
            bonus + e.bonus,
            clear_rules,
        );
        let settled = settle_rewards_in(hero, health, mana, opponent, (h, d, m), 0, rules);
        health = settled.actor_hp;
        opponent = settled.opponent_hp;
        let (actual, next) = preview_hit(hit_rules, &mut opponent, settled.damage);
        damage += actual;
        hit_rules = next;
        mana = settled.actor_mp;
    }
    SkillPreview {
        awakens_self: paid_hit.awakened,
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
    let mut high = preview_rewards_in(skill, board, first, hero, hp, mp, enemy_hp, bonus, rules);
    for y in 0..N {
        for x in 0..N {
            if !legal_target(hero, hp, skill, board, (x, y), rules) {
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
    if armed && crate::board::adjacent(a, c, hero) && a.0.abs_diff(c.0) + a.1.abs_diff(c.1) == 1 {
        crate::board::parallel_swap(&mut copy, a, c);
    } else if !crate::board::hero_swap(&mut copy, a, c, hero) {
        return None;
    }
    if hero == DWARF
        && (a.1 < N - reachable_rows(hero, hp, rules) || c.1 < N - reachable_rows(hero, hp, rules))
    {
        return None;
    }
    let rules = Rules {
        horizontal: a.1 == c.1,
        upper_swap: a.1 < 4 && c.1 < 4,
        ..rules
    };
    let cells = crate::matches(&copy);
    let (h, d, m) = clear_rewards_in(hero, hp, mp, &copy, &cells, 1, bonus, rules);
    let settled = settle_rewards_in(hero, hp, mp, enemy_hp, (h, d, m), 0, rules);
    let health_after = settled.actor_hp;
    let mana_after = settled.actor_mp;
    let mut opponent = settled.opponent_hp;
    let (damage, hit_rules) = preview_hit(rules, &mut opponent, settled.damage);
    Some(MovePreview {
        heal: health_after - hp,
        damage,
        enemy_health_after: opponent,
        awakens_enemy: rules.opponent.is_some_and(|(_, a)| !a)
            && hit_rules.opponent.is_some_and(|(_, a)| a),
        mana_gain: mana_after - mp,
        health_after,
        mana_after,
        cells,
    })
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
    let mut moves: Vec<_> = crate::board::move_hints(board, hero, armed)
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

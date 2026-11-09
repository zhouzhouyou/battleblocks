use super::*;

/// From this complete round onward, damage ramps up and healing decays.
/// The cap keeps the endgame finite without turning the first half into a race.
pub const FATIGUE_START_ROUND: u32 = 12;
pub const FATIGUE_DAMAGE_STEP: i32 = 12;
pub const FATIGUE_HEALING_STEP: i32 = 10;
pub fn fatigue_scales(round: u32) -> (i32, i32) {
    let steps = round.saturating_sub(FATIGUE_START_ROUND) as i32;
    (
        (100 + steps * FATIGUE_DAMAGE_STEP).min(300),
        (100 - steps * FATIGUE_HEALING_STEP).max(30),
    )
}

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
#[cfg(test)]
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
    clear_batch_in(hero, hp, mp, board, cells, chain, bonus, rules).totals()
}
pub fn clear_batch_in(
    hero: usize,
    hp: i32,
    mp: i32,
    board: &Board,
    cells: &[(usize, usize)],
    chain: usize,
    bonus: i32,
    rules: Rules,
) -> RewardBatch {
    let ctx = EventContext::new(hero, hp, mp, rules);
    let mut batch = RewardBatch::default();
    for &(x, y) in cells {
        let tile = board[y][x];
        if tile >= 7 {
            continue;
        }
        let mut gains = VALUES[tile as usize];
        let source = EventSource::Tile {
            position: (x, y),
            tile,
            chain,
            from_swap: rules.swap_clear,
        };
        dispatch_rules(
            &ctx,
            &mut RuleEvent::TileReward {
                source,
                gains: &mut gains,
            },
            &mut batch.triggers,
        );
        batch
            .events
            .extend(RewardBatch::gains(gains, 0, source).events);
    }
    let mut scale = reward_scale(chain, bonus);
    let mut damage_extra = 0;
    dispatch_rules(
        &ctx,
        &mut RuleEvent::ClearScale {
            chain,
            scale: &mut scale,
            damage_extra: &mut damage_extra,
        },
        &mut batch.triggers,
    );
    let mut gains = batch.totals();
    gains = (
        gains.0 * scale / 100,
        gains.1 * (scale + damage_extra) / 100,
        gains.2 * scale / 100,
    );
    dispatch_rules(
        &ctx,
        &mut RuleEvent::ClearTotals { gains: &mut gains },
        &mut batch.triggers,
    );
    batch.set_totals(gains);
    let mut triggers = std::mem::take(&mut batch.triggers);
    dispatch_rules(
        &ctx,
        &mut RuleEvent::ClearBatch { batch: &mut batch },
        &mut triggers,
    );
    batch.triggers = triggers;
    batch
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
    let mut blood = missing * 2;
    let mut allowed = false;
    dispatch_rules(
        &EventContext::new(
            hero,
            i32::MAX,
            0,
            Rules {
                awake,
                ..Default::default()
            },
        ),
        &mut RuleEvent::Payment {
            skill: None,
            missing,
            blood: &mut blood,
            allowed: &mut allowed,
        },
        &mut Vec::new(),
    );
    blood
}

pub fn payment_in(hero: usize, hp: i32, mp: i32, skill: Skill, rules: Rules) -> Option<Payment> {
    let missing = (skill_cost_in(hero, skill, rules) - mp).max(0);
    let mut blood = missing * 2;
    let mut allowed = missing == 0;
    dispatch_rules(
        &EventContext::new(hero, hp, mp, rules),
        &mut RuleEvent::Payment {
            skill: Some(skill),
            missing,
            blood: &mut blood,
            allowed: &mut allowed,
        },
        &mut Vec::new(),
    );
    if missing == 0 || allowed {
        Some(Payment {
            mana: skill_cost_in(hero, skill, rules).min(mp),
            blood,
        })
    } else {
        None
    }
}

/// Resolve an event batch through ability listeners, retaining every derivation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardSettlement {
    pub actor_hp: i32,
    pub actor_mp: i32,
    pub opponent_hp: i32,
    pub damage: i32,
    pub triggers: Vec<AbilityTrigger>,
    pub events: Vec<RewardEvent>,
}
impl RewardSettlement {
    /// Latest ability source that produced a live event for this resource and recipient.
    pub fn source_for(&self, resource: Resource, recipient: EventSide) -> Option<EventSource> {
        self.events
            .iter()
            .rev()
            .find(|event| {
                event.resource == resource && event.recipient == recipient && event.amount > 0
            })
            .map(RewardEvent::current_source)
    }
}
#[cfg(test)]
pub fn settle_rewards_in(
    hero: usize,
    hp: i32,
    mp: i32,
    opponent_hp: i32,
    gains: (i32, i32, i32),
    direct_opponent_heal: i32,
    rules: Rules,
) -> RewardSettlement {
    settle_batch_in(
        hero,
        hp,
        mp,
        opponent_hp,
        RewardBatch::gains(gains, direct_opponent_heal, EventSource::Unspecified),
        rules,
    )
}
pub fn settle_batch_in(
    hero: usize,
    hp: i32,
    mp: i32,
    opponent_hp: i32,
    mut batch: RewardBatch,
    rules: Rules,
) -> RewardSettlement {
    for reward in &mut batch.events {
        reward.amount = reward.amount * reward.probability as i32 / 100;
        reward.probability = 100;
    }
    let ctx = EventContext::new(hero, hp, mp, rules);
    let mut triggers = std::mem::take(&mut batch.triggers);
    let mut queue: std::collections::VecDeque<_> = std::mem::take(&mut batch.events).into();
    let mut processed = 0;
    while let Some(mut reward) = queue.pop_front() {
        processed += 1;
        assert!(
            processed <= 4096,
            "ability event chain exceeded safety limit"
        );
        if reward.amount <= 0 {
            batch.events.push(reward);
            continue;
        }
        let mut generated = Vec::new();
        let mut cancelled = false;
        dispatch_rules(
            &ctx,
            &mut RuleEvent::Reward {
                reward: &mut reward,
                generated: &mut generated,
                cancelled: &mut cancelled,
            },
            &mut triggers,
        );
        queue.extend(generated);
        if cancelled {
            continue;
        }
        let target_ctx = if reward.recipient == EventSide::Actor {
            Some(ctx)
        } else {
            rules.opponent.map(|(target, awake)| {
                let mut context = EventContext::new(
                    target,
                    opponent_hp,
                    rules.opponent_mp,
                    Rules {
                        awake,
                        board_mechanics: rules.board_mechanics,
                        statuses: rules.opponent_statuses,
                        opponent_statuses: rules.statuses,
                        opponent_mp: mp,
                        opponent: Some((hero, rules.awake)),
                        ..Default::default()
                    },
                );
                context.owner = EventSide::Opponent;
                context
            })
        };
        if let Some(target_ctx) = target_ctx {
            dispatch_rules(
                &target_ctx,
                &mut RuleEvent::Incoming {
                    reward: &mut reward,
                },
                &mut triggers,
            );
        }
        batch.events.push(reward);
    }
    let (damage_scale, healing_scale) = fatigue_scales(rules.round);
    if damage_scale != 100 || healing_scale != 100 {
        for event in &mut batch.events {
            event.amount = match event.resource {
                Resource::Damage => event.amount * damage_scale / 100,
                Resource::Healing => event.amount * healing_scale / 100,
                Resource::Mana => event.amount,
            };
        }
    }
    triggers.dedup();
    let (heal, damage, mana) = batch.totals();
    let opponent_heal = batch.total(Resource::Healing, EventSide::Opponent);
    let opponent_cap = rules
        .opponent
        .map(|(id, awake)| health_cap(id, awake))
        .unwrap_or(i32::MAX);
    RewardSettlement {
        actor_hp: (hp + heal).min(health_cap(hero, rules.awake)),
        actor_mp: (mp + mana).min(mana_cap(hero, rules.awake)),
        opponent_hp: (opponent_hp + opponent_heal).min(opponent_cap),
        damage,
        triggers,
        events: batch.events,
    }
}
pub fn turn_end_rewards(hero: usize, hp: i32, mp: i32, rules: Rules) -> RewardBatch {
    let mut batch = RewardBatch::default();
    let mut triggers = Vec::new();
    dispatch_rules(
        &EventContext::new(hero, hp, mp, rules),
        &mut RuleEvent::TurnEnd { batch: &mut batch },
        &mut triggers,
    );
    batch.triggers.extend(triggers);
    batch
}

/// Passive rewards consume immutable movement records, never replay gravity.
pub fn portal_transit_batch(
    hero: usize,
    hp: i32,
    mp: i32,
    falls: &[crate::board::TileFall],
    rules: Rules,
) -> RewardBatch {
    let ctx = EventContext::new(hero, hp, mp, rules);
    let mut batch = RewardBatch::default();
    let mut triggers = Vec::new();
    for fall in falls {
        for &(_, position) in &fall.portals {
            dispatch_rules(
                &ctx,
                &mut RuleEvent::PortalTransit {
                    tile: fall.kind,
                    position,
                    batch: &mut batch,
                },
                &mut triggers,
            );
        }
    }
    batch.triggers = triggers;
    batch
}

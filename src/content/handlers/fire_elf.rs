use super::*;

/// How many non-mana tiles Mana Affinity turns into mana tiles each turn.
const MANA_AFFINITY_TILES: usize = 2;

fn convert_one(
    board: &mut Board,
    eligible: &mut Vec<(usize, usize)>,
    mechanics: crate::board::BoardMechanics,
) -> bool {
    let attempts = eligible.len().saturating_mul(2).max(1);
    for _ in 0..attempts {
        if eligible.is_empty() {
            return false;
        }
        let index = crate::rng::gen_range(0, eligible.len());
        let (x, y) = eligible[index];
        let original = board[y][x];
        board[y][x] = 3;
        if mechanics.matches(board).is_empty() {
            eligible.swap_remove(index);
            return true;
        }
        board[y][x] = original;
    }
    false
}

pub fn handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Passive(passive) = id else {
        return;
    };
    if passive != Passive::ManaAffinity {
        return;
    }
    let RuleEvent::TurnStart { board } = event else {
        return;
    };
    let mut eligible: Vec<_> = (0..N)
        .flat_map(|y| (0..N).map(move |x| (x, y)))
        .filter(|&(x, y)| board[y][x] < 7 && VALUES[board[y][x] as usize].2 == 0)
        .collect();
    let mut changed = false;
    for _ in 0..MANA_AFFINITY_TILES {
        if !convert_one(board, &mut eligible, ctx.rules.board_mechanics) {
            break;
        }
        changed = true;
    }
    if changed {
        trigger(out, EventSide::Actor, id, None);
    }
}

pub fn active_handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    _out: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Active(skill) = id else {
        return;
    };
    if skill != Skill::EmberBlast {
        return;
    }
    match event {
        RuleEvent::SkillCost { skill, value } if *skill == Skill::EmberBlast => {
            // Nominal cost is the caster's current cap. Payment below spends all
            // current mana, so the displayed cost and the actual cost agree at full.
            **value = mana_cap(ctx.hero, ctx.rules.awake);
        }
        RuleEvent::Payment {
            skill,
            blood,
            allowed,
            ..
        } if skill.is_some_and(|skill| effective_skill(skill, ctx.rules) == Skill::EmberBlast) => {
            let cap = mana_cap(ctx.hero, ctx.rules.awake);
            let threshold = if ctx.rules.awake { cap * 80 / 100 } else { cap };
            **allowed = ctx.mp >= threshold;
            **blood = 0;
        }
        RuleEvent::Cast { skill, effect, .. } if *skill == Skill::EmberBlast => {
            **effect = super::fire_elf_effect::cast(*skill, ctx);
        }
        RuleEvent::CastState { skill, state, .. } if *skill == Skill::EmberBlast => {
            state.sleep_pending = true;
        }
        RuleEvent::AiUtility {
            board,
            bonus,
            swap_value,
            value,
            ..
        } => {
            **value = Some(ai_utility(ctx, board, *bonus, *swap_value));
        }
        _ => {}
    }
}

fn ai_utility(ctx: &EventContext, board: &Board, bonus: i32, swap_value: f32) -> i32 {
    let preview = preview_rewards_in(
        Skill::EmberBlast,
        board,
        (0, 0),
        ctx.hero,
        ctx.hp,
        ctx.mp,
        ctx.rules.opponent_hp.unwrap_or(10000),
        bonus,
        ctx.rules,
    );
    let full = ctx.mp >= mana_cap(ctx.hero, ctx.rules.awake);
    let board_value = (N * N) as i32 * 7 / 10;
    if preview.enemy_health_after <= 0 {
        // A guaranteed kill is always worth the nap.
        return preview.damage + board_value + 1000;
    }
    if !full {
        // At the awakened 80% threshold, wait for full unless the hit is lethal.
        return 0;
    }
    // Judge the cast by the damage that actually lands after the awakening
    // reset, not merely by whether the awakening is triggered. Sleeping gives
    // the opponent two turns, so low-value chip casts should lose to a normal
    // move; the best normal move (`swap_value`) is the comparison baseline.
    let sleep_cost = if ctx.hp < 250 { 150 } else { 110 };
    let value = preview.damage + board_value - sleep_cost;
    if value <= swap_value as i32 {
        swap_value as i32 - 1
    } else {
        value
    }
}

pub fn status_handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    if id != AbilityId::Status(StatusId::Sleep) {
        return;
    }
    match event {
        RuleEvent::TurnEnd { batch } => {
            let known = passives(ctx.hero, ctx.rules.awake);
            let sound = known.contains(&Passive::SoundSleep);
            // Sleep recovery is a fire-elf passive. A copied Ember Blast only
            // skips the caster's turn; it does not borrow this recovery.
            if !sound && !known.contains(&Passive::SleepRecovery) {
                return;
            }
            let healing = health_cap(ctx.hero, ctx.rules.awake) * if sound { 12 } else { 8 } / 100;
            let mana = mana_cap(ctx.hero, ctx.rules.awake) * if sound { 20 } else { 15 } / 100;
            batch.push(
                Resource::Healing,
                EventSide::Actor,
                healing,
                EventSource::Status(StatusId::Sleep),
            );
            batch.push(
                Resource::Mana,
                EventSide::Actor,
                mana,
                EventSource::Status(StatusId::Sleep),
            );
            let ability = if sound {
                AbilityId::Passive(Passive::SoundSleep)
            } else {
                AbilityId::Passive(Passive::SleepRecovery)
            };
            trigger(out, EventSide::Actor, ability, None);
        }
        RuleEvent::Incoming { reward }
            if reward.resource == Resource::Damage
                && reward.recipient == ctx.owner
                && passives(ctx.hero, ctx.rules.awake).contains(&Passive::SoundSleep)
                && !reward.has_source(EventSource::Status(StatusId::Sleep)) =>
        {
            reward.amount = reward.amount * 70 / 100;
            trigger(
                out,
                EventSide::Actor,
                AbilityId::Passive(Passive::SoundSleep),
                None,
            );
        }
        _ => {}
    }
}

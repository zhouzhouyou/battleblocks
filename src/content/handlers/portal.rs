use super::*;
#[path = "../skills/portal.rs"]
mod skill;
pub fn active(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    _: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Active(s) = id else {
        return;
    };
    match event {
        RuleEvent::Profile { value } => {
            value.paired_target = s == Skill::PortalPair;
            value.gravity_change = s == Skill::GravityShift;
            value.zero_ai_threshold = true;
        }
        RuleEvent::SkillCost { .. }
            if s == Skill::GravityShift && ctx.rules.awake && ctx.rules.copied_skill == Some(s) =>
        {
            // Copied actives retain their native awakened cost through the same rule.
            gravity_mastery_cost(event);
        }
        RuleEvent::Targeted { value, .. } => **value = true,
        RuleEvent::LegalTarget {
            target, allowed, ..
        } => {
            **allowed = skill::mechanics(s, ctx.rules, *target).is_some()
                && (s != Skill::GravityShift
                    || !ctx.rules.board_mechanics.gravity_matches_quadrant(*target));
        }
        RuleEvent::NoSwapUsable { value, board } => {
            **value = Some((0..N).any(|y| {
                (0..N).any(|x| {
                    skill::mechanics(s, ctx.rules, (x, y)).is_some_and(|next| {
                        !next.matches(board).is_empty()
                            || !crate::board::hero_options_in(board, ctx.hero, next).is_empty()
                    })
                })
            }));
        }
        RuleEvent::AiUtility {
            value,
            board,
            bonus,
            swap_value,
        } => {
            let target = crate::strategy::choose_target_in(
                s, board, ctx.hero, ctx.hp, ctx.mp, *bonus, ctx.rules,
            );
            let Some(next) = skill::mechanics(s, ctx.rules, target) else {
                return;
            };
            let preview = preview_rewards_in(
                s,
                board,
                target,
                ctx.hero,
                ctx.hp,
                ctx.mp,
                ctx.rules.opponent_hp.unwrap_or(10000),
                *bonus,
                ctx.rules,
            );
            // Immediate clears and the following exchange use the edited topology.
            // Never score an exchange on tiles already queued to clear.
            let followup = if next.matches(board).is_empty() {
                crate::strategy::best_move_score_in(
                    board,
                    ctx.hero,
                    preview.health_after,
                    preview.mana_after,
                    Rules {
                        board_mechanics: next,
                        ..ctx.rules
                    },
                ) - *swap_value
            } else {
                0.
            };
            **value = Some(
                ((preview.damage - preview.opponent_heal) as f32
                    + preview.heal as f32 * 0.8
                    + (preview.mana_after - ctx.mp) as f32 * 0.65
                    + followup) as i32,
            );
        }
        RuleEvent::Cast { target, effect, .. } => {
            if let Some(next) = skill::mechanics(s, ctx.rules, *target) {
                effect.board_change = Some(if s == Skill::PortalPair {
                    BoardChange::Portals(next.portals[ctx.rules.actor_enemy as usize].unwrap())
                } else {
                    BoardChange::Gravity(next.gravity())
                });
            }
        }
        _ => {}
    }
}
pub fn passive(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    let RuleEvent::PortalTransit {
        tile,
        position,
        batch,
    } = event
    else {
        return;
    };
    if id != AbilityId::Passive(Passive::PortalYield) || *tile >= 7 {
        return;
    }
    let base = VALUES[*tile as usize];
    let mut rewards = RewardBatch::gains(
        (
            (base.0 > 0) as i32,
            (base.1 > 0) as i32,
            (base.2 > 0) as i32,
        ),
        0,
        EventSource::Portal {
            position: *position,
            tile: *tile,
        },
    );
    for reward in &mut rewards.events {
        reward.derived_by(EventSource::Passive(Passive::PortalYield), EventSide::Actor);
    }
    batch.events.extend(rewards.events);
    trigger(out, ctx.owner, id, None);
}

fn gravity_mastery_cost(event: &mut RuleEvent<'_>) {
    if let RuleEvent::SkillCost {
        skill: Skill::GravityShift,
        value,
    } = event
    {
        **value = 15;
    }
}

pub fn mastery(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    _: &mut Vec<AbilityTrigger>,
) {
    if id == AbilityId::Passive(Passive::GravityMastery) && ctx.rules.awake {
        gravity_mastery_cost(event);
    }
}

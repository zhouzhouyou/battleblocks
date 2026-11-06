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
            value, swap_value, ..
        } => {
            // These setup skills leave the turn open for an ordinary exchange.
            **value = Some(if *swap_value > 0. {
                if s == Skill::PortalPair {
                    8
                } else {
                    4
                }
            } else {
                0
            });
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

use super::*;
#[path = "active_ai.rs"]
mod ai;
#[path = "../skills/dwarf.rs"]
mod dwarf;
#[path = "../skills/knight.rs"]
mod knight;
#[path = "../skills/legacy.rs"]
mod legacy;
#[path = "../skills/nun.rs"]
mod nun;
#[path = "../skills/priest.rs"]
mod priest;

pub fn handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    _out: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Active(listener) = id else {
        return;
    };
    match event {
        RuleEvent::Profile { value } => match listener {
            Skill::Copy => value.copy_slot = true,
            Skill::Jump => value.hop = true,
            Skill::Miracle => value.force_move = true,
            Skill::Hammer => value.random_target = true,
            Skill::Charge | Skill::Rush => value.delayed_damage = true,
            Skill::Greedy => value.initial_loot = true,
            Skill::Guard | Skill::Blood => value.healing_only = true,
            Skill::Landslide => value.requires_unarmed = true,
            Skill::Strike => {
                value.requires_no_bonus = true;
                value.zero_ai_threshold = true;
            }
            _ => {}
        },
        RuleEvent::ResolveSkill { value } if listener == Skill::Copy => {
            **value = ctx
                .rules
                .copied_skill
                .filter(|s| *s != Skill::Copy)
                .unwrap_or(Skill::Copy)
        }
        RuleEvent::NoSwapUsable { board, value } => match listener {
            Skill::Jump if !ctx.rules.statuses.contains(StatusId::ExtraReach) => {
                **value = Some(
                    !move_options_in(
                        board,
                        ctx.hero,
                        ctx.hp,
                        Rules { ..ctx.rules }.with_status(
                            StatusId::ExtraReach,
                            Skill::Jump,
                            StatusDuration::OwnerTurnEnd(1),
                        ),
                    )
                    .is_empty(),
                )
            }
            Skill::Miracle => **value = Some(true),
            _ => {}
        },
        RuleEvent::AiUtility {
            board,
            bonus,
            swap_value,
            value,
        } => **value = ai::utility(listener, ctx, board, *bonus, *swap_value),

        RuleEvent::CastState {
            skill,
            board,
            target,
            state,
        } if *skill == listener => match listener {
            Skill::Frenzy => {
                state
                    .statuses
                    .insert(StatusId::Frenzy, listener, StatusDuration::OwnerTurnEnd(1))
            }
            Skill::Rational => state.statuses.insert(
                StatusId::Rational,
                listener,
                StatusDuration::OwnerTurnEnd(1),
            ),
            Skill::Jump => {
                state.statuses.insert(
                    StatusId::ExtraReach,
                    listener,
                    StatusDuration::OwnerTurnEnd(1),
                );
            }
            Skill::Miracle => {
                state.force_armed = true;
                state.statuses.insert(
                    StatusId::ForcedMove,
                    listener,
                    StatusDuration::OwnerTurnEnd(1),
                );
            }
            Skill::Strike => state.statuses.insert(
                StatusId::DoubleRewards,
                listener,
                StatusDuration::UntilClear,
            ),
            Skill::Landslide => {
                state
                    .statuses
                    .insert(StatusId::ParallelSwap, listener, StatusDuration::UntilSwap)
            }
            Skill::Entomb => {
                if let Some(slot) = state.pockets.iter_mut().find(|p| p.is_none()) {
                    *slot = Some(board[target.1][target.0]);
                }
            }
            Skill::Resurrect => state.pockets[state.pocket_slot] = None,
            _ => {}
        },
        RuleEvent::Targeted { skill, value } if *skill == listener => {
            **value = matches!(
                listener,
                Skill::Charge
                    | Skill::Nature
                    | Skill::Greedy
                    | Skill::Chort
                    | Skill::Balance
                    | Skill::War
                    | Skill::Drain
                    | Skill::Rush
                    | Skill::Forge
                    | Skill::Entomb
                    | Skill::Resurrect
                    | Skill::Copy
            )
        }
        RuleEvent::LegalTarget {
            skill,
            board,
            target,
            allowed,
        } if *skill == listener => match listener {
            Skill::Copy => **allowed = false,
            Skill::Jump if ctx.rules.statuses.contains(StatusId::ExtraReach) => **allowed = false,
            Skill::Entomb
                if ctx.rules.pockets.iter().all(Option::is_some)
                    || board[target.1][target.0] >= 7 =>
            {
                **allowed = false
            }
            Skill::Resurrect
                if ctx
                    .rules
                    .pockets
                    .get(ctx.rules.pocket_slot)
                    .copied()
                    .flatten()
                    .is_none() =>
            {
                **allowed = false
            }
            _ => {}
        },
        RuleEvent::Cast {
            skill,
            board,
            target,
            effect,
        } if *skill == listener => {
            **effect = match listener {
                Skill::Rush | Skill::Miracle | Skill::Hammer => knight::effect(listener, *target),
                Skill::Jump | Skill::Forge => dwarf::effect(listener, board, *target),
                Skill::Entomb | Skill::Resurrect | Skill::Copy => {
                    priest::cast(listener, board, *target, ctx)
                }
                Skill::Frenzy | Skill::Healing | Skill::Rational => nun::cast(listener, ctx),
                _ => legacy::cast(listener, board, *target, ctx),
            }
        }
        _ => {}
    }
}

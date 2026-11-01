use super::*;
pub fn handle(
    id: AbilityId,
    _ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    match (id, event) {
        (AbilityId::Passive(Passive::KnightStrength), RuleEvent::TileReward { gains, .. })
            if gains.1 > 0 =>
        {
            gains.1 += 3;
            trigger(out, EventSide::Actor, id, None);
        }
        (AbilityId::Passive(Passive::Might), RuleEvent::TileReward { gains, .. })
            if gains.1 > 0 =>
        {
            gains.1 += 2;
            trigger(out, EventSide::Actor, id, None);
        }
        (
            AbilityId::Passive(Passive::Clumsy | Passive::ClumsyAwake),
            RuleEvent::Movement { profile },
        ) => profile.columns = true,
        (AbilityId::Passive(Passive::Clumsy), RuleEvent::ClearScale { chain, scale, .. })
            if *chain == 1 && _ctx.rules.horizontal =>
        {
            **scale += 8;
            trigger(out, EventSide::Actor, id, None);
        }
        (AbilityId::Passive(Passive::ClumsyAwake), RuleEvent::ClearScale { chain, scale, .. })
            if *chain == 1 && _ctx.rules.horizontal =>
        {
            **scale += 18;
            trigger(out, EventSide::Actor, id, None);
        }
        _ => {}
    }
}

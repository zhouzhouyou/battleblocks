use super::*;
pub fn handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    match (id, event) {
        (AbilityId::Passive(Passive::Short), RuleEvent::Capabilities { value }) => {
            value.rejection = Some("Dwarf.unreachable")
        }
        (AbilityId::Passive(Passive::Grudge), RuleEvent::Capabilities { value }) => {
            value.reach_visual = true
        }

        (AbilityId::Passive(Passive::DwarfStrength), RuleEvent::TileReward { gains, .. })
            if gains.1 > 0 =>
        {
            gains.1 += 2;
            trigger(out, EventSide::Actor, id, None);
        }
        (AbilityId::Passive(Passive::Short), RuleEvent::Incoming { reward })
            if reward.resource == Resource::Damage && reward.amount > 0 =>
        {
            if matches!(reward.current_source(),EventSource::Tile{position:(_,y),from_swap:true,..} if y<N/2)
            {
                reward.amount = 0;
                trigger(out, ctx.owner, id, Some("Dwarf.evade"));
            }
        }
        (AbilityId::Passive(Passive::Short), RuleEvent::Reach { rows }) => **rows = 4,
        (AbilityId::Passive(Passive::Grudge), RuleEvent::Reach { rows }) => {
            **rows += [80, 60, 40]
                .iter()
                .filter(|&&p| ctx.hp * 100 <= health_cap(ctx.hero, true) * p)
                .count()
        }
        _ => {}
    }
}

use super::*;
pub fn handle(
    id: AbilityId,
    _ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Status(status) = id else {
        return;
    };
    match (status, event) {
        (
            StatusId::Rational,
            RuleEvent::AbilityEnabled {
                ability: AbilityId::Passive(Passive::Madness),
                enabled,
            },
        ) => **enabled = false,

        (
            StatusId::Frenzy,
            RuleEvent::Reward {
                reward,
                generated,
                cancelled,
            },
        ) if !**cancelled
            && reward.resource == Resource::Healing
            && reward.recipient == EventSide::Opponent
            && !reward.has_source(EventSource::Status(status)) =>
        {
            let mut child = (**reward).clone();
            child.resource = Resource::Damage;
            child.derived_by(EventSource::Status(status), EventSide::Actor);
            generated.push(child);
            **cancelled = true;
            trigger(out, EventSide::Actor, id, None);
        }
        (StatusId::Frenzy, RuleEvent::Appearance { dark }) => **dark = true,
        (StatusId::Rational, RuleEvent::Appearance { dark }) => **dark = false,
        (StatusId::ExtraReach, RuleEvent::Reach { depth }) => **depth += 1,
        _ => {}
    }
}

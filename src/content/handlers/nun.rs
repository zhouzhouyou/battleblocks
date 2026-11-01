use super::*;
pub fn handle(
    id: AbilityId,
    _ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    match (id, event) {
        (AbilityId::Passive(Passive::Madness), RuleEvent::Appearance { dark }) => **dark = true,

        (
            AbilityId::Passive(Passive::FriendMilk),
            RuleEvent::Reward {
                reward, generated, ..
            },
        ) if reward.resource == Resource::Healing
            && reward.recipient == EventSide::Actor
            && !reward.has_source(EventSource::Passive(Passive::FriendMilk)) =>
        {
            let mut child = (**reward).clone();
            child.recipient = EventSide::Opponent;
            child.derived_by(EventSource::Passive(Passive::FriendMilk), EventSide::Actor);
            generated.push(child);
            trigger(out, EventSide::Actor, id, None);
        }
        (
            AbilityId::Passive(Passive::Madness),
            RuleEvent::Reward {
                reward,
                generated,
                cancelled,
            },
        ) if !**cancelled
            && reward.resource == Resource::Healing
            && !reward.has_source(EventSource::Passive(Passive::Madness)) =>
        {
            let mut child = (**reward).clone();
            child.resource = Resource::Damage;
            child.recipient = EventSide::Opponent;
            child.derived_by(EventSource::Passive(Passive::Madness), EventSide::Actor);
            generated.push(child);
            **cancelled = true;
            trigger(out, EventSide::Actor, id, None);
        }
        (AbilityId::Passive(Passive::AutoRecovery), RuleEvent::TurnEnd { batch }) => {
            batch.push(
                Resource::Healing,
                EventSide::Actor,
                6,
                EventSource::Passive(Passive::AutoRecovery),
            );
            batch.push(
                Resource::Mana,
                EventSide::Actor,
                8,
                EventSource::Passive(Passive::AutoRecovery),
            );
            trigger(out, EventSide::Actor, id, None);
        }
        _ => {}
    }
}

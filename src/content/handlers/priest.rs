use super::*;
pub fn handle(
    id: AbilityId,
    _ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    match (id, event) {
        (
            AbilityId::Passive(Passive::MindVision),
            RuleEvent::OpponentTurnEnd {
                cleared,
                state,
                commands,
            },
        ) => {
            if let Some(slot) = state.pockets.iter().position(Option::is_none) {
                if !cleared.is_empty() {
                    commands.push(StateCommand::FillPocket {
                        slot,
                        options: cleared.to_vec(),
                    });
                    trigger(out, EventSide::Actor, id, None);
                }
            }
        }
        (
            AbilityId::Passive(Passive::BorrowedSlot),
            RuleEvent::RefreshAbilities { options, commands },
        ) => {
            commands.push(StateCommand::CopySkill {
                options: options.to_vec(),
            });
            if !options.is_empty() {
                trigger(out, EventSide::Actor, id, None);
            }
        }
        (id, RuleEvent::Capabilities { value }) => match id {
            AbilityId::Passive(Passive::MindVision) => {
                value.pockets = true;
            }

            _ => {}
        },
        _ => {}
    }
}

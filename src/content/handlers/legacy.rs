use super::*;
pub fn handle(
    id: AbilityId,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    out: &mut Vec<AbilityTrigger>,
) {
    let AbilityId::Passive(p) = id else { return };
    match event {
        RuleEvent::Capabilities { value }
            if matches!(p, Passive::FleshCasting | Passive::FleshConversion) =>
        {
            value.blood_payment = true
        }
        RuleEvent::TileReward {
            source: EventSource::Tile { tile, .. },
            gains,
        } => match p {
            Passive::BalancedAffinity if *tile == 0 => gains.1 += 4,
            Passive::BalancedResonance if *tile == 0 => gains.1 += 6,
            Passive::ElementReturn if *tile == 0 => gains.2 += 2,
            _ => return,
        },
        RuleEvent::ClearScale { damage_extra, .. } if p == Passive::ElvenPower => {
            **damage_extra += 25
        }
        RuleEvent::ClearTotals { gains } => match p {
            Passive::BattleTraining => {
                gains.0 = gains.0 * 106 / 100;
                gains.1 = gains.1 * 106 / 100;
                gains.2 = gains.2 * 106 / 100;
            }
            Passive::ForestPact => {
                gains.0 = gains.0 * 93 / 100;
                gains.1 = gains.1 * 93 / 100;
                gains.2 = gains.2 * 93 / 100;
            }
            Passive::BloodFury | Passive::BoilingBlood
                if ctx.hp * 100
                    <= HEROES[ctx.hero].hp * if p == Passive::BoilingBlood { 50 } else { 35 } =>
            {
                gains.1 = gains.1 * 125 / 100
            }
            Passive::LootMastery if ctx.rules.initial_loot => gains.1 = gains.1 * 130 / 100,
            _ => return,
        },
        RuleEvent::ClearBatch { batch } => match p {
            Passive::ArcaneConversion if ctx.mp < mana_cap(ctx.hero, ctx.rules.awake) => {
                let amount = batch.total(Resource::Healing, EventSide::Actor) / 2;
                if amount == 0 {
                    return;
                }
                batch.derive_resource(
                    Resource::Healing,
                    EventSide::Actor,
                    Resource::Mana,
                    EventSide::Actor,
                    amount,
                    EventSource::Passive(p),
                    true,
                );
            }
            Passive::Recycling => {
                let amount = (ctx.hp + batch.total(Resource::Healing, EventSide::Actor)
                    - health_cap(ctx.hero, ctx.rules.awake))
                .max(0)
                    / 2;
                if amount == 0 {
                    return;
                }
                batch.derive_resource(
                    Resource::Healing,
                    EventSide::Actor,
                    Resource::Mana,
                    EventSide::Actor,
                    amount,
                    EventSource::Passive(p),
                    false,
                );
            }
            _ => return,
        },
        RuleEvent::ManaCap { value } if p == Passive::ManaSeal => **value = 250,
        RuleEvent::SkillCost { skill, value } => match p {
            Passive::GuardEconomy if *skill == Skill::Guard => **value = 80,
            Passive::MeteorEconomy if *skill == Skill::Meteor => **value = 140,
            _ => return,
        },
        RuleEvent::Payment {
            missing,
            blood,
            allowed,
        } => match p {
            Passive::FleshCasting => **allowed = ctx.hp > **blood,
            Passive::FleshConversion => {
                **blood = (*missing + 1) / 2;
                **allowed = ctx.hp > **blood;
            }
            _ => return,
        },
        RuleEvent::Movement { profile } => match p {
            Passive::ColumnFormation => profile.columns = true,
            Passive::ForestStep => profile.diagonal = true,
            _ => return,
        },
        RuleEvent::SkillEffect { skill, effect } => match p {
            Passive::BlastMastery if *skill == Skill::Imp => {
                effect.damage += effect.cells.len() as i32 * 2
            }
            Passive::ChargeMastery if *skill == Skill::Charge => {
                effect.damage = effect.damage * 125 / 100
            }
            Passive::LootMastery if *skill == Skill::Greedy => {
                effect.damage = effect.damage * 130 / 100
            }
            Passive::BloodSacrifice
                if payment_in(ctx.hero, ctx.hp, ctx.mp, *skill, ctx.rules)
                    .is_some_and(|p| p.blood > 0) =>
            {
                effect.damage = effect.damage * 120 / 100
            }
            _ => return,
        },
        _ => return,
    }
    if matches!(
        event,
        RuleEvent::TileReward { .. }
            | RuleEvent::ClearTotals { .. }
            | RuleEvent::ClearBatch { .. }
            | RuleEvent::SkillEffect { .. }
    ) {
        trigger(out, EventSide::Actor, id, None);
    }
}

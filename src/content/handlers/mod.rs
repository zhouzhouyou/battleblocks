use super::*;
mod active;
mod dwarf;
mod fire_elf;
#[path = "../skills/fire_elf.rs"]
mod fire_elf_effect;
mod knight;
mod legacy;
mod nun;
mod portal;
mod priest;
// Registration binds identities to implementations; rule behavior belongs to handlers.
pub fn passive_handler(p: Passive) -> RuleHandler {
    match p {
        Passive::PortalYield => portal::passive,
        Passive::GravityMastery => portal::mastery,
        Passive::DwarfStrength | Passive::Short | Passive::Grudge => dwarf::handle,
        Passive::KnightStrength | Passive::Might | Passive::Clumsy | Passive::ClumsyAwake => {
            knight::handle
        }
        Passive::FriendMilk | Passive::Madness | Passive::AutoRecovery => nun::handle,
        Passive::MindVision | Passive::BorrowedSlot => priest::handle,
        Passive::ManaAffinity | Passive::SleepRecovery | Passive::SoundSleep => fire_elf::handle,
        Passive::ColumnFormation
        | Passive::BattleTraining
        | Passive::ForestStep
        | Passive::ForestPact
        | Passive::ArcaneConversion
        | Passive::ManaSeal
        | Passive::ManaRelease
        | Passive::Recycling
        | Passive::BalancedAffinity
        | Passive::BalancedResonance
        | Passive::ElementReturn
        | Passive::BloodFury
        | Passive::BoilingBlood
        | Passive::FleshCasting
        | Passive::FleshConversion
        | Passive::ChargeMastery
        | Passive::GuardEconomy
        | Passive::ElvenPower
        | Passive::MeteorEconomy
        | Passive::LootMastery
        | Passive::BlastMastery
        | Passive::BloodSacrifice => legacy::handle,
    }
}
pub fn active_handler(skill: Skill) -> RuleHandler {
    match skill {
        Skill::PortalPair | Skill::GravityShift => portal::active,
        Skill::Charge
        | Skill::Guard
        | Skill::Landslide
        | Skill::Strike
        | Skill::Nature
        | Skill::Lightning
        | Skill::Meteor
        | Skill::Greedy
        | Skill::Imp
        | Skill::Chort
        | Skill::Corrupt
        | Skill::Blood
        | Skill::Balance
        | Skill::War
        | Skill::Drain
        | Skill::Rush
        | Skill::Miracle
        | Skill::Hammer
        | Skill::Jump
        | Skill::Forge
        | Skill::Entomb
        | Skill::Resurrect
        | Skill::Copy
        | Skill::Frenzy
        | Skill::Healing
        | Skill::Rational => active::handle,
        Skill::EmberBlast => fire_elf::active_handle,
    }
}

pub fn status_handler(status: StatusId) -> RuleHandler {
    match status {
        StatusId::Frenzy
        | StatusId::Rational
        | StatusId::ExtraReach
        | StatusId::DoubleRewards
        | StatusId::ParallelSwap => statuses::handle,
        StatusId::Sleep => fire_elf::status_handle,
    }
}
mod statuses;

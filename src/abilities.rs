use super::{payment_in, Rules, HEROES};

/// Stable identities: phase upgrades replace an identity, additive powers keep existing ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Passive {
    ColumnFormation,
    BattleTraining,
    ForestStep,
    ForestPact,
    ArcaneConversion,
    ManaSeal,
    ManaRelease,
    Recycling,
    BalancedAffinity,
    BalancedResonance,
    ElementReturn,
    BloodFury,
    BoilingBlood,
    FleshCasting,
    FleshConversion,
    ChargeMastery,
    GuardEconomy,
    ElvenPower,
    MeteorEconomy,
    LootMastery,
    BlastMastery,
    BloodSacrifice,
    KnightStrength,
    DwarfStrength,
    Might,
    Short,
    Grudge,
    Clumsy,
    ClumsyAwake,
    MindVision,
    BorrowedSlot,
    FriendMilk,
    AutoRecovery,
    Madness,
    ManaAffinity,
    SleepRecovery,
    SoundSleep,
    PortalYield,
}
use Passive::*;
pub struct AbilityLoadout {
    pub normal: &'static [Passive],
    pub awake: &'static [Passive],
    pub awake_skills: &'static [super::Skill],
    pub awakening: &'static str,
}
pub fn ability_loadout(hero: usize) -> &'static AbilityLoadout {
    static DATA: std::sync::OnceLock<Vec<AbilityLoadout>> = std::sync::OnceLock::new();
    &DATA.get_or_init(|| {
        include_str!("../config/ability-loadouts.txt")
            .lines()
            .filter(|s| !s.starts_with('#') && !s.is_empty())
            .enumerate()
            .map(|(index, line)| {
                let f: Vec<_> = line.split('|').collect();
                assert_eq!(f[0].parse::<usize>().unwrap(), index);
                let parse = |text: &str| -> &'static [Passive] {
                    Box::leak(
                        text.split(',')
                            .filter(|s| !s.is_empty())
                            .map(|name| {
                                *Passive::ALL
                                    .iter()
                                    .find(|p| format!("{p:?}") == name)
                                    .expect("passive identity")
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )
                };
                let active = Box::leak(
                    f[3].split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| super::Skill::from_index(s.parse().unwrap()).unwrap())
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                );
                AbilityLoadout {
                    normal: parse(f[1]),
                    awake: parse(f[2]),
                    awake_skills: active,
                    awakening: f[4],
                }
            })
            .collect()
    })[hero]
}
pub fn passives(hero: usize, awake: bool) -> &'static [Passive] {
    let data = ability_loadout(hero);
    if awake {
        data.awake
    } else {
        data.normal
    }
}
#[cfg(test)]
pub fn has_passive(hero: usize, awake: bool, passive: Passive) -> bool {
    passives(hero, awake).contains(&passive)
}
impl Passive {
    pub const ALL: &'static [Self] = &[
        Self::ColumnFormation,
        Self::BattleTraining,
        Self::ForestStep,
        Self::ForestPact,
        Self::ArcaneConversion,
        Self::ManaSeal,
        Self::ManaRelease,
        Self::Recycling,
        Self::BalancedAffinity,
        Self::BalancedResonance,
        Self::ElementReturn,
        Self::BloodFury,
        Self::BoilingBlood,
        Self::FleshCasting,
        Self::FleshConversion,
        Self::ChargeMastery,
        Self::GuardEconomy,
        Self::ElvenPower,
        Self::MeteorEconomy,
        Self::LootMastery,
        Self::BlastMastery,
        Self::BloodSacrifice,
        Self::KnightStrength,
        Self::DwarfStrength,
        Self::Might,
        Self::Short,
        Self::Grudge,
        Self::Clumsy,
        Self::ClumsyAwake,
        Self::MindVision,
        Self::BorrowedSlot,
        Self::FriendMilk,
        Self::AutoRecovery,
        Self::Madness,
        Self::ManaAffinity,
        Self::SleepRecovery,
        Self::SoundSleep,
        Self::PortalYield,
    ];
    fn is_new(self) -> bool {
        matches!(
            self,
            KnightStrength
                | DwarfStrength
                | Might
                | Short
                | Grudge
                | Clumsy
                | ClumsyAwake
                | MindVision
                | BorrowedSlot
                | FriendMilk
                | AutoRecovery
                | Madness
                | ManaAffinity
                | SleepRecovery
                | SoundSleep
                | PortalYield
        )
    }
    pub fn replaces(self) -> Option<Self> {
        match self {
            ClumsyAwake => Some(Clumsy),
            ManaRelease => Some(ManaSeal),
            BalancedResonance => Some(BalancedAffinity),
            BoilingBlood => Some(BloodFury),
            FleshConversion => Some(FleshCasting),
            SoundSleep => Some(SleepRecovery),

            _ => None,
        }
    }
    pub fn name(self, zh: bool) -> &'static str {
        if self.is_new() {
            return super::role_text(&format!("{self:?}.name"), zh);
        }
        let key = match self {
            KnightStrength | DwarfStrength | Might | Short | Grudge | Clumsy | ClumsyAwake
            | MindVision | BorrowedSlot | FriendMilk | AutoRecovery | Madness | ManaAffinity
            | SleepRecovery | SoundSleep | PortalYield => {
                unreachable!()
            }
            ColumnFormation => "passive.name.column_formation",
            BattleTraining => "passive.name.battle_training",
            ForestStep => "passive.name.forest_step",
            ForestPact => "passive.name.forest_pact",
            ArcaneConversion => "passive.name.arcane_conversion",
            ManaSeal => "passive.name.mana_seal",
            ManaRelease => "passive.name.mana_release",
            Recycling => "passive.name.recycling",
            BalancedAffinity => "passive.name.balanced_affinity",
            BalancedResonance => "passive.name.balanced_resonance",
            ElementReturn => "passive.name.element_return",
            BloodFury => "passive.name.blood_fury",
            BoilingBlood => "passive.name.boiling_blood",
            FleshCasting => "passive.name.flesh_casting",
            FleshConversion => "passive.name.flesh_conversion",
            ChargeMastery => "passive.name.charge_mastery",
            GuardEconomy => "passive.name.guard_economy",
            ElvenPower => "passive.name.elven_power",
            MeteorEconomy => "passive.name.meteor_economy",
            LootMastery => "passive.name.loot_mastery",
            BlastMastery => "passive.name.blast_mastery",
            BloodSacrifice => "passive.name.blood_sacrifice",
        };
        crate::tr_key(zh, key)
    }
    pub fn short_name(self, zh: bool) -> &'static str {
        if self.is_new() {
            return self.name(zh);
        }
        if zh {
            return self.name(true);
        }
        let key = match self {
            KnightStrength | DwarfStrength | Might | Short | Grudge | Clumsy | ClumsyAwake
            | MindVision | BorrowedSlot | FriendMilk | AutoRecovery | Madness | ManaAffinity
            | SleepRecovery | SoundSleep | PortalYield => {
                unreachable!()
            }
            ColumnFormation => "passive.short_name.column_formation",
            BattleTraining => "passive.short_name.battle_training",
            ForestStep => "passive.short_name.forest_step",
            ForestPact => "passive.short_name.forest_pact",
            ArcaneConversion => "passive.short_name.arcane_conversion",
            ManaSeal => "passive.short_name.mana_seal",
            ManaRelease => "passive.short_name.mana_release",
            Recycling => "passive.short_name.recycling",
            BalancedAffinity => "passive.short_name.balanced_affinity",
            BalancedResonance => "passive.short_name.balanced_resonance",
            ElementReturn => "passive.short_name.element_return",
            BloodFury => "passive.short_name.blood_fury",
            BoilingBlood => "passive.short_name.boiling_blood",
            FleshCasting => "passive.short_name.flesh_casting",
            FleshConversion => "passive.short_name.flesh_conversion",
            ChargeMastery => "passive.short_name.charge_mastery",
            GuardEconomy => "passive.short_name.guard_economy",
            ElvenPower => "passive.short_name.elven_power",
            MeteorEconomy => "passive.short_name.meteor_economy",
            LootMastery => "passive.short_name.loot_mastery",
            BlastMastery => "passive.short_name.blast_mastery",
            BloodSacrifice => "passive.short_name.blood_sacrifice",
        };
        crate::tr_key(false, key)
    }
    pub fn description(self, zh: bool) -> &'static str {
        if self.is_new() {
            return super::role_text(&format!("{self:?}.description"), zh);
        }
        let key = match self {
            KnightStrength | DwarfStrength | Might | Short | Grudge | Clumsy | ClumsyAwake
            | MindVision | BorrowedSlot | FriendMilk | AutoRecovery | Madness | ManaAffinity
            | SleepRecovery | SoundSleep | PortalYield => {
                unreachable!()
            }
            ColumnFormation => "passive.description.column_formation",
            BattleTraining => "passive.description.battle_training",
            ForestStep => "passive.description.forest_step",
            ForestPact => "passive.description.forest_pact",
            ArcaneConversion => "passive.description.arcane_conversion",
            ManaSeal => "passive.description.mana_seal",
            ManaRelease => "passive.description.mana_release",
            Recycling => "passive.description.recycling",
            BalancedAffinity => "passive.description.balanced_affinity",
            BalancedResonance => "passive.description.balanced_resonance",
            ElementReturn => "passive.description.element_return",
            BloodFury => "passive.description.blood_fury",
            BoilingBlood => "passive.description.boiling_blood",
            FleshCasting => "passive.description.flesh_casting",
            FleshConversion => "passive.description.flesh_conversion",
            ChargeMastery => "passive.description.charge_mastery",
            GuardEconomy => "passive.description.guard_economy",
            ElvenPower => "passive.description.elven_power",
            MeteorEconomy => "passive.description.meteor_economy",
            LootMastery => "passive.description.loot_mastery",
            BlastMastery => "passive.description.blast_mastery",
            BloodSacrifice => "passive.description.blood_sacrifice",
        };
        crate::tr_key(zh, key)
    }
    #[cfg(test)]
    pub fn summary_status(
        self,
        hero: usize,
        hp: i32,
        mp: i32,
        awake: bool,
        zh: bool,
    ) -> Option<(&'static str, bool)> {
        match self {
            BloodFury | BoilingBlood | FleshCasting | FleshConversion => {
                self.status(hero, hp, mp, awake, zh)
            }
            _ => None,
        }
    }
    pub fn badge_name(self, zh: bool, status: Option<(&str, bool)>) -> String {
        if matches!(self, BloodFury | BoilingBlood) {
            format!(
                "{}{}",
                self.short_name(zh),
                if status.is_some_and(|(_, on)| on) {
                    "+25%"
                } else {
                    crate::tr_key(zh, "ui.overlays.draw_passives.off")
                }
            )
        } else {
            self.short_name(zh).to_string()
        }
    }
    pub fn badge_highlight(self, status: Option<(&str, bool)>) -> bool {
        matches!(self, BloodFury | BoilingBlood) && status.is_some_and(|(_, on)| on)
    }
    pub fn status(
        self,
        hero: usize,
        hp: i32,
        mp: i32,
        awake: bool,
        zh: bool,
    ) -> Option<(&'static str, bool)> {
        let pair = match self {
            BloodFury | BoilingBlood => {
                let on = hp > 0
                    && hp * 100 <= HEROES[hero].hp * if self == BoilingBlood { 50 } else { 35 };
                (
                    if on {
                        crate::tr_key(zh, "passive.status.blood_fury_active")
                    } else {
                        crate::tr_key(zh, "passive.status.inactive")
                    },
                    on,
                )
            }
            ArcaneConversion => (
                if mp < super::mana_cap(hero, awake) {
                    crate::tr_key(zh, "passive.status.converting")
                } else {
                    crate::tr_key(zh, "passive.status.full_healing")
                },
                mp < super::mana_cap(hero, awake),
            ),
            FleshCasting | FleshConversion => {
                if hp <= 0 {
                    return None;
                }
                let missing: Vec<_> = HEROES[hero]
                    .skills
                    .iter()
                    .copied()
                    .filter(|&s| mp < super::skill_cost(hero, awake, s))
                    .collect();
                if missing.is_empty() {
                    return None;
                }
                let active = missing.into_iter().any(|s| {
                    payment_in(
                        hero,
                        hp,
                        mp,
                        s,
                        Rules {
                            awake,
                            ..Default::default()
                        },
                    )
                    .is_some()
                });
                (
                    crate::tr_key(
                        zh,
                        if active {
                            "health.blood_ready"
                        } else {
                            "health.blood_too_low"
                        },
                    ),
                    active,
                )
            }
            BloodSacrifice => {
                let can = HEROES[hero].skills.iter().any(|&s| {
                    payment_in(
                        hero,
                        hp,
                        mp,
                        s,
                        Rules {
                            awake,
                            ..Rules::default()
                        },
                    )
                    .is_some_and(|p| p.blood > 0)
                });
                (
                    if can {
                        crate::tr_key(zh, "passive.status.blood_paid_active")
                    } else {
                        crate::tr_key(zh, "passive.status.needs_blood_payment")
                    },
                    can,
                )
            }
            _ => return None,
        };
        Some(pair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upgrades_replace_identity_while_new_powers_keep_inherited_passives() {
        for hero in 0..7 {
            let base = passives(hero, false);
            let awake = passives(hero, true);
            assert!(awake.len() <= 4);
            let unique = awake
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>();
            assert_eq!(unique.len(), awake.len());
            for &old in base {
                let upgrades = awake.iter().filter(|p| p.replaces() == Some(old)).count();
                assert_eq!(usize::from(awake.contains(&old)) + upgrades, 1);
            }
            for &p in awake {
                if let Some(old) = p.replaces() {
                    assert!(base.contains(&old));
                    assert!(!awake.contains(&old));
                }
                for zh in [false, true] {
                    assert!(!p.name(zh).is_empty());
                    assert!(!p.description(zh).is_empty());
                }
            }
        }
        assert_eq!(passives(6, true), &[FleshConversion, BloodSacrifice]);
        assert!(passives(2, true).contains(&MeteorEconomy));
        assert!(!passives(2, true).contains(&ManaSeal));
    }
    #[test]
    fn displayed_phase_identities_drive_costs_cap_payment_and_live_status() {
        use super::super::{blood_cost, mana_cap, skill_cost, Skill};
        for awake in [false, true] {
            assert_eq!(has_passive(2, awake, MeteorEconomy), awake);
            assert_eq!(
                skill_cost(2, awake, Skill::Meteor),
                if awake { 140 } else { 160 }
            );
            assert_eq!(mana_cap(2, awake), if awake { 400 } else { 250 });
            assert_eq!(blood_cost(6, awake, 25), if awake { 13 } else { 50 });
        }
        assert_eq!(
            BloodFury.status(5, 211, 0, false, true),
            Some(("未触发", false))
        );
        assert_eq!(
            BoilingBlood.status(5, 282, 0, true, true),
            Some(("伤害+25%", true))
        );
        assert_eq!(
            ArcaneConversion.status(2, 250, 250, false, true),
            Some(("正常治疗", false))
        );
        assert_eq!(
            ArcaneConversion.status(2, 250, 250, true, true),
            Some(("转化中", true))
        );
        assert_eq!(
            BloodSacrifice.status(6, 341, 25, true, true),
            Some(("代付时+20%", true))
        );
        assert_eq!(
            BloodSacrifice.status(6, 341, 100, true, true),
            Some(("需生命代付", false))
        );
        // Selecting a phase (including an undo/reset) rebuilds the list; no stale upgrades remain.
        let original = passives(4, false).to_vec();
        let _ = passives(4, true);
        assert_eq!(passives(4, false), original);
    }
}

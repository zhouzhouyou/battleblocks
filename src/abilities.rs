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
}
use Passive::*;
pub fn passives(hero: usize, awake: bool) -> &'static [Passive] {
    match (hero, awake) {
        (0, false) => &[ColumnFormation, BattleTraining],
        (0, true) => &[ColumnFormation, BattleTraining, ChargeMastery, GuardEconomy],
        (1, false) => &[ForestStep, ForestPact],
        (1, true) => &[ForestStep, ForestPact, ElvenPower],
        (2, false) => &[ArcaneConversion, ManaSeal],
        (2, true) => &[ArcaneConversion, ManaRelease, MeteorEconomy],
        (3, false) => &[Recycling],
        (3, true) => &[Recycling, LootMastery],
        (4, false) => &[BalancedAffinity, ElementReturn],
        (4, true) => &[BalancedResonance, ElementReturn, BlastMastery],
        (5, false) => &[BloodFury],
        (5, true) => &[BoilingBlood],
        (6, false) => &[FleshCasting],
        (6, true) => &[FleshConversion, BloodSacrifice],
        _ => &[],
    }
}
pub fn has_passive(hero: usize, awake: bool, passive: Passive) -> bool {
    passives(hero, awake).contains(&passive)
}
impl Passive {
    pub fn replaces(self) -> Option<Self> {
        match self {
            ManaRelease => Some(ManaSeal),
            BalancedResonance => Some(BalancedAffinity),
            BoilingBlood => Some(BloodFury),
            FleshConversion => Some(FleshCasting),
            _ => None,
        }
    }
    pub fn name(self, zh: bool) -> &'static str {
        let (cn, en) = match self {
            ColumnFormation => ("列阵", "Column formation"),
            BattleTraining => ("战技训练", "Battle training"),
            ForestStep => ("林间穿行", "Forest step"),
            ForestPact => ("森林契约", "Forest pact"),
            ArcaneConversion => ("奥术转化", "Arcane conversion"),
            ManaSeal => ("魔力封印", "Mana seal"),
            ManaRelease => ("魔力解放", "Mana release"),
            Recycling => ("废物回收", "Scrap recovery"),
            BalancedAffinity => ("平衡亲和", "Balanced affinity"),
            BalancedResonance => ("平衡共鸣", "Balanced resonance"),
            ElementReturn => ("元素回流", "Element return"),
            BloodFury => ("血怒", "Blood fury"),
            BoilingBlood => ("沸血", "Boiling blood"),
            FleshCasting => ("血肉施法", "Flesh casting"),
            FleshConversion => ("血肉转化", "Flesh conversion"),
            ChargeMastery => ("冲锋专精", "Charge mastery"),
            GuardEconomy => ("守护节制", "Guard economy"),
            ElvenPower => ("精灵之力", "Elven power"),
            MeteorEconomy => ("陨星节制", "Meteor economy"),
            LootMastery => ("掠夺专精", "Loot mastery"),
            BlastMastery => ("爆裂专精", "Blast mastery"),
            BloodSacrifice => ("血祭", "Blood sacrifice"),
        };
        if zh {
            cn
        } else {
            en
        }
    }
    pub fn short_name(self, zh: bool) -> &'static str {
        if zh {
            return self.name(true);
        }
        match self {
            ColumnFormation => "Columns",
            BattleTraining => "Training",
            ForestStep => "Forest step",
            ForestPact => "Pact",
            ArcaneConversion => "Conversion",
            ManaSeal => "MP seal",
            ManaRelease => "MP release",
            Recycling => "Recycle",
            BalancedAffinity => "Affinity",
            BalancedResonance => "Resonance",
            ElementReturn => "MP return",
            BloodFury => "Fury",
            BoilingBlood => "Boiling",
            FleshCasting => "HP casting",
            FleshConversion => "HP casting+",
            ChargeMastery => "Charge+",
            GuardEconomy => "Guard cost",
            ElvenPower => "Elven power",
            MeteorEconomy => "Meteor cost",
            LootMastery => "Loot+",
            BlastMastery => "Blast+",
            BloodSacrifice => "Blood power",
        }
    }
    pub fn description(self, zh: bool) -> &'static str {
        let (cn,en)=match self {
            ColumnFormation=>("左右交换相邻两整列\n上下仍交换两个方块","Horizontal swaps move two columns.\nVertical swaps move two tiles."),
            BattleTraining=>("消除治疗、伤害和魔力\n收益增加6%","Clear healing, damage and mana\nincrease by 6%."),
            ForestStep=>("可交换斜向相邻方块\n仍按横向、纵向三消判定","Diagonal adjacent swaps allowed.\nMatches remain horizontal/vertical."),
            ForestPact=>("消除治疗、伤害和魔力\n收益降低7%","Clear healing, damage and mana\ndecrease by 7%."),
            ArcaneConversion=>("魔力未满时，一半消除治疗\n转为魔力；已满则正常治疗\n转换量向下取整","Below MP cap, half of clear healing\nbecomes MP (round down).\nAt full MP, heal normally."),
            ManaSeal=>("当前魔力上限锁定为250\n觉醒后解锁至400","Current MP cap is locked at 250.\nAwakening unlocks 400."),
            ManaRelease=>("魔力上限解锁至400","MP cap unlocked to 400."),
            Recycling=>("消除产生的溢出治疗\n每2点转为1魔力，受上限限制","Every 2 overheal from clears\nbecomes 1 MP, obeying the cap."),
            BalancedAffinity=>("有收益消除中，每个平衡\n方块额外造成4点伤害","Each balanced tile in rewarded\nclears deals 4 extra damage."),
            BalancedResonance=>("有收益消除中，每个平衡\n方块额外造成6点伤害","Each balanced tile in rewarded\nclears deals 6 extra damage."),
            ElementReturn=>("有收益消除中，每个平衡\n方块额外回复2魔力","Each balanced tile in rewarded\nclears restores 2 extra MP."),
            BloodFury=>("生命不高于初始上限35%时\n消除伤害增加25%\n每轮消除前判断","At or below 35% of initial HP,\nclear damage increases by 25%.\nChecked before each clear wave."),
            BoilingBlood=>("生命不高于初始上限50%时\n消除伤害增加25%\n每轮消除前判断","At or below 50% of initial HP,\nclear damage increases by 25%.\nChecked before each clear wave."),
            FleshCasting=>("缺少的每1魔力支付2生命\n支付后必须至少剩余1生命","Each missing MP costs 2 HP.\nMust leave at least 1 HP."),
            FleshConversion=>("1生命抵2点缺少的魔力\n生命消耗向上取整\n支付后必须至少剩余1生命","1 HP pays 2 missing MP.\nRound HP cost up.\nMust leave at least 1 HP."),
            ChargeMastery=>("骑士冲锋的伤害增加25%","Knight charge damage +25%."),
            GuardEconomy=>("坚守回复的魔力消耗\n从150降低至80","Guard MP cost reduced\nfrom 150 to 80."),
            ElvenPower=>("消除伤害额外增加基础值25%\n与消除增幅加算","Clear damage gains 25% of base.\nAdditive with clear amplification."),
            MeteorEconomy=>("陨星轰击的魔力消耗\n从160降低至140","Meteor MP cost reduced\nfrom 160 to 140."),
            LootMastery=>("区域掠夺直接与首轮消除\n伤害增加30%，后续连锁不加成","Area loot direct/first clear damage\n+30%; later cascades unchanged."),
            BlastMastery=>("平衡引爆每格额外造成2伤害\n单格伤害由12提高至14","Blast gains 2 damage per tile:\n12 increases to 14."),
            BloodSacrifice=>("使用生命代付的技能\n直接伤害增加20%\n不放大后续消除伤害","HP-paid skills deal 20% more\ndirect damage. Subsequent clear\ndamage is unchanged."),
        };
        if zh {
            cn
        } else {
            en
        }
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
                        if zh {
                            "伤害+25%"
                        } else {
                            "DMG +25%"
                        }
                    } else if zh {
                        "未触发"
                    } else {
                        "Inactive"
                    },
                    on,
                )
            }
            ArcaneConversion => (
                if mp < super::mana_cap(hero, awake) {
                    if zh {
                        "转化中"
                    } else {
                        "Converting"
                    }
                } else if zh {
                    "正常治疗"
                } else {
                    "Full healing"
                },
                mp < super::mana_cap(hero, awake),
            ),
            FleshCasting | FleshConversion => {
                return super::passive_status(hero, hp, mp, awake, zh)
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
                        if zh {
                            "代付时+20%"
                        } else {
                            "HP-paid +20%"
                        }
                    } else if zh {
                        "需生命代付"
                    } else {
                        "Needs HP payment"
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

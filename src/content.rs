#[path = "abilities.rs"]
mod abilities;
pub use abilities::{has_passive, Passive};
pub fn passives(hero: usize, awake: bool) -> &'static [Passive] {
    abilities::passives(hero, awake)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rules {
    pub awake: bool,
    pub initial_loot: bool,
    pub opponent_hp: Option<i32>,
    pub opponent: Option<(usize, bool)>,
}
pub const AWAKENING_LINES: [i32; 7] = [45, 35, 40, 50, 37, 35, 30];
pub const AWAKENING_CAPS: [i32; 7] = [55, 45, 50, 55, 50, 47, 44];
pub fn awakening_line(hero: usize) -> i32 {
    HEROES[hero].hp * AWAKENING_LINES[hero] / 100
}
pub fn health_cap(hero: usize, awake: bool) -> i32 {
    if awake {
        HEROES[hero].hp * AWAKENING_CAPS[hero] / 100
    } else {
        HEROES[hero].hp
    }
}
/// Wizard's reserve unlocks on awakening; other roles keep their base cap.
pub fn mana_cap(hero: usize, awake: bool) -> i32 {
    if has_passive(hero, awake, Passive::ManaSeal) {
        250
    } else {
        HEROES[hero].mp
    }
}
pub fn skill_cost(hero: usize, awake: bool, skill: Skill) -> i32 {
    if has_passive(hero, awake, Passive::GuardEconomy) && skill == Skill::Guard {
        80
    } else if has_passive(hero, awake, Passive::MeteorEconomy) && skill == Skill::Meteor {
        140
    } else {
        skill.cost()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageResult {
    pub damage: i32,
    pub awakened: bool,
    pub restored: i32,
}
/// One damage event. The triggering hit cannot spill into the second life bar.
pub fn take_damage(hero: usize, hp: &mut i32, awake: &mut bool, damage: i32) -> DamageResult {
    if damage <= 0 || *hp <= 0 {
        return DamageResult::default();
    }
    let before = *hp;
    let hit = before.saturating_sub(damage).max(0);
    let triggered = !*awake && hit <= awakening_line(hero);
    *hp = if triggered {
        health_cap(hero, true)
    } else {
        hit
    };
    *awake |= triggered;
    DamageResult {
        damage: before - hit,
        awakened: triggered,
        restored: if triggered { *hp - hit } else { 0 },
    }
}
pub fn awakening_effect(hero: usize, zh: bool) -> &'static str {
    let (en, cn) = match hero {
        0 => (
            "Charge damage +25%; Guard heals 20% of new HP cap; costs 80 MP",
            "冲锋伤害增加25%；坚守回复按新上限治疗20%，消耗80魔力",
        ),
        1 => (
            "Clear damage +25% of base (additive)",
            "消除伤害额外增加基础值25%（加算）",
        ),
        2 => (
            "MP cap: 250 → 400; Meteor cost: 160 → 140",
            "魔力上限250 → 400；陨星消耗160 → 140",
        ),
        3 => ("Area loot damage +30%", "区域掠夺伤害增加30%"),
        4 => (
            "Balanced bonus: +4 → +6; Blast: 12 → 14 per tile",
            "平衡方块额外伤害4 → 6；引爆每格伤害12 → 14",
        ),
        5 => ("Blood fury threshold: 35% → 50%", "血怒触发线35% → 50%"),
        _ => (
            "1 HP pays 2 MP (round HP up); blood-paid damage +20%",
            "1生命抵2魔力（生命向上取整）；代付技能伤害+20%",
        ),
    };
    if zh {
        cn
    } else {
        en
    }
}
fn preview_hit(rules: Rules, hp: &mut i32, damage: i32) -> (i32, Rules) {
    let mut next = rules;
    if let Some((hero, mut awake)) = rules.opponent {
        let hit = take_damage(hero, hp, &mut awake, damage);
        next.opponent = Some((hero, awake));
        (hit.damage, next)
    } else {
        let actual = damage.max(0).min(*hp);
        *hp -= actual;
        (actual, next)
    }
}
use super::{Board, N};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Skill {
    Charge,
    Guard,
    Landslide,
    Strike,
    Nature,
    Lightning,
    Meteor,
    Greedy,
    Imp,
    Chort,
    Corrupt,
    Blood,
    Balance,
    War,
    Drain,
}
impl Skill {
    pub fn cost(self) -> i32 {
        match self {
            Self::Blood => 80,
            Self::Balance => 60,
            Self::War => 80,
            Self::Drain => 50,
            Self::Charge => 100,
            Self::Guard => 150,
            Self::Landslide => 200,
            Self::Strike => 35,
            Self::Nature => 60,
            Self::Lightning => 60,
            Self::Meteor => 160,
            Self::Greedy => 25,
            Self::Imp => 60,
            Self::Chort => 70,
            Self::Corrupt => 80,
        }
    }
    pub fn name(self, zh: bool) -> &'static str {
        let names = match self {
            Self::Blood => ("Mana transfusion", "魔力换血"),
            Self::Balance => ("Balanced conversion", "均衡置换"),
            Self::War => ("Attack conversion", "战意转化"),
            Self::Drain => ("Life siphon", "生命汲取"),
            Self::Charge => ("Knight charge", "骑士冲锋"),
            Self::Guard => ("Steadfast recovery", "坚守回复"),
            Self::Landslide => ("Landslide", "三线换位"),
            Self::Strike => ("Sinister strike", "消除增幅"),
            Self::Nature => ("Nature's force", "自然转化"),
            Self::Lightning => ("Lightning", "雷能吸收"),
            Self::Meteor => ("Meteor", "陨星轰击"),
            Self::Greedy => ("Greedy", "区域掠夺"),
            Self::Imp => ("Imp's pact", "平衡引爆"),
            Self::Chort => ("Chort's pact", "同类收割"),
            Self::Corrupt => ("Corrupt", "伤害转生"),
        };
        if zh {
            names.1
        } else {
            names.0
        }
    }
    pub fn description(self, zh: bool) -> &'static str {
        let names = match self {
            Self::Blood => (
                "Spend 80 mana to heal 100.",
                "消耗80魔力回复100生命",
            ),
            Self::Balance => (
                "Turn selected 2×2 into\nbalanced tiles. Set up\nbalanced detonation.",
                "选择2×2变为平衡方块\n为平衡引爆创造目标\n形成三消自动结算",
            ),
            Self::War => (
                "Turn selected 2×2 into\nattack tiles. Set up\nsame-type harvest.",
                "选择2×2变为伤害方块\n为同类收割创造目标\n形成三消自动结算",
            ),
            Self::Drain => (
                "Clear selected 2×2.\nHeal 2× healing value,\ndeal 3×; no mana.",
                "清除所选2×2区域\n伤害为治疗值3倍，回复2倍\n不获得魔力",
            ),
            Self::Charge => (
                "Clear 2 adjacent columns.\nDeal 60 + their attack value.\nNo healing or mana gain.",
                "选择相邻两列并清除\n造成60＋方块伤害值总和\n不回复生命或魔力",
            ),
            Self::Guard => ("Restore 20% of max HP.\nConsume pure healing tiles.","回复最大生命值的20%\n清除纯治疗，不额外回血"),
            Self::Landslide => (
                "Next adjacent swap also\nswaps both parallel pairs.\nNo match required.",
                "下次相邻交换附带两侧交换\n同时交换三组平行方块\n不要求交换后形成三消",
            ),
            Self::Strike => (
                "Next match grants double\nhealing, damage and mana.\nOne cast per exchange.",
                "下次三消时，治疗、伤害\n和魔力翻倍\n每次交换前只能释放一次",
            ),
            Self::Nature => (
                "Change a selected 2×2\nto balanced tiles.\nHeal 10, deal 25.",
                "选择2×2变成平衡方块\n造成25伤害、回复10生命\n形成三消时自动结算",
            ),
            Self::Lightning => (
                "Clear both diagonals.\nGain their mana values.\nNo healing or damage.",
                "清除两条对角线\n回复方块魔力值总和\n不治疗、不造成伤害",
            ),
            Self::Meteor => (
                "Clear central diamond.\nDeal 60 + half attack.\nGain half mana values.",
                "清除中央菱形区域\n造成60＋方块伤害值一半\n回复魔力值一半，不回复生命",
            ),
            Self::Greedy => (
                "Clear a selected 2×2.\nGain 150% base rewards.\nNo size multiplier.",
                "选择2×2区域清除\n获得三种基础收益的150%\n按方块基础值结算",
            ),
            Self::Imp => (
                "Clear all balanced tiles.\nPer tile: 12 damage, 4 mana.\nNew cascades give rewards.",
                "清除所有平衡方块\n每格造成12伤害、回复4魔力\n后续下落三消正常结算",
            ),
            Self::Chort => (
                "Clear one selected type.\nPer tile: heal 2, deal 4,\nand restore 1 mana.",
                "点击一种方块，全部清除\n每格治疗2、造成4伤害\n并回复1魔力",
            ),
            Self::Corrupt => (
                "Turn attack to healing.\nPer tile: heal 3, deal 6.\nResolve new matches.",
                "纯伤害方块全部变为治疗\n每格治疗3、造成6伤害\n形成三消时自动结算",
            ),
        };
        if zh {
            names.1
        } else {
            names.0
        }
    }
}
pub struct Hero {
    pub en: &'static str,
    pub zh: &'static str,
    pub hp: i32,
    pub mp: i32,
    pub skills: &'static [Skill],
}
pub const HEROES: &[Hero] = &[
    Hero {
        en: "Knight",
        zh: "骑士",
        hp: 700,
        mp: 200,
        skills: &[Skill::Charge, Skill::Guard],
    },
    Hero {
        en: "Elf",
        zh: "精灵",
        hp: 600,
        mp: 120,
        skills: &[Skill::Strike, Skill::Nature],
    },
    Hero {
        en: "Wizard",
        zh: "巫师",
        hp: 550,
        mp: 400,
        skills: &[Skill::Lightning, Skill::Meteor],
    },
    Hero {
        en: "Goblin",
        zh: "哥布林",
        hp: 725,
        mp: 160,
        skills: &[Skill::Greedy, Skill::Blood],
    },
    Hero {
        en: "Imp",
        zh: "小恶魔",
        hp: 700,
        mp: 120,
        skills: &[Skill::Imp, Skill::Balance],
    },
    Hero {
        en: "Chort",
        zh: "角魔",
        hp: 600,
        mp: 160,
        skills: &[Skill::Chort, Skill::War],
    },
    Hero {
        en: "Big Zombie",
        zh: "大僵尸",
        hp: 775,
        mp: 100,
        skills: &[Skill::Corrupt, Skill::Drain],
    },
];
pub const VALUES: [(i32, i32, i32); 7] = [
    (2, 4, 4),
    (6, 0, 0),
    (0, 12, 0),
    (0, 0, 12),
    (3, 7, 0),
    (3, 0, 7),
    (0, 7, 7),
];
#[derive(Default)]
pub struct Effect {
    pub cells: Vec<(usize, usize)>,
    pub hp: i32,
    pub damage: i32,
    pub mp: i32,
    pub reward: bool,
    pub bonus: i32,
    pub armed: bool,
}
pub fn effect(skill: Skill, b: &mut Board) -> Effect {
    effect_at(skill, b, (3, 3))
}
pub fn targeted(skill: Skill) -> bool {
    matches!(
        skill,
        Skill::Charge
            | Skill::Nature
            | Skill::Greedy
            | Skill::Chort
            | Skill::Balance
            | Skill::War
            | Skill::Drain
    )
}
pub fn target_cells(skill: Skill, b: &Board, target: (usize, usize)) -> Vec<(usize, usize)> {
    let mut copy = *b;
    let e = effect_at(skill, &mut copy, target);
    let transformed_only = e.cells.is_empty();
    let mut cells = e.cells;
    for y in 0..N {
        for x in 0..N {
            if b[y][x] != copy[y][x] && !cells.contains(&(x, y)) {
                cells.push((x, y));
            }
        }
    }
    if transformed_only {
        for cell in crate::matches(&copy) {
            if !cells.contains(&cell) {
                cells.push(cell);
            }
        }
    }
    cells
}
pub fn effect_at(skill: Skill, b: &mut Board, target: (usize, usize)) -> Effect {
    let mut e = Effect::default();
    match skill {
        Skill::Blood => e.hp = 100,
        Skill::Balance | Skill::War | Skill::Drain => {
            for y in target.1.min(N - 2)..target.1.min(N - 2) + 2 {
                for x in target.0.min(N - 2)..target.0.min(N - 2) + 2 {
                    if skill == Skill::Drain {
                        e.cells.push((x, y));
                        e.hp += VALUES[b[y][x] as usize].0 * 2;
                        e.damage += VALUES[b[y][x] as usize].0 * 3;
                    } else {
                        b[y][x] = if skill == Skill::War { 2 } else { 0 };
                    }
                }
            }
        }
        Skill::Guard => {
            e.hp = HEROES[0].hp / 5;
            for (y, row) in b.iter().enumerate() {
                for (x, &kind) in row.iter().enumerate() {
                    if kind == 1 {
                        e.cells.push((x, y));
                    }
                }
            }
            // The consumed healing tiles deliberately grant no extra rewards.
        }
        Skill::Charge => {
            e.damage = 60;
            let x = target.0.min(N - 2);
            for y in 0..N {
                for c in x..x + 2 {
                    e.cells.push((c, y));
                    e.damage += VALUES[b[y][c] as usize].1;
                }
            }
        }
        Skill::Landslide => e.armed = true,
        Skill::Strike => e.bonus = 1,
        Skill::Nature => {
            e.damage = 25;
            e.hp = 10;
            for y in target.1.min(N - 2)..target.1.min(N - 2) + 2 {
                for x in target.0.min(N - 2)..target.0.min(N - 2) + 2 {
                    b[y][x] = 0;
                }
            }
        }

        Skill::Corrupt => {
            for row in b.iter_mut() {
                for t in row {
                    if *t == 2 {
                        *t = 1;
                        e.damage += 6;
                        e.hp += 3;
                    }
                }
            }
        }
        Skill::Lightning => {
            for y in 0..N {
                for x in [y, N - 1 - y] {
                    e.cells.push((x, y));
                    e.mp += VALUES[b[y][x] as usize].2;
                }
            }
        }
        Skill::Meteor => {
            e.damage = 60;
            for x in 0..N {
                let c = x.min(N - 1 - x);
                for y in N / 2 - 1 - c..=N / 2 + c {
                    e.cells.push((x, y));
                    let v = VALUES[b[y][x] as usize];
                    e.damage += v.1 / 2;
                    e.mp += v.2;
                }
            }
            e.mp /= 2;
        }
        Skill::Greedy => {
            e.reward = true;
            for y in target.1.min(N - 2)..target.1.min(N - 2) + 2 {
                for x in target.0.min(N - 2)..target.0.min(N - 2) + 2 {
                    e.cells.push((x, y));
                    let v = VALUES[b[y][x] as usize];
                    e.hp += v.0;
                    e.damage += v.1;
                    e.mp += v.2;
                }
            }
            e.hp /= 2;
            e.damage /= 2;
            e.mp /= 2;
        }

        Skill::Imp => {
            for y in 0..N {
                for x in 0..N {
                    if b[y][x] == 0 {
                        e.cells.push((x, y));
                        e.damage += 12;
                        e.mp += 4;
                    }
                }
            }
        }
        Skill::Chort => {
            let kind = b[target.1][target.0];
            for y in 0..N {
                for x in 0..N {
                    if b[y][x] == kind {
                        e.cells.push((x, y));
                    }
                }
            }

            e.hp = e.cells.len() as i32 * 2;
            e.damage = e.cells.len() as i32 * 4;
            e.mp = e.cells.len() as i32;
        }
    }
    e
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_shapes_and_rewards_match_skill_rules() {
        let mut b = [[2; N]; N];
        let blade = effect(Skill::Charge, &mut b);
        assert_eq!(blade.cells.len(), 16);
        assert_eq!(blade.damage, 60 + 2 * N as i32 * VALUES[2].1);
        assert!(!blade.reward);
        let lightning = effect(Skill::Lightning, &mut [[3; N]; N]);
        assert_eq!(lightning.cells.len(), 16);
        assert_eq!(lightning.mp, 2 * N as i32 * VALUES[3].2);
        let meteor = effect(Skill::Meteor, &mut b);
        assert_eq!(meteor.cells.len(), 40);
        assert_eq!(
            meteor.damage,
            60 + meteor.cells.len() as i32 * (VALUES[2].1 / 2)
        );
        let imp = effect(Skill::Imp, &mut [[0; N]; N]);
        assert_eq!(imp.cells.len(), 64);
        assert!(!imp.reward);
        assert!(effect(Skill::Landslide, &mut b).armed);
        assert_eq!(effect(Skill::Strike, &mut b).bonus, 1);
    }
    #[test]
    fn selected_targets_and_new_combos() {
        let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + y) % 7) as u8));
        let e = effect_at(Skill::Charge, &mut b.clone(), (7, 7));
        assert!(e.cells.iter().all(|&(x, _)| x >= 6));
        let mut changed = b;
        effect_at(Skill::War, &mut changed, (7, 7));
        assert_eq!(
            changed
                .iter()
                .flatten()
                .zip(b.iter().flatten())
                .filter(|(a, b)| a != b)
                .count(),
            4
        );
        assert_eq!(
            effect_at(Skill::Drain, &mut [[1; N]; N], (7, 7)).hp,
            8 * VALUES[1].0
        );
        assert_eq!(effect(Skill::Blood, &mut b.clone()).hp, 100);
        for hero in HEROES {
            assert_eq!(hero.skills.len(), 2);
        }
    }
    #[test]
    fn knight_charge_and_guard_rules() {
        let charge = effect_at(Skill::Charge, &mut [[1; N]; N], (0, 0));
        assert_eq!(charge.damage, 60);
        assert!(!charge.reward);
        let guard = effect(Skill::Guard, &mut [[2; N]; N]);
        assert_eq!(guard.hp, HEROES[0].hp / 5);
        assert!(guard.cells.is_empty());
        assert!(!guard.armed);
        assert_eq!(guard.bonus, 0);
        assert_eq!(Skill::Guard.cost(), 150);
        assert_eq!(HEROES[0].skills, &[Skill::Charge, Skill::Guard]);
    }
    #[test]
    fn all_heroes_can_pay_their_skills() {
        for h in HEROES {
            for s in h.skills {
                assert!(s.cost() <= h.mp);
            }
        }
    }
    #[test]
    fn effects_are_unique_and_in_bounds() {
        for s in [
            Skill::Charge,
            Skill::Lightning,
            Skill::Meteor,
            Skill::Greedy,
            Skill::Imp,
            Skill::Chort,
        ] {
            let mut b = [[0; N]; N];
            let e = effect(s, &mut b);
            let mut cells = e.cells.clone();
            cells.sort();
            cells.dedup();
            assert_eq!(cells.len(), e.cells.len());
            assert!(cells.iter().all(|&(x, y)| x < N && y < N));
        }
    }
    #[test]
    fn corrupt_and_nature_change_board() {
        let mut b = [[2; N]; N];
        effect(Skill::Corrupt, &mut b);
        assert!(b.iter().flatten().all(|&t| t == 1));
        effect(Skill::Nature, &mut b);
        assert_eq!(b.iter().flatten().filter(|&&t| t == 0).count(), 4);
    }
}

/// Exact rewards before the first random refill, including conversion matches, cost and caps.
pub struct SkillPreview {
    pub enemy_health_after: i32,
    pub awakens_enemy: bool,
    pub awakens_self: bool,
    pub heal: i32,
    pub damage: i32,
    pub mana_gain: i32,
    pub mana_after: i32,
    pub blood_cost: i32,
    pub health_after: i32,
}
#[cfg(test)]
pub fn preview_rewards(
    skill: Skill,
    board: &Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
) -> SkillPreview {
    preview_rewards_in(
        skill,
        board,
        target,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        Rules::default(),
    )
}
pub fn preview_rewards_in(
    skill: Skill,
    board: &Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    rules: Rules,
) -> SkillPreview {
    let mut copy = *board;
    let pay = payment_in(hero, hp, mp, skill, rules).unwrap_or(Payment {
        mana: skill_cost(hero, rules.awake, skill).min(mp),
        blood: 0,
    });
    let mut health = hp;
    let mut actor_awake = rules.awake;
    let paid_hit = if rules.opponent.is_some() {
        take_damage(hero, &mut health, &mut actor_awake, pay.blood)
    } else {
        health -= pay.blood;
        DamageResult::default()
    };
    let rules = Rules {
        awake: actor_awake,
        ..rules
    };
    let e = effect_at_in(skill, &mut copy, target, hero, hp, mp, rules);
    health = (health + e.hp).min(health_cap(hero, actor_awake));
    let mut opponent = enemy_hp;
    let (mut damage, mut hit_rules) = preview_hit(rules, &mut opponent, e.damage);
    let spent = mp - pay.mana;
    let mut mana = (spent + e.mp).min(mana_cap(hero, rules.awake));
    let (cells, rewarded) = if e.cells.is_empty() {
        (crate::matches(&copy), true)
    } else {
        (e.cells.clone(), e.reward)
    };
    if health > 0 && opponent > 0 && rewarded && !cells.is_empty() {
        let clear_rules = Rules {
            initial_loot: skill == Skill::Greedy && e.reward,
            ..rules
        };
        let (h, d, m) = clear_rewards_in(
            hero,
            health,
            mana,
            &copy,
            &cells,
            1,
            bonus + e.bonus,
            clear_rules,
        );
        health = (health + h).min(health_cap(hero, rules.awake));
        let (actual, next) = preview_hit(hit_rules, &mut opponent, d);
        damage += actual;
        hit_rules = next;
        mana = (mana + m).min(mana_cap(hero, rules.awake));
    }
    SkillPreview {
        awakens_self: paid_hit.awakened,
        heal: health - hp,
        damage,
        enemy_health_after: opponent,
        awakens_enemy: rules.opponent.is_some_and(|(_, a)| !a)
            && hit_rules.opponent.is_some_and(|(_, a)| a),
        mana_gain: mana - spent,
        mana_after: mana,
        blood_cost: pay.blood,
        health_after: health,
    }
}
#[cfg(test)]
pub fn preview_range(
    skill: Skill,
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
) -> (SkillPreview, SkillPreview) {
    preview_range_in(
        skill,
        board,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        Rules::default(),
    )
}
pub fn preview_range_in(
    skill: Skill,
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    rules: Rules,
) -> (SkillPreview, SkillPreview) {
    let mut low = preview_rewards_in(skill, board, (0, 0), hero, hp, mp, enemy_hp, bonus, rules);
    let mut high = preview_rewards_in(skill, board, (0, 0), hero, hp, mp, enemy_hp, bonus, rules);
    for y in 0..N {
        for x in 0..N {
            let p = preview_rewards_in(skill, board, (x, y), hero, hp, mp, enemy_hp, bonus, rules);
            low.enemy_health_after = low.enemy_health_after.min(p.enemy_health_after);
            high.enemy_health_after = high.enemy_health_after.max(p.enemy_health_after);
            low.awakens_enemy &= p.awakens_enemy;
            high.awakens_enemy |= p.awakens_enemy;
            low.awakens_self &= p.awakens_self;
            high.awakens_self |= p.awakens_self;
            low.health_after = low.health_after.min(p.health_after);
            high.health_after = high.health_after.max(p.health_after);
            low.heal = low.heal.min(p.heal);
            high.heal = high.heal.max(p.heal);
            low.damage = low.damage.min(p.damage);
            high.damage = high.damage.max(p.damage);
            low.mana_gain = low.mana_gain.min(p.mana_gain);
            high.mana_gain = high.mana_gain.max(p.mana_gain);
            low.mana_after = low.mana_after.min(p.mana_after);
            high.mana_after = high.mana_after.max(p.mana_after);
        }
    }
    (low, high)
}
#[cfg(test)]
mod preview_tests {
    use super::*;
    #[test]
    fn guard_consumes_pure_healing_only_without_additional_rewards() {
        let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        let original = board;
        let seed = crate::rng::state();
        let expected: Vec<_> = (0..N)
            .flat_map(|y| (0..N).filter_map(move |x| (board[y][x] == 1).then_some((x, y))))
            .collect();
        let e = effect(Skill::Guard, &mut board.clone());
        assert_eq!(e.cells, expected);
        assert!(!e.reward);
        assert_eq!(e.damage, 0);
        assert_eq!(e.mp, 0);
        assert_eq!(e.hp, HEROES[0].hp / 5);
        assert!(e
            .cells
            .iter()
            .all(|&(x, y)| board[y][x] != 4 && board[y][x] != 5));
        assert_eq!(target_cells(Skill::Guard, &board, (3, 3)), expected);
        let preview = preview_rewards(Skill::Guard, &board, (3, 3), 0, 400, 300, 700, 0);
        assert_eq!(preview.heal, HEROES[0].hp / 5);
        assert_eq!(preview.mana_after, 150);
        assert_eq!(preview.mana_gain, 0);
        assert_eq!(board, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn charge_target_preview_is_positive_and_matches_selected_columns() {
        let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        for x in 0..N {
            let p = preview_rewards(Skill::Charge, &b, (x, 3), 0, 700, 300, 2000, 0);
            let start = x.min(N - 2);
            let expected = 60
                + (0..N)
                    .flat_map(|y| (start..start + 2).map(move |c| VALUES[b[y][c] as usize].1))
                    .sum::<i32>();
            assert_eq!(p.damage, expected);
            assert!(p.damage >= 60);
            assert_eq!(p.mana_after, 200);
            assert_eq!(p.mana_gain, 0);
        }
    }
    #[test]
    fn transformations_count_first_matches_and_preview_is_pure() {
        let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        let mut board = board;
        board[0][0] = 1;
        board[0][1] = 1;
        board[0][2] = 2;
        let seed = crate::rng::state();
        let original = board;
        let p = preview_rewards(Skill::Corrupt, &board, (3, 3), 6, 500, 400, 700, 0);
        assert!(p.heal >= 3 * VALUES[1].0);
        assert_eq!(board, original);
        assert_eq!(crate::rng::state(), seed);
        let affected = target_cells(Skill::Corrupt, &board, (3, 3));
        assert!(affected.contains(&(0, 0)) && affected.contains(&(2, 0)));
    }
    #[test]
    fn target_ranges_cover_every_target() {
        let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        for skill in [
            Skill::Charge,
            Skill::Nature,
            Skill::Greedy,
            Skill::Chort,
            Skill::Balance,
            Skill::War,
            Skill::Drain,
        ] {
            let (low, high) = preview_range(skill, &board, 0, 400, 300, 700, 0);
            for y in 0..N {
                for x in 0..N {
                    let p = preview_rewards(skill, &board, (x, y), 0, 400, 300, 700, 0);
                    assert!((low.heal..=high.heal).contains(&p.heal));
                    assert!((low.damage..=high.damage).contains(&p.damage));
                    assert!((low.mana_gain..=high.mana_gain).contains(&p.mana_gain));
                }
            }
        }
    }
    #[test]
    fn lightning_shows_cost_and_actual_mana() {
        let b = [[3; N]; N];
        let p = preview_rewards(Skill::Lightning, &b, (3, 3), 2, 500, 200, 700, 0);
        assert_eq!(
            p.mana_gain,
            (2 * N as i32 * VALUES[3].2).min(250 - (200 - Skill::Lightning.cost()))
        );
        assert_eq!(
            p.mana_after,
            (200 - Skill::Lightning.cost() + 2 * N as i32 * VALUES[3].2).min(250)
        );
        assert_eq!(b, [[3; N]; N]);
    }
    #[test]
    fn healing_and_damage_obey_caps() {
        let p = preview_rewards(Skill::Guard, &[[1; N]; N], (3, 3), 0, 650, 300, 500, 0);
        assert_eq!(p.heal, 50);
        assert_eq!(p.mana_after, 150);
        let p = preview_rewards(Skill::Charge, &[[2; N]; N], (3, 3), 0, 700, 300, 20, 0);
        assert_eq!(p.damage, 20);
    }
}

/// First clear 100%; later cascades +10% per wave, capped at 130%.
/// An elf preparation adds 100% once. Debug bonus units follow the same rule.
pub fn reward_scale(chain: usize, bonus: i32) -> i32 {
    100 + chain.saturating_sub(1).min(3) as i32 * 10 + bonus.max(0) * 100
}
#[cfg(test)]
mod reward_tests {
    use super::*;
    #[test]
    fn cascades_are_bounded_and_first_wave_is_linear() {
        assert_eq!(reward_scale(1, 0), 100);
        assert_eq!(reward_scale(2, 0), 110);
        assert_eq!(reward_scale(100, 0), 130);
        assert_eq!(reward_scale(1, 1), 200);
    }
}

/// Stateless passives: same calculation for game, previews and AI.
#[cfg(test)]
pub fn clear_rewards(
    hero: usize,
    hp: i32,
    mp: i32,
    board: &Board,
    cells: &[(usize, usize)],
    chain: usize,
    bonus: i32,
) -> (i32, i32, i32) {
    clear_rewards_in(hero, hp, mp, board, cells, chain, bonus, Rules::default())
}
pub fn clear_rewards_in(
    hero: usize,
    hp: i32,
    mp: i32,
    board: &Board,
    cells: &[(usize, usize)],
    chain: usize,
    bonus: i32,
    rules: Rules,
) -> (i32, i32, i32) {
    let (mut h, mut d, mut m) = (0, 0, 0);
    for &(x, y) in cells {
        if board[y][x] == 255 {
            continue;
        }
        let (a, b, c) = VALUES[board[y][x] as usize];
        h += a;
        d += b;
        m += c;
        if board[y][x] == 0 {
            if has_passive(hero, rules.awake, Passive::BalancedAffinity) {
                d += 4;
            }
            if has_passive(hero, rules.awake, Passive::BalancedResonance) {
                d += 6;
            }
            if has_passive(hero, rules.awake, Passive::ElementReturn) {
                m += 2;
            }
        }
    }
    let scale = reward_scale(chain, bonus);
    h = h * scale / 100;
    d = d
        * (scale
            + if has_passive(hero, rules.awake, Passive::ElvenPower) {
                25
            } else {
                0
            })
        / 100;
    m = m * scale / 100;
    if has_passive(hero, rules.awake, Passive::BattleTraining) {
        h = h * 106 / 100;
        d = d * 106 / 100;
        m = m * 106 / 100;
    }
    if has_passive(hero, rules.awake, Passive::ForestPact) {
        h = h * 93 / 100;
        d = d * 93 / 100;
        m = m * 93 / 100;
    }
    if has_passive(hero, rules.awake, Passive::ArcaneConversion) && mp < mana_cap(hero, rules.awake)
    {
        let converted = h / 2;
        h -= converted;
        m += converted;
    }
    if has_passive(hero, rules.awake, Passive::Recycling) {
        m += (hp + h - health_cap(hero, rules.awake)).max(0) / 2;
    }
    if (has_passive(hero, rules.awake, Passive::BloodFury)
        || has_passive(hero, rules.awake, Passive::BoilingBlood))
        && hp * 100
            <= HEROES[hero].hp
                * if has_passive(hero, rules.awake, Passive::BoilingBlood) {
                    50
                } else {
                    35
                }
    {
        d = d * 125 / 100;
    }
    if has_passive(hero, rules.awake, Passive::LootMastery) && rules.initial_loot {
        d = d * 130 / 100;
    }
    (h, d, m)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Payment {
    pub mana: i32,
    pub blood: i32,
}
#[cfg(test)]
pub fn payment(hero: usize, hp: i32, mp: i32, skill: Skill) -> Option<Payment> {
    payment_in(hero, hp, mp, skill, Rules::default())
}
pub fn blood_cost(hero: usize, awake: bool, missing: i32) -> i32 {
    let missing = missing.max(0);
    if has_passive(hero, awake, Passive::FleshConversion) {
        (missing + 1) / 2
    } else {
        missing * 2
    }
}
pub fn payment_in(hero: usize, hp: i32, mp: i32, skill: Skill, rules: Rules) -> Option<Payment> {
    let missing = (skill_cost(hero, rules.awake, skill) - mp).max(0);
    let blood = blood_cost(hero, rules.awake, missing);
    if missing == 0 || (hero == 6 && hp > blood) {
        Some(Payment {
            mana: skill_cost(hero, rules.awake, skill).min(mp),
            blood,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod passive_tests {
    use super::*;
    #[test]
    fn payment_requires_surviving_before_any_heal() {
        assert_eq!(
            payment(6, 101, 0, Skill::Drain),
            Some(Payment {
                mana: 0,
                blood: 100
            })
        );
        assert_eq!(payment(6, 100, 0, Skill::Drain), None);
        assert_eq!(
            payment(6, 60, 25, Skill::Drain),
            Some(Payment {
                mana: 25,
                blood: 50
            })
        );
        assert_eq!(payment(6, 50, 25, Skill::Drain), None);
        assert_eq!(payment(0, 700, 25, Skill::Drain), None);
        assert_eq!(
            payment(6, 1, 50, Skill::Drain),
            Some(Payment { mana: 50, blood: 0 })
        );
    }
    #[test]
    fn clear_passives_have_exact_rewards_and_thresholds() {
        let cells = vec![(0, 0), (1, 0), (2, 0)];
        assert_eq!(
            clear_rewards(2, 300, 0, &[[1; N]; N], &cells, 1, 0),
            (9, 0, 9)
        );
        assert_eq!(
            clear_rewards(3, 720, 490, &[[1; N]; N], &cells, 1, 0),
            (18, 0, 6)
        );
        assert_eq!(
            clear_rewards(4, 300, 0, &[[0; N]; N], &cells, 1, 0),
            (6, 24, 18)
        );
        assert_eq!(
            clear_rewards(5, 210, 0, &[[2; N]; N], &cells, 1, 0),
            (0, 45, 0)
        );
        assert_eq!(
            clear_rewards(5, 211, 0, &[[2; N]; N], &cells, 1, 0),
            (0, 36, 0)
        );
        assert_eq!(
            clear_rewards(4, 300, 0, &[[0; N]; N], &cells, 2, 1),
            (12, 50, 37)
        );
    }
    #[test]
    fn blood_preview_is_pure_accounts_for_cost_caps_and_targets() {
        let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        let original = b;
        let seed = crate::rng::state();
        for target in [(0, 0), (3, 3), (7, 7)] {
            let p = preview_rewards(Skill::Drain, &b, target, 6, 200, 25, 700, 0);
            let e = effect_at(Skill::Drain, &mut b.clone(), target);
            assert_eq!(p.blood_cost, 50);
            assert_eq!(p.health_after, 150 + e.hp);
            assert_eq!(p.heal, e.hp - 50);
            assert_eq!(p.mana_after, 0);
        }
        let p = preview_rewards(
            Skill::Drain,
            &[[1; N]; N],
            (0, 0),
            6,
            HEROES[6].hp - 1,
            49,
            700,
            0,
        );
        assert_eq!(p.health_after, HEROES[6].hp);
        assert_eq!(p.blood_cost, 2);
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn passive_skill_preview_matches_manual_first_wave_with_caps() {
        let b = [[0; N]; N];
        for id in 0..7 {
            let skill = Skill::Nature;
            let hp = HEROES[id].hp - 5;
            let mp = mana_cap(id, false);
            let p = preview_rewards(skill, &b, (0, 0), id, hp, mp, 700, 0);
            let mut copy = b;
            let e = effect_at(skill, &mut copy, (0, 0));
            let health = (hp + e.hp).min(HEROES[id].hp);
            let mana = mp - skill.cost() + e.mp;
            let (h, d, m) = clear_rewards(id, health, mana, &copy, &crate::matches(&copy), 1, 0);
            assert_eq!(p.health_after, (health + h).min(HEROES[id].hp));
            assert_eq!(p.damage, (e.damage + d).min(700));
            assert_eq!(p.mana_after, (mana + m).min(mana_cap(id, false)));
        }
    }
}

/// Actual actions that can continue a no-swap turn. A preparation which only
/// multiplies future rewards does not by itself unlock a dead board.
#[cfg(test)]
pub fn usable_skills(board: &Board, hero: usize, hp: i32, mp: i32) -> Vec<Skill> {
    usable_skills_in(board, hero, hp, mp, Rules::default())
}
pub fn usable_skills_in(board: &Board, hero: usize, hp: i32, mp: i32, rules: Rules) -> Vec<Skill> {
    HEROES[hero]
        .skills
        .iter()
        .copied()
        .filter(|&skill| {
            let Some(pay) = payment_in(hero, hp, mp, skill, rules) else {
                return false;
            };
            (0..N).any(|y| {
                (0..N).any(|x| {
                    let mut copy = *board;
                    let e = effect_at_in(skill, &mut copy, (x, y), hero, hp, mp, rules);
                    !e.cells.is_empty()
                        || copy != *board
                        || e.armed
                        || e.damage > 0
                        || (e.hp > pay.blood && hp - pay.blood < health_cap(hero, rules.awake))
                        || (mp - pay.mana + e.mp).min(mana_cap(hero, rules.awake)) > mp
                })
            })
        })
        .collect()
}
#[cfg(test)]
pub fn must_refresh(board: &Board, hero: usize, hp: i32, mp: i32, armed: bool) -> bool {
    must_refresh_in(board, hero, hp, mp, armed, Rules::default())
}
pub fn must_refresh_in(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    armed: bool,
    rules: Rules,
) -> bool {
    !armed
        && crate::board::hero_options(board, hero).is_empty()
        && usable_skills_in(board, hero, hp, mp, rules).is_empty()
}

#[cfg(test)]
mod deadlock_tests {
    use super::*;
    fn dead() -> Board {
        std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
    }
    #[test]
    fn deadlock_respects_role_moves_and_preparation() {
        let b = dead();
        for hero in 0..7 {
            assert!(must_refresh(&b, hero, 1, 0, false));
        }
        assert!(!must_refresh(&b, 0, 700, 0, true));
        // Strike alone cannot unlock a board, Nature can.
        assert!(must_refresh(&b, 1, 600, 35, false));
        assert!(!must_refresh(&b, 1, 600, 60, false));
        let mut diagonal = b;
        diagonal[0][0] = 6;
        diagonal[0][1] = 6;
        diagonal[0][2] = 2;
        diagonal[1][3] = 6;
        assert!(!must_refresh(&diagonal, 1, 600, 0, false));
        let mut columns = b;
        columns[0][0] = 6;
        columns[0][1] = 2;
        columns[0][2] = 6;
        columns[0][3] = 6;
        assert!(!must_refresh(&columns, 0, 700, 0, false));
    }
    #[test]
    fn skill_escape_checks_effects_caps_and_blood_before_heal() {
        let b = dead();
        let original = b;
        let seed = crate::rng::state();
        assert!(!must_refresh(&b, 0, 700, 100, false));
        assert!(!must_refresh(&b, 3, 500, 80, false));
        assert!(!must_refresh(&b, 6, 101, 0, false));
        assert!(must_refresh(&b, 6, 100, 0, false));
        assert!(!must_refresh(&b, 6, 51, 25, false));
        assert!(must_refresh(&b, 6, 50, 25, false));
        let no_targets =
            std::array::from_fn(|y| std::array::from_fn(|x| [3, 4, 5, 6][(x + 2 * y) % 4]));
        assert!(usable_skills(&no_targets, 0, 700, 150).contains(&Skill::Charge));
        assert!(!usable_skills(&no_targets, 0, 700, 150).contains(&Skill::Guard));
        assert!(!usable_skills(&no_targets, 4, 700, 60).contains(&Skill::Imp));
        assert!(usable_skills(&no_targets, 4, 700, 60).contains(&Skill::Balance));
        assert!(!usable_skills(&no_targets, 6, 775, 160).contains(&Skill::Corrupt));
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
}

pub struct MovePreview {
    pub enemy_health_after: i32,
    pub awakens_enemy: bool,
    pub heal: i32,
    pub damage: i32,
    pub mana_gain: i32,
    pub health_after: i32,
    pub mana_after: i32,
    pub cells: Vec<(usize, usize)>,
}
/// Exact first-wave rewards for an exchange, before any random refill.
#[cfg(test)]
pub fn preview_move(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
    a: (usize, usize),
    c: (usize, usize),
) -> Option<MovePreview> {
    preview_move_in(
        board,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        armed,
        a,
        c,
        Rules::default(),
    )
}
pub fn preview_move_in(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
    a: (usize, usize),
    c: (usize, usize),
    rules: Rules,
) -> Option<MovePreview> {
    let mut copy = *board;
    if armed && crate::board::adjacent(a, c, hero) && a.0.abs_diff(c.0) + a.1.abs_diff(c.1) == 1 {
        crate::board::parallel_swap(&mut copy, a, c);
    } else if !crate::board::hero_swap(&mut copy, a, c, hero) {
        return None;
    }
    let cells = crate::matches(&copy);
    let (h, d, m) = clear_rewards_in(hero, hp, mp, &copy, &cells, 1, bonus, rules);
    let health_after = (hp + h).min(health_cap(hero, rules.awake));
    let mana_after = (mp + m).min(mana_cap(hero, rules.awake));
    let mut opponent = enemy_hp;
    let (damage, hit_rules) = preview_hit(rules, &mut opponent, d);
    Some(MovePreview {
        heal: health_after - hp,
        damage,
        enemy_health_after: opponent,
        awakens_enemy: rules.opponent.is_some_and(|(_, a)| !a)
            && hit_rules.opponent.is_some_and(|(_, a)| a),
        mana_gain: mana_after - mp,
        health_after,
        mana_after,
        cells,
    })
}
/// Every listed exchange with its preview, best first: by heal + damage + mana, then by
/// heal, damage and mana, each from high to low. Equal rewards keep the board order.
#[cfg(test)]
pub fn ranked_moves(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
) -> Vec<(((usize, usize), (usize, usize)), Option<MovePreview>)> {
    ranked_moves_in(
        board,
        hero,
        hp,
        mp,
        enemy_hp,
        bonus,
        armed,
        Rules::default(),
    )
}
pub fn ranked_moves_in(
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
    armed: bool,
    rules: Rules,
) -> Vec<(((usize, usize), (usize, usize)), Option<MovePreview>)> {
    let mut moves: Vec<_> = crate::board::move_hints(board, hero, armed)
        .into_iter()
        .map(|(a, c)| {
            let p = preview_move_in(board, hero, hp, mp, enemy_hp, bonus, armed, a, c, rules);
            ((a, c), p)
        })
        .collect();
    moves.sort_by_key(|(_, p)| {
        std::cmp::Reverse(p.as_ref().map(|p| {
            (
                p.heal + p.damage + p.mana_gain,
                p.heal,
                p.damage,
                p.mana_gain,
            )
        }))
    });
    moves
}

#[cfg(test)]
mod move_preview_tests {
    use super::*;
    fn base() -> Board {
        std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
    }
    #[test]
    fn previews_use_role_exchange_rules_rewards_caps_and_bonus() {
        let mut b = base();
        b[0][0] = 6;
        b[0][1] = 2;
        b[0][2] = 6;
        b[0][3] = 6;
        let p = preview_move(&b, 0, 500, 0, 700, 0, false, (0, 0), (1, 0)).unwrap();
        assert_eq!((p.heal, p.damage, p.mana_gain), (0, 22, 22));
        let p = preview_move(&b, 0, 500, 0, 700, 1, false, (0, 0), (1, 0)).unwrap();
        assert_eq!((p.damage, p.mana_gain), (44, 44));
        assert!(preview_move(&base(), 0, 700, 0, 700, 0, false, (0, 0), (1, 0)).is_none());
        assert!(preview_move(&b, 0, 700, 0, 700, 0, false, (0, 0), (1, 1)).is_none());
        let mut b = base();
        b[0][0] = 6;
        b[0][1] = 6;
        b[0][2] = 2;
        b[1][3] = 6;
        let p = preview_move(&b, 1, 599, 119, 7, 0, false, (2, 0), (3, 1)).unwrap();
        assert_eq!((p.heal, p.damage, p.mana_gain), (0, 7, 1));
        assert_eq!((p.health_after, p.mana_after), (599, 120));
        let mut b = base();
        b[0][0] = 1;
        b[0][1] = 2;
        b[0][2] = 1;
        b[0][3] = 1;
        let p = preview_move(&b, 0, 699, 0, 700, 0, false, (0, 0), (1, 0)).unwrap();
        assert_eq!((p.heal, p.health_after), (1, 700));
    }
    #[test]
    fn every_hint_has_pure_preview_including_prepared_swaps() {
        crate::rng::srand(5017);
        let b = crate::board::fresh();
        let original = b;
        let seed = crate::rng::state();
        for hero in 0..7 {
            for armed in [false, true] {
                for (a, c) in crate::board::move_hints(&b, hero, armed) {
                    let p = preview_move(
                        &b,
                        hero,
                        HEROES[hero].hp - 1,
                        mana_cap(hero, false) - 1,
                        7,
                        1,
                        armed,
                        a,
                        c,
                    )
                    .unwrap();
                    assert!((0..=1).contains(&p.heal));
                    assert!((0..=1).contains(&p.mana_gain));
                    assert!((0..=7).contains(&p.damage));
                    assert_eq!(p.health_after, HEROES[hero].hp - 1 + p.heal);
                    assert_eq!(p.mana_after, mana_cap(hero, false) - 1 + p.mana_gain);
                }
            }
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
        let p = preview_move(&base(), 0, 700, 0, 700, 0, true, (0, 0), (1, 0)).unwrap();
        assert!(p.cells.is_empty());
        assert_eq!((p.heal, p.damage, p.mana_gain), (0, 0, 0));
    }
    #[test]
    fn ranked_moves_sort_by_total_then_heal_damage_mana() {
        // Rows 0, 2, 4 and 6 each get one swap that lines up three tiles of one kind:
        // damage+mana (total 42), heal+damage, heal+mana and balanced (each total 30).
        let mut b = base();
        for (y, kind, other) in [(0, 6, 2), (2, 4, 3), (4, 5, 2), (6, 0, 3)] {
            b[y][0] = kind;
            b[y][1] = other;
            b[y][2] = kind;
            b[y][3] = kind;
        }
        b[4][4] = 0;
        assert!(crate::matches(&b).is_empty());
        let original = b;
        let seed = crate::rng::state();
        // The flesh-casting zombie has no passive reward change.
        let ranked = ranked_moves(&b, 6, 500, 0, 700, 0, false);
        let rank = |y: usize| {
            let i = ranked
                .iter()
                .position(|&((a, c), _)| (a, c) == ((0, y), (1, y)) || (a, c) == ((1, y), (0, y)))
                .unwrap();
            let p = ranked[i].1.as_ref().unwrap();
            (i, (p.heal, p.damage, p.mana_gain))
        };
        let (damage_mana, heal_damage, heal_mana, balanced) = (rank(0), rank(2), rank(4), rank(6));
        assert_eq!(damage_mana.1, (0, 21, 21));
        assert_eq!(heal_damage.1, (9, 21, 0));
        assert_eq!(heal_mana.1, (9, 0, 21));
        assert_eq!(balanced.1, (6, 12, 12));
        assert!(damage_mana.0 < heal_damage.0);
        assert!(heal_damage.0 < heal_mana.0);
        assert!(heal_mana.0 < balanced.0);
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn ranked_moves_keep_every_hint_with_its_pure_preview() {
        crate::rng::srand(2718);
        for _ in 0..6 {
            let b = crate::board::fresh();
            let original = b;
            let seed = crate::rng::state();
            for hero in 0..7 {
                for armed in [false, true] {
                    let (hp, mp) = (HEROES[hero].hp / 2, HEROES[hero].mp / 2);
                    let ranked = ranked_moves(&b, hero, hp, mp, 300, 0, armed);
                    let mut listed: Vec<_> = ranked.iter().map(|(m, _)| *m).collect();
                    let mut hints = crate::board::move_hints(&b, hero, armed);
                    listed.sort();
                    hints.sort();
                    assert_eq!(listed, hints);
                    let key = |p: &MovePreview| {
                        (
                            p.heal + p.damage + p.mana_gain,
                            p.heal,
                            p.damage,
                            p.mana_gain,
                        )
                    };
                    for pair in ranked.windows(2) {
                        let (first, second) = (pair[0].1.as_ref(), pair[1].1.as_ref());
                        assert!(second.is_none() || key(first.unwrap()) >= key(second.unwrap()));
                    }
                    for &((a, c), ref p) in &ranked {
                        let expected = preview_move(&b, hero, hp, mp, 300, 0, armed, a, c);
                        assert_eq!(p.as_ref().map(key), expected.as_ref().map(key));
                    }
                }
            }
            assert_eq!(b, original);
            assert_eq!(crate::rng::state(), seed);
        }
    }
}

pub fn effect_at_in(
    skill: Skill,
    b: &mut Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    rules: Rules,
) -> Effect {
    let mut e = effect_at(skill, b, target);
    if rules.awake {
        if hero == 0 && skill == Skill::Guard {
            e.hp = health_cap(hero, true) / 5;
        }
        if has_passive(hero, rules.awake, Passive::BlastMastery) && skill == Skill::Imp {
            e.damage += e.cells.len() as i32 * 2;
        }
        if (has_passive(hero, rules.awake, Passive::ChargeMastery) && skill == Skill::Charge)
            || (has_passive(hero, rules.awake, Passive::LootMastery) && skill == Skill::Greedy)
        {
            e.damage = e.damage * if hero == 0 { 125 } else { 130 } / 100;
        }
        if has_passive(hero, rules.awake, Passive::BloodSacrifice)
            && payment_in(hero, hp, mp, skill, rules).is_some_and(|p| p.blood > 0)
        {
            e.damage = e.damage * 120 / 100;
        }
    }
    e
}

pub fn skill_description(skill: Skill, awake: bool, zh: bool) -> &'static str {
    if awake && skill == Skill::Imp {
        return if zh {
            "清除所有平衡方块\n每格造成14伤害、回复4魔力\n后续下落三消正常结算"
        } else {
            "Clear all balanced tiles.\nPer tile: 14 damage, 4 mana.\nNew cascades give rewards."
        };
    }
    if awake && skill == Skill::Guard {
        if zh {
            "回复觉醒后生命上限的20%\n清除纯治疗，不额外回血"
        } else {
            "Heal 20% of the new HP cap.\nConsume pure healing tiles."
        }
    } else {
        skill.description(zh)
    }
}

#[cfg(test)]
mod awakening_tests {
    use super::*;
    fn rules(awake: bool, enemy: usize, other_awake: bool) -> Rules {
        Rules {
            awake,
            opponent: Some((enemy, other_awake)),
            ..Rules::default()
        }
    }
    fn board() -> Board {
        std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
    }
    #[test]
    fn every_role_triggers_exactly_at_its_line_and_survives_a_lethal_hit() {
        for hero in 0..7 {
            let threshold = awakening_line(hero);
            let mut hp = threshold + 2;
            let mut awake = false;
            assert!(!take_damage(hero, &mut hp, &mut awake, 1).awakened);
            assert!(take_damage(hero, &mut hp, &mut awake, 1).awakened);
            assert_eq!(hp, health_cap(hero, true));
            assert!(awake);
            let hit = take_damage(hero, &mut hp, &mut awake, 10_000);
            assert!(!hit.awakened);
            assert_eq!(hp, 0);
            hp = HEROES[hero].hp;
            awake = false;
            assert!(take_damage(hero, &mut hp, &mut awake, 10_000).awakened);
            assert_eq!(hp, health_cap(hero, true));
            assert_eq!(take_damage(hero, &mut hp, &mut awake, 80).damage, 80);
            assert_eq!(hp, health_cap(hero, true) - 80);
        }
        let mut hp = 0;
        let mut awake = false;
        assert!(!take_damage(0, &mut hp, &mut awake, 10).awakened);
        hp = 300;
        assert!(!take_damage(0, &mut hp, &mut awake, 0).awakened);
    }
    #[test]
    fn imp_strength_moves_from_blast_to_rewarded_clears_in_both_phases() {
        let b = board();
        let original = b;
        let seed = crate::rng::state();
        let cells = vec![(0, 0)]; // balanced tile: base 2 HP / 4 damage / 4 MP
        let count = b.iter().flatten().filter(|&&t| t == 0).count() as i32;
        for (awake, blast, bonus) in [(false, 12, 4), (true, 14, 6)] {
            let r = rules(awake, 0, true);
            assert_eq!(
                clear_rewards_in(4, 300, 0, &b, &cells, 1, 0, r),
                (2, 4 + bonus, 6)
            );
            let e = effect_at_in(Skill::Imp, &mut b.clone(), (0, 0), 4, 300, 60, r);
            assert_eq!((e.damage, e.mp), (count * blast, count * 4));
            assert!(!e.reward); // direct blast does not also receive clear/passive rewards
            assert!(payment_in(4, 300, 59, Skill::Imp, r).is_none());
            let p = preview_rewards_in(Skill::Imp, &b, (0, 0), 4, 300, 60, 385, 0, r);
            assert_eq!(p.damage, count * blast);
            assert_eq!(p.mana_after, count * 4);
            let p = preview_rewards_in(Skill::Imp, &[[0; N]; N], (0, 0), 4, 300, 120, 385, 0, r);
            assert_eq!(p.mana_after, 120);
            assert!(skill_description(Skill::Imp, awake, true).contains(&format!("{blast}伤害")));
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn tuned_enemy_caps_match_real_hits_for_every_target_and_phase() {
        let b = board();
        let original = b;
        let seed = crate::rng::state();
        for enemy in [2, 4, 5, 6] {
            for awake in [false, true] {
                let hp = if awake {
                    health_cap(enemy, true)
                } else {
                    awakening_line(enemy) + 1
                };
                for target in [(0, 0), (3, 0), (6, 0)] {
                    let mut copy = b;
                    let e = effect_at_in(
                        Skill::Charge,
                        &mut copy,
                        target,
                        0,
                        700,
                        100,
                        Rules::default(),
                    );
                    let (mut actual_hp, mut actual_awake) = (hp, awake);
                    take_damage(enemy, &mut actual_hp, &mut actual_awake, e.damage);
                    let preview = preview_rewards_in(
                        Skill::Charge,
                        &b,
                        target,
                        0,
                        700,
                        100,
                        hp,
                        0,
                        rules(false, enemy, awake),
                    );
                    assert_eq!(preview.enemy_health_after, actual_hp);
                    assert_eq!(preview.awakens_enemy, !awake && actual_awake);
                    assert_eq!(preview.mana_after, 0);
                }
            }
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn phase_health_changes_reach_actual_caps_without_changing_costs_or_preview_state() {
        let mut b = board();
        b[0][..3].copy_from_slice(&[1, 0, 1]);
        b[1][1] = 1;
        let original = b;
        let seed = crate::rng::state();
        for hero in [2, 4, 5, 6] {
            let cap = health_cap(hero, true);
            let r = rules(true, 0, true);
            let p =
                preview_move_in(&b, hero, cap - 1, 0, 385, 0, false, (1, 0), (1, 1), r).unwrap();
            let mut actual = b;
            assert!(crate::board::hero_swap(&mut actual, (1, 0), (1, 1), hero));
            let (heal, damage, mana) =
                clear_rewards_in(hero, cap - 1, 0, &actual, &crate::matches(&actual), 1, 0, r);
            assert_eq!(p.health_after, (cap - 1 + heal).min(cap));
            assert_eq!(
                (p.heal, p.damage, p.mana_after),
                (1, damage, mana.min(mana_cap(hero, true)))
            );
            for &skill in HEROES[hero].skills {
                let cost = skill_cost(hero, true, skill);
                assert_eq!(
                    payment_in(hero, cap, cost, skill, r),
                    Some(Payment {
                        mana: cost,
                        blood: 0
                    })
                );
            }
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn same_role_has_independent_flags_and_reset_is_a_plain_snapshot_restore() {
        let mut hp = [400, 400];
        let mut flags = [false, false];
        let original = (hp, flags);
        take_damage(0, &mut hp[1], &mut flags[1], 1000);
        assert_eq!(flags, [false, true]);
        assert_eq!(hp, [400, 385]);
        (hp, flags) = original;
        assert_eq!(hp, [400, 400]);
        assert_eq!(flags, [false, false]);
    }
    #[test]
    fn awakened_buffs_costs_and_caps_use_the_actual_shared_rules() {
        let cells = vec![(0, 0), (1, 0), (2, 0)];
        let r = rules(true, 2, true);
        let b = [[2; N]; N];
        assert_eq!(clear_rewards_in(1, 200, 0, &b, &cells, 1, 1, r).1, 75); // (200+25)% then elf -7%
        assert_eq!(
            clear_rewards_in(4, 200, 0, &[[0; N]; N], &cells, 1, 0, r).1,
            30
        );
        assert_eq!(clear_rewards_in(5, 270, 0, &b, &cells, 1, 0, r).1, 45);
        assert_eq!(clear_rewards_in(5, 301, 0, &b, &cells, 1, 0, r).1, 36);
        assert_eq!(
            clear_rewards_in(3, health_cap(3, true) - 1, 0, &[[1; N]; N], &cells, 1, 0, r).2,
            8
        );
        assert_eq!(skill_cost(0, true, Skill::Guard), 80);
        assert_eq!(skill_cost(2, true, Skill::Meteor), 140);
        assert!(payment_in(2, 200, 139, Skill::Meteor, r).is_none());
        assert_eq!(payment_in(2, 200, 140, Skill::Meteor, r).unwrap().mana, 140);
        let p = preview_rewards_in(Skill::Guard, &board(), (0, 0), 0, 380, 80, 500, 0, r);
        assert_eq!((p.health_after, p.heal, p.mana_after), (385, 5, 0));
        assert_eq!(
            effect_at_in(Skill::Charge, &mut b.clone(), (0, 0), 0, 385, 100, r).damage,
            315
        );
    }
    #[test]
    fn awakened_guard_heals_its_new_cap_and_preserves_tile_consumption() {
        let r = rules(true, 0, true);
        let b = board();
        let p = preview_rewards_in(Skill::Guard, &b, (0, 0), 0, 200, 80, 385, 0, r);
        assert_eq!((p.health_after, p.heal, p.mana_after), (277, 77, 0));
        let e = effect_at_in(Skill::Guard, &mut b.clone(), (0, 0), 0, 200, 80, r);
        assert_eq!(e.hp, 77);
        assert!(!e.reward);
        assert!(e.cells.iter().all(|&(x, y)| b[y][x] == 1));
        assert!(payment_in(0, 200, 79, Skill::Guard, r).is_none());
    }
    #[test]
    fn loot_boost_applies_only_to_its_first_clear_and_mana_still_caps() {
        let r = rules(true, 0, true);
        let b = [[2; N]; N];
        let p = preview_rewards_in(Skill::Greedy, &b, (0, 0), 3, 300, 500, 700, 0, r);
        assert_eq!(p.damage, 93); // 24*1.3 floor + 48*1.3 floor
        let cells = vec![(0, 0), (1, 0), (0, 1), (1, 1)];
        assert_eq!(clear_rewards_in(3, 300, 0, &b, &cells, 1, 0, r).1, 48);
        assert_eq!(
            clear_rewards_in(
                3,
                300,
                0,
                &b,
                &cells,
                1,
                0,
                Rules {
                    initial_loot: true,
                    ..r
                }
            )
            .1,
            62
        );
        let p = preview_rewards_in(Skill::Meteor, &[[3; N]; N], (0, 0), 2, 200, 400, 700, 0, r);
        assert_eq!(p.mana_after, 400);
        assert_eq!(p.mana_gain, 140);
    }
    #[test]
    fn wizard_full_mana_heals_normally_and_checks_phase_cap_before_clear() {
        let b = [[1; N]; N];
        let cells = vec![(0, 0), (1, 0), (2, 0)];
        for (awake, cap) in [(false, 250), (true, 400)] {
            let r = rules(awake, 0, true);
            assert_eq!(
                clear_rewards_in(2, 200, cap, &b, &cells, 1, 0, r),
                (18, 0, 0)
            );
            assert_eq!(
                clear_rewards_in(2, 200, cap - 1, &b, &cells, 1, 0, r),
                (9, 0, 9)
            );
            // Filling MP in this wave still uses the below-cap rule for that wave.
            let mut swap = board();
            swap[0][..3].copy_from_slice(&[1, 0, 1]);
            swap[1][1] = 1;
            let original = swap;
            let seed = crate::rng::state();
            let full =
                preview_move_in(&swap, 2, 200, cap, 700, 0, false, (1, 0), (1, 1), r).unwrap();
            let low =
                preview_move_in(&swap, 2, 200, cap - 1, 700, 0, false, (1, 0), (1, 1), r).unwrap();
            assert_eq!((full.heal, full.mana_gain), (18, 0));
            assert_eq!((low.heal, low.mana_gain), (9, 1));
            assert_eq!(swap, original);
            assert_eq!(crate::rng::state(), seed);
        }
    }
    #[test]
    fn reduced_reserves_cover_each_roles_largest_skill() {
        assert_eq!(
            HEROES.iter().map(|h| h.mp).collect::<Vec<_>>(),
            [200, 120, 400, 160, 120, 160, 100]
        );
        for hero in 0..7 {
            for awake in [false, true] {
                for &skill in HEROES[hero].skills {
                    assert!(skill_cost(hero, awake, skill) <= mana_cap(hero, awake));
                }
            }
        }
    }
    #[test]
    fn wizard_reserve_caps_skill_and_swap_rewards_in_both_phases() {
        let b = [[3; N]; N];
        let original = b;
        let seed = crate::rng::state();
        for (awake, cap) in [(false, 250), (true, 400)] {
            assert_eq!(mana_cap(2, awake), cap);
            let r = rules(awake, 0, true);
            assert!(payment_in(2, 200, 59, Skill::Lightning, r).is_none());
            assert_eq!(
                payment_in(2, 200, 60, Skill::Lightning, r),
                Some(Payment { mana: 60, blood: 0 })
            );
            let p = preview_rewards_in(Skill::Lightning, &b, (0, 0), 2, 250, cap - 10, 700, 0, r);
            assert_eq!(p.mana_after, cap);
            assert_eq!(p.mana_gain, 10 + Skill::Lightning.cost());
            let mut swapped = board();
            swapped[0][..3].copy_from_slice(&[3, 0, 3]);
            swapped[1][1] = 3;
            let p = preview_move_in(&swapped, 2, 250, cap - 1, 700, 0, false, (1, 0), (1, 1), r)
                .unwrap();
            assert_eq!((p.mana_gain, p.mana_after), (1, cap));
        }
        for hero in [0, 1, 3, 4, 5, 6] {
            assert_eq!(mana_cap(hero, false), HEROES[hero].mp);
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn awakened_flesh_casting_rounds_up_and_requires_survival() {
        let r = rules(true, 0, true);
        for (mp, blood) in [(0, 25), (1, 25), (25, 13), (47, 2), (49, 1), (50, 0)] {
            let pay = payment_in(6, 100, mp, Skill::Drain, r).unwrap();
            assert_eq!(
                pay,
                Payment {
                    mana: mp.min(50),
                    blood
                }
            );
            if blood > 0 {
                assert!(payment_in(6, blood, mp, Skill::Drain, r).is_none());
                assert!(payment_in(6, blood + 1, mp, Skill::Drain, r).is_some());
            }
        }
        assert_eq!(
            payment_in(6, 101, 0, Skill::Drain, rules(false, 0, true))
                .unwrap()
                .blood,
            100
        );
        assert!(payment_in(0, 100, 0, Skill::Drain, r).is_none());
    }
    #[test]
    fn awakened_flesh_preview_matches_payment_effect_targets_and_caps() {
        let b = board();
        let original = b;
        let seed = crate::rng::state();
        let r = rules(true, 0, true);
        let cap = health_cap(6, true);
        let mut damage = Vec::new();
        for target in [(0, 0), (1, 0), (2, 0)] {
            let p = preview_rewards_in(Skill::Drain, &b, target, 6, cap, 25, 700, 0, r);
            let mut copy = b;
            let e = effect_at_in(Skill::Drain, &mut copy, target, 6, cap, 25, r);
            assert_eq!(p.blood_cost, 13);
            assert_eq!(p.health_after, (cap - 13 + e.hp).min(cap));
            assert_eq!(p.mana_after, 0);
            assert_eq!(p.damage, e.damage);
            assert!(!p.awakens_self);
            damage.push(p.damage);
        }
        assert!(damage.windows(2).any(|w| w[0] != w[1]));
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn payment_can_awaken_before_healing_and_boost_that_same_skill() {
        let b = board();
        let original = b;
        let seed = crate::rng::state();
        let r = rules(false, 0, false);
        let p = preview_rewards_in(Skill::Drain, &b, (0, 0), 6, 250, 25, 700, 0, r);
        let e = effect_at_in(
            Skill::Drain,
            &mut b.clone(),
            (0, 0),
            6,
            250,
            25,
            Rules { awake: true, ..r },
        );
        assert!(p.awakens_self);
        assert_eq!(p.blood_cost, 50);
        assert_eq!(p.health_after, health_cap(6, true));
        assert_eq!(p.damage, e.damage);
        assert_eq!(p.mana_after, 0);
        assert_eq!(payment_in(6, 100, 0, Skill::Drain, r), None); // still must be payable
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
    #[test]
    fn previews_simulate_opponent_awakening_and_subsequent_damage_without_mutation() {
        let b = board();
        let original = b;
        let seed = crate::rng::state();
        let r = rules(true, 0, false);
        let p = preview_rewards_in(Skill::Charge, &b, (0, 0), 0, 385, 300, 330, 0, r);
        assert!(p.awakens_enemy);
        assert_eq!(p.enemy_health_after, 385);
        let p2 = preview_rewards_in(
            Skill::Charge,
            &b,
            (0, 0),
            0,
            385,
            300,
            330,
            0,
            rules(true, 0, true),
        );
        assert!(!p2.awakens_enemy);
        assert!(p2.enemy_health_after < 330);
        let (low, high) = preview_range_in(Skill::Charge, &b, 0, 385, 300, 700, 0, r);
        for y in 0..N {
            for x in 0..N {
                let p = preview_rewards_in(Skill::Charge, &b, (x, y), 0, 385, 300, 700, 0, r);
                assert!((low.damage..=high.damage).contains(&p.damage));
                assert!((low.enemy_health_after..=high.enemy_health_after)
                    .contains(&p.enemy_health_after));
            }
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
}

/// Persistent indicators for conditional passives; phase itself has no badge.
pub fn passive_status(
    hero: usize,
    hp: i32,
    mp: i32,
    awake: bool,
    zh: bool,
) -> Option<(&'static str, bool)> {
    if hero == 5 {
        let passive = if has_passive(hero, awake, Passive::BoilingBlood) {
            Passive::BoilingBlood
        } else {
            Passive::BloodFury
        };
        return passive.status(hero, hp, mp, awake, zh);
    }
    if hero == 6 && hp > 0 {
        let missing = HEROES[hero]
            .skills
            .iter()
            .copied()
            .filter(|&s| mp < skill_cost(hero, awake, s))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            let active = missing.into_iter().any(|s| {
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
                .is_some()
            });
            return Some((
                if active {
                    if zh {
                        "可代付"
                    } else {
                        "HP ready"
                    }
                } else if zh {
                    "代付不足"
                } else {
                    "HP too low"
                },
                active,
            ));
        }
    }
    None
}
#[cfg(test)]
mod passive_status_tests {
    use super::*;
    #[test]
    fn badges_follow_health_thresholds_and_real_payment_capacity() {
        assert_eq!(
            passive_status(5, 210, 0, false, true),
            Some(("伤害+25%", true))
        );
        assert_eq!(
            passive_status(5, 211, 0, false, true),
            Some(("未触发", false))
        );
        assert_eq!(
            passive_status(5, 270, 0, true, true),
            Some(("伤害+25%", true))
        );
        assert_eq!(
            passive_status(6, 101, 0, false, true),
            Some(("可代付", true))
        );
        assert_eq!(
            passive_status(6, 100, 0, false, true),
            Some(("代付不足", false))
        );
        assert_eq!(passive_status(6, 200, 80, true, true), None);
        assert_eq!(passive_status(0, 100, 0, false, true), None);
    }
}

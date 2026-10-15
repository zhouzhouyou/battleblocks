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
            Self::Lightning => 40,
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
                "Spend 80 mana to heal 100.\nConsumes your turn.",
                "消耗80魔力回复100生命\n释放后结束回合",
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
            Self::Guard => ("Restore 20% of max HP.\nConsume pure healing tiles.\nRefill; resolve cascades.","回复最大生命值的20%\n清除纯治疗，不额外回血\n掉落后结算三消，再换人"),
            Self::Landslide => (
                "Next adjacent swap also\nswaps both parallel pairs.\nNo match required.",
                "下次相邻交换附带两侧交换\n同时交换三组平行方块\n不要求交换后形成三消",
            ),
            Self::Strike => (
                "Next rewarded clear gets\n+100% rewards.\nOne cast per exchange.",
                "下次有收益的消除收益+100%\n增加治疗、伤害和魔力\n每次交换前只能释放一次",
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
                "Clear all balanced tiles.\nPer tile: 16 damage, 4 mana.\nNew cascades give rewards.",
                "清除所有平衡方块\n每格造成16伤害、回复4魔力\n后续下落三消正常结算",
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
        mp: 300,
        skills: &[Skill::Charge, Skill::Guard],
    },
    Hero {
        en: "Elf",
        zh: "精灵",
        hp: 600,
        mp: 500,
        skills: &[Skill::Strike, Skill::Nature],
    },
    Hero {
        en: "Wizard",
        zh: "巫师",
        hp: 550,
        mp: 800,
        skills: &[Skill::Lightning, Skill::Meteor],
    },
    Hero {
        en: "Goblin",
        zh: "哥布林",
        hp: 700,
        mp: 500,
        skills: &[Skill::Greedy, Skill::Blood],
    },
    Hero {
        en: "Imp",
        zh: "小恶魔",
        hp: 700,
        mp: 500,
        skills: &[Skill::Imp, Skill::Balance],
    },
    Hero {
        en: "Chort",
        zh: "角魔",
        hp: 600,
        mp: 700,
        skills: &[Skill::Chort, Skill::War],
    },
    Hero {
        en: "Big Zombie",
        zh: "大僵尸",
        hp: 750,
        mp: 400,
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
                        e.damage += 16;
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
    pub heal: i32,
    pub damage: i32,
    pub mana_gain: i32,
    pub mana_after: i32,
}
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
    let mut copy = *board;
    let e = effect_at(skill, &mut copy, target);
    let mut health = (hp + e.hp).min(HEROES[hero].hp);
    let mut opponent = (enemy_hp - e.damage).max(0);
    let spent = mp - skill.cost();
    let mut mana = (spent + e.mp).min(HEROES[hero].mp);
    let (cells, rewarded) = if e.cells.is_empty() {
        (crate::matches(&copy), true)
    } else {
        (e.cells.clone(), e.reward)
    };
    if rewarded && !cells.is_empty() {
        let multiplier = reward_scale(1, bonus + e.bonus);
        let (mut h, mut d, mut m) = (0, 0, 0);
        for &(x, y) in &cells {
            let v = VALUES[copy[y][x] as usize];
            h += v.0;
            d += v.1;
            m += v.2;
        }
        health = (health + h * multiplier / 100).min(HEROES[hero].hp);
        opponent = (opponent - d * multiplier / 100).max(0);
        mana = (mana + m * multiplier / 100).min(HEROES[hero].mp);
    }
    SkillPreview {
        heal: health - hp,
        damage: enemy_hp - opponent,
        mana_gain: mana - spent,
        mana_after: mana,
    }
}
pub fn preview_range(
    skill: Skill,
    board: &Board,
    hero: usize,
    hp: i32,
    mp: i32,
    enemy_hp: i32,
    bonus: i32,
) -> (SkillPreview, SkillPreview) {
    let mut low = preview_rewards(skill, board, (0, 0), hero, hp, mp, enemy_hp, bonus);
    let mut high = preview_rewards(skill, board, (0, 0), hero, hp, mp, enemy_hp, bonus);
    for y in 0..N {
        for x in 0..N {
            let p = preview_rewards(skill, board, (x, y), hero, hp, mp, enemy_hp, bonus);
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
        assert_eq!(p.mana_gain, 2 * N as i32 * VALUES[3].2);
        assert_eq!(
            p.mana_after,
            200 - Skill::Lightning.cost() + 2 * N as i32 * VALUES[3].2
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

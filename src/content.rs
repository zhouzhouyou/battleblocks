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
        hp: 725,
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
        hp: 775,
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
    pub blood_cost: i32,
    pub health_after: i32,
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
    let pay = payment(hero, hp, mp, skill).unwrap_or(Payment {
        mana: skill.cost().min(mp),
        blood: 0,
    });
    let mut health = (hp - pay.blood + e.hp).min(HEROES[hero].hp);
    let mut opponent = (enemy_hp - e.damage).max(0);
    let spent = mp - pay.mana;
    let mut mana = (spent + e.mp).min(HEROES[hero].mp);
    let (cells, rewarded) = if e.cells.is_empty() {
        (crate::matches(&copy), true)
    } else {
        (e.cells.clone(), e.reward)
    };
    if rewarded && !cells.is_empty() {
        let (h, d, m) = clear_rewards(hero, health, mana, &copy, &cells, 1, bonus + e.bonus);
        health = (health + h).min(HEROES[hero].hp);
        opponent = (opponent - d).max(0);
        mana = (mana + m).min(HEROES[hero].mp);
    }
    SkillPreview {
        heal: health - hp,
        damage: enemy_hp - opponent,
        mana_gain: mana - spent,
        mana_after: mana,
        blood_cost: pay.blood,
        health_after: health,
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

/// Stateless passives: same calculation for game, previews and AI.
pub fn clear_rewards(
    hero: usize,
    hp: i32,
    _mp: i32,
    board: &Board,
    cells: &[(usize, usize)],
    chain: usize,
    bonus: i32,
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
        if hero == 4 && board[y][x] == 0 {
            d += 2;
            m += 2;
        }
    }
    let scale = reward_scale(chain, bonus);
    h = h * scale / 100;
    d = d * scale / 100;
    m = m * scale / 100;
    if hero == 0 {
        h = h * 106 / 100;
        d = d * 106 / 100;
        m = m * 106 / 100;
    }
    if hero == 1 {
        h = h * 93 / 100;
        d = d * 93 / 100;
        m = m * 93 / 100;
    }
    if hero == 2 {
        let converted = h / 2;
        h -= converted;
        m += converted;
    }
    if hero == 3 {
        m += (hp + h - HEROES[hero].hp).max(0) / 2;
    }
    if hero == 5 && hp * 100 <= HEROES[hero].hp * 35 {
        d = d * 125 / 100;
    }
    (h, d, m)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Payment {
    pub mana: i32,
    pub blood: i32,
}
pub fn payment(hero: usize, hp: i32, mp: i32, skill: Skill) -> Option<Payment> {
    let missing = (skill.cost() - mp).max(0);
    let blood = missing * 2;
    if missing == 0 || (hero == 6 && hp > blood) {
        Some(Payment {
            mana: skill.cost().min(mp),
            blood,
        })
    } else {
        None
    }
}
pub fn passive(hero: usize, zh: bool) -> &'static str {
    let (en,cn)=match hero {
        0=>("Column formation: horizontal\nswaps move two whole columns.\nClear rewards +6%.","列阵：左右交换相邻整列\n上下交换仍为两个方块\n消除收益增加6%"),
        1=>("Forest step: diagonal adjacent\nswaps are also allowed.\nClear rewards reduced by 7%.","林间穿行：可斜向交换\n仅交换斜向相邻的两格\n横纵三消，消除收益降低7%"),
        2=>("Arcane conversion: half of\nclear healing becomes mana.\nDirect skill healing is unchanged.","奥术转化：消除治疗的一半\n等量转为魔力（向下取整）\n技能直接治疗不变"),
        3=>("Scrap recovery: overhealing\nfrom clears becomes mana at 2:1.\nMana still obeys the MP cap.","废物回收：消除溢出治疗\n每2点转为1点魔力\n魔力仍受上限限制"),
        4=>("Balanced affinity: each balanced\ntile in a rewarded clear gives\n+2 damage and +2 mana.","平衡亲和：有收益消除时\n每个平衡方块额外提供\n2点伤害、2点魔力"),
        5=>("Blood fury: at 35% HP or less,\nclear damage increases by 25%.\nChecked before each clear wave.","血怒：生命不高于35%时\n消除伤害增加25%\n每轮消除前判断生命"),
        _=>("Flesh casting: missing mana\ncosts 2 HP per point.\nMust leave at least 1 HP.","血肉施法：魔力不足时\n每缺1魔力支付2生命\n必须至少剩余1生命"),
    };
    if zh {
        cn
    } else {
        en
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
            (6, 18, 18)
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
            (12, 37, 37)
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
            let mp = HEROES[id].mp;
            let p = preview_rewards(skill, &b, (0, 0), id, hp, mp, 700, 0);
            let mut copy = b;
            let e = effect_at(skill, &mut copy, (0, 0));
            let health = (hp + e.hp).min(HEROES[id].hp);
            let mana = mp - skill.cost() + e.mp;
            let (h, d, m) = clear_rewards(id, health, mana, &copy, &crate::matches(&copy), 1, 0);
            assert_eq!(p.health_after, (health + h).min(HEROES[id].hp));
            assert_eq!(p.damage, (e.damage + d).min(700));
            assert_eq!(p.mana_after, (mana + m).min(HEROES[id].mp));
        }
    }
}

/// Actual actions that can continue a no-swap turn. A preparation which only
/// multiplies future rewards does not by itself unlock a dead board.
pub fn usable_skills(board: &Board, hero: usize, hp: i32, mp: i32) -> Vec<Skill> {
    HEROES[hero]
        .skills
        .iter()
        .copied()
        .filter(|&skill| {
            let Some(pay) = payment(hero, hp, mp, skill) else {
                return false;
            };
            (0..N).any(|y| {
                (0..N).any(|x| {
                    let mut copy = *board;
                    let e = effect_at(skill, &mut copy, (x, y));
                    !e.cells.is_empty()
                        || copy != *board
                        || e.armed
                        || e.damage > 0
                        || (e.hp > pay.blood && hp - pay.blood < HEROES[hero].hp)
                        || (mp - pay.mana + e.mp).min(HEROES[hero].mp) > mp
                })
            })
        })
        .collect()
}
pub fn must_refresh(board: &Board, hero: usize, hp: i32, mp: i32, armed: bool) -> bool {
    !armed
        && crate::board::hero_options(board, hero).is_empty()
        && usable_skills(board, hero, hp, mp).is_empty()
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

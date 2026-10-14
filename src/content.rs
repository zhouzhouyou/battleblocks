use super::{Board, N};
use macroquad::rand;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Skill {
    Blade,
    Landslide,
    Strike,
    Nature,
    Lightning,
    Meteor,
    Greedy,
    Imp,
    Chort,
    Corrupt,
}
impl Skill {
    pub fn cost(self) -> i32 {
        match self {
            Self::Blade => 100,
            Self::Landslide => 200,
            Self::Strike => 200,
            Self::Nature => 125,
            Self::Lightning => 150,
            Self::Meteor => 600,
            Self::Greedy => 50,
            Self::Imp => 100,
            Self::Chort => 200,
            Self::Corrupt => 400,
        }
    }
    pub fn name(self, zh: bool) -> &'static str {
        let names = match self {
            Self::Blade => ("Blade slash", "剑刃斩击"),
            Self::Landslide => ("Landslide", "山崩打击"),
            Self::Strike => ("Sinister strike", "阴险打击"),
            Self::Nature => ("Nature's force", "自然之力"),
            Self::Lightning => ("Lightning", "闪电"),
            Self::Meteor => ("Meteor", "陨石"),
            Self::Greedy => ("Greedy", "贪婪"),
            Self::Imp => ("Imp's pact", "小恶魔契约"),
            Self::Chort => ("Chort's pact", "角魔契约"),
            Self::Corrupt => ("Corrupt", "腐化"),
        };
        if zh {
            names.1
        } else {
            names.0
        }
    }
    pub fn description(self, zh: bool) -> &'static str {
        let names = match self {
            Self::Blade => (
                "Clear 2 adjacent columns.\nDeal their attack value.\nNo healing or mana gain.",
                "随机清除相邻两列（16格）\n按方块伤害值攻击敌人\n不回复生命或魔力",
            ),
            Self::Landslide => (
                "Next adjacent swap also\nswaps both parallel pairs.\nNo match required.",
                "下次相邻交换附带两侧交换\n同时交换三组平行方块\n不要求交换后形成三消",
            ),
            Self::Strike => (
                "Next rewarded clear gets\n+1 reward multiplier.\nRepeat casts stack.",
                "下次有收益的消除倍率+1\n增加治疗、伤害和魔力\n重复释放可叠加",
            ),
            Self::Nature => (
                "Change 4 random tiles\nto balanced tiles.\nResolve any new matches.",
                "随机四格变成平衡方块\n平衡方块含三种收益\n形成三消时自动结算",
            ),
            Self::Lightning => (
                "Clear both diagonals.\nGain half their mana.\nNo healing or damage.",
                "清除两条对角线（16格）\n回复方块魔力值总和的一半\n不治疗、不造成伤害",
            ),
            Self::Meteor => (
                "Clear central diamond\n(40 tiles); gain their\nattack and mana values.",
                "清除中央菱形区域（40格）\n按方块数值造成伤害\n回复魔力，不回复生命",
            ),
            Self::Greedy => (
                "Clear random tiles.\nGain normal rewards;\nmore tiles = higher gain.",
                "随机清除若干不重复方块\n获得正常三种消除收益\n清除越多，收益倍率越高",
            ),
            Self::Imp => (
                "Clear all balanced tiles.\nNo immediate rewards.\nNew cascades give rewards.",
                "清除所有平衡方块\n本次清除不获得收益\n后续下落三消正常结算",
            ),
            Self::Chort => (
                "Clear one random type.\nPer tile: heal 3, deal 3,\nand restore 3 mana.",
                "随机选一种方块，全部清除\n每格治疗3、造成3伤害\n并回复3魔力",
            ),
            Self::Corrupt => (
                "All pure attack tiles\nbecome healing tiles.\nResolve any new matches.",
                "纯伤害方块全部变为治疗\n混合类型方块不受影响\n形成三消时自动结算",
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
        skills: &[Skill::Blade, Skill::Landslide],
    },
    Hero {
        en: "Elf",
        zh: "精灵",
        hp: 550,
        mp: 500,
        skills: &[Skill::Strike, Skill::Nature],
    },
    Hero {
        en: "Wizard",
        zh: "巫师",
        hp: 500,
        mp: 800,
        skills: &[Skill::Lightning, Skill::Meteor],
    },
    Hero {
        en: "Goblin",
        zh: "哥布林",
        hp: 600,
        mp: 500,
        skills: &[Skill::Greedy],
    },
    Hero {
        en: "Imp",
        zh: "小恶魔",
        hp: 400,
        mp: 500,
        skills: &[Skill::Imp],
    },
    Hero {
        en: "Chort",
        zh: "角魔",
        hp: 600,
        mp: 700,
        skills: &[Skill::Chort],
    },
    Hero {
        en: "Big Zombie",
        zh: "大僵尸",
        hp: 800,
        mp: 400,
        skills: &[Skill::Corrupt],
    },
];
pub const VALUES: [(i32, i32, i32); 7] = [
    (3, 3, 3),
    (10, 0, 0),
    (0, 10, 0),
    (0, 0, 10),
    (5, 5, 0),
    (5, 0, 5),
    (0, 5, 5),
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
    let mut e = Effect::default();
    match skill {
        Skill::Blade => {
            let x = rand::gen_range(0, N - 1);
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
            let mut cells = Vec::new();
            while cells.len() < 4 {
                let c = (rand::gen_range(0, N), rand::gen_range(0, N));
                if !cells.contains(&c) {
                    cells.push(c);
                }
            }
            for (x, y) in cells {
                b[y][x] = 0;
            }
        }
        Skill::Corrupt => {
            for row in b.iter_mut() {
                for t in row {
                    if *t == 2 {
                        *t = 1;
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
            e.mp /= 2;
        }
        Skill::Meteor => {
            for x in 0..N {
                let c = x.min(N - 1 - x);
                for y in N / 2 - 1 - c..=N / 2 + c {
                    e.cells.push((x, y));
                    let v = VALUES[b[y][x] as usize];
                    e.damage += v.1;
                    e.mp += v.2;
                }
            }
        }
        Skill::Greedy => {
            e.reward = true;
            loop {
                let c = (rand::gen_range(0, N), rand::gen_range(0, N));
                if !e.cells.contains(&c) {
                    e.cells.push(c);
                }
                if e.cells.len() == N * N || rand::gen_range(0, 7) >= 5 {
                    break;
                }
            }
        }
        Skill::Imp => {
            for y in 0..N {
                for x in 0..N {
                    if b[y][x] == 0 {
                        e.cells.push((x, y));
                    }
                }
            }
        }
        Skill::Chort => {
            let kind = rand::gen_range(0, 7);
            for y in 0..N {
                for x in 0..N {
                    if b[y][x] == kind {
                        e.cells.push((x, y));
                    }
                }
            }
            let n = e.cells.len() as i32 * 3;
            e.hp = n;
            e.damage = n;
            e.mp = n;
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
        let blade = effect(Skill::Blade, &mut b);
        assert_eq!(blade.cells.len(), 16);
        assert_eq!(blade.damage, 160);
        assert!(!blade.reward);
        let lightning = effect(Skill::Lightning, &mut [[3; N]; N]);
        assert_eq!(lightning.cells.len(), 16);
        assert_eq!(lightning.mp, 80);
        let meteor = effect(Skill::Meteor, &mut b);
        assert_eq!(meteor.cells.len(), 40);
        assert_eq!(meteor.damage, 400);
        let imp = effect(Skill::Imp, &mut [[0; N]; N]);
        assert_eq!(imp.cells.len(), 64);
        assert!(!imp.reward);
        assert!(effect(Skill::Landslide, &mut b).armed);
        assert_eq!(effect(Skill::Strike, &mut b).bonus, 1);
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
            Skill::Blade,
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

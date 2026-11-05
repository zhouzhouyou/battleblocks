use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    Rush,
    Miracle,
    Hammer,
    Jump,
    Forge,
    Entomb,
    Resurrect,
    Copy,
    Frenzy,
    Healing,
    Rational,
    EmberBlast,
    PortalPair,
    GravityShift,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileOperation {
    Eliminate,
    Destroy,
    Transform,
    Store,
    Insert,
    Prepare,
    Heal,
    Copy,
}
impl TileOperation {
    fn parse(value: &str) -> Self {
        match value {
            "eliminate" => Self::Eliminate,
            "destroy" => Self::Destroy,
            "transform" => Self::Transform,
            "store" => Self::Store,
            "insert" => Self::Insert,
            "prepare" => Self::Prepare,
            "heal" => Self::Heal,
            "copy" => Self::Copy,
            _ => panic!("unknown tile operation: {value}"),
        }
    }
}
impl Skill {
    pub fn operation(self) -> TileOperation {
        self.definition().operation
    }
    pub fn cost(self) -> i32 {
        self.definition().cost
    }
    pub const ALL: &[Skill] = &[
        Skill::Charge,
        Skill::Guard,
        Skill::Landslide,
        Skill::Strike,
        Skill::Nature,
        Skill::Lightning,
        Skill::Meteor,
        Skill::Greedy,
        Skill::Imp,
        Skill::Chort,
        Skill::Corrupt,
        Skill::Blood,
        Skill::Balance,
        Skill::War,
        Skill::Drain,
        Skill::Rush,
        Skill::Miracle,
        Skill::Hammer,
        Skill::Jump,
        Skill::Forge,
        Skill::Entomb,
        Skill::Resurrect,
        Skill::Copy,
        Skill::Frenzy,
        Skill::Healing,
        Skill::Rational,
        Skill::EmberBlast,
        Skill::PortalPair,
        Skill::GravityShift,
    ];
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
    fn definition(self) -> &'static SkillDefinition {
        static DEFINITIONS: std::sync::OnceLock<Vec<SkillDefinition>> = std::sync::OnceLock::new();
        let all = DEFINITIONS.get_or_init(|| {
            include_str!("../../config/skills.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.is_empty())
                .map(|line| {
                    let fields: Vec<_> = line.split('|').collect();
                    assert_eq!(fields.len(), 11, "invalid skill configuration: {line}");
                    SkillDefinition {
                        cost: fields[9].parse().expect("invalid cost"),
                        operation: TileOperation::parse(fields[10]),
                        id: fields[0],
                        en: fields[1],
                        zh: fields[2],
                        description_en: fields[3].replace("\\n", "\n"),
                        description_zh: fields[4].replace("\\n", "\n"),
                        cooldown: fields[5].parse().expect("invalid cooldown"),
                        ends_turn: fields[6].parse().expect("invalid end-turn flag"),
                        awake_en: fields[7].replace("\\n", "\n"),
                        awake_zh: fields[8].replace("\\n", "\n"),
                    }
                })
                .collect()
        });
        let id = format!("{self:?}");
        all.iter()
            .find(|entry| entry.id == id)
            .expect("missing skill configuration")
    }
    pub fn name(self, zh: bool) -> &'static str {
        let d = self.definition();
        if zh {
            d.zh
        } else {
            d.en
        }
    }
    pub fn description(self, zh: bool) -> &'static str {
        let d = self.definition();
        if zh {
            &d.description_zh
        } else {
            &d.description_en
        }
    }
    pub fn cooldown(self) -> u8 {
        self.definition().cooldown
    }
    pub fn ends_turn(self) -> bool {
        self.definition().ends_turn
    }
    pub fn index(self) -> usize {
        self as usize
    }
}
struct SkillDefinition {
    operation: TileOperation,
    cost: i32,
    id: &'static str,
    en: &'static str,
    zh: &'static str,
    description_en: String,
    description_zh: String,
    cooldown: u8,
    ends_turn: bool,
    awake_en: String,
    awake_zh: String,
}

/// Per-side cooldowns, in the owner's turns. A zero value is immediately usable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cooldowns(pub [[u8; Skill::ALL.len()]; 2]);
impl Default for Cooldowns {
    fn default() -> Self {
        Self([[0; Skill::ALL.len()]; 2])
    }
}
impl Cooldowns {
    pub fn remaining(&self, side: usize, skill: Skill) -> u8 {
        self.0[side][skill.index()]
    }
    pub fn cast(&mut self, side: usize, skill: Skill) {
        self.0[side][skill.index()] = skill.cooldown();
    }
    pub fn start_turn(&mut self, side: usize) {
        for remaining in &mut self.0[side] {
            *remaining = remaining.saturating_sub(1);
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
pub static HEROES: std::sync::LazyLock<Vec<Hero>> = std::sync::LazyLock::new(|| {
    include_str!("../../config/heroes.txt")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|line| {
            let f: Vec<_> = line.split('|').collect();
            assert_eq!(f.len(), 5);
            let skills = f[4]
                .split(',')
                .map(|id| Skill::from_index(id.parse().expect("skill id")).expect("skill"))
                .collect::<Vec<_>>();
            Hero {
                en: f[0],
                zh: f[1],
                hp: f[2].parse().expect("HP"),
                mp: f[3].parse().expect("MP"),
                skills: Box::leak(skills.into_boxed_slice()),
            }
        })
        .collect()
});
pub fn skills(hero: usize, awake: bool) -> &'static [Skill] {
    let configured = ability_loadout(hero).awake_skills;
    if awake && !configured.is_empty() {
        configured
    } else {
        HEROES[hero].skills
    }
}

pub fn effective_skill(skill: Skill, rules: Rules) -> Skill {
    let mut value = skill;
    dispatch_active(
        skill,
        &EventContext::new(0, 0, 0, rules),
        &mut RuleEvent::ResolveSkill { value: &mut value },
        &mut Vec::new(),
    );
    value
}

pub fn skill_cost_in(hero: usize, skill: Skill, rules: Rules) -> i32 {
    let effective = effective_skill(skill, rules);
    // Use the full rules here: a copied active must see copied_skill and its own
    // dynamic cost, which the awake-only `skill_cost` helper cannot resolve.
    let mut base = effective.cost();
    dispatch_rules(
        &EventContext::new(hero, 0, 0, rules),
        &mut RuleEvent::SkillCost {
            skill: effective,
            value: &mut base,
        },
        &mut Vec::new(),
    );
    let repeat = rules.repeat_casts[effective.index()] as i32;
    if effective.cooldown() == 0 && !effective.ends_turn() {
        base + repeat * 15
    } else {
        base
    }
}
pub fn skill_name_in(skill: Skill, rules: Rules, zh: bool) -> String {
    if effective_skill(skill, rules) != skill {
        format!(
            "{} · {}",
            skill.name(zh),
            effective_skill(skill, rules).name(zh)
        )
    } else {
        skill.name(zh).to_string()
    }
}
pub fn skill_description_in(skill: Skill, rules: Rules, zh: bool) -> &'static str {
    skill_description(effective_skill(skill, rules), rules.awake, zh)
}
pub fn role_text(key: &str, zh: bool) -> &'static str {
    static TEXT: std::sync::OnceLock<Vec<(&'static str, String, String)>> =
        std::sync::OnceLock::new();
    let entries = TEXT.get_or_init(|| {
        include_str!("../../config/role-text.txt")
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|line| {
                let f: Vec<_> = line.split('|').collect();
                assert_eq!(f.len(), 3);
                (f[0], f[1].replace("\\n", "\n"), f[2].replace("\\n", "\n"))
            })
            .collect()
    });
    let entry = entries
        .iter()
        .find(|(k, _, _)| *k == key)
        .expect("missing role text");
    if zh {
        &entry.2
    } else {
        &entry.1
    }
}
pub fn awakening_effect(hero: usize, zh: bool) -> &'static str {
    let key = ability_loadout(hero).awakening;
    if key.starts_with("hero.") {
        crate::tr_key(zh, key)
    } else {
        role_text(key, zh)
    }
}

pub fn skill_description(skill: Skill, awake: bool, zh: bool) -> &'static str {
    let d = skill.definition();
    let awakened = if zh { &d.awake_zh } else { &d.awake_en };
    if awake && !awakened.is_empty() {
        awakened
    } else {
        skill.description(zh)
    }
}

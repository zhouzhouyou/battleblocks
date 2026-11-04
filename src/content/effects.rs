use super::*;
pub const VALUES: [(i32, i32, i32); 7] = [
    (2, 4, 4),
    (6, 0, 0),
    (0, 12, 0),
    (0, 0, 12),
    (3, 7, 0),
    (3, 0, 7),
    (0, 7, 7),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardChange {
    Portals(crate::board::PortalPair),
    Gravity(crate::board::Gravity),
}
impl BoardChange {
    pub fn apply(
        self,
        board: &mut crate::board::BoardMechanics,
        owner: bool,
    ) -> Result<(), &'static str> {
        match self {
            Self::Portals(mut pair) => {
                pair.owner = owner;
                board.place(pair)
            }
            Self::Gravity(g) => {
                board.set_gravity(g, owner);
                Ok(())
            }
        }
    }
}
#[derive(Default)]
pub struct Effect {
    pub board_change: Option<BoardChange>,
    pub cells: Vec<(usize, usize)>,
    pub hp: i32,
    pub opponent_heal: i32,
    pub damage: i32,
    pub mp: i32,
    pub reward: bool,
    pub bonus: i32,
    pub armed: bool,
    pub triggers: Vec<AbilityTrigger>,
    pub events: Option<RewardBatch>,
}
impl Effect {
    pub fn reward_batch(&self, skill: Skill) -> RewardBatch {
        let mut batch = self.events.clone().unwrap_or_else(|| {
            RewardBatch::gains(
                (self.hp, self.damage, self.mp),
                self.opponent_heal,
                EventSource::Skill(skill),
            )
        });
        batch.set_totals((self.hp, self.damage, self.mp));
        batch.set_total(Resource::Healing, EventSide::Opponent, self.opponent_heal);
        batch.triggers.extend(self.triggers.clone());
        batch
    }
}
pub fn effect(skill: Skill, b: &mut Board) -> Effect {
    effect_at(skill, b, (3, 3))
}
pub fn targeted(skill: Skill) -> bool {
    let mut value = false;
    dispatch_active(
        skill,
        &EventContext::new(0, HEROES[0].hp, 0, Rules::default()),
        &mut RuleEvent::Targeted {
            skill,
            value: &mut value,
        },
        &mut Vec::new(),
    );
    value
}
#[cfg(test)]
pub fn target_cells(skill: Skill, b: &Board, target: (usize, usize)) -> Vec<(usize, usize)> {
    target_cells_in(skill, b, target, 0, HEROES[0].hp, 0, Rules::default())
}
pub fn target_cells_in(
    skill: Skill,
    b: &Board,
    target: (usize, usize),
    hero: usize,
    hp: i32,
    mp: i32,
    mut rules: Rules,
) -> Vec<(usize, usize)> {
    let mut copy = *b;
    let e = effect_at_in(skill, &mut copy, target, hero, hp, mp, rules);
    if let Some(change) = e.board_change {
        let _ = change.apply(&mut rules.board_mechanics, rules.actor_enemy);
    }
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
        for cell in rules.board_mechanics.matches(&copy) {
            if !cells.contains(&cell) {
                cells.push(cell);
            }
        }
    }
    cells
}
pub fn trigger_tiles(
    board: &Board,
    cells: &[(usize, usize)],
    healing_percent: i32,
    damage_percent: i32,
    mana_percent: i32,
) -> (i32, i32, i32) {
    let (mut h, mut d, mut m) = (0, 0, 0);
    for &(x, y) in cells {
        if board[y][x] < 7 {
            let v = VALUES[board[y][x] as usize];
            h += v.0;
            d += v.1;
            m += v.2;
        }
    }
    (
        h * healing_percent / 100,
        d * damage_percent / 100,
        m * mana_percent / 100,
    )
}
pub fn effect_at(skill: Skill, b: &mut Board, target: (usize, usize)) -> Effect {
    effect_at_in(skill, b, target, 0, HEROES[0].hp, 0, Rules::default())
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
    let skill = effective_skill(skill, rules);
    let ctx = EventContext::new(hero, hp, mp, rules);
    let mut effect = Effect::default();
    dispatch_active(
        skill,
        &ctx,
        &mut RuleEvent::Cast {
            skill,
            board: b,
            target,
            effect: &mut effect,
        },
        &mut Vec::new(),
    );
    let mut triggers = std::mem::take(&mut effect.triggers);
    dispatch_rules(
        &ctx,
        &mut RuleEvent::SkillEffect {
            skill,
            effect: &mut effect,
        },
        &mut triggers,
    );
    effect.triggers = triggers;
    effect
}

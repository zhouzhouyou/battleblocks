//! What each side did in the current and the previous turn, shown in the two-row turn
//! indicator above the board. The records are display-only and never affect play.
use crate::content::Skill;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// An exchange; `columns` marks the knight's whole-column swap.
    Swap {
        from: (usize, usize),
        to: (usize, usize),
        columns: bool,
    },
    Skill(Skill),
    /// The board was rerolled, by choice or because nothing was playable.
    Reroll,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub enemy: bool,
    pub actions: Vec<Action>,
    /// The actor's net HP change, the damage it dealt and the mana it gained.
    pub heal: i32,
    pub damage: i32,
    pub mana: i32,
    /// Deepest cascade reached.
    pub chain: i32,
}

impl Turn {
    pub fn new(enemy: bool) -> Self {
        Self {
            enemy,
            actions: Vec::new(),
            heal: 0,
            damage: 0,
            mana: 0,
            chain: 0,
        }
    }
    pub fn add(&mut self, heal: i32, damage: i32, mana: i32) {
        self.heal += heal;
        self.damage += damage;
        self.mana += mana;
    }
    fn is_empty(&self) -> bool {
        self.actions.is_empty() && self.results_text(false).is_none()
    }
    pub fn last_skill(&self) -> Option<Skill> {
        self.actions.iter().rev().find_map(|a| match a {
            Action::Skill(skill) => Some(*skill),
            _ => None,
        })
    }
    /// Actions in order, e.g. "消除增幅 → (3,4)↔(4,4)"; swaps read as in the move hints.
    pub fn actions_text(&self, zh: bool) -> String {
        self.actions
            .iter()
            .map(|action| match *action {
                Action::Swap { from, to, columns } => swap_text(zh, from, to, columns),
                Action::Skill(skill) => skill.name(zh).to_string(),
                Action::Reroll => crate::tr(zh, "Board reroll", "重排棋盘").to_string(),
            })
            .collect::<Vec<_>>()
            .join(" → ")
    }
    /// The non-zero results, e.g. "伤害 24 · 魔力 +12"; `None` when nothing changed.
    pub fn results_text(&self, zh: bool) -> Option<String> {
        let parts: Vec<String> = [
            (self.heal, crate::tr(zh, "HP", "生命"), true),
            (self.damage, crate::tr(zh, "damage", "伤害"), false),
            (self.mana, crate::tr(zh, "MP", "魔力"), true),
        ]
        .into_iter()
        .filter(|&(value, _, _)| value != 0)
        .map(|(value, name, signed)| match (zh, signed) {
            (true, true) => format!("{name} {value:+}"),
            (true, false) => format!("{name} {value}"),
            (false, true) => format!("{value:+} {name}"),
            (false, false) => format!("{value} {name}"),
        })
        .collect();
        (!parts.is_empty()).then(|| parts.join(" · "))
    }
}

/// An exchange as listed in the move hints, 1-based: "(3,1)↔(4,1) 整列".
pub fn swap_text(zh: bool, a: (usize, usize), c: (usize, usize), columns: bool) -> String {
    format!(
        "({},{})↔({},{}){}",
        a.0 + 1,
        a.1 + 1,
        c.0 + 1,
        c.1 + 1,
        if columns {
            crate::tr(zh, " COL", " 整列")
        } else {
            ""
        }
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Log {
    pub current: Turn,
    pub previous: Option<Turn>,
}

impl Log {
    /// A fresh battle: the player acts first and nothing came before.
    pub fn new() -> Self {
        Self {
            current: Turn::new(false),
            previous: None,
        }
    }
    /// Starts a turn for `enemy`; the finished turn becomes the previous one unless
    /// nothing happened in it.
    pub fn next(&mut self, enemy: bool) {
        let finished = std::mem::replace(&mut self.current, Turn::new(enemy));
        if !finished.is_empty() {
            self.previous = Some(finished);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn turns_describe_actions_and_only_nonzero_results() {
        let mut turn = Turn::new(false);
        assert_eq!(turn.actions_text(true), "");
        assert_eq!(turn.results_text(true), None);
        turn.actions.push(Action::Skill(Skill::Strike));
        turn.actions.push(Action::Swap {
            from: (2, 3),
            to: (3, 3),
            columns: false,
        });
        turn.add(0, 24, 12);
        turn.add(0, 6, 0);
        assert_eq!(turn.actions_text(true), "消除增幅 → (3,4)↔(4,4)");
        assert_eq!(turn.results_text(true).unwrap(), "伤害 30 · 魔力 +12");
        assert_eq!(turn.actions_text(false), "Sinister strike → (3,4)↔(4,4)");
        assert_eq!(turn.results_text(false).unwrap(), "30 damage · +12 MP");
        assert_eq!(turn.last_skill(), Some(Skill::Strike));
        // Flesh casting can leave a net HP loss.
        let mut paid = Turn::new(true);
        paid.add(-50, 0, 0);
        assert_eq!(paid.results_text(true).unwrap(), "生命 -50");
        assert_eq!(swap_text(true, (2, 0), (3, 0), true), "(3,1)↔(4,1) 整列");
        assert_eq!(swap_text(false, (2, 0), (3, 0), true), "(3,1)↔(4,1) COL");
    }
    #[test]
    fn next_turn_keeps_the_last_turn_that_did_something() {
        let mut log = Log::new();
        log.current.actions.push(Action::Reroll);
        log.next(true);
        assert!(log.current.enemy && log.current.actions.is_empty());
        assert_eq!(log.previous.as_ref().unwrap().actions, [Action::Reroll]);
        // An empty turn (a debug turn switch) does not replace the previous one.
        log.next(false);
        assert!(!log.current.enemy);
        assert_eq!(log.previous.as_ref().unwrap().actions, [Action::Reroll]);
    }
}

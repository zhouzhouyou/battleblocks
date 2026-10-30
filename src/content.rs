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
    pub jumped: bool,
    pub miracle: bool,
    pub horizontal: bool,
    pub upper_swap: bool,
    pub pockets: [Option<u8>; 2],
    pub pocket_slot: usize,
    pub copied_skill: Option<Skill>,
}
pub const KNIGHT: usize = 7;
pub const DWARF: usize = 8;
pub const PRIEST: usize = 9;
pub const ROSTER: &[usize] = &[DWARF, KNIGHT, PRIEST, 1, 2, 3, 4, 5, 6];
use crate::board::{Board, N};

// Stable rules API shared by the game, AI, previews and balance probe.
#[path = "content/definitions.rs"]
mod definitions;
pub use definitions::*;
#[path = "content/health.rs"]
mod health;
pub use health::*;
#[path = "content/movement.rs"]
mod movement;
pub use movement::*;
#[path = "content/effects.rs"]
mod effects;
pub use effects::*;
#[path = "content/previews.rs"]
mod previews;
pub use previews::*;
#[path = "content/rewards.rs"]
mod rewards;
use health::preview_hit;
pub use rewards::*;
#[cfg(test)]
#[path = "content/tests/awakening_tests.rs"]
mod awakening_tests;
#[cfg(test)]
#[path = "content/tests/cooldown_tests.rs"]
mod cooldown_tests;
#[cfg(test)]
#[path = "content/tests/deadlock_tests.rs"]
mod deadlock_tests;
#[cfg(test)]
#[path = "content/tests/keyword_tests.rs"]
mod keyword_tests;
#[cfg(test)]
#[path = "content/tests/move_preview_tests.rs"]
mod move_preview_tests;
#[cfg(test)]
#[path = "content/tests/new_role_integration_tests.rs"]
mod new_role_integration_tests;
#[cfg(test)]
#[path = "content/tests/passive_status_tests.rs"]
mod passive_status_tests;
#[cfg(test)]
#[path = "content/tests/passive_tests.rs"]
mod passive_tests;
#[cfg(test)]
#[path = "content/tests/preview_tests.rs"]
mod preview_tests;
#[cfg(test)]
#[path = "content/tests/reward_tests.rs"]
mod reward_tests;
#[cfg(test)]
#[path = "content/tests/tests.rs"]
mod tests;

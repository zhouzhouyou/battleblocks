#[path = "abilities.rs"]
mod abilities;
pub use abilities::{ability_loadout, Passive};
pub fn passives(hero: usize, awake: bool) -> &'static [Passive] {
    abilities::passives(hero, awake)
}
#[path = "content/lore.rs"]
mod lore;
#[allow(unused_imports)]
pub use lore::*;
#[path = "content/status.rs"]
mod status;
pub use status::*;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rules {
    pub projectile_aim: Option<projectile::Aim>,
    pub actor_enemy: bool,
    pub launch_from_right: bool,
    pub board_mechanics: crate::board::BoardMechanics,
    pub target_anchor: Option<(usize, usize)>,
    pub awake: bool,
    pub initial_loot: bool,
    pub opponent_hp: Option<i32>,
    pub opponent_mp: i32,
    pub opponent_statuses: StatusSet,
    pub opponent: Option<(usize, bool)>,
    pub horizontal: bool,
    pub swap_clear: bool,
    pub pockets: [Option<u8>; 2],
    pub pocket_slot: usize,
    pub copied_skill: Option<Skill>,
    pub statuses: StatusSet,
    /// Complete rounds played, starting at 1. Drives the anti-stall overdrive.
    pub round: u32,
    /// Casts already made this turn, used by the no-cooldown repeat surcharge.
    pub repeat_casts: [u8; Skill::ALL.len()],
}
impl Rules {
    pub fn with_status(mut self, id: StatusId, source: Skill, duration: StatusDuration) -> Self {
        self.statuses.insert(id, source, duration);
        self
    }
}
pub const KNIGHT: usize = 7;
pub const DWARF: usize = 8;
pub const PRIEST: usize = 9;
pub const NUN: usize = 10;
pub const FIRE_ELF: usize = 11;
pub const PORTAL_CUBE: usize = 12;
pub const SLINGSHOT_USER: usize = 13;
#[path = "content/roster.rs"]
mod roster;
pub use roster::*;

pub fn turn_start_effects(
    hero: usize,
    hp: i32,
    mp: i32,
    board: &mut Board,
    rules: Rules,
) -> Vec<AbilityTrigger> {
    let mut triggers = Vec::new();
    dispatch_rules(
        &EventContext::new(hero, hp, mp, rules),
        &mut RuleEvent::TurnStart { board },
        &mut triggers,
    );
    triggers
}
use crate::board::{Board, N};

// Stable rules API shared by the game, AI, previews and balance probe.
#[path = "content/definitions.rs"]
mod definitions;
pub use definitions::*;
#[path = "content/health.rs"]
mod health;
pub use health::*;
#[path = "content/events.rs"]
mod events;
pub use events::*;
#[path = "content/movement.rs"]
mod movement;
pub use movement::*;
#[path = "content/effects.rs"]
mod effects;
pub mod projectile;
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
#[path = "content/tests/fire_elf_tests.rs"]
mod fire_elf_tests;
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
#[path = "content/tests/nun_tests.rs"]
mod nun_tests;
#[cfg(test)]
#[path = "content/tests/passive_status_tests.rs"]
mod passive_status_tests;
#[cfg(test)]
#[path = "content/tests/passive_tests.rs"]
mod passive_tests;
#[cfg(test)]
#[path = "content/tests/portal_tests.rs"]
mod portal_tests;
#[cfg(test)]
#[path = "content/tests/preview_tests.rs"]
mod preview_tests;
#[cfg(test)]
#[path = "content/tests/reward_tests.rs"]
mod reward_tests;
#[cfg(test)]
#[path = "content/tests/rework_tests.rs"]
mod rework_tests;
#[cfg(test)]
#[path = "content/tests/tests.rs"]
mod tests;

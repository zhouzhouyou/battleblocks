//! Snapshot-owned state for new roles. Board changes use the same rules in the
//! live loop and previews; random choices are supplied by the saved RNG stream.
use crate::board::{self, Board, N};
use crate::content::{self, Skill};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewRoleState {
    pub pockets: [Option<u8>; 2],
    pub jumped: bool,
    pub miracle: bool,
    pub copied_skill: Option<usize>,
    pub pocket_slot: usize,
    pub direct_tiles: Vec<u8>,
    pub force_armed: bool,
    pub horizontal: bool,
    pub upper_swap: bool,
}
impl NewRoleState {
    pub fn select_occupied_pocket(&mut self) {
        if self.pockets[self.pocket_slot].is_none() {
            self.pocket_slot = self.pockets.iter().position(Option::is_some).unwrap_or(0);
        }
    }
    pub fn on_cast(&mut self, effective: Skill, before: &Board, target: (usize, usize)) {
        self.horizontal = false;
        self.upper_swap = false;
        match effective {
            Skill::Jump => self.jumped = true,
            Skill::Miracle => self.force_armed = true,
            Skill::Entomb => {
                if let Some(slot) = self.pockets.iter_mut().find(|p| p.is_none()) {
                    *slot = Some(before[target.1][target.0]);
                }
            }
            Skill::Resurrect => self.pockets[self.pocket_slot] = None,
            _ => {}
        }
        self.select_occupied_pocket();
    }
    pub fn record_swap(&mut self, a: (usize, usize), b: (usize, usize)) {
        self.horizontal = a.1 == b.1;
        self.upper_swap = a.1 < 4 && b.1 < 4;
    }
    pub fn start_turn(&mut self) {
        self.jumped = false;
        self.miracle = false;
        self.force_armed = false;
        self.horizontal = false;
        self.upper_swap = false;
        self.direct_tiles.clear();
        self.select_occupied_pocket();
    }
    /// Select from the multiset of directly eliminated tiles, not distinct types.
    pub fn mind_vision(&mut self, direct_tiles: &[u8], pick: usize) -> bool {
        let Some(slot) = self.pockets.iter_mut().find(|p| p.is_none()) else {
            return false;
        };
        if direct_tiles.is_empty() {
            return false;
        }
        *slot = Some(direct_tiles[pick % direct_tiles.len()]);
        self.select_occupied_pocket();
        true
    }
    pub fn complete_force(
        &mut self,
        board: &mut Board,
        a: (usize, usize),
        b: (usize, usize),
        hero: usize,
    ) -> bool {
        if !self.force_armed || !force_swap(board, a, b, hero) {
            return false;
        }
        self.force_armed = false;
        self.miracle = true;
        self.record_swap(a, b);
        true
    }
}
/// Force a movement obeying the caster's movement passive; reject any movement
/// that produces a match, with no mutation on rejection.
pub fn force_swap(board: &mut Board, a: (usize, usize), b: (usize, usize), hero: usize) -> bool {
    if !board::adjacent(a, b, hero) {
        return false;
    }
    let mut copy = *board;
    if matches!(hero, 0 | content::KNIGHT) && a.1 == b.1 {
        for y in 0..N {
            board::swap(&mut copy, (a.0, y), (b.0, y));
        }
    } else {
        board::swap(&mut copy, a, b);
    }
    if copy == *board || !board::matches(&copy).is_empty() {
        return false;
    }
    *board = copy;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::{Cooldowns, Rules};
    fn board() -> Board {
        std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
    }
    #[test]
    fn new_cooldowns_share_config_and_restore_with_snapshots() {
        let mut cd = Cooldowns::default();
        let saved = cd.clone();
        cd.cast(0, Skill::Jump);
        assert_eq!(cd.remaining(0, Skill::Jump), 2);
        cd.start_turn(1);
        assert_eq!(cd.remaining(0, Skill::Jump), 2);
        cd.start_turn(0);
        assert_eq!(cd.remaining(0, Skill::Jump), 1);
        cd.start_turn(0);
        assert_eq!(cd.remaining(0, Skill::Jump), 0);
        cd = saved;
        assert_eq!(cd.remaining(0, Skill::Jump), 0);
        assert!(!Skill::Resurrect.ends_turn());
        assert!(!Skill::Miracle.ends_turn());
        assert!(Skill::Hammer.ends_turn());
        assert_eq!(Skill::Copy.cooldown(), 1);
    }
    #[test]
    fn pocket_capacity_and_selection_follow_actual_contents() {
        let mut state = NewRoleState::default();
        assert!(state.mind_vision(&[2, 3, 3], 2));
        assert_eq!(state.pockets[0], Some(3));
        assert!(state.mind_vision(&[1], 0));
        let full = state.clone();
        assert!(!state.mind_vision(&[0], 0));
        assert_eq!(state, full);
        state.on_cast(Skill::Resurrect, &board(), (0, 0));
        assert_eq!(state.pockets, [None, Some(1)]);
        assert_eq!(state.pocket_slot, 1);
    }
    #[test]
    fn resurrection_inserts_and_destroys_bottom_without_tile_rewards() {
        let mut b = board();
        let original = b;
        let r = Rules {
            pockets: [Some(2), None],
            ..Rules::default()
        };
        let e = content::effect_at_in(
            Skill::Resurrect,
            &mut b,
            (4, 3),
            content::PRIEST,
            500,
            50,
            r,
        );
        assert_eq!(b[3][4], 2);
        for y in 4..N {
            assert_eq!(b[y][4], original[y - 1][4]);
        }
        for y in 0..N {
            for x in 0..N {
                if x != 4 || y < 3 {
                    assert_eq!(b[y][x], original[y][x]);
                }
            }
        }
        assert_eq!((e.hp, e.damage, e.mp), (0, 0, 0));
        assert!(e.cells.is_empty());
    }
    #[test]
    fn force_swap_obeys_caster_movement_and_is_transactional() {
        let mut b = board();
        let original = b;
        let mut s = NewRoleState {
            force_armed: true,
            ..Default::default()
        };
        assert!(s.complete_force(&mut b, (0, 0), (1, 0), content::KNIGHT));
        for y in 0..N {
            assert_eq!(b[y][0], original[y][1]);
        }
        assert!(s.miracle);
        assert!(!s.force_armed);
        let mut priest = original;
        assert!(force_swap(&mut priest, (0, 0), (1, 0), content::PRIEST));
        assert_eq!(priest[1], original[1]);
        let before = priest;
        assert!(!force_swap(&mut priest, (0, 0), (7, 7), content::PRIEST));
        assert_eq!(priest, before);
    }
}

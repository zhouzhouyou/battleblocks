use super::*;
use std::rc::Rc;

type RankedHints = Vec<(Move, Option<content::MovePreview>)>;

#[derive(PartialEq, Eq)]
pub(super) struct HintKey {
    board: Board,
    hero: usize,
    hp: i32,
    mp: i32,
    opponent_hp: i32,
    bonus: i32,
    armed: bool,
    rules: content::Rules,
}

pub(super) struct HintCache {
    key: HintKey,
    ranked: Rc<RankedHints>,
}

impl Session {
    /// A derived, read-only cache shared by input and drawing, never saved as game state.
    pub(in crate::app) fn ranked_hints(&mut self, side: bool) -> Rc<RankedHints> {
        let (hero, hp, mp, opponent_hp) = if side {
            (self.enemy_id, self.enemy, self.enemy_mp, self.hp)
        } else {
            (self.hero_id, self.hp, self.mp, self.enemy)
        };
        let key = HintKey {
            board: self.b,
            hero,
            hp,
            mp,
            opponent_hp,
            bonus: self.extra_bonus,
            armed: self.armed,
            rules: self.rules(side),
        };
        if let Some(cached) = self.hint_cache.as_ref().filter(|cached| cached.key == key) {
            return Rc::clone(&cached.ranked);
        }
        let ranked = Rc::new(content::ranked_moves_in(
            &key.board,
            key.hero,
            key.hp,
            key.mp,
            key.opponent_hp,
            key.bonus,
            key.armed,
            key.rules,
        ));
        self.hint_cache = Some(HintCache {
            key,
            ranked: Rc::clone(&ranked),
        });
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_reuse_only_identical_state_and_never_change_rules_or_rng() {
        let mut session = Session::new_headless();
        let before = session.snapshot();
        let first = session.ranked_hints(false);
        assert!(Rc::ptr_eq(&first, &session.ranked_hints(false)));
        assert_eq!(rng::state(), before.rng_state);
        assert_eq!(session.b, before.board);
        session.mp += 1;
        let changed = session.ranked_hints(false);
        assert!(!Rc::ptr_eq(&first, &changed));
        session
            .board_mechanics
            .set_gravity(board::Gravity::Left, false);
        let gravity = session.ranked_hints(false);
        assert!(!Rc::ptr_eq(&changed, &gravity));
        session.new_role_states[0].statuses.insert(
            content::StatusId::ExtraReach,
            content::Skill::Jump,
            content::StatusDuration::OwnerTurnEnd(1),
        );
        assert!(!Rc::ptr_eq(&gravity, &session.ranked_hints(false)));
        session.restore(before);
        assert!(session.hint_cache.is_none());
        assert!(!Rc::ptr_eq(&first, &session.ranked_hints(false)));
    }
}

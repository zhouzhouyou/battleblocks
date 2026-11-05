//! Per-action view clocks. Signals are supplied by the canonical Session.
use crate::{
    content::Skill,
    sprites::{ActorCast, ActorSignal, AnimationClip},
};

pub(super) struct Cast {
    pub skill: Skill,
    pub enemy: bool,
    pub action: u64,
    pub cells: Vec<(usize, usize)>,
    pub spec: ActorCast,
    started: f64,
    hit: Option<f64>,
}
impl Cast {
    pub fn new(
        skill: Skill,
        enemy: bool,
        action: u64,
        cells: &[(usize, usize)],
        mut spec: ActorCast,
        now: f64,
    ) -> Self {
        // Guard is legal without consumable tiles. It must never wait forever.
        if spec.signal == ActorSignal::InitialTargetsCleared && cells.is_empty() {
            spec.signal = ActorSignal::CastApplied;
            spec.clip.impact = None;
        }
        Self {
            skill,
            enemy,
            action,
            cells: cells.to_vec(),
            spec,
            started: now,
            hit: None,
        }
    }
    pub fn notify(
        &mut self,
        signal: ActorSignal,
        enemy: bool,
        action: u64,
        cells: &[(usize, usize)],
        now: f64,
    ) {
        if self.enemy != enemy
            || self.action != action
            || self.hit.is_some()
            || self.spec.signal != signal
        {
            return;
        }
        if signal == ActorSignal::InitialTargetsCleared
            && (cells.len() != self.cells.len() || !cells.iter().all(|c| self.cells.contains(c)))
        {
            return;
        }
        self.hit = Some(now);
    }
    pub fn elapsed(&self, now: f64, over: bool) -> f64 {
        let elapsed = (now - self.started).max(0.);
        match self.spec.clip.impact {
            Some(key) => match self.hit {
                // Enter the impact pose even when summing frame durations rounds
                // a decimal boundary a fraction above its authored timestamp.
                Some(hit) => key + (now - hit).max(0.) + 1e-9,
                // Terminal sessions cannot deliver a pending clear. Bounded visual
                // recovery is not a synthetic model hit or reward notification.
                None if over => elapsed,
                None => elapsed.min(key - 0.000001),
            },
            None => elapsed,
        }
    }
    pub fn sample(&self, action: u64, now: f64, over: bool) -> Option<(AnimationClip, f64)> {
        let time = self.elapsed(now, over);
        (self.action == action && time < self.spec.clip.duration())
            .then_some((self.spec.clip, time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cast(id: usize, skill: Skill, cells: &[(usize, usize)]) -> Cast {
        Cast::new(
            skill,
            false,
            17,
            cells,
            crate::sprites::AWAKENED[id].actor_cast(skill).unwrap(),
            10.,
        )
    }
    #[test]
    fn delayed_collection_waits_for_own_initial_clear_and_preserves_recovery() {
        let mut c = cast(3, Skill::Greedy, &[(0, 0), (1, 0)]);
        assert_eq!(c.spec.clip.frame(c.elapsed(12., false)), 17);
        c.notify(
            ActorSignal::InitialTargetsCleared,
            true,
            17,
            &[(0, 0), (1, 0)],
            12.,
        );
        c.notify(
            ActorSignal::InitialTargetsCleared,
            false,
            18,
            &[(0, 0), (1, 0)],
            12.,
        );
        c.notify(
            ActorSignal::InitialTargetsCleared,
            false,
            17,
            &[(0, 0)],
            12.,
        );
        assert!(c.hit.is_none());
        c.notify(
            ActorSignal::InitialTargetsCleared,
            false,
            17,
            &[(1, 0), (0, 0)],
            12.,
        );
        assert_eq!(c.spec.clip.frame(c.elapsed(12., false)), 18);
        c.notify(
            ActorSignal::InitialTargetsCleared,
            false,
            17,
            &[(0, 0), (1, 0)],
            12.3,
        );
        assert_eq!(c.hit, Some(12.));
        assert!(c.sample(17, 12.99, false).is_some());
        assert!(c.sample(17, 13.01, false).is_none());
        assert!(c.sample(18, 12.1, false).is_none());
    }
    #[test]
    fn charge_damage_is_independent_of_later_clear_and_awakening() {
        let mut c = cast(0, Skill::Charge, &[(0, 0)]);
        c.notify(
            ActorSignal::InitialTargetsCleared,
            false,
            17,
            &[(0, 0)],
            10.8,
        );
        assert!(c.hit.is_none());
        c.notify(ActorSignal::DamageApplied, false, 17, &[], 11.1);
        assert_eq!(c.spec.clip.frame(c.elapsed(11.1, false)), 18);
        assert!(c.sample(17, 11.54, false).is_some());
        assert!(c.sample(17, 11.56, false).is_none());
    }
    #[test]
    fn immediate_skills_empty_guard_terminal_and_action_replacement_do_not_wait() {
        for (id, skill, cells) in [
            (1, Skill::Strike, vec![]),
            (1, Skill::Nature, vec![(0, 0)]),
            (3, Skill::Blood, vec![]),
            (0, Skill::Guard, vec![]),
        ] {
            let c = cast(id, skill, &cells);
            assert!(c.sample(17, 10., false).is_some());
            assert!(c.sample(17, 11.41, false).is_none());
        }
        let c = cast(3, Skill::Greedy, &[(0, 0)]);
        assert!(c.sample(17, 30., false).is_some());
        assert!(c.sample(17, 30., true).is_none());
        assert!(c.sample(18, 10.1, false).is_none());
    }
}

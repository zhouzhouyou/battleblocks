use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusId {
    Frenzy,
    Rational,
    ExtraReach,
    ForcedMove,
    ClearPenalty,
    DoubleRewards,
    ParallelSwap,
}
impl StatusId {
    pub fn description(self, zh: bool) -> &'static str {
        role_text(
            match self {
                Self::Frenzy => "Status.frenzy.description",
                Self::Rational => "Status.rational.description",
                Self::ExtraReach => "Status.reach.description",
                Self::ForcedMove => "Status.force.description",
                Self::ClearPenalty => "Status.penalty.description",
                Self::DoubleRewards => "Status.double.description",
                Self::ParallelSwap => "Status.parallel.description",
            },
            zh,
        )
    }
    pub fn name(self, zh: bool) -> &'static str {
        role_text(
            match self {
                Self::Frenzy => "Status.frenzy",
                Self::Rational => "Status.rational",
                Self::ExtraReach => "Status.reach",
                Self::ForcedMove => "Status.force",
                Self::ClearPenalty => "Status.penalty",
                Self::DoubleRewards => "Status.double",
                Self::ParallelSwap => "Status.parallel",
            },
            zh,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusDuration {
    OwnerTurnEnd(u8),
    UntilSwap,
    UntilClear,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusEffect {
    pub id: StatusId,
    pub applied_by: Skill,
    pub duration: StatusDuration,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatusSet(pub [Option<StatusEffect>; 16]);
impl StatusSet {
    #[cfg(test)]
    pub fn single(id: StatusId, applied_by: Skill, duration: StatusDuration) -> Self {
        let mut s = Self::default();
        s.insert(id, applied_by, duration);
        s
    }
    pub fn contains(self, id: StatusId) -> bool {
        self.0.iter().flatten().any(|s| s.id == id)
    }
    pub fn insert(&mut self, id: StatusId, applied_by: Skill, duration: StatusDuration) {
        if let Some(slot) = self.0.iter_mut().find(|s| s.is_some_and(|s| s.id == id)) {
            *slot = Some(StatusEffect {
                id,
                applied_by,
                duration,
            });
        } else {
            let slot = self
                .0
                .iter_mut()
                .find(|s| s.is_none())
                .expect("too many simultaneous status effects");
            *slot = Some(StatusEffect {
                id,
                applied_by,
                duration,
            });
        }
    }

    pub fn remove(&mut self, id: StatusId) {
        for s in &mut self.0 {
            if s.is_some_and(|s| s.id == id) {
                *s = None;
            }
        }
    }
    pub fn consume(&mut self, duration: StatusDuration) {
        for s in &mut self.0 {
            if s.is_some_and(|s| s.duration == duration) {
                *s = None;
            }
        }
    }
    pub fn end_owner_turn(&mut self) {
        for slot in &mut self.0 {
            if let Some(s) = slot {
                if let StatusDuration::OwnerTurnEnd(n) = s.duration {
                    if n <= 1 {
                        *slot = None;
                    } else {
                        s.duration = StatusDuration::OwnerTurnEnd(n - 1);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_turn_countdown_and_consumed_preparations_are_independent() {
        let mut effects = StatusSet::default();
        effects.insert(
            StatusId::Frenzy,
            Skill::Frenzy,
            StatusDuration::OwnerTurnEnd(2),
        );
        effects.insert(
            StatusId::DoubleRewards,
            Skill::Strike,
            StatusDuration::UntilClear,
        );
        effects.end_owner_turn();
        assert!(effects.contains(StatusId::Frenzy) && effects.contains(StatusId::DoubleRewards));
        effects.end_owner_turn();
        assert!(!effects.contains(StatusId::Frenzy) && effects.contains(StatusId::DoubleRewards));
        effects.consume(StatusDuration::UntilSwap);
        assert!(effects.contains(StatusId::DoubleRewards));
        effects.consume(StatusDuration::UntilClear);
        assert!(!effects.contains(StatusId::DoubleRewards));
    }
}

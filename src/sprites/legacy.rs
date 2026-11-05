//! Awakened-only playback contracts. No combat rules or normal sprite metadata.
use super::{AnimationClip, Playback, SpriteAsset, CAST_FRAME_TIME};
use crate::content::Skill;

/// Explicit opt-in clip used only by the three Issue 28 awakened assets.
pub const LEGACY_AWAKE_CAST: AnimationClip = AnimationClip {
    indices: &[12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
    durations: &CAST_FRAME_TIME,
    playback: Playback::Once,
    impact: None,
};
const CHARGE: [f64; 12] = [
    0.08, 0.12, 0.15, 0.16, 0.19, 0.25, 0.08, 0.08, 0.08, 0.07, 0.07, 0.07,
];
const COLLECT: [f64; 12] = [
    0.06, 0.06, 0.07, 0.07, 0.07, 0.07, 0.16, 0.16, 0.16, 0.16, 0.18, 0.18,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActorSignal {
    CastApplied,
    DamageApplied,
    InitialTargetsCleared,
}

#[derive(Clone, Copy, Debug)]
pub struct ActorCast {
    pub clip: AnimationClip,
    pub signal: ActorSignal,
}

impl SpriteAsset {
    pub fn legacy_awake(&self) -> bool {
        self.cast == Some(LEGACY_AWAKE_CAST)
    }
    pub fn actor_cast(&self, skill: Skill) -> Option<ActorCast> {
        if !self.legacy_awake() {
            return None;
        }
        let (durations, impact, signal) = match skill {
            Skill::Charge => (&CHARGE, Some(0.95), ActorSignal::DamageApplied),
            Skill::Guard | Skill::Greedy => {
                (&COLLECT, Some(0.4), ActorSignal::InitialTargetsCleared)
            }
            Skill::Strike | Skill::Nature | Skill::Blood => {
                (&CAST_FRAME_TIME, None, ActorSignal::CastApplied)
            }
            _ => return None,
        };
        Some(ActorCast {
            clip: AnimationClip {
                durations,
                impact,
                ..LEGACY_AWAKE_CAST
            },
            signal,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_three_awakened_assets_opt_in_and_each_skill_covers_twelve_poses() {
        for asset in super::super::SPRITES {
            assert!(!asset.legacy_awake());
        }
        for (id, asset) in super::super::AWAKENED.iter().enumerate() {
            assert_eq!(asset.legacy_awake(), matches!(id, 0 | 1 | 3));
            if asset.legacy_awake() {
                for &skill in crate::content::skills(id, true) {
                    let spec = asset.actor_cast(skill).unwrap();
                    assert!((spec.clip.duration() - 1.4).abs() < 1e-9);
                    let mut t = 0.;
                    for (i, &d) in spec.clip.durations.iter().enumerate() {
                        assert_eq!(spec.clip.frame(t + d / 2.), 12 + i);
                        t += d;
                    }
                    assert_eq!(spec.clip.frame(100.), 23);
                }
            }
        }
    }
}

//! Awakened-only playback contracts. No combat rules or normal sprite metadata.
use super::{AnimationClip, Playback, SpriteAsset, CAST_FRAME_TIME};
use crate::content::Skill;

/// Explicit opt-in for legacy awakened assets; normal sprites never use it.
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
// The zombie's club reaches its low point one pose after the two demons.
const DRAIN: [f64; 12] = [
    0.04, 0.05, 0.06, 0.06, 0.06, 0.06, 0.07, 0.2, 0.2, 0.2, 0.2, 0.2,
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
            Skill::LightSlingshot | Skill::HeavySlingshot => {
                (&CAST_FRAME_TIME, None, ActorSignal::CastApplied)
            }
            Skill::Charge => (&CHARGE, Some(0.95), ActorSignal::DamageApplied),
            Skill::Guard | Skill::Greedy | Skill::Imp | Skill::Chort => {
                (&COLLECT, Some(0.4), ActorSignal::InitialTargetsCleared)
            }
            Skill::Drain => (&DRAIN, Some(0.4), ActorSignal::InitialTargetsCleared),
            Skill::Strike
            | Skill::Nature
            | Skill::Blood
            | Skill::Balance
            | Skill::War
            | Skill::Corrupt => (&CAST_FRAME_TIME, None, ActorSignal::CastApplied),
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
    fn only_legacy_awakened_assets_opt_in_and_each_skill_covers_twelve_poses() {
        for asset in super::super::SPRITES {
            assert!(!asset.legacy_awake());
        }
        for (id, asset) in super::super::AWAKENED.iter().enumerate() {
            assert_eq!(asset.legacy_awake(), matches!(id, 0 | 1 | 3 | 4 | 5 | 6) || id == crate::content::SLINGSHOT_USER);
            if asset.legacy_awake() {
                for &skill in crate::content::skills(id, true) {
                    let spec = asset.actor_cast(skill).unwrap();
                    assert_eq!(
                        super::super::cast_clip(id, true, Some(skill)),
                        Some(spec.clip),
                        "shared menu/HUD selection must preserve the authored skill timing"
                    );
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

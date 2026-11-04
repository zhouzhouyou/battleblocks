//! Character animation metadata and sampling shared by the menu and battle HUD.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Playback {
    Loop,
    Once,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationClip {
    pub indices: &'static [usize],
    pub durations: &'static [f64],
    pub playback: Playback,
}

impl AnimationClip {
    pub fn duration(self) -> f64 {
        self.durations.iter().sum()
    }

    pub fn frame(self, elapsed: f64) -> usize {
        debug_assert_eq!(self.indices.len(), self.durations.len());
        let duration = self.duration();
        let elapsed = elapsed.max(0.);
        if self.playback == Playback::Once && elapsed >= duration {
            return *self.indices.last().expect("nonempty animation");
        }
        let mut time = elapsed % duration;
        for (&index, &hold) in self.indices.iter().zip(self.durations) {
            if time < hold {
                return index;
            }
            time -= hold;
        }
        *self.indices.last().expect("nonempty animation")
    }
}

pub struct SpriteAsset {
    pub bytes: &'static [u8],
    pub w: f32,
    pub h: f32,
    pub durations: &'static [f64],
    pub indices: &'static [usize],
    pub cast: Option<AnimationClip>,
    pub sleep: Option<AnimationClip>,
    /// Explicit foot anchor for the common production display scale.
    pub anchor: Option<[f32; 2]>,
}

impl SpriteAsset {
    pub fn idle(&self) -> AnimationClip {
        AnimationClip {
            indices: self.indices,
            durations: self.durations,
            playback: Playback::Loop,
        }
    }

    /// Jump uses the full idle sequence, with lift supplied by the landing effect.
    /// It must not play a hammer strike in midair or collapse to a repeated pose.
    pub fn action(&self, jumping: bool) -> Option<AnimationClip> {
        if jumping {
            self.cast.map(|_| AnimationClip {
                indices: self.indices,
                durations: &CAST_FRAME_TIME,
                playback: Playback::Once,
            })
        } else {
            self.cast
        }
    }
}

pub const CAST_FRAME_TIME: [f64; 12] = [crate::presentation::CAST_TIME / 12.; 12];
pub const NORMAL_CAST: AnimationClip = AnimationClip {
    indices: &[24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35],
    durations: &CAST_FRAME_TIME,
    playback: Playback::Once,
};
pub const AWAKE_CAST: AnimationClip = AnimationClip {
    indices: &[36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47],
    ..NORMAL_CAST
};
pub const SLEEP: AnimationClip = AWAKE_CAST;

/// View-only transition clock. Reset with other presentation state on rollback.
#[derive(Default)]
pub struct SleepClock {
    started: Option<f64>,
}
impl SleepClock {
    pub fn elapsed(&mut self, sleeping: bool, now: f64) -> f64 {
        if sleeping {
            (now - *self.started.get_or_insert(now)).max(0.)
        } else {
            self.started = None;
            0.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_shots_cover_all_poses_and_hold_the_last_at_and_after_the_end() {
        for clip in [NORMAL_CAST, AWAKE_CAST, SLEEP] {
            assert_eq!(clip.frame(-1.), clip.indices[0]);
            let mut elapsed = 0.;
            for (&frame, &duration) in clip.indices.iter().zip(clip.durations) {
                assert_eq!(clip.frame(elapsed + duration / 2.), frame);
                elapsed += duration;
            }
            assert!((elapsed - crate::presentation::CAST_TIME).abs() < 1e-9);
            assert_eq!(clip.frame(elapsed), *clip.indices.last().unwrap());
            assert_eq!(clip.frame(elapsed * 100.), *clip.indices.last().unwrap());
        }
    }
    #[test]
    fn idle_wraps_but_sleep_restarts_only_after_leaving_sleep() {
        let idle = AnimationClip {
            indices: &[0, 1, 2],
            durations: &[0.1, 0.2, 0.3],
            playback: Playback::Loop,
        };
        assert_eq!(idle.frame(0.05), 0);
        assert_eq!(idle.frame(0.15), 1);
        assert_eq!(idle.frame(0.4), 2);
        assert_eq!(idle.frame(0.65), 0);
        let mut clock = SleepClock::default();
        assert_eq!(clock.elapsed(true, 42.), 0.);
        assert_eq!(clock.elapsed(true, 44.), 2.);
        assert_eq!(SLEEP.frame(clock.elapsed(true, 44.)), 47);
        clock.elapsed(false, 45.);
        assert_eq!(clock.elapsed(true, 50.), 0.);
    }
}

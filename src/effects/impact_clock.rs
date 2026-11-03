//! GUI-only timing, notified by the same Session clear as the model.
pub(super) const IMPACT: f32 = 0.4;
pub(super) struct ImpactClock {
    started: f64,
    impact: Option<f64>,
}
impl ImpactClock {
    pub fn new(now: f64) -> Self {
        Self {
            started: now,
            impact: None,
        }
    }
    pub fn cleared(&mut self, now: f64) {
        if self.impact.is_none() {
            self.impact = Some(now);
        }
    }
    pub fn waiting(&self) -> bool {
        self.impact.is_none()
    }
    pub fn elapsed(&self, now: f64, over: bool) -> f32 {
        self.elapsed_at(now, over, IMPACT)
    }
    pub fn elapsed_at(&self, now: f64, over: bool, impact: f32) -> f32 {
        if let Some(hit) = self.impact {
            impact + (now - hit) as f32
        } else if over {
            (now - self.started) as f32
        } else {
            ((now - self.started) as f32).min(impact - 1. / 15.)
        }
    }
}

//! Shared by the GUI and offline review renderer; no model or graphics context.
pub const TAKEOFF: f32 = 0.12;
pub const LANDING: f32 = 0.65;
pub const DUST_DURATION: f32 = 0.7;

pub fn lift(elapsed: f32) -> f32 {
    let phase = ((elapsed - TAKEOFF) / (LANDING - TAKEOFF)).clamp(0., 1.);
    // The raised-hammer pose leaves 15 logical pixels below the mana bar.
    (14. * 4. * phase * (1. - phase)).round()
}

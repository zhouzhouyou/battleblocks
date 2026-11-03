//! Shared by the GUI and offline renderer; describes motion, never target selection.
pub const IMPACT: f32 = 0.9;
pub const FPS: f32 = 10.;
pub const END: f32 = 2.8;
pub const COLUMNS: usize = 6;
pub const FRAMES: usize = 29;
pub const FADE_TIME: f32 = 0.3;
pub fn wave_radius(age: f32) -> f32 {
    20. + 276. * (age / 1.1).clamp(0., 1.)
}
pub fn arrival(meteor: bool, x: usize, y: usize) -> f32 {
    let distance = ((x as f32 - 3.5) * 64.).hypot((y as f32 - 3.5) * 64.);
    if meteor {
        ((distance - 20.) / 276.).max(0.) * 1.1
    } else {
        0.18 + distance / 320. * 0.22
    }
}
pub fn tile_progress(meteor: bool, x: usize, y: usize, age: f32) -> f32 {
    let distance = ((x as f32 - 3.5) * 64.).hypot((y as f32 - 3.5) * 64.);
    if meteor && wave_radius(age) < distance {
        return 0.;
    }
    ((age - arrival(meteor, x, y)) / FADE_TIME).clamp(0., 1.)
}
pub fn fall_delay(meteor: bool) -> f64 {
    if meteor {
        1.25
    } else {
        0.85
    }
}

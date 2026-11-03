//! A single board-sized atlas. ZZZ is an independent, status-driven loop.
pub(super) use super::impact_clock::ImpactClock as Blast;
#[cfg(test)]
use super::impact_clock::IMPACT;
use macroquad::prelude::*;
pub const DURATION: f32 = 1.4;

impl Blast {
    pub fn draw(&self, texture: &Texture2D, over: bool) {
        if let Some(source) = board_frame(self.elapsed(get_time(), over)) {
            draw_texture_ex(
                texture,
                256.,
                110.,
                WHITE,
                DrawTextureParams {
                    source: Some(source),
                    ..Default::default()
                },
            );
        }
    }
}
fn board_frame(elapsed: f32) -> Option<Rect> {
    if !elapsed.is_finite() || !(0.0..DURATION).contains(&elapsed) {
        return None;
    }
    let frame = (elapsed * 15.).floor() as usize;
    Some(Rect::new(
        (frame % 6) as f32 * 512.,
        (frame / 6) as f32 * 512.,
        512.,
        512.,
    ))
}
pub(super) fn sleeping(active: bool, pending: bool, casting: bool) -> bool {
    active || (pending && !casting)
}

pub(super) fn draw_sleep(texture: &Texture2D, area: Rect, foot: Vec2, lying: bool, enemy: bool) {
    let head = if lying {
        foot + vec2(-12., -84.)
    } else {
        vec2(foot.x, area.y + 42.)
    };
    let origin = (head - vec2(24., 70.)).round();
    let panel = crate::graphics::side_panel_rect(enemy);
    let clip = Rect::new(panel.x, 188., panel.w, 188.);
    let Some(visible) = Rect::new(origin.x, origin.y, 96., 96.).intersect(clip) else {
        return;
    };
    let frame = ((get_time().rem_euclid(2.4) * 15.) as usize).min(35);
    let source = Rect::new(
        (frame % 6) as f32 * 96. + visible.x - origin.x,
        (frame / 6) as f32 * 96. + visible.y - origin.y,
        visible.w,
        visible.h,
    );
    draw_texture_ex(
        texture,
        visible.x,
        visible.y,
        WHITE,
        DrawTextureParams {
            source: Some(source),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    #[test]
    fn blast_waits_for_real_clear_and_can_finish_after_a_lethal_hit() {
        let mut blast = Blast::new(10.);
        assert!(blast.elapsed(11.3, false) < IMPACT); // Opponent awakening can hold settlement.
        blast.cleared(11.5);
        assert_eq!(blast.elapsed(11.5, false), IMPACT);
        assert!((blast.elapsed(12., false) - 0.9).abs() < 0.001);
        assert!(board_frame(blast.elapsed(12.6, false)).is_none());
        let lethal = Blast::new(10.);
        assert!((lethal.elapsed(10.6, true) - 0.6).abs() < 0.001);
        assert_eq!(
            board_frame(IMPACT).unwrap(),
            Rect::new(0., 512., 512., 512.)
        );
        assert!(!sleeping(false, false, false));
        assert!(!sleeping(false, true, true));
        assert!(sleeping(false, true, false));
        assert!(sleeping(true, false, true));
    }
    #[test]
    fn blast_peak_covers_every_corner_and_sleep_frames_remain_visible() {
        let fire = image::load_from_memory(include_bytes!("../../assets/effects/ember-blast.webp"))
            .unwrap()
            .to_rgba8();
        assert_eq!(fire.dimensions(), (3072, 2048));
        assert!(fire
            .view(0, 512, 512, 512)
            .pixels()
            .all(|(_, _, p)| p.0[3] >= 200));
        assert!(fire
            .view(3 * 512, 3 * 512, 512, 512)
            .pixels()
            .all(|(_, _, p)| p.0[3] == 0));
        let sleep = image::load_from_memory(include_bytes!("../../assets/effects/sleep-zzz.webp"))
            .unwrap()
            .to_rgba8();
        assert_eq!(sleep.dimensions(), (576, 576));
        for frame in 0..36 {
            assert!(sleep
                .view(frame % 6 * 96, frame / 6 * 96, 96, 96)
                .pixels()
                .any(|(_, _, p)| p.0[3] > 150));
        }
    }
}

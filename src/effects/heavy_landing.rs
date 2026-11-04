//! Layered dust around a character's existing foot anchor, never on the board.
use super::landing_motion::{DUST_DURATION, LANDING};
use macroquad::prelude::*;
const W: f32 = 176.;
const H: f32 = 80.;
const COLUMNS: usize = 7;
const ROWS: usize = 4;

fn placement(foot: Vec2, elapsed: f32, front: bool, clip: Rect) -> Option<(Vec2, Rect)> {
    let age = elapsed - LANDING;
    if !age.is_finite() || !(0.0..DUST_DURATION).contains(&age) {
        return None;
    }
    let index = (age * 30.).floor() as usize;
    let origin = (foot - vec2(88., 60.)).round();
    let visible = Rect::new(origin.x, origin.y, W, H).intersect(clip)?;
    let source = Rect::new(
        (index % COLUMNS) as f32 * W + visible.x - origin.x,
        (index / COLUMNS + if front { ROWS } else { 0 }) as f32 * H + visible.y - origin.y,
        visible.w,
        visible.h,
    );
    Some((vec2(visible.x, visible.y), source))
}

pub(super) fn draw(texture: &Texture2D, foot: Vec2, elapsed: f32, front: bool, enemy: bool) {
    let panel = crate::graphics::side_panel_rect(enemy);
    // The effect ends at the passive row, so dust never covers status text or cards.
    let clip = Rect::new(
        panel.x,
        188.,
        panel.w,
        crate::graphics::passive_rect(enemy).y - 188.,
    );
    if let Some((at, source)) = placement(foot, elapsed, front, clip) {
        draw_texture_ex(
            texture,
            at.x,
            at.y,
            WHITE,
            DrawTextureParams {
                source: Some(source),
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::landing_motion::{lift, TAKEOFF};
    use super::*;
    use image::GenericImageView;
    #[test]
    fn dust_starts_only_after_feet_land_and_layers_stay_in_their_panels() {
        // Jump keeps the weapon at rest but animates every authored idle pose.
        let dwarf = &crate::sprites::SPRITES[crate::content::DWARF];
        let jump = dwarf.action(true).unwrap();
        assert_eq!(jump.indices, dwarf.indices);
        assert_eq!(jump.indices.len(), 12);
        assert_ne!(jump.frame(0.1), jump.frame(0.6));
        assert_eq!(jump.frame(1.4), *dwarf.indices.last().unwrap());
        assert_eq!(lift(TAKEOFF), 0.);
        assert_eq!(lift(LANDING), 0.);
        assert_eq!(lift((TAKEOFF + LANDING) / 2.), 14.);
        for enemy in [false, true] {
            let foot = vec2(if enemy { 900. } else { 124. }, 372.);
            let clip = Rect::new(if enemy { 788. } else { 12. }, 188., 224., 188.);
            assert!(placement(foot, LANDING - 0.001, false, clip).is_none());
            for i in 0..21 {
                let elapsed = LANDING + i as f32 / 30. + 0.0001;
                assert_eq!(lift(elapsed), 0.);
                let (back, source) = placement(foot, elapsed, false, clip).unwrap();
                let (front, near) = placement(foot, elapsed, true, clip).unwrap();
                assert_eq!(back, front);
                assert_eq!(source.x, near.x);
                assert_eq!(near.y - source.y, 320.);
                assert!(back.x >= clip.x && back.x + source.w <= clip.right());
                assert!(back.y >= clip.y && back.y + source.h <= clip.bottom());
                assert!(source.bottom() <= 320. && near.bottom() <= 640.);
            }
            assert!(placement(foot, 1.351, false, clip).is_none());
        }
    }
    #[test]
    fn landing_atlas_has_two_transparent_ending_layers() {
        let atlas =
            image::load_from_memory(include_bytes!("../../assets/effects/heavy-landing.webp"))
                .unwrap()
                .to_rgba8();
        assert_eq!(atlas.dimensions(), (1232, 640));
        for layer in 0..2 {
            assert!(atlas
                .view(0, layer * 320, 176, 80)
                .pixels()
                .any(|(_, _, p)| p.0[3] > 0));
            assert!(atlas
                .view(0, layer * 320 + 240, 176, 80)
                .pixels()
                .all(|(_, _, p)| p.0[3] == 0));
        }
    }
}

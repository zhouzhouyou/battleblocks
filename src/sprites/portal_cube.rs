//! Final PC-A artwork registration. Skill-to-clip mappings are presentation data.
use super::*;
use crate::content::{Skill, PORTAL_CUBE};

const IDLE_TIME: [f64; 16] = [0.045; 16];
pub const PORTAL_CAST_TIME: [f64; 16] = [
    0.088, 0.088, 0.088, 0.088, 0.088, 0.088, 0.088, 0.088, 0.087, 0.087, 0.087, 0.087, 0.087,
    0.087, 0.087, 0.087,
];
const NORMAL_PORTAL: AnimationClip = AnimationClip {
    indices: &[
        32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
    ],
    durations: &PORTAL_CAST_TIME,
    playback: Playback::Once,
    impact: None,
};
const AWAKE_PORTAL: AnimationClip = AnimationClip {
    indices: &[
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
    ],
    ..NORMAL_PORTAL
};
const NORMAL_GRAVITY: AnimationClip = AnimationClip {
    indices: &[
        64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
    ],
    ..NORMAL_PORTAL
};
const AWAKE_GRAVITY: AnimationClip = AnimationClip {
    indices: &[
        80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    ],
    ..NORMAL_PORTAL
};
pub const PORTAL_NORMAL: SpriteAsset = SpriteAsset {
    bytes: include_bytes!("../../assets/roles/portal-cube/runtime/atlas.webp"),
    w: 384.,
    h: 384.,
    durations: &IDLE_TIME,
    indices: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    cast: Some(NORMAL_PORTAL),
    sleep: None,
    anchor: Some([192., 336.]),
};
pub const PORTAL_AWAKE: SpriteAsset = SpriteAsset {
    indices: &[
        16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    ],
    cast: Some(AWAKE_PORTAL),
    ..PORTAL_NORMAL
};
pub const SKILL_CASTS: &[(usize, bool, Skill, AnimationClip)] = &[
    (PORTAL_CUBE, false, Skill::PortalPair, NORMAL_PORTAL),
    (PORTAL_CUBE, true, Skill::PortalPair, AWAKE_PORTAL),
    (PORTAL_CUBE, false, Skill::GravityShift, NORMAL_GRAVITY),
    (PORTAL_CUBE, true, Skill::GravityShift, AWAKE_GRAVITY),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_skills_and_forms_use_all_sixteen_correct_poses_and_hold_the_last() {
        for awake in [false, true] {
            let asset = if awake { &PORTAL_AWAKE } else { &PORTAL_NORMAL };
            assert_eq!(asset.idle().indices.len(), 16);
            assert!((asset.idle().duration() - 0.72).abs() < 1e-9);
            assert_eq!(asset.idle().indices[0], if awake { 16 } else { 0 });
            for (skill, first) in [
                (Skill::PortalPair, if awake { 48 } else { 32 }),
                (Skill::GravityShift, if awake { 80 } else { 64 }),
            ] {
                let clip = crate::sprites::cast_clip(PORTAL_CUBE, awake, Some(skill)).unwrap();
                assert_eq!(clip.indices, (first..first + 16).collect::<Vec<_>>());
                assert_eq!(clip.playback, Playback::Once);
                assert!((clip.duration() - crate::presentation::CAST_TIME).abs() < 1e-9);
                let mut t = 0.;
                for (&frame, &hold) in clip.indices.iter().zip(clip.durations) {
                    assert_eq!(clip.frame(t + hold / 2.), frame);
                    t += hold;
                }
                assert_eq!(clip.frame(t), first + 15);
                assert_eq!(clip.frame(t + 10.), first + 15);
            }
            // A generic jump preview must also have one duration per pose.
            let jump = asset.action(true).unwrap();
            assert_eq!(jump.indices.len(), jump.durations.len());
            assert!((jump.duration() - crate::presentation::CAST_TIME).abs() < 1e-9);
        }
        assert_eq!(
            crate::sprites::cast_clip(crate::content::KNIGHT, false, Some(Skill::Rush)),
            crate::sprites::SPRITES[crate::content::KNIGHT].cast
        );
    }

    #[test]
    fn runtime_packing_preserves_every_pixel_of_the_final_rigid_body_artwork() {
        let runtime = image::load_from_memory(PORTAL_NORMAL.bytes)
            .unwrap()
            .to_rgba8();
        let source=image::load_from_memory(include_bytes!("../../assets-source/roles/portal-cube/animation/atlas.webp")).unwrap().to_rgba8();
        assert_eq!(runtime.dimensions(), (3840, 3840));
        assert!(runtime.width() <= 4096 && runtime.height() <= 4096);
        assert_ne!(PORTAL_NORMAL.bytes, crate::sprites::SPRITES[3].bytes);
        assert_eq!(PORTAL_NORMAL.anchor, Some([192., 336.]));
        for n in 0..96 {
            let from =
                image::imageops::crop_imm(&source, n % 8 * 384, n / 8 * 384, 384, 384).to_image();
            let to = image::imageops::crop_imm(&runtime, n % 10 * 384, n / 10 * 384, 384, 384)
                .to_image();
            assert_eq!(from, to, "runtime altered final pose {n}");
        }
    }

    #[test]
    fn dedicated_thumbnail_slot_is_visible_and_not_the_goblin_placeholder() {
        let thumbs = image::load_from_memory(include_bytes!("../../assets/ui/hero-thumbnails.webp"))
            .unwrap()
            .to_rgba8();
        assert!(thumbs.width() >= 13 * 40);
        let cube = image::imageops::crop_imm(&thumbs, 12 * 40, 0, 40, 48).to_image();
        let goblin = image::imageops::crop_imm(&thumbs, 3 * 40, 0, 40, 48).to_image();
        assert!(cube.pixels().filter(|p| p[3] > 0).count() > 100);
        assert_ne!(cube, goblin);
        assert_eq!(crate::sprites::thumbnail_index(PORTAL_CUBE), PORTAL_CUBE);
    }
}

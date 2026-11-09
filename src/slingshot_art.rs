//! One equipment sprite per weapon, shared by the launcher and skill icons.
use crate::{content::Skill, graphics::Sheet};
use macroquad::prelude::*;

const LIGHT: crate::assets::Asset = crate::asset!("roles/slingshot/runtime/light.webp");
const HEAVY: crate::assets::Asset = crate::asset!("roles/slingshot/runtime/heavy.webp");
thread_local! {
    static SHEETS: std::cell::RefCell<[Sheet; 3]> = const {
        std::cell::RefCell::new([Sheet::new(LIGHT), Sheet::new(HEAVY), Sheet::new(crate::sprites::SLINGSHOT_NORMAL.asset)])
    };
}

pub fn thumbnail(area: Rect) {
    SHEETS.with(|sheets| {
        let sheets = sheets.borrow();
        draw_texture_ex(
            sheets[2].get(),
            area.x + (area.w - area.h) / 2.,
            area.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0., 0., 384., 384.)),
                dest_size: Some(vec2(area.h, area.h)),
                ..Default::default()
            },
        );
    });
}

pub fn release() {
    SHEETS.with(|sheets| {
        for sheet in sheets.borrow_mut().iter_mut() {
            sheet.release();
        }
    });
}

fn rotate(point: Vec2, pivot: Vec2, angle: f32) -> Vec2 {
    pivot + Vec2::from_angle(angle).rotate(point - pivot)
}

fn rig(skill: Skill, area: Rect, angle: f32, pivot: Vec2, pouch: Vec2) {
    SHEETS.with(|sheets| {
        let sheets = sheets.borrow();
        draw_texture_ex(
            sheets[usize::from(skill == Skill::HeavySlingshot)].get(),
            area.x,
            area.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(area.w, area.h)),
                rotation: angle,
                pivot: Some(pivot),
                ..Default::default()
            },
        );
    });
    for x in [0.25, 0.75] {
        let tip = rotate(
            vec2(area.x + area.w * x, area.y + area.h * 0.18),
            pivot,
            angle,
        );
        draw_line(
            tip.x,
            tip.y,
            pouch.x,
            pouch.y,
            (area.w * 0.05).max(1.),
            Color::from_rgba(64, 35, 29, 255),
        );
        draw_line(
            tip.x,
            tip.y - 1.,
            pouch.x,
            pouch.y - 1.,
            (area.w * 0.018).max(0.6),
            Color::from_rgba(218, 181, 115, 255),
        );
    }
    let size = (area.w * 0.1).max(2.);
    draw_rectangle(
        pouch.x - size,
        pouch.y - size * 0.5,
        size * 2.,
        size,
        Color::from_rgba(88, 60, 49, 255),
    );
}

/// One stone/alchemical pellet renderer serves the pouch, launcher and live flight.
pub fn draw_ammo(at: Vec2, radius: f32, explosive: bool) {
    draw_poly(
        at.x.round(),
        at.y.round(),
        8,
        radius + 1.,
        22.5,
        Color::from_rgba(37, 38, 51, 255),
    );
    draw_poly(
        at.x.round(),
        at.y.round(),
        8,
        radius,
        22.5,
        if explosive {
            Color::from_rgba(88, 60, 49, 255)
        } else {
            Color::from_rgba(120, 112, 128, 255)
        },
    );
    draw_rectangle(
        at.x - radius * 0.4,
        at.y - radius * 0.5,
        radius * 0.6,
        radius * 0.35,
        LIGHTGRAY,
    );
    if explosive {
        let r = radius * 0.55;
        draw_triangle_lines(
            at + vec2(0., -r),
            at + vec2(-r, r),
            at + vec2(r, r),
            1.,
            GOLD,
        );
    }
}

/// The normalized artwork anchors are shared at every display size.
pub fn draw(skill: Skill, area: Rect, pull: Vec2) {
    let pouch = vec2(area.x + area.w * 0.5, area.y + area.h * 0.18) - pull;
    rig(skill, area, 0., vec2(area.x, area.y), pouch);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_character_atlas_has_four_independent_twelve_pose_clips() {
        use image::GenericImageView;
        let normal = &crate::sprites::SPRITES[crate::content::SLINGSHOT_USER];
        let awake = &crate::sprites::AWAKENED[crate::content::SLINGSHOT_USER];
        assert_eq!(normal.asset.path, awake.asset.path);
        let body = image::load_from_memory(normal.bytes()).unwrap().to_rgba8();
        assert_eq!(body.dimensions(), (2304, 3072));
        assert!(body.pixels().all(|p| matches!(p.0[3], 0 | 255)));
        let colors: std::collections::HashSet<_> =
            body.pixels().filter(|p| p.0[3] > 0).map(|p| p.0).collect();
        assert!(colors.len() <= 64);
        let mut poses = std::collections::HashSet::new();
        for i in 0..48 {
            let frame = body
                .view((i % 6) as u32 * 384, (i / 6) as u32 * 384, 384, 384)
                .to_image();
            assert!(poses.insert(frame.into_raw()));
        }
        assert_eq!(normal.cast.unwrap().indices[0], 24);
        assert_eq!(awake.cast.unwrap().indices[0], 36);
    }

    #[test]
    fn animation_sources_have_a_transparent_backdrop() {
        for bytes in [
            include_bytes!("../assets-source/roles/slingshot/animation/normal-idle-source.webp")
                .as_slice(),
            include_bytes!("../assets-source/roles/slingshot/animation/awake-idle-source.webp")
                .as_slice(),
            include_bytes!("../assets-source/roles/slingshot/animation/normal-cast-source.webp")
                .as_slice(),
            include_bytes!("../assets-source/roles/slingshot/animation/awake-cast-source.webp")
                .as_slice(),
        ] {
            let image = image::load_from_memory(bytes).unwrap().to_rgba8();
            let transparent = image.pixels().filter(|pixel| pixel.0[3] < 8).count();
            assert!(transparent * 100 > image.width() as usize * image.height() as usize * 45);
        }
    }

    #[test]
    fn shared_equipment_is_compact_palette_limited_and_binary_alpha() {
        for asset in [LIGHT, HEAVY] {
            let image = image::load_from_memory(asset.bytes()).unwrap().to_rgba8();
            assert_eq!(image.dimensions(), (128, 128));
            assert!(image.pixels().all(|p| matches!(p.0[3], 0 | 255)));
            let colors: std::collections::HashSet<_> = image.pixels().map(|p| p.0).collect();
            assert!(colors.len() <= 64);
            assert!(image.pixels().any(|p| p.0[3] == 255));
        }
    }
}

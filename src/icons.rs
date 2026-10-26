use crate::content::Skill;
use macroquad::prelude::*;
pub fn draw(skill: Skill, x: f32, y: f32, size: f32) {
    let color = match skill {
        Skill::Charge | Skill::Landslide => GOLD,
        Skill::Guard => GREEN,
        Skill::Nature | Skill::Strike => GREEN,
        Skill::Lightning | Skill::Meteor => SKYBLUE,
        Skill::Greedy | Skill::Blood => ORANGE,
        Skill::Imp | Skill::Balance => VIOLET,
        Skill::Chort | Skill::War => RED,
        Skill::Corrupt | Skill::Drain => LIME,
    };
    draw_rectangle(
        x,
        y,
        size,
        size,
        Color::new(color.r * 0.15, color.g * 0.15, color.b * 0.15, 1.),
    );
    draw_rectangle_lines(x + 1., y + 1., size - 2., size - 2., 1., color);
    let cx = x + size / 2.;
    let cy = y + size / 2.;
    let r = size * 0.3;
    match skill {
        Skill::Charge => {
            for d in [-0.45, 0.45] {
                draw_line(
                    cx + r * d - r * 0.5,
                    cy + r,
                    cx + r * d + r * 0.5,
                    cy - r,
                    3.,
                    color,
                );
            }
        }
        Skill::Guard => {
            draw_triangle(
                vec2(cx - r, cy - r * 0.7),
                vec2(cx + r, cy - r * 0.7),
                vec2(cx, cy + r),
                color,
            );
            draw_rectangle(cx - r, cy - r, r * 2., r * 0.5, color);
            draw_line(
                cx - r * 0.45,
                cy - r * 0.25,
                cx + r * 0.45,
                cy - r * 0.25,
                2.,
                WHITE,
            );
            draw_line(cx, cy - r * 0.7, cx, cy + r * 0.2, 2., WHITE);
        }
        Skill::Landslide => {
            for d in [-1., 0., 1.] {
                draw_line(
                    cx - r,
                    cy + d * r * 0.7,
                    cx + r,
                    cy + d * r * 0.7,
                    2.,
                    color,
                );
                draw_triangle(
                    vec2(cx + r, cy + d * r * 0.7),
                    vec2(cx + r * 0.5, cy + d * r * 0.7 - 3.),
                    vec2(cx + r * 0.5, cy + d * r * 0.7 + 3.),
                    color,
                );
            }
        }
        Skill::Lightning => {
            draw_line(cx + r * 0.4, cy - r, cx - r * 0.4, cy, 3., color);
            draw_line(cx - r * 0.4, cy, cx + r * 0.4, cy, 3., color);
            draw_line(cx + r * 0.4, cy, cx - r * 0.4, cy + r, 3., color);
        }
        Skill::Meteor => {
            draw_circle(cx - r * 0.3, cy + r * 0.3, r * 0.55, color);
            for d in [0., 0.5, 1.] {
                draw_line(
                    cx - r * 0.3 + d * 4.,
                    cy + r * 0.3,
                    cx + r,
                    cy - r + d * 4.,
                    2.,
                    color,
                );
            }
        }
        Skill::Nature => {
            draw_ellipse(cx, cy, r * 0.6, r, -35., color);
            draw_line(cx - r * 0.5, cy + r, cx + r * 0.5, cy - r, 2., WHITE);
        }
        Skill::Strike => {
            draw_poly(cx, cy, 4, r, 0., color);
            draw_line(cx - r * 0.5, cy, cx + r * 0.5, cy, 2., WHITE);
            draw_line(cx, cy - r * 0.5, cx, cy + r * 0.5, 2., WHITE);
        }
        Skill::Greedy => {
            draw_circle_lines(cx, cy, r, 2., color);
            draw_line(cx, cy - r * 0.65, cx, cy + r * 0.65, 3., color);
        }
        Skill::Blood => {
            draw_triangle(
                vec2(cx, cy - r),
                vec2(cx - r * 0.7, cy + r * 0.3),
                vec2(cx + r * 0.7, cy + r * 0.3),
                color,
            );
            draw_circle(cx, cy + r * 0.3, r * 0.7, color);
        }
        Skill::Imp => {
            draw_poly(cx, cy, 6, r, 30., color);
            draw_circle(cx, cy, r * 0.35, BLACK);
        }
        Skill::Balance => {
            for (dx, dy) in [(-1., -1.), (1., -1.), (-1., 1.), (1., 1.)] {
                draw_circle(cx + dx * r * 0.55, cy + dy * r * 0.55, r * 0.35, color);
            }
        }
        Skill::Chort => {
            draw_circle_lines(cx, cy, r, 2., color);
            draw_circle_lines(cx, cy, r * 0.5, 2., color);
        }
        Skill::War => {
            draw_triangle(
                vec2(cx, cy - r),
                vec2(cx - r, cy + r),
                vec2(cx + r, cy + r),
                color,
            );
            draw_line(cx, cy, cx, cy + r * 0.7, 2., BLACK);
        }
        Skill::Corrupt => {
            draw_arc(cx, cy, 20, r, 35., 3., 280., color);
            draw_triangle(
                vec2(cx + r, cy),
                vec2(cx + r - 6., cy - 6.),
                vec2(cx + r - 6., cy + 6.),
                color,
            );
        }
        Skill::Drain => {
            draw_circle_lines(cx, cy, r, 2., color);
            draw_line(cx - r * 0.6, cy, cx + r * 0.6, cy, 3., color);
            draw_line(cx, cy - r * 0.6, cx, cy + r * 0.6, 3., color);
        }
    }
}

/// Compact, code-drawn passive glyphs; no texture or decoder dependency.
pub fn draw_passive(passive: crate::content::Passive, x: f32, y: f32, size: f32) {
    use crate::content::Passive::*;
    let skill = match passive {
        ColumnFormation => Skill::Landslide,
        BattleTraining => Skill::Charge,
        ForestStep => Skill::Nature,
        ForestPact => Skill::Strike,
        ArcaneConversion => Skill::Lightning,
        ManaSeal | ManaRelease => Skill::Lightning,
        Recycling => Skill::Greedy,
        BalancedAffinity | BalancedResonance => Skill::Balance,
        ElementReturn => Skill::Lightning,
        BloodFury | BoilingBlood => Skill::War,
        FleshCasting | FleshConversion => Skill::Drain,
        ChargeMastery => Skill::Charge,
        GuardEconomy => Skill::Guard,
        ElvenPower => Skill::Strike,
        MeteorEconomy => Skill::Meteor,
        LootMastery => Skill::Greedy,
        BlastMastery => Skill::Imp,
        BloodSacrifice => Skill::Corrupt,
    };
    if matches!(passive, ManaSeal | ManaRelease) {
        draw_rectangle(x, y, size, size, Color::from_rgba(25, 39, 54, 255));
        let scale = size / 22.;
        if passive == ManaSeal {
            crate::draw_lock(x + size * 0.25, y + size * 0.15, SKYBLUE);
        } else {
            draw_rectangle(
                x + size * 0.24,
                y + size * 0.48,
                size * 0.52,
                size * 0.38,
                SKYBLUE,
            );
            draw_line(
                x + size * 0.35,
                y + size * 0.5,
                x + size * 0.35,
                y + size * 0.18,
                1.5 * scale,
                SKYBLUE,
            );
            draw_line(
                x + size * 0.35,
                y + size * 0.18,
                x + size * 0.7,
                y + size * 0.18,
                1.5 * scale,
                SKYBLUE,
            );
        }
    } else {
        draw(skill, x, y, size);
        if passive == ColumnFormation {
            draw_rectangle(
                x + size * 0.24,
                y + size * 0.15,
                size * 0.16,
                size * 0.7,
                GOLD,
            );
            draw_rectangle(
                x + size * 0.6,
                y + size * 0.15,
                size * 0.16,
                size * 0.7,
                GOLD,
            );
        }
        if matches!(passive, GuardEconomy | MeteorEconomy | ForestPact) {
            draw_line(
                x + size * 0.6,
                y + size * 0.8,
                x + size * 0.9,
                y + size * 0.8,
                2.,
                WHITE,
            );
        }
        if passive == BattleTraining {
            draw_circle(x + size * 0.78, y + size * 0.23, size * 0.16, GREEN);
        }
        if passive == ElementReturn {
            draw_circle(x + size * 0.78, y + size * 0.78, size * 0.14, SKYBLUE);
        }
    }
    if passive.replaces().is_some() {
        draw_triangle(
            vec2(x + size - 6., y),
            vec2(x + size, y),
            vec2(x + size, y + 6.),
            GOLD,
        );
    }
}

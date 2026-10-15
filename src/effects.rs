//! Skill-specific, deterministic visuals. Rendering never touches battle state or RNG.
use crate::content::Skill;
use macroquad::prelude::*;
fn tint(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a.clamp(0., 1.))
}
fn spark(center: Vec2, p: f32, color: Color, count: usize) {
    for i in 0..count {
        let angle = i as f32 * std::f32::consts::TAU / count as f32;
        let direction = vec2(angle.cos(), angle.sin());
        let a = center + direction * (6. + p * 24.);
        let b = center + direction * (12. + p * 34.);
        draw_line(a.x, a.y, b.x, b.y, 2., tint(color, 1. - p));
    }
}
fn bolt(a: Vec2, b: Vec2, seed: usize, alpha: f32) {
    let mut previous = a;
    let direction = (b - a).normalize_or_zero();
    let normal = vec2(-direction.y, direction.x);
    for i in 1..=7 {
        let t = i as f32 / 7.;
        let offset = if i == 7 {
            0.
        } else {
            ((seed * 13 + i * 7) as f32).sin() * 14.
        };
        let next = a.lerp(b, t) + normal * offset;
        draw_line(
            previous.x,
            previous.y,
            next.x,
            next.y,
            5.,
            tint(SKYBLUE, alpha * 0.5),
        );
        draw_line(
            previous.x,
            previous.y,
            next.x,
            next.y,
            2.,
            tint(WHITE, alpha),
        );
        previous = next;
    }
}
fn cross(center: Vec2, size: f32, color: Color) {
    draw_rectangle(center.x - size, center.y - 2., size * 2., 4., color);
    draw_rectangle(center.x - 2., center.y - size, 4., size * 2., color);
}
pub fn draw(skill: Skill, cells: &[(usize, usize)], progress: f32, enemy: bool) {
    let p = progress.clamp(0., 1.);
    let alpha = 1. - p;
    let actor = vec2(if enemy { 890. } else { 110. }, 300.);
    match skill {
        Skill::Guard => {
            draw_poly_lines(
                actor.x,
                actor.y,
                6,
                30. + p * 30.,
                30.,
                3.,
                tint(GREEN, alpha),
            );
            for i in 0..4 {
                cross(
                    actor + vec2((i as f32 * 1.7).sin() * 35., 35. - p * 80. - i as f32 * 10.),
                    7.,
                    tint(GREEN, alpha),
                );
            }
            draw_circle_lines(actor.x, actor.y + 50., 45. + p * 15., 3., tint(GOLD, alpha));
        }
        Skill::Blood => {
            for i in 0..6 {
                let angle = i as f32 * std::f32::consts::TAU / 6.;
                let from = actor + vec2(angle.cos(), angle.sin()) * 70.;
                let at = from.lerp(actor, p);
                draw_circle(
                    at.x,
                    at.y,
                    5.,
                    tint(if p < 0.45 { SKYBLUE } else { RED }, alpha),
                );
            }
            cross(actor - vec2(0., p * 35.), 12., tint(GREEN, alpha));
        }
        Skill::Strike | Skill::Landslide => {
            draw_circle_lines(actor.x, actor.y + 50., 45., 3., tint(GREEN, alpha));
            for i in 0..3 {
                let y = actor.y + 25. - p * 75. - i as f32 * 15.;
                draw_line(actor.x - 20., y + 12., actor.x, y, 3., tint(GOLD, alpha));
                draw_line(actor.x, y, actor.x + 20., y + 12., 3., tint(GOLD, alpha));
            }
        }
        _ => {}
    }
    // Grid effects finish before falling tiles obscure their original targets.
    let q = (p / 0.62).clamp(0., 1.);
    if q >= 1. {
        return;
    }
    for (i, &(x, y)) in cells.iter().enumerate() {
        let c = vec2(288. + x as f32 * 64., 142. + y as f32 * 64.);
        match skill {
            Skill::Lightning => {
                if q < 0.6 {
                    bolt(
                        vec2(c.x + (i as f32).sin() * 25., 112.),
                        c,
                        i,
                        1. - q / 0.65,
                    );
                }
                draw_circle_lines(c.x, c.y, 5. + q * 28., 2., tint(SKYBLUE, 1. - q));
                // Sparks converge on the caster: this spell restores mana.
                let t = (q - 0.2).max(0.) / 0.8;
                let at = c.lerp(actor, t);
                draw_circle(at.x, at.y, 3., tint(SKYBLUE, 1. - q));
            }
            Skill::Meteor => {
                let t = (q / 0.5).min(1.);
                let start = c - vec2(80., 160.);
                let at = start.lerp(c, t * t);
                if q < 0.5 && at.x >= 268. && at.x <= 756. && at.y >= 122. && at.y <= 610. {
                    draw_line(at.x - 25., at.y - 50., at.x, at.y, 8., tint(RED, 0.5));
                    draw_circle(at.x, at.y, 11., ORANGE);
                    draw_circle(at.x, at.y, 5., YELLOW);
                } else if q >= 0.5 {
                    let impact = (q - 0.5) * 2.;
                    draw_circle_lines(c.x, c.y, 5. + impact * 30., 3., tint(ORANGE, 1. - impact));
                    spark(c, impact, ORANGE, 5);
                }
            }
            Skill::Nature => {
                draw_line(
                    c.x,
                    c.y + 24.,
                    c.x,
                    c.y + 24. - q * 45.,
                    3.,
                    tint(GREEN, 1. - q),
                );
                draw_ellipse(
                    c.x - 8.,
                    c.y + 10. - q * 25.,
                    9.,
                    4.,
                    -35.,
                    tint(LIME, 1. - q),
                );
                draw_ellipse(c.x + 8., c.y - q * 25., 9., 4., 35., tint(GREEN, 1. - q));
                spark(c, q, GREEN, 4);
            }
            Skill::Balance => {
                let colors = [GREEN, SKYBLUE, ORANGE];
                for (j, color) in colors.iter().enumerate() {
                    let a = q * 5. + j as f32 * std::f32::consts::TAU / 3.;
                    let at = c + vec2(a.cos(), a.sin()) * (24. * (1. - q));
                    draw_circle(at.x, at.y, 5., tint(*color, 1. - q));
                }
                draw_poly_lines(
                    c.x,
                    c.y,
                    4,
                    25. * (1. - q) + 5.,
                    45. + q * 90.,
                    2.,
                    tint(VIOLET, 1. - q),
                );
            }
            Skill::War => {
                let t = (q / 0.4).min(1.);
                let at = c - vec2(0., 60. * (1. - t));
                draw_line(at.x, at.y - 15., at.x, at.y + 15., 4., tint(RED, 1. - q));
                draw_line(
                    at.x - 9.,
                    at.y - 4.,
                    at.x + 9.,
                    at.y - 4.,
                    3.,
                    tint(GOLD, 1. - q),
                );
                if q > 0.35 {
                    spark(c, (q - 0.35) / 0.65, ORANGE, 6);
                }
            }
            Skill::Imp => {
                draw_circle(c.x, c.y, 24. * (1. - q), tint(ORANGE, 0.4 * (1. - q)));
                draw_circle_lines(c.x, c.y, 8. + q * 32., 3., tint(RED, 1. - q));
                spark(c, q, ORANGE, 6);
            }
            Skill::Chort => {
                draw_poly_lines(
                    c.x,
                    c.y,
                    6,
                    30. * (1. - q) + 5.,
                    q * 90.,
                    2.,
                    tint(RED, 1. - q),
                );
                let at = c.lerp(actor, q);
                draw_circle(at.x, at.y, 4., tint(ORANGE, 1. - q));
            }
            Skill::Corrupt => {
                draw_rectangle(
                    c.x - 28.,
                    c.y + 28. - q * 56.,
                    56.,
                    8.,
                    tint(LIME, 0.7 * (1. - q)),
                );
                for j in 0..3 {
                    draw_circle(
                        c.x + (j as f32 * 2.1).sin() * 20.,
                        c.y + 18. - q * 45. - j as f32 * 6.,
                        3. + q * 3.,
                        tint(GREEN, 1. - q),
                    );
                }
            }
            Skill::Drain => {
                let at = c.lerp(actor, q);
                draw_line(c.x, c.y, at.x, at.y, 2., tint(GREEN, 0.4 * (1. - q)));
                draw_circle(at.x, at.y, 5., tint(LIME, 1. - q));
                cross(actor + vec2(0., -q * 25.), 5., tint(GREEN, (1. - q) * 0.5));
            }
            Skill::Greedy => {
                draw_circle_lines(c.x, c.y, 22. * (1. - q) + 3., 2., tint(GOLD, 1. - q));
                let at = c.lerp(actor, q) + vec2(0., -(q * std::f32::consts::PI).sin() * 55.);
                draw_circle(at.x, at.y, 5., tint(GOLD, 1. - q));
                draw_line(at.x, at.y - 3., at.x, at.y + 3., 1., tint(WHITE, 1. - q));
            }
            Skill::Guard => {
                // The board contribution is consumption, not additional healing.
                draw_rectangle_lines(
                    c.x - 24. * (1. - q),
                    c.y - 24. * (1. - q),
                    48. * (1. - q),
                    48. * (1. - q),
                    2.,
                    tint(GREEN, 1. - q),
                );
                for j in 0..3 {
                    draw_circle(
                        c.x + (j as f32 * 2.).sin() * 14.,
                        c.y - q * 28. - j as f32 * 5.,
                        3. * (1. - q) + 1.,
                        tint(LIME, 1. - q),
                    );
                }
            }
            Skill::Charge | Skill::Blood | Skill::Strike | Skill::Landslide => {}
        }
    }
}

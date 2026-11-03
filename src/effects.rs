//! Skill-specific, deterministic visuals. Rendering never touches battle state or RNG.
use crate::content::Skill;
use crate::outline;
use macroquad::prelude::*;
mod ember_blast;
mod hammer_smash;
mod heavy_landing;
mod landing_motion;
pub use landing_motion::lift as jump_lift;

pub fn jump_pose(frames: Option<&[usize]>, elapsed: f64) -> Option<[usize; 4]> {
    if elapsed >= landing_motion::LANDING as f64 {
        return None;
    }
    // Hold the existing wind-up frame; the following cast frames contain a weapon swing.
    Some([*frames?.first()?; 4])
}

/// GUI-only resources. Headless sessions never decode or upload these textures.
pub struct Textures {
    hammer_smash: Texture2D,
    heavy_landing: Texture2D,
    ember_blast: Texture2D,
    sleep_zzz: Texture2D,
    blast: Option<ember_blast::Blast>,
    cast_tiles: Vec<(usize, usize, u8)>,
    jumps: [Option<f64>; 2],
}
impl Textures {
    pub fn load() -> Self {
        Self {
            hammer_smash: crate::graphics::texture(include_bytes!(
                "../assets/effects/hammer-smash.webp"
            )),
            heavy_landing: crate::graphics::texture(include_bytes!(
                "../assets/effects/heavy-landing.webp"
            )),
            cast_tiles: Vec::new(),
            ember_blast: crate::graphics::texture(include_bytes!(
                "../assets/effects/ember-blast.webp"
            )),
            sleep_zzz: crate::graphics::texture(include_bytes!("../assets/effects/sleep-zzz.webp")),
            blast: None,
            jumps: [None; 2],
        }
    }
    /// Capture presentation-only artwork before the model removes or replaces the tiles.
    pub fn capture(
        &mut self,
        skill: Skill,
        enemy: bool,
        cells: &[(usize, usize)],
        board: &crate::Board,
    ) {
        self.cast_tiles = cells.iter().map(|&(x, y)| (x, y, board[y][x])).collect();
        if skill == Skill::Jump {
            self.jumps[enemy as usize] = Some(get_time());
        }
        if skill == Skill::EmberBlast {
            self.blast = Some(ember_blast::Blast::new(get_time()));
        }
    }
    pub fn clear(&mut self) {
        self.cast_tiles.clear();
        self.jumps = [None; 2];
        self.blast = None;
    }
    pub fn board_cleared(&mut self, cells: &[(usize, usize)]) {
        if cells.len() == crate::N * crate::N {
            if let Some(blast) = &mut self.blast {
                blast.cleared(get_time());
            }
        }
    }
    pub fn holds_board(&self) -> bool {
        self.blast.as_ref().is_some_and(|blast| blast.waiting())
    }
    pub fn draw_board_blast(&self, over: bool) {
        if let Some(blast) = &self.blast {
            blast.draw(&self.ember_blast, over);
        }
    }
    pub fn draw_sleep(&self, area: Rect, foot: Vec2, lying: bool, enemy: bool) {
        ember_blast::draw_sleep(&self.sleep_zzz, area, foot, lying, enemy);
    }
    /// A subsequent non-turn-ending cast must not cancel a jump already in flight.
    pub fn jump_elapsed(&self, enemy: bool) -> Option<f32> {
        let elapsed = (get_time() - self.jumps[enemy as usize]?) as f32;
        (elapsed < landing_motion::LANDING + landing_motion::DUST_DURATION).then_some(elapsed)
    }
    pub fn draw_landing(&self, foot: Vec2, elapsed: f32, front: bool, enemy: bool) {
        heavy_landing::draw(&self.heavy_landing, foot, elapsed, front, enemy);
    }
    pub fn draw(
        &self,
        skill: Skill,
        cells: &[(usize, usize)],
        progress: f32,
        enemy: bool,
        tiles: &[Texture2D; 7],
    ) {
        if skill == Skill::Hammer {
            let elapsed = progress * crate::presentation::CAST_TIME as f32;
            hammer_smash::draw_fragments(tiles, &self.cast_tiles, cells, elapsed);
            hammer_smash::draw(&self.hammer_smash, cells, elapsed);
        } else if skill != Skill::Jump && skill != Skill::EmberBlast {
            draw(skill, cells, progress, enemy);
        }
    }
}

/// Hammer's targets retain their mass until impact; ordinary clears keep their fade.
pub fn holds_tiles(skill: Option<Skill>, elapsed: f32) -> bool {
    matches!(skill, Some(Skill::Hammer | Skill::EmberBlast))
        && elapsed < crate::presentation::CLEAR_TIME as f32
}

pub fn sleeping(active: bool, pending: bool, casting: bool) -> bool {
    ember_blast::sleeping(active, pending, casting)
}
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
fn draw(skill: Skill, cells: &[(usize, usize)], progress: f32, enemy: bool) {
    let skill = match skill {
        Skill::Rush => Skill::Charge,
        Skill::Miracle | Skill::Copy => Skill::Strike,
        Skill::Forge => Skill::War,
        Skill::Entomb => Skill::Greedy,
        Skill::Resurrect => Skill::Nature,
        other => other,
    };
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
                outline(
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
            _ => {}
        }
    }
}

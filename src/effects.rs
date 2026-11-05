//! Skill-specific, deterministic visuals. Rendering never touches battle state or RNG.
use crate::content::Skill;
use crate::outline;
use macroquad::prelude::*;
mod ember_blast;
mod hammer_smash;
mod heavy_landing;
mod impact_clock;
mod landing_motion;
mod legacy_actors;
mod nun_heal;
mod wizard_spells;
mod wizard_timing;
pub use landing_motion::lift as jump_lift;

/// GUI-only resources. Headless sessions never decode or upload these textures.
pub struct Textures {
    hammer_smash: Texture2D,
    heavy_landing: Texture2D,
    ember_blast: Texture2D,
    sleep_zzz: Texture2D,
    blast: Option<ember_blast::Blast>,
    cast_tiles: Vec<(usize, usize, u8)>,
    jumps: [Option<f64>; 2],
    sleep_clocks: [crate::sprites::SleepClock; 2],
    idle_clocks: [crate::sprites::IdleClock; 2],
    wizard: wizard_spells::Effects,
    heal: nun_heal::Effects,
    actors: [Option<legacy_actors::Cast>; 2],
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
            sleep_clocks: Default::default(),
            idle_clocks: Default::default(),
            wizard: wizard_spells::Effects::load(),
            heal: nun_heal::Effects::load(),
            actors: [None, None],
        }
    }
    /// Capture presentation-only artwork before the model removes or replaces the tiles.
    pub fn capture(
        &mut self,
        skill: Skill,
        enemy: bool,
        cells: &[(usize, usize)],
        board: &crate::Board,
    ) -> f64 {
        self.cast_tiles = cells.iter().map(|&(x, y)| (x, y, board[y][x])).collect();
        self.wizard.capture(skill, enemy, cells, &self.cast_tiles);
        if skill == Skill::Jump {
            self.jumps[enemy as usize] = Some(get_time());
        }
        if skill == Skill::EmberBlast {
            self.blast = Some(ember_blast::Blast::new(get_time()));
        }
        if matches!(skill, Skill::Lightning | Skill::Meteor) {
            (wizard_timing::IMPACT - impact_clock::IMPACT) as f64
        } else {
            0.
        }
    }
    pub fn clear(&mut self) {
        self.cast_tiles.clear();
        self.jumps = [None; 2];
        self.sleep_clocks = Default::default();
        self.idle_clocks = Default::default();
        self.blast = None;
        self.wizard.clear();
        self.heal.clear();
        self.actors = [None, None];
    }
    pub fn begin_actor(
        &mut self,
        spec: Option<crate::sprites::ActorCast>,
        skill: Skill,
        enemy: bool,
        action: u64,
        cells: &[(usize, usize)],
    ) {
        self.actors[enemy as usize] = spec
            .map(|spec| legacy_actors::Cast::new(skill, enemy, action, cells, spec, get_time()));
    }
    pub fn actor_signal(
        &mut self,
        signal: crate::sprites::ActorSignal,
        enemy: bool,
        action: u64,
        cells: &[(usize, usize)],
    ) {
        if let Some(cast) = &mut self.actors[enemy as usize] {
            cast.notify(signal, enemy, action, cells, get_time());
        }
    }
    pub fn cancel_actors(&mut self) {
        self.actors = [None, None];
    }
    pub fn resume_actor(
        &mut self,
        spec: Option<crate::sprites::ActorCast>,
        skill: Skill,
        enemy: bool,
        action: u64,
        cells: &[(usize, usize)],
        elapsed: f64,
    ) {
        self.actors[enemy as usize] = spec.map(|spec| {
            legacy_actors::Cast::new(skill, enemy, action, cells, spec, get_time() - elapsed)
        });
    }
    pub fn actor_sample(
        &self,
        enemy: bool,
        action: u64,
        over: bool,
    ) -> Option<(crate::sprites::AnimationClip, f64)> {
        self.actors[enemy as usize]
            .as_ref()?
            .sample(action, get_time(), over)
    }
    pub fn owns_actor_effect(&self, enemy: bool, action: u64) -> bool {
        self.actors[enemy as usize]
            .as_ref()
            .is_some_and(|cast| cast.action == action)
    }
    pub fn draw_actor_effects(&self, action: u64, over: bool) {
        for cast in self.actors.iter().flatten() {
            let Some((clip, elapsed)) = cast.sample(action, get_time(), over) else {
                continue;
            };
            if cast.skill == Skill::Charge {
                continue;
            }
            let progress = if let Some(key) = clip.impact {
                // Collecting particles wait with the actor, then complete their
                // trip on the same real clear notification. Normal effects never
                // enter this opt-in branch.
                let q = if elapsed < key {
                    0.5 * elapsed / key
                } else {
                    0.5 + 0.5 * (elapsed - key) / (clip.duration() - key)
                };
                (q * 0.62) as f32
            } else {
                (elapsed / clip.duration()) as f32
            };
            draw(cast.skill, &cast.cells, progress, cast.enemy);
        }
    }
    pub fn healing_rewards(&mut self, events: &[crate::content::RewardEvent], enemy: bool) {
        self.heal.rewards(events, enemy);
    }
    pub fn healing_active(&self, enemy: bool) -> bool {
        self.heal.active(enemy)
    }
    pub fn sleep_elapsed(&mut self, enemy: bool, sleeping: bool) -> f64 {
        self.sleep_clocks[enemy as usize].elapsed(sleeping, get_time())
    }
    pub fn actor_moment(&self, enemy: bool, over: bool) -> Option<crate::sprites::CastMoment> {
        self.wizard.actor_moment(enemy, over)
    }
    pub fn idle_elapsed(&mut self, enemy: bool, casting: bool, awake: bool) -> f64 {
        self.idle_clocks[enemy as usize].elapsed(casting, awake, get_time())
    }
    pub fn draw_heal(&self, foot: Vec2, front: bool, enemy: bool) {
        self.heal.draw(foot, front, enemy);
    }
    pub fn board_cleared(&mut self, cells: &[(usize, usize)], enemy: bool, action: u64) -> f64 {
        self.actor_signal(
            crate::sprites::ActorSignal::InitialTargetsCleared,
            enemy,
            action,
            cells,
        );
        let delay = self.wizard.cleared(cells);
        if cells.len() == crate::N * crate::N {
            if let Some(blast) = &mut self.blast {
                blast.cleared(get_time());
            }
        }
        delay
    }
    pub fn holds_board(&self) -> bool {
        self.blast.as_ref().is_some_and(|blast| blast.waiting())
    }
    pub fn holds_cell(&self, x: usize, y: usize) -> bool {
        self.wizard.holds(x, y)
    }
    pub fn draw_board_blast(&self, over: bool, tiles: &[Texture2D; 7]) {
        self.wizard.draw(over, tiles);
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
        } else if !matches!(
            skill,
            Skill::Jump | Skill::EmberBlast | Skill::Lightning | Skill::Meteor
        ) {
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
    for &(x, y) in cells {
        let c = vec2(288. + x as f32 * 64., 142. + y as f32 * 64.);
        match skill {
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

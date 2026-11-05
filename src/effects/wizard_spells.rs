//! Legacy wizard visuals. The model supplies the cells; this module only paints them.
use super::impact_clock::ImpactClock;
use super::wizard_timing as timing;
use crate::content::Skill;
use macroquad::prelude::*;
pub(super) struct Effects {
    lightning: Option<Texture2D>,
    meteor: Option<Texture2D>,
    cast: Option<Cast>,
}
struct Cast {
    skill: Skill,
    enemy: bool,
    cells: Vec<(usize, usize)>,
    tiles: Vec<(usize, usize, u8)>,
    clock: ImpactClock,
    actor_active: bool,
}
impl Cast {
    fn actor_moment(
        &self,
        enemy: bool,
        over: bool,
        now: f64,
    ) -> Option<crate::sprites::CastMoment> {
        (self.actor_active && self.enemy == enemy).then(|| crate::sprites::CastMoment {
            elapsed: self.clock.elapsed_at(now, over, timing::IMPACT) as f64,
            impact: timing::IMPACT as f64,
        })
    }
}
fn frame(skill: Skill, elapsed: f32) -> Option<Rect> {
    if !matches!(skill, Skill::Lightning | Skill::Meteor) {
        return None;
    }
    if !elapsed.is_finite() || !(0.0..timing::END).contains(&elapsed) {
        return None;
    }
    let index = ((elapsed * timing::FPS).floor() as usize).min(timing::FRAMES - 1);
    let columns = timing::COLUMNS;
    Some(Rect::new(
        (index % columns) as f32 * 512.,
        (index / columns) as f32 * 512.,
        512.,
        512.,
    ))
}
fn selected(cells: &[(usize, usize)], x: usize, y: usize) -> bool {
    cells.contains(&(x, y))
}
impl Effects {
    pub fn load() -> Self {
        Self {
            lightning: Some(crate::graphics::texture(include_bytes!(
                "../../assets/effects/wizard-lightning.webp"
            ))),
            meteor: Some(crate::graphics::texture(include_bytes!(
                "../../assets/effects/wizard-meteor.webp"
            ))),
            cast: None,
        }
    }
    pub fn capture(
        &mut self,
        skill: Skill,
        enemy: bool,
        cells: &[(usize, usize)],
        original: &[(usize, usize, u8)],
    ) {
        if !matches!(skill, Skill::Lightning | Skill::Meteor) {
            if let Some(cast) = self.cast.as_mut().filter(|cast| cast.enemy == enemy) {
                // Keep the board aftermath, but a newer action owns this actor.
                cast.actor_active = false;
            }
            return;
        }
        self.cast = Some(Cast {
            skill,
            enemy,
            cells: cells.to_vec(),
            tiles: original.to_vec(),
            clock: ImpactClock::new(get_time()),
            actor_active: true,
        });
    }
    pub fn actor_moment(&self, enemy: bool, over: bool) -> Option<crate::sprites::CastMoment> {
        self.cast.as_ref()?.actor_moment(enemy, over, get_time())
    }
    pub fn clear(&mut self) {
        self.cast = None;
    }
    pub fn cleared(&mut self, cells: &[(usize, usize)]) -> f64 {
        if let Some(cast) = &mut self.cast {
            if cast.clock.waiting()
                && cells.len() == cast.cells.len()
                && cells.iter().all(|&(x, y)| selected(&cast.cells, x, y))
            {
                cast.clock.cleared(get_time());
                return timing::fall_delay(cast.skill == Skill::Meteor);
            }
        }
        0.
    }
    pub fn holds(&self, x: usize, y: usize) -> bool {
        self.cast
            .as_ref()
            .is_some_and(|cast| cast.clock.waiting() && selected(&cast.cells, x, y))
    }
    pub fn draw(&self, over: bool, tiles: &[Texture2D; 7]) {
        let Some(cast) = &self.cast else {
            return;
        };
        let elapsed = cast.clock.elapsed_at(get_time(), over, timing::IMPACT);
        let texture = match cast.skill {
            Skill::Lightning => self.lightning.as_ref(),
            _ => self.meteor.as_ref(),
        };
        let age = elapsed - timing::IMPACT;
        // The model has settled once. Paint retained originals until the wave removes them;
        // survivor motion is delayed by Presentation, so incoming tiles do not cover them.
        if !cast.clock.waiting() {
            for &(x, y, kind) in &cast.tiles {
                let Some(texture) = tiles.get(kind as usize) else {
                    continue;
                };
                let progress = timing::tile_progress(cast.skill == Skill::Meteor, x, y, age);
                if progress >= 1. {
                    continue;
                }
                let size = (64. * (1. - progress * 0.12)).round();
                draw_texture_ex(
                    texture,
                    256. + x as f32 * 64. + (64. - size) / 2.,
                    110. + y as f32 * 64. + (64. - size) / 2.,
                    Color::new(1., 1., 1., 1. - progress),
                    DrawTextureParams {
                        dest_size: Some(vec2(size, size)),
                        ..Default::default()
                    },
                );
            }
        }
        if let (Some(source), Some(texture)) = (frame(cast.skill, elapsed), texture) {
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
        // Lightning restores mana. Its motes converge on the caster's mana bar, never the opponent.
        if cast.skill == Skill::Lightning && (1.5..2.6).contains(&elapsed) {
            let target = vec2(if cast.enemy { 900. } else { 124. }, 180.);
            for (i, &(x, y)) in cast.cells.iter().enumerate() {
                let t = ((elapsed - 1.5 - i as f32 * 0.008) / 0.9).clamp(0., 1.);
                if t <= 0. || t >= 1. {
                    continue;
                }
                let start = vec2(288. + x as f32 * 64., 142. + y as f32 * 64.);
                let at = (start.lerp(target, t)
                    + vec2(0., -(t * std::f32::consts::PI).sin() * 32.))
                .round();
                draw_rectangle(
                    at.x - 3.,
                    at.y - 3.,
                    7.,
                    7.,
                    Color::new(0.3, 0.55, 0.8, 0.3),
                );
                draw_rectangle(at.x - 1., at.y - 1., 3., 3., Color::new(0.88, 0.94, 1., 1.));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    #[test]
    fn actor_waits_with_both_spells_and_recovers_from_the_actual_clear() {
        for skill in [Skill::Lightning, Skill::Meteor] {
            let mut cast = Cast {
                skill,
                enemy: false,
                cells: vec![],
                tiles: vec![],
                clock: ImpactClock::new(10.),
                actor_active: true,
            };
            let clip = crate::sprites::WIZARD_CAST;
            let frame = |cast: &Cast, now, over| {
                clip.frame(clip.aligned_time(cast.actor_moment(false, over, now).unwrap()))
            };
            assert_eq!(frame(&cast, 10., false), 24);
            assert_eq!(frame(&cast, 10.35, false), 28);
            assert_eq!(frame(&cast, 12., false), 30); // beyond the old 1.4s lifetime
            assert!(cast.actor_moment(true, false, 12.).is_none());
            cast.clock.cleared(12.);
            assert_eq!(frame(&cast, 12., false), 31);
            assert_eq!(frame(&cast, 12.45, false), 35);
            assert!(
                clip.aligned_time(cast.actor_moment(false, false, 12.51).unwrap())
                    > clip.duration()
            );
            cast.actor_active = false;
            assert!(cast.actor_moment(false, false, 12.6).is_none());
            let terminal = Cast {
                clock: ImpactClock::new(20.),
                actor_active: true,
                ..cast
            };
            assert_eq!(frame(&terminal, 20.95, true), 31); // no clear after victory
            assert!(
                clip.aligned_time(terminal.actor_moment(false, true, 22.).unwrap())
                    > clip.duration()
            );
        }
    }
    #[test]
    fn wave_clears_inner_tiles_before_outer_tiles_and_waits_before_falling() {
        assert!(timing::arrival(true, 3, 3) < timing::arrival(true, 0, 3));
        let mid = (timing::arrival(true, 3, 3) + timing::arrival(true, 0, 3)) / 2.;
        assert_eq!(timing::tile_progress(true, 3, 3, mid), 1.);
        assert_eq!(timing::tile_progress(true, 0, 3, mid), 0.);
        for y in 0..8 {
            for x in 0..8 {
                // A motion profile does not choose targets; test only the supplied diamond fixture.
                if (x as f32 - 3.5).abs() + (y as f32 - 3.5).abs() <= 4. {
                    assert_eq!(
                        timing::tile_progress(true, x, y, timing::fall_delay(true) as f32),
                        1.
                    );
                }
            }
        }
        assert_eq!(
            frame(Skill::Lightning, 0.9).unwrap(),
            Rect::new(1536., 512., 512., 512.)
        );
        assert!(frame(Skill::Meteor, 2.8).is_none());
        let mut clock = ImpactClock::new(10.);
        assert!(clock.elapsed_at(12., false, timing::IMPACT) < timing::IMPACT);
        clock.cleared(12.);
        assert_eq!(clock.elapsed_at(12., false, timing::IMPACT), timing::IMPACT);
    }
    #[test]
    fn expanded_spell_atlases_hit_and_end_transparent() {
        for bytes in [
            &include_bytes!("../../assets/effects/wizard-lightning.webp")[..],
            &include_bytes!("../../assets/effects/wizard-meteor.webp")[..],
        ] {
            let atlas = image::load_from_memory(bytes).unwrap().to_rgba8();
            assert_eq!(atlas.dimensions(), (3072, 2560));
            assert!(atlas
                .view(1536, 512, 512, 512)
                .pixels()
                .any(|(_, _, p)| p.0[3] > 200));
            let last = timing::FRAMES as u32 - 1;
            assert!(atlas
                .view(last % 6 * 512, last / 6 * 512, 512, 512)
                .pixels()
                .all(|(_, _, p)| p.0[3] == 0));
        }
    }
}

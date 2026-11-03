//! Target-local holy/heal overlays driven by final, uncancelled reward events.
use crate::content::{EventSide, EventSource, Resource, RewardEvent, Skill};
use macroquad::prelude::*;
const DURATION: f64 = 2.;

/// Keep both intentions if a target receives healing and converted damage together.
/// Full-health healing still has an event; cancelled ancestors never reach this list.
pub(super) fn targets(events: &[RewardEvent], actor_enemy: bool) -> [[bool; 2]; 2] {
    let mut targets = [[false; 2]; 2];
    for e in events {
        if e.amount <= 0 || !e.has_source(EventSource::Skill(Skill::Healing)) {
            continue;
        }
        let variant = match e.resource {
            Resource::Healing => 0,
            Resource::Damage => 1,
            Resource::Mana => continue,
        };
        let enemy = actor_enemy ^ (e.recipient == EventSide::Opponent);
        targets[enemy as usize][variant] = true;
    }
    targets
}
pub(super) struct Effects {
    textures: [Texture2D; 2],
    started: [[Option<f64>; 2]; 2],
}
impl Effects {
    pub fn load() -> Self {
        Self {
            textures: [
                crate::graphics::texture(include_bytes!(
                    "../../assets/effects/nun-heal-normal.webp"
                )),
                crate::graphics::texture(include_bytes!(
                    "../../assets/effects/nun-heal-damage.webp"
                )),
            ],
            started: [[None; 2]; 2],
        }
    }
    pub fn clear(&mut self) {
        self.started = [[None; 2]; 2];
    }
    pub fn rewards(&mut self, events: &[RewardEvent], actor_enemy: bool) {
        let now = get_time();
        for (side, variants) in targets(events, actor_enemy).iter().enumerate() {
            for (variant, &active) in variants.iter().enumerate() {
                if active {
                    self.started[side][variant] = Some(now);
                }
            }
        }
    }
    pub fn active(&self, enemy: bool) -> bool {
        self.started[enemy as usize]
            .iter()
            .flatten()
            .any(|start| (0.0..DURATION).contains(&(get_time() - start)))
    }
    pub fn draw(&self, foot: Vec2, front: bool, enemy: bool) {
        let panel = crate::graphics::side_panel_rect(enemy);
        let clip = Rect::new(
            panel.x,
            188.,
            panel.w,
            crate::graphics::passive_rect(enemy).y - 188.,
        );
        for variant in 0..2 {
            let Some(start) = self.started[enemy as usize][variant] else {
                continue;
            };
            let age = get_time() - start;
            if !(0.0..DURATION).contains(&age) {
                continue;
            }
            let i = (age * 20.).floor() as usize;
            let origin = (foot - vec2(80., 176.)).round();
            let Some(visible) = Rect::new(origin.x, origin.y, 160., 192.).intersect(clip) else {
                continue;
            };
            let source = Rect::new(
                (i % 8) as f32 * 160. + visible.x - origin.x,
                (i / 8 + if front { 5 } else { 0 }) as f32 * 192. + visible.y - origin.y,
                visible.w,
                visible.h,
            );
            draw_texture_ex(
                &self.textures[variant],
                visible.x,
                visible.y,
                WHITE,
                DrawTextureParams {
                    source: Some(source),
                    ..Default::default()
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::*;
    #[test]
    fn heal_variants_follow_settled_conversions_caps_and_actor_side() {
        let mut frenzy = StatusSet::default();
        frenzy.insert(
            StatusId::Frenzy,
            Skill::Frenzy,
            StatusDuration::OwnerTurnEnd(1),
        );
        let mut rational = StatusSet::default();
        rational.insert(
            StatusId::Rational,
            Skill::Rational,
            StatusDuration::OwnerTurnEnd(1),
        );
        for (awake, statuses, expected) in [
            (false, StatusSet::default(), [[true, false], [true, false]]),
            (false, frenzy, [[true, false], [false, true]]),
            (true, StatusSet::default(), [[false, false], [false, true]]),
            (true, rational, [[true, false], [true, false]]),
        ] {
            let batch = RewardBatch::gains((43, 0, 0), 49, EventSource::Skill(Skill::Healing));
            let settled = settle_batch_in(
                NUN,
                health_cap(NUN, awake),
                100,
                700,
                batch,
                Rules {
                    awake,
                    statuses,
                    opponent: Some((KNIGHT, false)),
                    ..Default::default()
                },
            );
            assert_eq!(targets(&settled.events, false), expected);
            assert_eq!(targets(&settled.events, true), [expected[1], expected[0]]);
        }
        let unrelated =
            RewardBatch::gains((8, 0, 0), 0, EventSource::Passive(Passive::AutoRecovery));
        assert_eq!(targets(&unrelated.events, false), [[false; 2]; 2]);
    }
    #[test]
    fn atlases_have_two_layers_and_transparent_endings() {
        for bytes in [
            include_bytes!("../../assets/effects/nun-heal-normal.webp").as_slice(),
            include_bytes!("../../assets/effects/nun-heal-damage.webp").as_slice(),
        ] {
            let im = image::load_from_memory(bytes).unwrap().to_rgba8();
            assert_eq!(im.dimensions(), (1280, 1920));
            for layer in 0..2 {
                assert!(im
                    .enumerate_pixels()
                    .filter(|(x, y, _)| *x >= 1120
                        && *y >= (4 + layer * 5) * 192
                        && *y < (5 + layer * 5) * 192)
                    .all(|(_, _, p)| p[3] == 0));
            }
        }
    }
}

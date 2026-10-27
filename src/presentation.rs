use crate::{board::TileFall, label, Board, N};
use macroquad::prelude::*;
pub const CHARGE_TIME: f64 = 0.95;
pub const SWAP_TIME: f64 = 0.35;
pub const CLEAR_TIME: f64 = 0.4;
pub const FALL_TIME: f64 = 0.4;
pub const TURN_HOLD: f64 = 0.85;
/// Single-frame skins breathe and gently shift their weight around planted feet.
/// Multi-frame original sprites retain their authored animation unchanged.
pub fn idle_pose(size: Vec2, time: f64, single_frame: bool) -> (Vec2, f32) {
    if !single_frame {
        return (size, 0.);
    }
    let phase = (time * std::f64::consts::TAU / 2.4) as f32;
    let breath = phase.sin();
    (
        vec2(
            size.x * (1. - 0.008 * breath),
            size.y * (0.982 + 0.018 * breath),
        ),
        0.009 * (phase * 0.5).sin(),
    )
}
struct Motion {
    from: Vec2,
    to: Vec2,
    kind: u8,
}
struct Popup {
    enemy: bool,
    before: i32,
    after: i32,
    time: f32,
    source: String,
}
struct Awakening {
    enemy: bool,
    before: i32,
    hit: i32,
    old_max: i32,
    new_max: i32,
    time: f64,
}
const AWAKEN_TIME: f64 = 1.1;
#[derive(Default)]
pub struct Presentation {
    awakenings: Vec<Awakening>,
    motions: Vec<Motion>,
    time: f64,
    duration: f64,
    popups: Vec<Popup>,
    actor_enemy: bool,
    swapping: bool,
    charge_column: Option<usize>,
}
fn pos(x: usize, y: isize) -> Vec2 {
    vec2(256. + x as f32 * 64., 110. + y as f32 * 64.)
}
impl Presentation {
    pub fn stop_motion(&mut self) {
        self.motions.clear();
        self.time = 0.;
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn tick(&mut self, dt: f64) {
        self.time = (self.time - dt).max(0.);
        if self.time == 0. {
            self.charge_column = None;
            self.motions.clear();
        }
        for a in &mut self.awakenings {
            a.time -= dt;
        }
        self.awakenings.retain(|a| a.time > 0.);
        for p in &mut self.popups {
            p.time -= dt as f32;
        }
        self.popups.retain(|p| p.time > 0.);
    }
    pub fn charge(&mut self, column: usize, remaining: f64) {
        self.motions.clear();
        self.charge_column = Some(column.min(N - 2));
        self.time = remaining;
        self.duration = CHARGE_TIME;
        self.swapping = false;
    }
    pub fn charging(&self) -> bool {
        self.charge_column.is_some() && self.busy()
    }
    pub fn charge_remaining(&self) -> f64 {
        if self.charging() {
            self.time
        } else {
            0.
        }
    }
    pub fn destruction(&self, x: usize, y: usize) -> Option<f32> {
        let column = self.charge_column?;
        if x < column || x > column + 1 {
            return None;
        }
        let head = -0.8 + (1. - self.time / CHARGE_TIME) as f32 * (N as f32 + 1.6);
        Some(((head - y as f32) * 2.).clamp(0., 1.))
    }
    pub fn draw_charge(&self, texture: &Texture2D, w: f32, h: f32, indices: &[usize]) {
        let Some(column) = self.charge_column else {
            return;
        };
        let t = (1. - self.time / CHARGE_TIME) as f32;
        let center = 256. + (column as f32 + 1.) * 64.;
        let y = 62. + t * (N as f32 * 64. + 100.);
        let width = w / h * 72.;
        for i in 1..=4 {
            let length = 20. + i as f32 * 12.;
            draw_line(
                center - 28. + i as f32 * 10.,
                (y - length).clamp(110., 622.),
                center - 28. + i as f32 * 10.,
                (y - 8.).clamp(110., 622.),
                3.,
                Color::new(1., 0.8, 0.2, 0.35),
            );
        }
        let index = indices[((t * 12.) as usize) % indices.len()];
        let top = (110. - (y - 50.)).max(0.);
        let bottom = (y + 22. - 622.).max(0.);
        let visible = 72. - top - bottom;
        if visible > 0. {
            draw_texture_ex(
                texture,
                center - width / 2.,
                y - 50. + top,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, visible)),
                    source: Some(Rect::new(
                        index as f32 * w,
                        top / 72. * h,
                        w,
                        visible / 72. * h,
                    )),
                    ..Default::default()
                },
            );
        }
        if (110.0..=622.0).contains(&(y + 20.)) {
            draw_line(center - 55., y + 20., center + 55., y + 20., 4., GOLD);
        }
    }
    pub fn swapping(&self) -> Option<bool> {
        self.busy().then_some(self.swapping)
    }
    pub fn busy(&self) -> bool {
        self.time > 0. || !self.awakenings.is_empty()
    }
    pub fn awakening(
        &mut self,
        enemy: bool,
        before: i32,
        old_max: i32,
        new_max: i32,
        restored: i32,
    ) {
        self.awakenings.retain(|a| a.enemy != enemy);
        self.awakenings.push(Awakening {
            enemy,
            before,
            hit: new_max - restored,
            old_max,
            new_max,
            time: AWAKEN_TIME,
        });
    }
    pub fn awakened_visible(&self, enemy: bool) -> bool {
        self.awakenings
            .iter()
            .find(|a| a.enemy == enemy)
            .is_none_or(|a| a.time < AWAKEN_TIME - 0.45)
    }
    pub fn health_bar(&self, enemy: bool, hp: i32, max: i32) -> (f32, f32, f32) {
        let Some(a) = self.awakenings.iter().find(|a| a.enemy == enemy) else {
            return (hp as f32, max as f32, 1.);
        };
        let t = (AWAKEN_TIME - a.time) as f32;
        let lerp = |x: f32, y: f32, t: f32| x + (y - x) * t.clamp(0., 1.);
        let cap = lerp(a.old_max as f32, a.new_max as f32, (t - 0.15) / 0.3);
        let value = if t < 0.15 {
            lerp(a.before as f32, a.hit as f32, t / 0.15)
        } else {
            lerp(a.hit as f32, a.new_max as f32, (t - 0.45) / 0.35)
        };
        let extent = if t < 0.8 {
            cap / a.old_max as f32
        } else {
            lerp(a.new_max as f32 / a.old_max as f32, 1., (t - 0.8) / 0.3)
        };
        (value, cap, extent)
    }
    pub fn awakening_progress(&self, enemy: bool) -> Option<f32> {
        self.awakenings
            .iter()
            .find(|a| a.enemy == enemy)
            .map(|a| (AWAKEN_TIME - a.time) as f32)
    }
    pub fn mana_bar(&self, enemy: bool, mp: i32, old_max: i32, max: i32) -> (f32, f32, f32) {
        let progress = self
            .awakenings
            .iter()
            .find(|a| a.enemy == enemy)
            .map_or(1., |a| {
                (((AWAKEN_TIME - a.time) as f32 - 0.15) / 0.65).clamp(0., 1.)
            });
        (
            mp as f32,
            old_max as f32 + (max - old_max) as f32 * progress,
            1.,
        )
    }
    pub fn draw_awakening(&self, zh: bool) {
        for a in &self.awakenings {
            let panel = crate::side_panel_rect(a.enemy);
            let t = (AWAKEN_TIME - a.time) as f32;
            let alpha = if t < 0.2 {
                t / 0.2
            } else {
                ((1.1 - t) / 0.35).clamp(0., 1.)
            };
            let x = panel.x + panel.w / 2.;
            crate::label_centered(
                if zh { "觉醒" } else { "AWAKENING" },
                x,
                255. - t * 9.,
                24.,
                Color::new(1., 0.8, 0.4, alpha),
            );
            if t > 0.45 {
                let text = format!("+{} {}", a.new_max - a.hit, if zh { "生命" } else { "HP" });
                crate::label_centered(
                    &text,
                    x,
                    282. - t * 9.,
                    18.,
                    Color::new(0.6, 1., 0.7, alpha),
                );
            }
        }
    }
    pub fn hides(&self, x: usize, y: usize) -> bool {
        self.busy() && self.motions.iter().any(|m| m.to == pos(x, y as isize))
    }
    pub fn swap(
        &mut self,
        before: &Board,
        a: (usize, usize),
        b: (usize, usize),
        parallel: bool,
        actor_enemy: bool,
        hero: usize,
    ) {
        self.actor_enemy = actor_enemy;
        self.swapping = true;
        let mut pairs = vec![(a, b)];
        if parallel {
            for delta in [-1isize, 1] {
                let (p, q) = if a.0 == b.0 {
                    (
                        (a.0 as isize + delta, a.1 as isize),
                        (b.0 as isize + delta, b.1 as isize),
                    )
                } else {
                    (
                        (a.0 as isize, a.1 as isize + delta),
                        (b.0 as isize, b.1 as isize + delta),
                    )
                };
                if [p.0, p.1, q.0, q.1]
                    .iter()
                    .all(|&v| v >= 0 && v < N as isize)
                {
                    pairs.push(((p.0 as usize, p.1 as usize), (q.0 as usize, q.1 as usize)));
                }
            }
        }
        if hero == 0 && a.1 == b.1 && !parallel {
            pairs = (0..N).map(|y| ((a.0, y), (b.0, y))).collect();
        }
        self.motions.clear();
        for (a, b) in pairs {
            for (from, to) in [(a, b), (b, a)] {
                self.motions.push(Motion {
                    from: pos(from.0, from.1 as isize),
                    to: pos(to.0, to.1 as isize),
                    kind: before[from.1][from.0],
                });
            }
        }
        self.time = SWAP_TIME;
        self.duration = SWAP_TIME;
    }
    pub fn fall(&mut self, falls: Vec<TileFall>) {
        self.swapping = false;
        self.motions = falls
            .into_iter()
            .map(|f| Motion {
                from: pos(f.x, f.from_y),
                to: pos(f.x, f.to_y as isize),
                kind: f.kind,
            })
            .collect();
        self.time = FALL_TIME;
        self.duration = FALL_TIME;
    }
    pub fn health(&mut self, before: [i32; 2], after: [i32; 2], source: &str) {
        for i in 0..2 {
            if before[i] != after[i] {
                self.popups.push(Popup {
                    enemy: i == 1,
                    before: before[i],
                    after: after[i],
                    time: 2.4,
                    source: source.to_string(),
                });
            }
        }
    }
    pub fn draw_tiles(&self, tiles: &[Texture2D; 7]) {
        if self.time <= 0. {
            return;
        }
        let t = 1. - (self.time / self.duration) as f32;
        let eased = t * t * (3. - 2. * t);
        for m in &self.motions {
            let p = m.from.lerp(m.to, eased);
            let crop = (110. - p.y).max(0.);
            if crop >= 64. {
                continue;
            }
            let texture = &tiles[m.kind as usize];
            draw_texture_ex(
                texture,
                p.x,
                p.y + crop,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(64., 64. - crop)),
                    source: Some(Rect::new(
                        0.,
                        crop / 64. * texture.height(),
                        texture.width(),
                        (1. - crop / 64.) * texture.height(),
                    )),
                    ..Default::default()
                },
            );
            if self.swapping {
                draw_rectangle_lines(
                    p.x + 2.,
                    p.y + 2.,
                    60.,
                    60.,
                    3.,
                    if self.actor_enemy { ORANGE } else { SKYBLUE },
                );
            }
        }
    }
    pub fn draw_health(&self, zh: bool) {
        for side in [false, true] {
            for (i, p) in self
                .popups
                .iter()
                .filter(|p| p.enemy == side)
                .rev()
                .take(3)
                .enumerate()
            {
                let x = if side { 802. } else { 22. };
                let y = 246. + i as f32 * 58. - (2.4 - p.time) * 8.;
                let delta = p.after - p.before;
                let color = if delta < 0 { ORANGE } else { GREEN };
                draw_rectangle(x, y - 22., 200., 52., Color::new(0.04, 0.05, 0.08, 0.93));
                label(
                    &format!(
                        "{} {}",
                        if delta < 0 {
                            if zh {
                                "受伤"
                            } else {
                                "Damage"
                            }
                        } else {
                            if zh {
                                "回复"
                            } else {
                                "Heal"
                            }
                        },
                        delta.abs()
                    ),
                    x + 5.,
                    y,
                    24.,
                    color,
                );
                label(
                    &format!("{} → {} · {}", p.before, p.after, p.source),
                    x + 5.,
                    y + 20.,
                    13.,
                    WHITE,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn charge_sweep_clears_only_selected_columns_in_order() {
        let mut p = Presentation::default();
        p.charge(2, CHARGE_TIME);
        assert_eq!(p.destruction(2, 0), Some(0.));
        assert_eq!(p.destruction(4, 0), None);
        p.tick(CHARGE_TIME * 0.5);
        assert_eq!(p.destruction(2, 0), Some(1.));
        assert_eq!(p.destruction(2, N - 1), Some(0.));
        assert!(p.charge_remaining() > 0.);
        p.tick(CHARGE_TIME);
        assert!(!p.charging());
    }
    #[test]
    fn motion_finishes_without_resetting_health_feedback() {
        let b = [[2; N]; N];
        let mut p = Presentation::default();
        p.swap(&b, (0, 0), (1, 0), false, true, 2);
        p.health([700, 500], [700, 350], "charge");
        assert!(p.hides(0, 0));
        assert!(p.actor_enemy);
        p.tick(SWAP_TIME + 0.01);
        assert!(!p.busy());
        assert_eq!(p.popups.len(), 1);
        assert_eq!(p.popups[0].before - p.popups[0].after, 150);
        p.tick(3.);
        assert!(p.popups.is_empty());
    }
    #[test]
    fn animation_covers_every_parallel_pair_and_rollback_clears_it() {
        let mut p = Presentation::default();
        p.swap(&[[2; N]; N], (3, 3), (4, 3), true, false, 2);
        assert_eq!(p.motions.len(), 6);
        p.clear();
        assert!(!p.busy());
        assert!(!p.hides(3, 3));
    }
}

/// A two-headed arrow identifies the two exchange endpoints. Knight horizontal
/// inputs outline the entire columns, rather than implying a single tile swap.
pub fn draw_move_hint(
    a: (usize, usize),
    b: (usize, usize),
    columns: bool,
    number: usize,
    focused: bool,
) {
    let center = |p: (usize, usize)| vec2(288. + p.0 as f32 * 64., 142. + p.1 as f32 * 64.);
    let (from, to) = (center(a), center(b));
    let color = if focused {
        WHITE
    } else if columns {
        GOLD
    } else {
        SKYBLUE
    };
    if columns {
        for x in [a.0, b.0] {
            draw_rectangle_lines(256. + x as f32 * 64. + 5., 115., 54., 502., 2., GOLD);
        }
    }
    let dir = (to - from).normalize();
    let normal = vec2(-dir.y, dir.x);
    let start = from + dir * 13.;
    let end = to - dir * 13.;
    draw_line(
        start.x,
        start.y,
        end.x,
        end.y,
        7.,
        Color::new(0.02, 0.04, 0.08, 0.95),
    );
    draw_line(start.x, start.y, end.x, end.y, 3., color);
    for (tip, inward) in [(start, dir), (end, -dir)] {
        for side in [-1., 1.] {
            let wing = tip + inward * 9. + normal * side * 6.;
            draw_line(
                tip.x,
                tip.y,
                wing.x,
                wing.y,
                5.,
                Color::new(0.02, 0.04, 0.08, 0.95),
            );
            draw_line(tip.x, tip.y, wing.x, wing.y, 2., color);
        }
    }
    let badge = move_hint_anchor(a, b);
    draw_rectangle(
        badge.x - 10.,
        badge.y - 8.,
        22.,
        17.,
        Color::new(0.03, 0.05, 0.09, 0.95),
    );
    label(&number.to_string(), badge.x - 8., badge.y + 5., 12., color);
}

pub fn move_hint_anchor(a: (usize, usize), b: (usize, usize)) -> Vec2 {
    let center = |p: (usize, usize)| vec2(288. + p.0 as f32 * 64., 142. + p.1 as f32 * 64.);
    let (from, to) = (center(a), center(b));
    let dir = (to - from).normalize();
    (from + to) * 0.5 + vec2(-dir.y, dir.x) * 12.
}

#[cfg(test)]
mod awakening_animation_tests {
    use super::*;
    #[test]
    fn wizard_unlock_animation_keeps_mana_and_other_side_independent() {
        let mut p = Presentation::default();
        p.awakening(false, 230, 550, 275, 275);
        assert_eq!(p.mana_bar(false, 250, 250, 400), (250., 250., 1.));
        assert_eq!(p.mana_bar(true, 200, 250, 400), (200., 400., 1.));
        p.tick(0.45);
        let (mp, cap, _) = p.mana_bar(false, 250, 250, 400);
        assert_eq!(mp, 250.);
        assert!(cap > 250. && cap < 400.);
        assert!(p.awakening_progress(false).is_some());
        p.tick(0.7);
        assert_eq!(p.mana_bar(false, 250, 250, 400), (250., 400., 1.));
        assert!(p.awakening_progress(false).is_none());
    }
    #[test]
    fn awakening_shrinks_refills_normalizes_and_blocks_resolution() {
        let mut p = Presentation::default();
        p.awakening(false, 330, 700, 385, 385);
        assert!(p.busy());
        assert!(!p.awakened_visible(false));
        assert_eq!(p.health_bar(false, 385, 385), (330., 700., 1.));
        p.tick(0.3);
        let (hp, cap, width) = p.health_bar(false, 385, 385);
        assert_eq!(hp, 0.);
        assert!(cap > 385. && cap < 700.);
        assert!(width < 1.);
        p.tick(0.35);
        let (hp, cap, width) = p.health_bar(false, 385, 385);
        assert!(hp > 0. && hp < 385.);
        assert_eq!(cap, 385.);
        assert!(width < 1.);
        assert!(p.awakened_visible(false));
        p.tick(0.46);
        assert!(!p.busy());
        assert_eq!(p.health_bar(false, 385, 385), (385., 385., 1.));
        p.awakening(true, 20, 700, 385, 385);
        p.clear();
        assert!(!p.busy());
    }
}

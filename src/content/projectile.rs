//! Board-cell units, f64 analytic trajectories and continuous circle/rectangle contacts.
//! Equal-time events use a 1 ns tolerance and row-major order inside each hit batch.
use super::*;
use crate::board;
const EPS: f64 = 1e-9;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ammo {
    #[default]
    Solid,
    Explosive,
}
impl Ammo {
    pub fn name(self, zh: bool) -> &'static str {
        crate::tr_key(
            zh,
            match self {
                Self::Solid => "projectile.ammo.solid",
                Self::Explosive => "projectile.ammo.explosive",
            },
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aim {
    pub ammo: Ammo,
    pub angle_mdeg: i32,
    pub power_permille: u16,
}
impl Default for Aim {
    fn default() -> Self {
        Self {
            ammo: Ammo::Solid,
            angle_mdeg: 35000,
            power_permille: 1000,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct RewardRule {
    pub percent: [i32; 3],
    pub probability: [u8; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Explosion {
    pub mass: f64,
    pub radius: f64,
    pub landing: bool,
    pub fuse: Option<f64>,
    pub penetrations: Option<usize>,
    pub reward: RewardRule,
}
#[derive(Clone, Copy, Debug)]
pub struct ProjectileConfig {
    pub from_right: bool,
    pub min_speed: f64,
    pub max_speed: f64,
    pub min_angle: i32,
    pub max_angle: i32,
    pub gravity: f64,
    pub gravity_direction: board::Gravity,
    pub radius: f64,
    pub direct_reward: RewardRule,
    pub explosion: Option<Explosion>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
pub const ORIGIN: Point = Point { x: -0.3, y: 7.2 };
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Landing,
    OutOfBounds,
    Fuse,
    Penetrations,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub cell: (usize, usize),
    pub time: f64,
    pub position: Point,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Flight {
    pub ammo: Ammo,
    pub origin: Point,
    pub velocity: Point,
    pub gravity: f64,
    pub gravity_direction: board::Gravity,
    pub duration: f64,
    pub end: End,
    pub hits: Vec<Hit>,
    pub explosion_cells: Vec<(usize, usize)>,
    pub explosion_radius: Option<f64>,
}
impl Flight {
    pub fn position(&self, time: f64) -> Point {
        let t = time.clamp(0., self.duration);
        let acceleration = self.acceleration();
        Point {
            x: self.origin.x + self.velocity.x * t + 0.5 * acceleration.x * t * t,
            y: self.origin.y + self.velocity.y * t + 0.5 * acceleration.y * t * t,
        }
    }
    fn acceleration(&self) -> Point {
        let (x, y) = self.gravity_direction.vector();
        Point {
            x: x as f64 * self.gravity,
            y: y as f64 * self.gravity,
        }
    }
    pub fn cells(&self) -> Vec<(usize, usize)> {
        self.hits
            .iter()
            .map(|h| h.cell)
            .chain(self.explosion_cells.iter().copied())
            .collect()
    }
}
impl ProjectileConfig {
    pub fn aim_for_target(&self, target: (usize, usize)) -> Aim {
        Aim {
            angle_mdeg: self.min_angle
                + (self.max_angle - self.min_angle) * target.1.min(N - 1) as i32 / (N - 1) as i32,
            power_permille: (target.0.min(N - 1) * 1000 / (N - 1)) as u16,
            ..Default::default()
        }
    }
    pub fn clamp_aim(&self, aim: Aim) -> Aim {
        Aim {
            ammo: if self.explosion.is_some() {
                aim.ammo
            } else {
                Ammo::Solid
            },
            angle_mdeg: aim.angle_mdeg.clamp(self.min_angle, self.max_angle),
            power_permille: aim.power_permille.min(1000),
            ..Default::default()
        }
    }
    pub fn solve(&self, board: &Board, aim: Aim) -> Flight {
        if self.from_right {
            let mut mirrored = *board;
            for row in &mut mirrored {
                row.reverse();
            }
            let mut flight = Self {
                from_right: false,
                gravity_direction: match self.gravity_direction {
                    board::Gravity::Left => board::Gravity::Right,
                    board::Gravity::Right => board::Gravity::Left,
                    direction => direction,
                },
                ..*self
            }
            .solve(&mirrored, aim);
            flight.gravity_direction = self.gravity_direction;
            flight.origin.x = N as f64 - flight.origin.x;
            flight.velocity.x = -flight.velocity.x;
            for hit in &mut flight.hits {
                hit.cell.0 = N - 1 - hit.cell.0;
                hit.position.x = N as f64 - hit.position.x;
            }
            flight.hits.sort_by(|a, b| {
                a.time
                    .total_cmp(&b.time)
                    .then(a.cell.1.cmp(&b.cell.1))
                    .then(a.cell.0.cmp(&b.cell.0))
            });
            for cell in &mut flight.explosion_cells {
                cell.0 = N - 1 - cell.0;
            }
            flight.explosion_cells.sort_by_key(|&(x, y)| (y, x));
            return flight;
        }
        let aim = self.clamp_aim(aim);
        let explosion = self.explosion.filter(|_| aim.ammo == Ammo::Explosive);
        let angle = (aim.angle_mdeg as f64 / 1000.).to_radians();
        let speed =
            self.min_speed + (self.max_speed - self.min_speed) * aim.power_permille as f64 / 1000.;
        let speed = speed / explosion.map_or(1., |ammo| ammo.mass.sqrt());
        let velocity = Point {
            x: speed * angle.cos(),
            y: -speed * angle.sin(),
        };
        let mut flight = Flight {
            ammo: aim.ammo,
            origin: ORIGIN,
            velocity,
            gravity: self.gravity,
            gravity_direction: self.gravity_direction,
            duration: 0.,
            end: End::OutOfBounds,
            hits: vec![],
            explosion_cells: vec![],
            explosion_radius: None,
        };
        let acceleration = flight.acceleration();
        // A finite bound on horizontal exit, including a shot that reverses before
        // entering the board. The launch lane remains outside the board edge.
        let horizon = if acceleration.x.abs() > EPS {
            (2. * velocity.x.abs() + (2. * acceleration.x.abs() * (N as f64 + 2.)).sqrt())
                / acceleration.x.abs()
        } else {
            (N as f64 + self.radius - ORIGIN.x) / velocity.x
        };
        let (gx, gy) = self.gravity_direction.vector();
        let landing_origin = ORIGIN.x * gx as f64 + ORIGIN.y * gy as f64;
        let landing_velocity = velocity.x * gx as f64 + velocity.y * gy as f64;
        let landing_edge = if gx + gy > 0 {
            N as f64 - self.radius
        } else {
            -self.radius
        };
        let landing = roots(
            &[
                landing_origin - landing_edge,
                landing_velocity,
                self.gravity * 0.5,
            ],
            0.,
            horizon,
        )
        .into_iter()
        .find(|t| landing_velocity + self.gravity * t > EPS);
        let boundary = [(ORIGIN.x - self.radius, -1.), (N as f64 + self.radius, 1.)]
            .into_iter()
            .flat_map(|(edge, sign)| {
                roots(
                    &[ORIGIN.x - edge, velocity.x, acceleration.x * 0.5],
                    0.,
                    horizon,
                )
                .into_iter()
                .filter(move |t| (velocity.x + acceleration.x * t) * sign > EPS)
            })
            .min_by(f64::total_cmp)
            .unwrap_or(horizon);
        flight.duration = landing.unwrap_or(f64::INFINITY).min(boundary);
        flight.end = if landing.is_some_and(|t| t <= boundary) {
            End::Landing
        } else {
            End::OutOfBounds
        };
        if let Some(explosion) = explosion {
            if let Some(fuse) = explosion.fuse.filter(|t| *t <= flight.duration + EPS) {
                flight.duration = fuse.min(flight.duration);
                flight.end = End::Fuse;
            }
        }
        for (y, row) in board.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if *tile < 7 {
                    if let Some(time) = contact(&flight, (x, y), self.radius) {
                        flight.hits.push(Hit {
                            cell: (x, y),
                            time,
                            position: flight.position(time),
                        });
                    }
                }
            }
        }
        flight.hits.sort_by(|a, b| {
            a.time
                .total_cmp(&b.time)
                .then(a.cell.1.cmp(&b.cell.1))
                .then(a.cell.0.cmp(&b.cell.0))
        });
        if let Some(explosion) = explosion {
            if let Some(limit) = explosion.penetrations.filter(|n| *n <= flight.hits.len()) {
                let time = flight.hits[limit - 1].time;
                // Include the complete simultaneous contact batch before checking the threshold.
                flight.hits.retain(|hit| hit.time <= time + EPS);
                flight.duration = time;
                flight.end = End::Penetrations;
            }
            if matches!(flight.end, End::Fuse | End::Penetrations)
                || (flight.end == End::Landing && explosion.landing)
            {
                flight.explosion_radius = Some(explosion.radius);
                let center = flight.position(flight.duration);
                for (y, row) in board.iter().enumerate() {
                    for (x, tile) in row.iter().enumerate() {
                        if *tile < 7
                            && !flight.hits.iter().any(|h| h.cell == (x, y))
                            && distance_squared(center, (x, y)) <= explosion.radius.powi(2) + EPS
                        {
                            flight.explosion_cells.push((x, y));
                        }
                    }
                }
            }
        }
        flight
    }
}
fn distance_squared(p: Point, cell: (usize, usize)) -> f64 {
    let x = p.x - p.x.clamp(cell.0 as f64, cell.0 as f64 + 1.);
    let y = p.y - p.y.clamp(cell.1 as f64, cell.1 as f64 + 1.);
    x * x + y * y
}
// Isolate polynomial roots between derivative roots. This handles tangent contact
// as well as crossings, with fixed bisection counts independent of render frames.
fn roots(coefficients: &[f64], lo: f64, hi: f64) -> Vec<f64> {
    let mut degree = coefficients.len() - 1;
    while degree > 0 && coefficients[degree].abs() < 1e-14 {
        degree -= 1;
    }
    if degree == 0 {
        return vec![];
    }
    if degree == 1 {
        let t = -coefficients[0] / coefficients[1];
        return if t >= lo - EPS && t <= hi + EPS {
            vec![t.clamp(lo, hi)]
        } else {
            vec![]
        };
    }
    let derivative: Vec<_> = (1..=degree).map(|i| coefficients[i] * i as f64).collect();
    let mut boundaries = vec![lo];
    boundaries.extend(roots(&derivative, lo, hi));
    boundaries.push(hi);
    let value = |t: f64| {
        coefficients[..=degree]
            .iter()
            .rev()
            .fold(0., |v, c| v * t + c)
    };
    let mut found = vec![];
    for &t in &boundaries {
        if value(t).abs() < 1e-10 {
            found.push(t);
        }
    }
    for pair in boundaries.windows(2) {
        let (mut a, mut b) = (pair[0], pair[1]);
        let sign = value(a).is_sign_positive();
        if sign == value(b).is_sign_positive() {
            continue;
        }
        for _ in 0..48 {
            let mid = (a + b) * 0.5;
            if value(mid).is_sign_positive() == sign {
                a = mid;
            } else {
                b = mid;
            }
        }
        found.push((a + b) * 0.5);
    }
    found.sort_by(f64::total_cmp);
    found.dedup_by(|a, b| (*a - *b).abs() < EPS);
    found
}
fn contact(flight: &Flight, cell: (usize, usize), radius: f64) -> Option<f64> {
    let (x, y) = (cell.0 as f64, cell.1 as f64);
    let acceleration = flight.acceleration();
    let (vx, vy) = (flight.velocity.x, flight.velocity.y);
    let (gx, gy) = (acceleration.x * 0.5, acceleration.y * 0.5);
    let mut candidates = vec![0.];
    for edge in [x - radius, x + 1. + radius] {
        candidates.extend(roots(
            &[flight.origin.x - edge, vx, gx],
            0.,
            flight.duration,
        ));
    }
    for edge in [y - radius, y + 1. + radius] {
        candidates.extend(roots(
            &[flight.origin.y - edge, vy, gy],
            0.,
            flight.duration,
        ));
    }
    for cx in [x, x + 1.] {
        for cy in [y, y + 1.] {
            let (dx, dy) = (flight.origin.x - cx, flight.origin.y - cy);
            candidates.extend(roots(
                &[
                    dx * dx + dy * dy - radius * radius,
                    2. * (dx * vx + dy * vy),
                    vx * vx + vy * vy + 2. * (dx * gx + dy * gy),
                    2. * (vx * gx + vy * gy),
                    gx * gx + gy * gy,
                ],
                0.,
                flight.duration,
            ));
        }
    }
    candidates
        .into_iter()
        .filter(|t| distance_squared(flight.position(*t), cell) <= radius * radius + EPS)
        .min_by(f64::total_cmp)
}
impl Flight {
    pub fn json(&self) -> String {
        let hits = self
            .hits
            .iter()
            .map(|h| {
                format!(
                    "{{\"cell\":[{},{}],\"time\":{},\"position\":[{},{}]}}",
                    h.cell.0, h.cell.1, h.time, h.position.x, h.position.y
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"ammo\":\"{:?}\",\"origin\":[{},{}],\"velocity\":[{},{}],\"gravity\":{},\"gravity_direction\":\"{:?}\",\"duration\":{},\"end\":\"{:?}\",\"hits\":[{}],\"explosion_cells\":[{}],\"explosion_radius\":{}}}", self.ammo,self.origin.x,self.origin.y,self.velocity.x,self.velocity.y,self.gravity,self.gravity_direction,self.duration,self.end,hits,
            self.explosion_cells.iter().map(|(x,y)| format!("[{x},{y}]")).collect::<Vec<_>>().join(","), self.explosion_radius.map(|r| r.to_string()).unwrap_or("null".into()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config(skill: Skill, awake: bool) -> ProjectileConfig {
        skill_profile(
            skill,
            Rules {
                awake,
                ..Default::default()
            },
        )
        .projectile
        .unwrap()
    }
    #[test]
    fn gravity_changes_acceleration_contacts_and_downstream_landing() {
        for direction in [
            board::Gravity::Down,
            board::Gravity::Up,
            board::Gravity::Left,
            board::Gravity::Right,
        ] {
            let mut c = config(Skill::HeavySlingshot, true);
            c.gravity_direction = direction;
            let flight = c.solve(&[[0; N]; N], Aim::default());
            let (gx, gy) = direction.vector();
            let t = flight.duration * 0.5;
            let p = flight.position(t);
            assert!(
                (p.x - ORIGIN.x - flight.velocity.x * t - 0.5 * c.gravity * gx as f64 * t * t)
                    .abs()
                    < EPS
            );
            assert!(
                (p.y - ORIGIN.y - flight.velocity.y * t - 0.5 * c.gravity * gy as f64 * t * t)
                    .abs()
                    < EPS
            );
            assert!(flight.duration.is_finite() && flight.duration > 0.);
            assert!(!flight.hits.is_empty());
            for hit in &flight.hits {
                assert_eq!(hit.position, flight.position(hit.time));
                assert!((distance_squared(hit.position, hit.cell) - c.radius.powi(2)).abs() < EPS);
                let before = (hit.time - 1e-6).max(0.);
                assert!(distance_squared(flight.position(before), hit.cell) > c.radius.powi(2));
            }
            if flight.end == End::Landing {
                let end = flight.position(flight.duration);
                let depth = match direction {
                    board::Gravity::Down => N as f64 - end.y,
                    board::Gravity::Up => end.y,
                    board::Gravity::Left => end.x,
                    board::Gravity::Right => N as f64 - end.x,
                };
                assert!((depth - c.radius).abs() < EPS);
            }
        }
        // Leftward gravity can reverse a weak shot before it enters the board.
        let mut c = config(Skill::LightSlingshot, false);
        c.gravity_direction = board::Gravity::Left;
        let flight = c.solve(
            &[[0; N]; N],
            Aim {
                angle_mdeg: c.max_angle,
                power_permille: 0,
                ..Default::default()
            },
        );
        assert_eq!(flight.end, End::OutOfBounds);
        assert!(flight.duration.is_finite() && flight.duration > 0.);
    }

    #[test]
    fn mirrored_launch_hits_the_reflected_board_and_keeps_the_same_timing() {
        let mut board = [[0; N]; N];
        board[7][0] = 255;
        board[6][4] = 255;
        let mut reflected = board;
        for row in &mut reflected {
            row.reverse();
        }
        for direction in [
            board::Gravity::Down,
            board::Gravity::Up,
            board::Gravity::Left,
            board::Gravity::Right,
        ] {
            for skill in [Skill::LightSlingshot, Skill::HeavySlingshot] {
                for ammo in [Ammo::Solid, Ammo::Explosive] {
                    let left = ProjectileConfig {
                        gravity_direction: direction,
                        ..config(skill, true)
                    };
                    let right = ProjectileConfig {
                        from_right: true,
                        gravity_direction: match direction {
                            board::Gravity::Left => board::Gravity::Right,
                            board::Gravity::Right => board::Gravity::Left,
                            other => other,
                        },
                        ..left
                    };
                    let aim = Aim {
                        ammo,
                        ..Default::default()
                    };
                    let a = left.solve(&board, aim);
                    let b = right.solve(&reflected, aim);
                    assert_eq!(a.duration, b.duration);
                    assert_eq!(a.end, b.end);
                    assert_eq!(a.explosion_radius, b.explosion_radius);
                    for time in [0., a.duration / 2., a.duration] {
                        assert!((a.position(time).x + b.position(time).x - N as f64).abs() < 1e-9);
                        assert!((a.position(time).y - b.position(time).y).abs() < 1e-9);
                    }
                    let expected: std::collections::HashSet<_> =
                        a.cells().into_iter().map(|(x, y)| (N - 1 - x, y)).collect();
                    assert_eq!(expected, b.cells().into_iter().collect());
                    for hit in &a.hits {
                        let mirrored = b
                            .hits
                            .iter()
                            .find(|h| h.cell == (N - 1 - hit.cell.0, hit.cell.1))
                            .unwrap();
                        assert_eq!(hit.time, mirrored.time);
                    }
                }
            }
        }
    }
    #[test]
    fn analytic_motion_range_and_phase_constraints() {
        let board = [[0; N]; N];
        for awake in [false, true] {
            let light = config(Skill::LightSlingshot, awake);
            let heavy = config(Skill::HeavySlingshot, false);
            assert!(light.explosion.is_none());
            assert_eq!(light.gravity, heavy.gravity);
            assert!(light.max_speed < heavy.max_speed);
            let mut far_angles = 0;
            for angle in (10000..=65000).step_by(1000) {
                let aim = Aim {
                    angle_mdeg: angle,
                    power_permille: 1000,
                    ..Default::default()
                };
                let short = light.solve(&board, aim);
                assert!(short.position(short.duration).x < 6.);
                let long = heavy.solve(&board, aim);
                if long.hits.iter().any(|h| h.cell.0 == N - 1) {
                    far_angles += 1;
                }
                for t in [0., long.duration * 0.25, long.duration * 0.75] {
                    let p = long.position(t);
                    assert!(
                        (p.y - (ORIGIN.y + long.velocity.y * t + 0.5 * heavy.gravity * t * t))
                            .abs()
                            < EPS
                    );
                }
            }
            assert!(far_angles >= 15, "reachable, operable angle interval");
        }
        assert!(config(Skill::HeavySlingshot, true).explosion.is_some());
        assert!(config(Skill::HeavySlingshot, true).explosion.is_some());
    }
    #[test]
    fn light_full_power_reaches_fourth_column_across_practical_angles() {
        let board = [[0; N]; N];
        for awake in [false, true] {
            let light = config(Skill::LightSlingshot, awake);
            for angle in (20000..=60000).step_by(1000) {
                let flight = light.solve(
                    &board,
                    Aim {
                        angle_mdeg: angle,
                        power_permille: 1000,
                        ..Default::default()
                    },
                );
                assert!(
                    flight.hits.iter().any(|hit| hit.cell.0 == 3),
                    "awake={awake} angle={angle}"
                );
            }
            let default_flight = light.solve(&board, Aim::default());
            assert!(
                default_flight.position(default_flight.duration).x >= 4.,
                "default aim crosses the full fourth column"
            );
        }
    }
    #[test]
    fn continuous_contacts_radius_tangent_and_simultaneous_batch() {
        let flight = Flight {
            ammo: Ammo::Solid,
            origin: Point { x: -1., y: 1. },
            velocity: Point { x: 1000., y: 0. },
            gravity: 0.,
            gravity_direction: board::Gravity::Down,
            duration: 0.02,
            end: End::OutOfBounds,
            hits: vec![],
            explosion_cells: vec![],
            explosion_radius: None,
        };
        let a = contact(&flight, (3, 0), 0.15).unwrap();
        let b = contact(&flight, (3, 1), 0.15).unwrap();
        assert!((a - b).abs() < EPS);
        assert!((a - 0.00385).abs() < EPS);
        assert!(contact(&flight, (3, 2), 0.15).is_none());
        let tangent = Flight {
            origin: Point { x: -1., y: -0.15 },
            ..flight
        };
        assert!(contact(&tangent, (3, 0), 0.15).is_some());
    }
    #[test]
    fn penetration_trigger_preserves_the_entire_simultaneous_batch() {
        let mut c = config(Skill::HeavySlingshot, true);
        c.radius = 0.1;
        let angle = 50000;
        let radians = (angle as f64 / 1000.).to_radians();
        let time = (0.3 - c.radius) / (c.max_speed * radians.cos());
        c.gravity = 2. * (7. - ORIGIN.y + c.max_speed * radians.sin() * time) / (time * time);
        let mut explosion = c.explosion.unwrap();
        explosion.mass = 1.;
        explosion.penetrations = Some(1);
        explosion.fuse = None;
        c.explosion = Some(explosion);
        let mut board = [[255; N]; N];
        board[6][0] = 0;
        board[7][0] = 0;
        let flight = c.solve(
            &board,
            Aim {
                ammo: Ammo::Explosive,
                angle_mdeg: angle,
                power_permille: 1000,
                ..Default::default()
            },
        );
        assert_eq!(flight.end, End::Penetrations);
        assert_eq!(flight.hits.len(), 2);
        assert!((flight.hits[0].time - flight.hits[1].time).abs() < EPS);
    }
    #[test]
    fn triggers_end_flight_and_explosion_cells_are_unique() {
        let board = [[0; N]; N];
        let mut c = config(Skill::HeavySlingshot, true);
        let mut explosion = c.explosion.unwrap();
        for (landing, fuse, penetrations, expected) in [
            (true, None, None, End::Landing),
            (false, Some(0.2), None, End::Fuse),
            (false, None, Some(3), End::Penetrations),
            (true, Some(0.8), Some(3), End::Penetrations),
        ] {
            explosion.landing = landing;
            explosion.fuse = fuse;
            explosion.penetrations = penetrations;
            c.explosion = Some(explosion);
            let flight = c.solve(
                &board,
                Aim {
                    ammo: Ammo::Explosive,
                    angle_mdeg: 45000,
                    power_permille: 500,
                    ..Default::default()
                },
            );
            assert_eq!(flight.end, expected);
            assert!(flight.explosion_radius.is_some());
            let cells = flight.cells();
            let unique: std::collections::HashSet<_> = cells.iter().collect();
            assert_eq!(unique.len(), cells.len());
            assert!(flight.hits.iter().all(|h| h.time <= flight.duration + EPS));
        }
        explosion.fuse = Some(100.);
        explosion.penetrations = None;
        explosion.landing = true;
        c.explosion = Some(explosion);
        c.max_speed = 14.;
        let outside = c.solve(
            &board,
            Aim {
                ammo: Ammo::Explosive,
                angle_mdeg: 35000,
                power_permille: 1000,
                ..Default::default()
            },
        );
        assert_eq!(outside.end, End::OutOfBounds);
        assert!(outside.explosion_radius.is_none());
        assert!((outside.position(outside.duration).x - (N as f64 + c.radius)).abs() < EPS);
    }
    #[test]
    fn trajectory_clears_are_separate_sources_and_preview_is_pure() {
        let board = [[0; N]; N];
        let rules = Rules {
            projectile_aim: Some(Aim::default()),
            opponent: Some((8, false)),
            opponent_hp: Some(500),
            ..Default::default()
        };
        let rng_before = crate::rng::state();
        let mut copy = board;
        let effect = effect_at_in(
            Skill::LightSlingshot,
            &mut copy,
            (3, 3),
            SLINGSHOT_USER,
            625,
            100,
            rules,
        );
        assert!(!effect.reward);
        let batch = effect.reward_batch(Skill::LightSlingshot);
        assert!(batch.events.iter().all(|e| matches!(
            e.current_source(),
            EventSource::Projectile {
                explosion: false,
                ..
            }
        )));
        let values = trigger_tiles(&board, &effect.cells, 75, 75, 75);
        assert_eq!(batch.totals(), (values.0, values.1, values.2));
        let preview = preview_rewards_in(
            Skill::LightSlingshot,
            &board,
            (3, 3),
            SLINGSHOT_USER,
            625,
            100,
            500,
            0,
            rules,
        );
        assert_eq!(preview.damage, values.1);
        assert_eq!(preview.mana_after, 100 - 30 + values.2);
        assert_eq!(preview.heal, values.0.min(25));
        assert_eq!(copy, board);
        assert_eq!(crate::rng::state(), rng_before);
        let preview = preview_rewards_in(
            Skill::LightSlingshot,
            &board,
            (3, 3),
            SLINGSHOT_USER,
            650,
            160,
            500,
            0,
            rules,
        );
        assert_eq!(preview.heal, 0);
        assert!(preview.mana_after <= 160);
    }
    #[test]
    fn probability_preview_does_not_draw_live_rng() {
        let mut batch =
            RewardBatch::gains((10, 20, 30), 0, EventSource::Skill(Skill::HeavySlingshot));
        for event in &mut batch.events {
            event.probability = 50;
        }
        let before = crate::rng::state();
        let result = settle_batch_in(SLINGSHOT_USER, 600, 50, 500, batch, Rules::default());
        assert_eq!(
            (result.actor_hp, result.damage, result.actor_mp),
            (605, 10, 65)
        );
        assert_eq!(crate::rng::state(), before);
    }
}

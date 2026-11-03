//! Offline Rust adaptation of PixelFX impacts/heavy-landing, Patch / Hugo Duprez, MIT.
//! Split dust rings behind/in front of the existing character. No stock body or font.
use std::f32::consts::TAU;
use std::io::{self, Write};
#[path = "../../../../src/effects/landing_motion.rs"]
mod motion;
const W: usize = 176;
const H: usize = 80;
const COUNT: usize = 22;
const RAMP: [[u8; 3]; 4] = [
    [55, 56, 71],
    [83, 80, 100],
    [120, 112, 128],
    [172, 160, 161],
];
const HOT: [u8; 3] = [226, 212, 188];
fn random(i: u32, c: u32) -> f32 {
    let mut x = i
        .wrapping_mul(747796405)
        .wrapping_add(c.wrapping_mul(2891336453))
        .wrapping_add(14);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    ((x >> 22) ^ x) as f32 / u32::MAX as f32
}
fn fade(x: f32) -> f32 {
    let x = x.clamp(0., 1.);
    x * x * (3. - 2. * x)
}
fn out(x: f32) -> f32 {
    1. - (1. - x.clamp(0., 1.)).powi(3)
}
struct Canvas(Vec<u8>);
impl Canvas {
    fn new() -> Self {
        Self(vec![0; W * H * 4])
    }
    fn dot(&mut self, x: f32, y: f32, c: [u8; 3], a: f32) {
        let (x, y) = (x.round() as i32 + 88, y.round() as i32 + 60);
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let a = a.clamp(0., 1.);
        if a <= 0. {
            return;
        }
        let p = (y as usize * W + x as usize) * 4;
        let old = self.0[p + 3] as f32 / 255.;
        let combined = a + old * (1. - a);
        for k in 0..3 {
            self.0[p + k] = ((c[k] as f32 * a + self.0[p + k] as f32 * old * (1. - a)) / combined)
                .round() as u8;
        }
        self.0[p + 3] = (combined * 255.).round() as u8;
    }
    fn puff(&mut self, x: f32, y: f32, rx: f32, ry: f32, a: f32) {
        for yy in (y - ry).floor() as i32..=(y + ry).ceil() as i32 {
            for xx in (x - rx).floor() as i32..=(x + rx).ceil() as i32 {
                let (dx, dy) = (xx as f32 - x, yy as f32 - y);
                let d = (dx / rx).powi(2) + (dy / ry).powi(2);
                if d > 1. {
                    continue;
                }
                let step = if dx + dy < -rx * 0.35 {
                    3
                } else if dx + dy < rx * 0.5 {
                    2
                } else {
                    1
                };
                self.dot(
                    xx as f32,
                    yy as f32,
                    RAMP[step],
                    a * if d > 0.8 { 0.4 } else { 1. },
                );
            }
        }
    }
}
fn frame(t: f32, front: bool) -> Canvas {
    let mut p = Canvas::new();
    if !(0.0..motion::DUST_DURATION).contains(&t) {
        return p;
    }
    // The two staggered elliptical rings follow PixelFX's heavy-landing choreography.
    // Both use coherent clusters rather than noise; the near halves cover only the feet.
    for ring in 0..2 {
        let delay = 0.02 + ring as f32 * 0.09;
        let life = 0.32 + ring as f32 * 0.2;
        let q = (t - delay) / life;
        if !(0.0..1.0).contains(&q) {
            continue;
        }
        let radius = 5. + (57. - ring as f32 * 14.) * out(q);
        let alpha = 0.8 * (1. - fade(q)) * fade(q / 0.12);
        for i in 0..20 {
            let angle = i as f32 / 20. * TAU + ring as f32 * 0.17;
            if (angle.sin() >= 0.) != front {
                continue;
            }
            let rx = (4. + random(i, ring) * 2.5) * (0.65 + q * 0.8);
            let x = angle.cos() * radius;
            let y = angle.sin() * radius * 0.18 - 1.;
            p.puff(x, y, rx, rx * 0.7, alpha * (0.55 + random(i, 2) * 0.3));
        }
        if ring == 0 && q < 0.7 {
            for i in 0..220 {
                let a = i as f32 / 220. * TAU;
                if (a.sin() >= 0.) != front {
                    continue;
                }
                p.dot(
                    a.cos() * (radius + 1.),
                    a.sin() * (radius + 1.) * 0.18,
                    HOT,
                    (1. - q / 0.7) * 0.65,
                );
            }
        }
    }
    // Low wisps at the feet, without a tall column or flashes on the character's face.
    for i in 0..10 {
        if (i % 2 == 0) != front {
            continue;
        }
        let delay = random(i, 5) * 0.04;
        let q = (t - delay) / 0.58;
        if !(0.0..1.0).contains(&q) {
            continue;
        }
        let side = if i % 3 == 0 { -1. } else { 1. };
        let x = side * (3. + q * (7. + random(i, 6) * 18.));
        let y = -2. - q * (4. + random(i, 7) * 9.);
        p.puff(
            x,
            y,
            4. + random(i, 8) * 3. + q * 2.,
            3. + q * 2.,
            0.6 * (1. - fade(q)),
        );
    }
    // Small pebbles hop once then settle. Fixed seeds leave battle randomness untouched.
    for i in 0..12 {
        if (i % 2 == 0) != front {
            continue;
        }
        let life = 0.44 + random(i, 9) * 0.2;
        if t >= life {
            continue;
        }
        let side = if i % 3 == 0 { -1. } else { 1. };
        let x = side * (4. + (30. + random(i, 10) * 65.) * t);
        let y = (-(52. + random(i, 11) * 48.) * t + 210. * t * t).min(0.);
        let size: i32 = if random(i, 12) > 0.6 { 2 } else { 1 };
        let alpha = 1. - fade((t - (life - 0.13)) / 0.13);
        for yy in -size..=size {
            for xx in -size..=size {
                if xx.abs() + yy.abs() > size + 1 {
                    continue;
                }
                p.dot(
                    x + xx as f32,
                    y + yy as f32,
                    if xx + yy < 0 { RAMP[3] } else { RAMP[1] },
                    alpha,
                );
            }
        }
    }
    // A restrained flat contact glint; no magical aura or persistent floor damage.
    if front && t < 0.067 {
        for x in -18..=18 {
            let a = (1. - (x as f32 / 19.).abs()) * (1. - t / 0.067);
            p.dot(x as f32, 0., HOT, a);
        }
    }
    p
}
fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--motion") {
        let values: Vec<String> = (0..43)
            .map(|i| motion::lift(i as f32 / 30.).to_string())
            .collect();
        println!("[{}]", values.join(","));
        return Ok(());
    }
    let stdout = io::stdout();
    let mut output = stdout.lock();
    for front in [false, true] {
        for i in 0..COUNT {
            output.write_all(&frame(i as f32 / 30., front).0)?;
        }
    }
    Ok(())
}

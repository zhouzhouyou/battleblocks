//! Rust adaptations of PixelFX lightning/lightning-strike and fire/meteor.
//! Patch / Hugo Duprez, MIT; see licenses/pixelfx. No stock body or font.
use std::f32::consts::TAU;
use std::io::{self, Write};
const W: usize = 512;
const BLUE: [[u8; 3]; 4] = [
    [41, 59, 88],
    [69, 101, 138],
    [118, 153, 183],
    [226, 239, 249],
];
const FIRE: [[u8; 3]; 5] = [
    [101, 51, 65],
    [159, 67, 80],
    [232, 131, 61],
    [255, 208, 112],
    [255, 240, 205],
];
fn unit(x: f32) -> f32 {
    x.clamp(0., 1.)
}
fn fade(a: f32, b: f32, t: f32) -> f32 {
    let v = unit((t - a) / (b - a));
    v * v * (3. - 2. * v)
}
fn out(t: f32) -> f32 {
    1. - (1. - unit(t)).powi(3)
}
fn rand(i: u32, c: u32) -> f32 {
    let mut x = i
        .wrapping_mul(747796405)
        .wrapping_add(c.wrapping_mul(2891336453))
        .wrapping_add(31);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    ((x >> 22) ^ x) as f32 / u32::MAX as f32
}
struct Canvas(Vec<u8>);
impl Canvas {
    fn new() -> Self {
        Self(vec![0; W * W * 4])
    }
    fn dot(&mut self, x: f32, y: f32, c: [u8; 3], a: f32) {
        let (x, y) = (x.round() as i32, y.round() as i32);
        if x < 0 || y < 0 || x >= 512 || y >= 512 {
            return;
        }
        let a = unit(a);
        if a <= 0. {
            return;
        }
        let p = (y as usize * W + x as usize) * 4;
        let old = self.0[p + 3] as f32 / 255.;
        let sum = a + old * (1. - a);
        for k in 0..3 {
            self.0[p + k] =
                ((c[k] as f32 * a + self.0[p + k] as f32 * old * (1. - a)) / sum).round() as u8;
        }
        self.0[p + 3] = (sum * 255.).round() as u8;
    }
    fn line(&mut self, a: (f32, f32), b: (f32, f32), c: [u8; 3], alpha: f32, width: i32) {
        let n = (a.0 - b.0).abs().max((a.1 - b.1).abs()).ceil().max(1.) as usize;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            for dy in -width..=width {
                for dx in -width..=width {
                    if dx * dx + dy * dy <= width * width {
                        self.dot(x + dx as f32, y + dy as f32, c, alpha);
                    }
                }
            }
        }
    }
    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32, c: [u8; 3], alpha: f32) {
        for yy in (y - ry).floor().max(0.) as i32..=(y + ry).ceil().min(511.) as i32 {
            for xx in (x - rx).floor().max(0.) as i32..=(x + rx).ceil().min(511.) as i32 {
                if ((xx as f32 - x) / rx).powi(2) + ((yy as f32 - y) / ry).powi(2) <= 1. {
                    self.dot(xx as f32, yy as f32, c, alpha);
                }
            }
        }
    }
    fn ring(&mut self, x: f32, y: f32, r: f32, sy: f32, c: [u8; 3], alpha: f32) {
        let n = (r * TAU * 2.).ceil() as usize;
        for i in 0..=n {
            let a = i as f32 / n as f32 * TAU;
            self.dot(x + a.cos() * r, y + a.sin() * r * sy, c, alpha);
        }
    }
    fn puff(&mut self, x: f32, y: f32, r: f32, fire: bool, alpha: f32) {
        for yy in (y - r).floor().max(0.) as i32..=(y + r).ceil().min(511.) as i32 {
            for xx in (x - r).floor().max(0.) as i32..=(x + r).ceil().min(511.) as i32 {
                let (dx, dy) = (xx as f32 - x, yy as f32 - y);
                let d = dx.hypot(dy) / r;
                let edge = 1. + (dy.atan2(dx) * 5. + x * 0.1).sin() * 0.06;
                if d > edge {
                    continue;
                }
                let hot = (1. - d) * 0.7 - (dx + dy) / r * 0.12;
                let c = if fire {
                    FIRE[(hot * 4. + 1.).round().clamp(0., 4.) as usize]
                } else {
                    let colors = [[37, 38, 51], [55, 56, 71], [83, 80, 100], [120, 112, 128]];
                    colors[((1. - d) * 2. + if dx + dy < 0. { 1. } else { 0. })
                        .round()
                        .clamp(0., 3.) as usize]
                };
                self.dot(
                    xx as f32,
                    yy as f32,
                    c,
                    alpha * if d > 0.88 { 0.5 } else { 1. },
                );
            }
        }
    }
}
fn lightning(t: f32) -> Canvas {
    let mut p = Canvas::new();
    if !(0.0..0.75).contains(&t) {
        return p;
    }
    let age = t - 0.4;
    let tick = (age.max(0.) * 20.).floor() as u32;
    let flicker = [1., 0.55, 0.85, 0.4, 0.7, 0.28, 0.1];
    let strength = if age < 0. {
        unit((t - 0.08) / 0.32) * 0.6
    } else {
        flicker[(tick as usize).min(6)]
    };
    let leader = if age < 0. { unit(t / 0.4) * 7. } else { 7. };
    for diagonal in 0..2 {
        for segment in 0..7 {
            if segment as f32 > leader {
                continue;
            }
            let x0 = if diagonal == 0 {
                32. + segment as f32 * 64.
            } else {
                480. - segment as f32 * 64.
            };
            let y0 = 32. + segment as f32 * 64.;
            let sx = if diagonal == 0 { 1. } else { -1. };
            let mut last = (x0, y0);
            for j in 1..=8 {
                let f = j as f32 / 8.;
                if segment as f32 + f > leader {
                    break;
                }
                let offset = if j == 8 {
                    0.
                } else {
                    (rand((diagonal * 100 + segment * 8 + j) as u32, tick / 2) - 0.5) * 15.
                };
                let next = (x0 + sx * 64. * f + offset, y0 + 64. * f - offset * sx);
                if age >= 0. {
                    p.line(last, next, BLUE[1], strength * 0.2, 7);
                }
                p.line(last, next, BLUE[2], strength, if age < 0. { 1 } else { 3 });
                if age >= 0. {
                    p.line(last, next, BLUE[3], strength, 1);
                }
                if (j == 3 || j == 6) && age >= 0. {
                    let side = if j == 3 { 1. } else { -1. };
                    p.line(
                        next,
                        (next.0 + sx * 13. * side, next.1 - 8. * side),
                        BLUE[2],
                        strength * 0.75,
                        0,
                    );
                }
                last = next;
            }
        }
        for i in 0..8 {
            let x = if diagonal == 0 {
                32. + i as f32 * 64.
            } else {
                480. - i as f32 * 64.
            };
            let y = 32. + i as f32 * 64.;
            if age < 0. {
                if i as f32 <= leader {
                    p.ring(x, y, 12., 1., BLUE[1], strength * 0.55);
                }
                continue;
            }
            let a = (1. - age / 0.3).max(0.);
            p.ellipse(x, y, 11., 11., BLUE[2], a * 0.5);
            p.ellipse(x, y, 5., 5., BLUE[3], a);
            for ray in 0..5 {
                let angle = ray as f32 / 5. * TAU + rand(i, 2) * TAU;
                let radius = 8. + age * 70.;
                p.line(
                    (
                        x + angle.cos() * radius * 0.5,
                        y + angle.sin() * radius * 0.5,
                    ),
                    (x + angle.cos() * radius, y + angle.sin() * radius),
                    BLUE[3],
                    a,
                    0,
                );
            }
        }
    }
    p
}
fn meteor(t: f32) -> Canvas {
    let mut p = Canvas::new();
    if !(0.0..1.15).contains(&t) {
        return p;
    }
    let age = t - 0.4;
    if age < 0. {
        let f = (t / 0.4).powi(2);
        let (x, y) = (-70. + 326. * f, -110. + 366. * f);
        // One dense comet tail: broad red back steps, orange folds, cream leading edge.
        for k in (0..90).rev() {
            let q = k as f32 / 90.;
            let cx = x - q * 116.;
            let cy = y - q * 136.;
            let r = 38. * (1. - q).powf(0.65) + 2.;
            p.ellipse(
                cx,
                cy,
                r,
                r * 0.75,
                FIRE[if q > 0.65 {
                    0
                } else if q > 0.3 {
                    1
                } else {
                    2
                }],
                1. - q * 0.8,
            );
        }
        for i in 0..5 {
            let d = i as f32 * 19.;
            p.puff(x - d * 0.7, y - d, 20. - i as f32 * 2.3, true, 0.8);
        }
        // The projectile is faceted stone, not a plain glowing ball.
        for yy in -43..=43 {
            for xx in -43..=43 {
                let a = (yy as f32).atan2(xx as f32);
                let edge = 41. * (0.95 + (a * 5. + 0.7).sin() * 0.06);
                let r = (xx as f32).hypot(yy as f32);
                if r > edge {
                    continue;
                }
                let c = if r > edge - 3. {
                    FIRE[3]
                } else if xx + yy < -12 {
                    [172, 160, 161]
                } else if xx + yy < 14 {
                    [83, 80, 100]
                } else {
                    [37, 38, 51]
                };
                p.dot(x + xx as f32, y + yy as f32, c, 1.);
            }
        }
        // Molten cracks break up the stone facets without turning its core into a fireball.
        for (a, b) in [
            ((-12., -23.), (-3., -5.)),
            ((-3., -5.), (-13., 8.)),
            ((-3., -5.), (13., 2.)),
            ((13., 2.), (20., 20.)),
        ] {
            p.line((x + a.0, y + a.1), (x + b.0, y + b.1), FIRE[2], 0.9, 1);
            p.line((x + a.0, y + a.1), (x + b.0, y + b.1), FIRE[3], 0.8, 0);
        }
        return p;
    }
    // The ground-wave expands as a diamond, matching the attack's visual footprint.
    let fade_all = 1. - fade(0.4, 0.75, age);
    for ring in 0..2 {
        let k = (age - ring as f32 * 0.045) / 0.36;
        if !(0.0..1.0).contains(&k) {
            continue;
        }
        let r = 20. + 276. * out(k);
        let alpha = (1. - k) * 0.85;
        let points = [
            (256., 256. - r),
            (256. + r, 256.),
            (256., 256. + r),
            (256. - r, 256.),
            (256., 256. - r),
        ];
        for pair in points.windows(2) {
            p.line(pair[0], pair[1], FIRE[3], alpha, 1);
        }
    }
    // Rolling smoke stays low; fire puffs flare then cool out, leaving no permanent crater.
    for i in 0..17 {
        let a = i as f32 / 17. * TAU;
        let radius = (35. + rand(i, 3) * 115.) * out(age / 0.4);
        let x = 256. + a.cos() * radius;
        let y = 256. + a.sin() * radius * 0.55 - age * 25.;
        p.puff(
            x,
            y,
            18. + rand(i, 4) * 18. + age * 12.,
            false,
            fade_all * 0.38,
        );
    }
    if age < 0.3 {
        for i in 0..13 {
            let a = i as f32 / 13. * TAU;
            let r = (24. + rand(i, 5) * 96.) * out(age / 0.2);
            p.puff(
                256. + a.cos() * r,
                256. + a.sin() * r * 0.65 - age * 32.,
                16. + rand(i, 6) * 24.,
                true,
                1. - fade(0.12, 0.3, age),
            );
        }
    }
    // Stone fragments and hot chips arc away from the single impact.
    for i in 0..28 {
        let a = rand(i, 7) * TAU;
        let speed = 50. + rand(i, 8) * 210.;
        let life = 0.32 + rand(i, 9) * 0.36;
        if age >= life {
            continue;
        }
        let x = 256. + a.cos() * speed * age;
        let y = 256. + a.sin() * speed * age * 0.6 - age * 80. + 250. * age * age;
        let alpha = 1. - fade(life - 0.12, life, age);
        let size: i32 = if i % 4 == 0 { 3 } else { 2 };
        for yy in -size..=size {
            for xx in -size..=size {
                if xx.abs() + yy.abs() > size + 1 {
                    continue;
                }
                let c = if i % 3 == 0 {
                    FIRE[3]
                } else if xx + yy < 0 {
                    [172, 160, 161]
                } else {
                    [55, 56, 71]
                };
                p.dot(x + xx as f32, y + yy as f32, c, alpha);
            }
        }
    }
    if age < 0.1 {
        let r = 70. * (1. - age / 0.12);
        for y in -(r as i32)..=r as i32 {
            for x in -(r as i32)..=r as i32 {
                let a = (y as f32).atan2(x as f32);
                let edge = r * (0.7 + 0.3 * (a * 7.).cos().abs());
                if (x as f32).hypot(y as f32) < edge {
                    p.dot(256. + x as f32, 256. + y as f32, FIRE[4], 1.);
                }
            }
        }
    }
    p
}
fn main() -> io::Result<()> {
    let lightning_mode = std::env::args().any(|s| s == "--lightning");
    let count = if lightning_mode { 16 } else { 24 };
    let stdout = io::stdout();
    let mut output = stdout.lock();
    for i in 0..count {
        let t = i as f32 / 20.;
        output.write_all(
            &if lightning_mode {
                lightning(t)
            } else {
                meteor(t)
            }
            .0,
        )?;
    }
    Ok(())
}

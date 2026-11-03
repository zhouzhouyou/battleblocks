//! Offline Rust adaptation of PixelFX melee/hammer-smash (Patch / Hugo Duprez, MIT).
//! See licenses/pixelfx. No character, weapon, font, or game RNG is used.
//! Coordinates are integer logical pixels; the impact is the atlas pivot (96, 88).
use std::f32::consts::{PI, TAU};
use std::io::{self, Write};

const W: usize = 192;
const H: usize = 128;
const FRAMES: usize = 28;
const HIT: f32 = 0.4;
type Color = [u8; 3];
const INK: Color = [22, 24, 32];
const STONE: [Color; 4] = [
    [55, 56, 71],
    [83, 80, 100],
    [120, 112, 128],
    [172, 160, 161],
];
const LIGHT: [Color; 5] = [
    [101, 51, 65],
    [159, 67, 80],
    [188, 133, 82],
    [226, 212, 188],
    [255, 240, 205],
];
fn unit(x: f32) -> f32 {
    x.clamp(0., 1.)
}
fn fade(a: f32, b: f32, t: f32) -> f32 {
    let x = unit((t - a) / (b - a));
    x * x * (3. - 2. * x)
}
fn out(x: f32) -> f32 {
    1. - (1. - unit(x)).powi(3)
}
// A private, stable visual seed. Never touches the game's random stream.
fn rand(i: u32, channel: u32) -> f32 {
    let mut x = i
        .wrapping_mul(747796405)
        .wrapping_add(channel.wrapping_mul(2891336453))
        .wrapping_add(9);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    ((x >> 22) ^ x) as f32 / u32::MAX as f32
}
struct Canvas(Vec<u8>);
impl Canvas {
    fn new() -> Self {
        Self(vec![0; W * H * 4])
    }
    fn pixel(&mut self, x: f32, y: f32, c: Color, alpha: f32) {
        let (x, y) = (x.round() as i32 + 96, y.round() as i32 + 88);
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        // Fade whole hard-edged clusters instead of dissolving them into noisy lone pixels.
        let alpha = unit(alpha);
        if alpha <= 0. {
            return;
        }
        let p = (y as usize * W + x as usize) * 4;
        let old = self.0[p + 3] as f32 / 255.;
        let combined = alpha + old * (1. - alpha);
        for k in 0..3 {
            self.0[p + k] = ((c[k] as f32 * alpha + self.0[p + k] as f32 * old * (1. - alpha))
                / combined)
                .round() as u8;
        }
        self.0[p + 3] = (combined * 255.).round() as u8;
    }
    fn line(&mut self, a: (f32, f32), b: (f32, f32), c: Color, alpha: f32) {
        let steps = (a.0 - b.0).abs().max((a.1 - b.1).abs()).ceil().max(1.) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.pixel(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, c, alpha);
        }
    }
    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32, c: Color, alpha: f32) {
        for yy in (y - ry).floor() as i32..=(y + ry).ceil() as i32 {
            for xx in (x - rx).floor() as i32..=(x + rx).ceil() as i32 {
                if ((xx as f32 - x) / rx).powi(2) + ((yy as f32 - y) / ry).powi(2) <= 1. {
                    self.pixel(xx as f32, yy as f32, c, alpha);
                }
            }
        }
    }
    fn ring(&mut self, rx: f32, ry: f32, c: Color, alpha: f32) {
        for i in 0..360 {
            let a = i as f32 / 360. * TAU;
            self.pixel(a.cos() * rx, a.sin() * ry, c, alpha);
        }
    }
    fn dust(&mut self, x: f32, y: f32, r: f32, alpha: f32) {
        for yy in (y - r).floor() as i32..=(y + r).ceil() as i32 {
            for xx in (x - r).floor() as i32..=(x + r).ceil() as i32 {
                let dx = xx as f32 - x;
                let dy = yy as f32 - y;
                let d = (dx * dx + dy * dy) / (r * r);
                if d > 1. {
                    continue;
                }
                let (sx, sy) = (xx + 96, yy + 88);
                if sx < 0 || sy < 0 || sx >= W as i32 || sy >= H as i32 {
                    continue;
                }
                // Three shaped tonal steps, top-left lit, with a quiet transparent edge.
                let shade = if dx + dy < -r * 0.45 {
                    2
                } else if dx + dy < r * 0.6 {
                    1
                } else {
                    0
                };
                let a = unit(alpha) * if d > 0.8 { 0.35 } else { 1. };
                let p = (sy as usize * W + sx as usize) * 4;
                let old = self.0[p + 3] as f32 / 255.;
                let combined = a + old * (1. - a);
                if combined <= 0. {
                    continue;
                }
                for k in 0..3 {
                    self.0[p + k] = ((STONE[shade][k] as f32 * a
                        + self.0[p + k] as f32 * old * (1. - a))
                        / combined)
                        .round() as u8;
                }
                self.0[p + 3] = (combined * 255.).round() as u8;
            }
        }
    }
}
fn frame(t: f32) -> Canvas {
    let mut p = Canvas::new();
    if t < 0.18 || t >= 0.9 {
        return p;
    }
    let q = t - HIT;
    // Accelerating comet wedge: the hammer's path, without drawing another weapon.
    // PixelFX's onSwing/smear recipe, moved so its final point is exactly the pivot.
    let h = unit((t - 0.18) / (HIT - 0.18));
    let end = -2.25 + 2.95 * (0.28 * h + 0.72 * h * h);
    let after = fade(HIT, HIT + 0.045, t);
    let start = (end - 0.25 - 1.0 * h).max(-2.25) + (0.25 + 1.0 * h) * after;
    if after < 1. {
        for y in -72..=8 {
            for x in -80..=25 {
                let dx = x as f32 + 40. * 0.7_f32.cos();
                let dy = y as f32 + 40. * 0.7_f32.sin();
                let r = dx.hypot(dy);
                let a = dy.atan2(dx);
                let tail = (end - a) / (end - start).max(0.01);
                let width = (3. + 7. * h) * (1. - unit(tail)).powf(0.6);
                if a < start || a > end || r > 40. || r < 40. - width {
                    continue;
                }
                let step = if end - a < 0.06 {
                    4
                } else {
                    (3. - unit(tail) * 3.).round() as usize
                };
                p.pixel(x as f32, y as f32, LIGHT[step], 1. - after);
            }
        }
    }
    if q < 0. {
        return p;
    }
    let gone = 1. - fade(0.27, 0.49, q);
    // Small dent and branched cracks, cooling from ivory to dark stone.
    p.ellipse(0., 1., 14., 4., INK, gone);
    for i in 0..7 {
        let a = (i as f32 / 7.) * TAU + 0.2;
        let length = (22. + rand(i, 1) * 12.) * out(q / 0.08);
        let mut last = (0., 1.);
        for j in 1..=4 {
            let d = length * j as f32 / 4.;
            let point = (
                a.cos() * d,
                a.sin() * d * 0.33 + (rand(i * 4 + j, 2) - 0.5) * 4.,
            );
            p.line(
                (last.0, last.1 + 1.),
                (point.0, point.1 + 1.),
                STONE[2],
                gone * 0.65,
            );
            p.line(last, point, if q < 0.05 { LIGHT[4] } else { INK }, gone);
            last = point;
        }
    }
    // Two rapidly expanding ground rings and the rising shell of force.
    for i in 0..2 {
        let k = (q - i as f32 * 0.045) / 0.24;
        if (0.0..1.0).contains(&k) {
            let r = 10. + 54. * out(k);
            p.ring(r, r * 0.24, LIGHT[4 - i], 1. - k);
            p.ring(r - 1., r * 0.24 - 1., LIGHT[2], (1. - k) * 0.65);
        }
    }
    let d = (q - 0.025) / 0.22;
    if (0.0..1.0).contains(&d) {
        let r = 12. + 32. * out(d);
        let thick = 1. + 2. * (1. - d);
        for y in -(r.ceil() as i32)..=0 {
            for x in -(r.ceil() as i32)..=r.ceil() as i32 {
                let edge = r - (x as f32).hypot(y as f32);
                if edge >= 0. && edge < thick {
                    p.pixel(
                        x as f32,
                        y as f32,
                        if edge < 1. { LIGHT[4] } else { LIGHT[2] },
                        1. - fade(0.5, 1., d),
                    );
                }
            }
        }
    }
    // Dust rolls both ways. Keep it low and translucent so tiles remain readable.
    for i in 0..12 {
        let s = if i % 2 == 0 { -1. } else { 1. };
        let life = 0.24 + rand(i, 3) * 0.18;
        let k = unit(q / life);
        if k >= 1. {
            continue;
        }
        let x = s * (5. + (30. + rand(i, 4) * 35.) * out(k));
        let y = -2. - rand(i, 5) * 8. - k * (5. + rand(i, 6) * 8.);
        let r = (4. + rand(i, 7) * 5.) * (0.6 + 0.7 * k);
        p.dust(x, y, r, 0.7 * (1. - fade(0.35, 1., k)));
    }
    // Seeded tumbling stone clusters with top-left facets. Their fall is analytic.
    for i in 0..18 {
        let life = 0.29 + rand(i, 8) * 0.17;
        if q >= life {
            continue;
        }
        let s = if i % 2 == 0 { -1. } else { 1. };
        let vx = s * (40. + rand(i, 9) * 130.);
        let vy = -(95. + rand(i, 10) * 165.);
        let x = vx * q;
        let y = (vy * q + 400. * q * q).min(0.);
        let size = 2. + (rand(i, 11) * 3.).floor();
        let a = rand(i, 12) * TAU + q * (rand(i, 13) - 0.5) * 20.;
        let alpha = 1. - fade(life - 0.1, life, q);
        for yy in -(size as i32 + 1)..=size as i32 + 1 {
            for xx in -(size as i32 + 1)..=size as i32 + 1 {
                let dx = xx as f32 * a.cos() - yy as f32 * a.sin();
                let dy = xx as f32 * a.sin() + yy as f32 * a.cos();
                if dx.abs() + dy.abs() * 0.8 > size {
                    continue;
                }
                let c = if dx + dy < -size * 0.4 {
                    STONE[3]
                } else if dx + dy < size * 0.5 {
                    STONE[1]
                } else {
                    INK
                };
                p.pixel(x + xx as f32, y + yy as f32, c, alpha);
            }
        }
    }
    // Short hot chips and impact rays, not a lingering fire effect.
    for i in 0..16 {
        let life = 0.1 + rand(i, 14) * 0.15;
        if q >= life {
            continue;
        }
        let a = -PI / 2. + (rand(i, 15) - 0.5) * 2.6;
        let speed = 90. + rand(i, 16) * 170.;
        let x = a.cos() * speed * q;
        let y = a.sin() * speed * q + 180. * q * q;
        p.line(
            (x - a.cos() * 3., y - a.sin() * 3.),
            (x, y),
            LIGHT[3],
            1. - q / life,
        );
        p.pixel(x, y, LIGHT[4], 1. - q / life);
    }
    if q < 0.065 {
        // Two compact hero frames make contact readable over the board's busy materials.
        if q < 0.04 {
            let r = 18. - q * 120.;
            for y in -(r as i32)..=r as i32 {
                for x in -(r as i32)..=r as i32 {
                    let dx = x as f32;
                    let dy = y as f32;
                    let a = dy.atan2(dx);
                    let edge = r * (0.58 + 0.42 * (a * 5. + 0.4).cos().abs());
                    if dx.hypot(dy * 1.25) < edge {
                        p.pixel(dx, dy - 3., LIGHT[4], 1.);
                    }
                }
            }
        }
        p.ellipse(0., 0., 18. - q * 110., 4. - q * 25., LIGHT[4], 1.);
        for i in 0..7 {
            let a = -PI * 0.9 + i as f32 / 6. * PI * 0.8;
            let length = (14. + rand(i, 17) * 18.) * out(q / 0.025);
            p.line(
                (a.cos() * 5., a.sin() * 5.),
                (a.cos() * length, a.sin() * length),
                LIGHT[4],
                1. - q / 0.065,
            );
        }
    }
    p
}
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for i in 0..FRAMES {
        out.write_all(&frame(i as f32 / 30.).0)?;
    }
    Ok(())
}

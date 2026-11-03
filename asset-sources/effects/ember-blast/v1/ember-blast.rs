//! Offline pixel fire: rising tongues, cooling puffs and expanding blast inspired by
//! PixelFX fire/fire-ring and explosions/big-explosion (Patch / Hugo Duprez, MIT).
//! Our board composition uses a fixed visual seed; no bodies, fonts or game rules.
use std::f32::consts::TAU;
use std::io::{self, Write};
#[path = "../../sleep/v1/sleep.rs"]
mod sleep;
const W: usize = 512;
const COUNT: usize = 22;
const FIRE: [[u8; 3]; 8] = [
    [53, 39, 54],
    [101, 51, 65],
    [159, 67, 80],
    [214, 96, 69],
    [232, 131, 61],
    [245, 163, 75],
    [255, 208, 112],
    [255, 240, 205],
];
fn unit(x: f32) -> f32 {
    x.clamp(0., 1.)
}
fn fade(a: f32, b: f32, t: f32) -> f32 {
    let q = unit((t - a) / (b - a));
    q * q * (3. - 2. * q)
}
fn rand(i: u32, c: u32) -> f32 {
    let mut x = i
        .wrapping_mul(747796405)
        .wrapping_add(c.wrapping_mul(2891336453))
        .wrapping_add(23);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    ((x >> 22) ^ x) as f32 / u32::MAX as f32
}
fn noise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (u, v) = (fx * fx * (3. - 2. * fx), fy * fy * (3. - 2. * fy));
    let a = rand(ix as u32, iy as u32);
    let b = rand(ix.wrapping_add(1) as u32, iy as u32);
    let c = rand(ix as u32, iy.wrapping_add(1) as u32);
    let d = rand(ix.wrapping_add(1) as u32, iy.wrapping_add(1) as u32);
    (a + (b - a) * u) * (1. - v) + (c + (d - c) * u) * v
}
fn frame(t: f32) -> Vec<u8> {
    let mut out = vec![0; W * W * 4];
    if !(0.0..1.4).contains(&t) {
        return out;
    }
    let mut heat = vec![0f32; W * W];
    let mut opacity = vec![0f32; W * W];
    if t < 0.4 {
        // Two coherent winding arms funnel into the center; all target tiles remain intact.
        let charge = unit(t / 0.4);
        for y in 0..W {
            for x in 0..W {
                let (dx, dy) = (x as f32 - 256., y as f32 - 256.);
                let r = dx.hypot(dy);
                let a = dy.atan2(dx);
                let spiral = (a * 2. + r * 0.055 + t * 11.).sin();
                let reach = 28. + charge * 140.;
                let edge = (1. - ((r - reach * 0.55) / (reach * 0.55)).abs()).max(0.);
                let core = (1. - r / (8. + charge * 18.)).max(0.);
                if spiral > 0.94 && r < reach && r > 14. {
                    heat[y * W + x] = 0.45 + charge * 0.4;
                    opacity[y * W + x] = edge * charge * 0.85;
                }
                if core > 0. {
                    heat[y * W + x] = 0.8 + core * 0.2;
                    opacity[y * W + x] = core * charge;
                }
            }
        }
    } else {
        let age = t - 0.4;
        let tail = 1. - fade(0.68, 1.38, t);
        // A tiled coherent underglow fills the complete square, including its corners.
        for y in 0..W {
            for x in 0..W {
                let n = noise(x as f32 / 42., y as f32 / 54. - age * 3.);
                heat[y * W + x] = 0.18 + n * 0.3;
                opacity[y * W + x] = tail * (0.82 - age * 0.3).max(0.);
            }
        }
        // Radially expanding, overlapping lobes, rather than a repeated wall of flame tiles.
        for ring in 0..6 {
            let radius = ring as f32 * 73.;
            let count = if ring == 0 { 1 } else { ring * 8 };
            for site in 0..count {
                let id = (ring * 53 + site) as u32;
                let angle =
                    (site as f32 + rand(id, 1) * 0.5) / count as f32 * TAU + ring as f32 * 0.37;
                let growth = 0.75 + 0.25 * (1. - (1. - unit(age / 0.32)).powi(3));
                let r = radius * growth + age * (8. + rand(id, 2) * 17.);
                let cx = 256. + angle.cos() * r;
                let cy = 256. + angle.sin() * r - age * 20.;
                let size = 44. + rand(id, 3) * 28. + age * 12.;
                for y in (cy - size * 1.2).floor().max(0.) as usize
                    ..(cy + size * 1.2).ceil().min(512.) as usize
                {
                    for x in (cx - size * 1.2).floor().max(0.) as usize
                        ..(cx + size * 1.2).ceil().min(512.) as usize
                    {
                        let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                        let a = dy.atan2(dx);
                        let shape = 1.
                            + (a * 3. + rand(id, 4) * TAU + age * 3.).sin() * 0.09
                            + (a * 5. + id as f32).cos() * 0.05;
                        let distance = dx.hypot(dy) / size;
                        if distance > shape {
                            continue;
                        }
                        let billow =
                            noise(x as f32 / 23. + rand(id, 5) * 3., y as f32 / 27. - age * 3.);
                        let hot =
                            0.14 + (1. - distance / shape).max(0.).powf(0.7) * 0.46 + billow * 0.14
                                - (dx + dy) / size * 0.08;
                        let index = y * W + x;
                        heat[index] = heat[index].max(hot);
                        opacity[index] =
                            opacity[index].max(tail * (0.82 + unit(1. - distance) * 0.18));
                    }
                }
            }
        }
        // The first burst is broad and brief, then a hard-edged wave reaches every corner.
        for y in 0..W {
            for x in 0..W {
                let index = y * W + x;
                let radius = (x as f32 - 256.).hypot(y as f32 - 256.);
                let flash = (-age / 0.035).exp();
                let wave_r = 28. + (1. - (1. - unit(age / 0.26)).powi(3)) * 380.;
                let rim = (1. - ((radius - wave_r) / 12.).abs()).max(0.) * (1. - unit(age / 0.35));
                let core = (1. - radius / 140.).max(0.) * (-age / 0.3).exp() * 0.35;
                heat[index] = ((heat[index] + core) * (1. - fade(0.7, 1.35, t) * 0.4)
                    + flash * 0.85
                    + rim * 0.35)
                    .min(1.);
            }
        }
    }
    for y in 0..W {
        for x in 0..W {
            let index = y * W + x;
            let alpha = opacity[index];
            if alpha < 0.003 {
                continue;
            }
            let level = (heat[index] * 7.).round().clamp(0., 7.) as usize;
            out[index * 4..index * 4 + 3].copy_from_slice(&FIRE[level]);
            out[index * 4 + 3] = (alpha * 255.).round() as u8;
        }
    }
    // Sparse warm embers rise above the dying field; seeded and integer-aligned.
    if t > 0.65 && t < 1.33 {
        let age = t - 0.65;
        let a = 1. - fade(0.2, 0.68, age);
        for i in 0..72 {
            let x = (rand(i, 10) * 508. + (age * 7. + i as f32).sin() * 2.).round() as i32;
            let y = (rand(i, 11) * 512. - age * (28. + rand(i, 12) * 60.)).round() as i32;
            for yy in 0..2 {
                for xx in 0..2 {
                    if x + xx < 0 || y + yy < 0 || x + xx >= 512 || y + yy >= 512 {
                        continue;
                    }
                    let p = ((y + yy) as usize * W + (x + xx) as usize) * 4;
                    out[p..p + 3].copy_from_slice(&FIRE[6]);
                    out[p + 3] = (a * 255.).round() as u8;
                }
            }
        }
    }
    out
}
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if std::env::args().any(|s| s == "--sleep") {
        for i in 0..sleep::COUNT {
            output.write_all(&sleep::frame(i as f32 / 15.))?;
        }
    } else {
        for i in 0..COUNT {
            output.write_all(&frame(i as f32 / 15.))?;
        }
    }
    Ok(())
}

//! Original bitmap Zs; PixelFX status/sleep's staggered rising motif, no font or body.
pub const W: usize = 96;
pub const H: usize = 96;
pub const COUNT: usize = 36;
pub fn frame(t: f32) -> Vec<u8> {
    let mut out = vec![0; W * H * 4];
    let put = |out: &mut Vec<u8>, x: i32, y: i32, c: [u8; 3], alpha: f32| {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let p = (y as usize * W + x as usize) * 4;
        out[p..p + 3].copy_from_slice(&c);
        out[p + 3] = (alpha.clamp(0., 1.) * 255.).round() as u8;
    };
    for i in (0..3).rev() {
        let phase = (t / 2.4 + i as f32 / 3.).rem_euclid(1.);
        let alpha = (phase / 0.1).min((1. - phase) / 0.2).clamp(0., 1.);
        let size = if phase < 0.38 {
            7
        } else if phase < 0.7 {
            9
        } else {
            11
        };
        let stroke = if size > 7 { 2 } else { 1 };
        let x = (24. + 12. + phase * 20. + (phase * 6.28 + i as f32).sin() * 2.).round() as i32;
        let y = (70. - 9. - phase * 34.).round() as i32;
        let filled = |xx: i32, yy: i32| -> bool {
            if xx < 0 || yy < 0 || xx >= size || yy >= size {
                return false;
            }
            yy < stroke || yy >= size - stroke || (xx - (size - 1 - yy)).abs() < stroke
        };
        for yy in -1..=size {
            for xx in -1..=size {
                if filled(xx, yy) {
                    let c = if yy < stroke {
                        [255, 240, 205]
                    } else if xx + yy < size {
                        [226, 212, 188]
                    } else {
                        [118, 153, 183]
                    };
                    put(&mut out, x + xx, y + yy, c, alpha);
                } else if (-1..=1).any(|dy| (-1..=1).any(|dx| filled(xx + dx, yy + dy))) {
                    put(&mut out, x + xx, y + yy, [37, 38, 51], alpha);
                }
            }
        }
    }
    out
}

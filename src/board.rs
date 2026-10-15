use crate::rng;
pub const N: usize = 8;
pub type Board = [[u8; N]; N];
pub fn matches(b: &Board) -> Vec<(usize, usize)> {
    let mut hit = [[false; N]; N];
    for y in 0..N {
        for x in 0..N {
            if b[y][x] == 255 {
                continue;
            }
            if x + 2 < N && b[y][x] == b[y][x + 1] && b[y][x] == b[y][x + 2] {
                hit[y][x] = true;
                hit[y][x + 1] = true;
                hit[y][x + 2] = true;
            }
            if y + 2 < N && b[y][x] == b[y + 1][x] && b[y][x] == b[y + 2][x] {
                hit[y][x] = true;
                hit[y + 1][x] = true;
                hit[y + 2][x] = true;
            }
        }
    }
    let mut out = Vec::new();
    for y in 0..N {
        for x in 0..N {
            if hit[y][x] {
                out.push((x, y));
            }
        }
    }
    out
}
pub fn new_board() -> Board {
    let mut b = [[255; N]; N];
    for y in 0..N {
        for x in 0..N {
            loop {
                b[y][x] = rng::gen_range(0, 7);
                if !(x >= 2 && b[y][x] == b[y][x - 1] && b[y][x] == b[y][x - 2])
                    && !(y >= 2 && b[y][x] == b[y - 1][x] && b[y][x] == b[y - 2][x])
                {
                    break;
                }
            }
        }
    }
    b
}
pub fn swap(b: &mut Board, a: (usize, usize), c: (usize, usize)) {
    let t = b[a.1][a.0];
    b[a.1][a.0] = b[c.1][c.0];
    b[c.1][c.0] = t;
}
pub fn valid_swap(b: &mut Board, a: (usize, usize), c: (usize, usize)) -> bool {
    if a.0.abs_diff(c.0) + a.1.abs_diff(c.1) != 1 {
        return false;
    }
    swap(b, a, c);
    if matches(b).is_empty() {
        swap(b, a, c);
        false
    } else {
        true
    }
}
pub fn parallel_swap(b: &mut Board, a: (usize, usize), c: (usize, usize)) {
    swap(b, a, c);
    for delta in [-1isize, 1] {
        let (p, q) = if a.0 == c.0 {
            (
                (a.0 as isize + delta, a.1 as isize),
                (c.0 as isize + delta, c.1 as isize),
            )
        } else {
            (
                (a.0 as isize, a.1 as isize + delta),
                (c.0 as isize, c.1 as isize + delta),
            )
        };
        if [p.0, p.1, q.0, q.1]
            .iter()
            .all(|&v| v >= 0 && v < N as isize)
        {
            swap(
                b,
                (p.0 as usize, p.1 as usize),
                (q.0 as usize, q.1 as usize),
            );
        }
    }
}
pub fn playable(b: &Board) -> bool {
    for y in 0..N {
        for x in 0..N {
            for c in [(x + 1, y), (x, y + 1)] {
                if c.0 < N && c.1 < N {
                    let mut copy = *b;
                    if valid_swap(&mut copy, (x, y), c) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
pub fn fresh() -> Board {
    loop {
        let b = new_board();
        if playable(&b) {
            return b;
        }
    }
}
#[derive(Clone, Debug)]
pub struct TileFall {
    pub x: usize,
    pub from_y: isize,
    pub to_y: usize,
    pub kind: u8,
}
#[allow(dead_code)] // Used by the headless balance probe and board tests.
pub fn drop_board(b: &mut Board) {
    let _ = drop_board_animated(b);
}
pub fn drop_board_animated(b: &mut Board) -> Vec<TileFall> {
    let mut falls = Vec::new();
    for x in 0..N {
        let mut dest = N;
        for y in (0..N).rev() {
            if b[y][x] != 255 {
                dest -= 1;
                let kind = b[y][x];
                b[dest][x] = kind;
                if dest != y {
                    falls.push(TileFall {
                        x,
                        from_y: y as isize,
                        to_y: dest,
                        kind,
                    });
                }
            }
        }
        for y in 0..dest {
            b[y][x] = rng::gen_range(0, 7);
            falls.push(TileFall {
                x,
                from_y: y as isize - dest as isize,
                to_y: y,
                kind: b[y][x],
            });
        }
    }
    falls
}
#[cfg(test)]
mod fall_tests {
    use super::*;
    #[test]
    fn falling_motion_tracks_survivors_and_refill() {
        let mut b = [[2; N]; N];
        b[N - 1][0] = 255;
        b[N - 2][0] = 3;
        let falls = drop_board_animated(&mut b);
        assert_eq!(b[N - 1][0], 3);
        assert!(falls
            .iter()
            .any(|f| f.x == 0 && f.from_y == (N - 2) as isize && f.to_y == N - 1 && f.kind == 3));
        assert!(falls
            .iter()
            .any(|f| f.x == 0 && f.from_y < 0 && f.to_y == 0));
        assert!(b.iter().flatten().all(|&v| v < 7));
    }
}

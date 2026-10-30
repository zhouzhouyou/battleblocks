//! Stable compaction and refill; records describe motion independently of direction.
use super::*;

#[derive(Clone, Debug)]
pub struct TileFall {
    pub from: (isize, isize),
    pub to: (usize, usize),
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
                        from: (x as isize, y as isize),
                        to: (x, dest),
                        kind,
                    });
                }
            }
        }
        for y in 0..dest {
            b[y][x] = rng::gen_range(0, 7);
            falls.push(TileFall {
                from: (x as isize, y as isize - dest as isize),
                to: (x, y),
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
            .any(|f| f.from == (0, (N - 2) as isize) && f.to == (0, N - 1) && f.kind == 3));
        assert!(falls
            .iter()
            .any(|f| f.from.0 == 0 && f.from.1 < 0 && f.to == (0, 0)));
        assert!(b.iter().flatten().all(|&v| v < 7));
    }
}

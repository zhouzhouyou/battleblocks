//! Stable compaction and refill; records describe motion independently of direction.
use super::*;

#[derive(Clone, Debug)]
pub struct TileFall {
    pub from: (isize, isize),
    pub to: (usize, usize),
    pub kind: u8,
    pub portals: Vec<(Cell, Cell)>,
}
#[allow(dead_code)] // Used by the headless balance probe and board tests.
pub fn drop_board(b: &mut Board) {
    let _ = drop_board_animated(b);
}
pub fn drop_board_animated(b: &mut Board) -> Vec<TileFall> {
    drop_board_in(b, BoardMechanics::default()).expect("default gravity topology")
}
/// Compress each source-to-sink chain once, then refill from its boundary source.
pub fn drop_board_in(
    b: &mut Board,
    mechanics: BoardMechanics,
) -> Result<Vec<TileFall>, &'static str> {
    settle_board_in(b, mechanics, || Some(rng::gen_range(0, 7)))
}
/// Compact known survivors without generating tiles or consuming randomness.
pub fn compact_board_in(
    b: &mut Board,
    mechanics: BoardMechanics,
) -> Result<Vec<TileFall>, &'static str> {
    settle_board_in(b, mechanics, || None)
}
fn settle_board_in(
    b: &mut Board,
    mechanics: BoardMechanics,
    mut generate: impl FnMut() -> Option<u8>,
) -> Result<Vec<TileFall>, &'static str> {
    let topology = mechanics.topology()?;
    let mut falls = Vec::new();
    let (dx, dy) = mechanics.gravity().vector();
    for chain in &topology.chains {
        let survivors: Vec<_> = chain
            .iter()
            .enumerate()
            .filter_map(|(i, &(x, y))| (b[y][x] != 255).then_some((i, b[y][x])))
            .collect();
        let missing = chain.len() - survivors.len();
        for (offset, &(source, kind)) in survivors.iter().enumerate() {
            let dest = missing + offset;
            let (x, y) = chain[dest];
            b[y][x] = kind;
            if source != dest {
                falls.push(TileFall {
                    from: (chain[source].0 as isize, chain[source].1 as isize),
                    to: (x, y),
                    kind,
                    portals: chain[source..=dest]
                        .windows(2)
                        .filter(|w| topology.portal_links.contains(&(w[0], w[1])))
                        .map(|w| (w[0], w[1]))
                        .collect(),
                });
            }
        }
        for dest in 0..missing {
            let (x, y) = chain[dest];
            let Some(kind) = generate() else {
                b[y][x] = 255;
                continue;
            };
            b[y][x] = kind;
            let distance = (missing - dest) as isize;
            falls.push(TileFall {
                from: (
                    chain[0].0 as isize - dx * distance,
                    chain[0].1 as isize - dy * distance,
                ),
                to: (x, y),
                kind,
                portals: chain[..=dest]
                    .windows(2)
                    .filter(|w| topology.portal_links.contains(&(w[0], w[1])))
                    .map(|w| (w[0], w[1]))
                    .collect(),
            });
        }
    }
    Ok(falls)
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

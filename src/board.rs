use crate::rng;
pub const N: usize = 8;
pub type Board = [[u8; N]; N];
#[path = "board/topology.rs"]
mod topology;
pub use topology::*;
pub fn matches(b: &Board) -> Vec<(usize, usize)> {
    matches_in(b, BoardMechanics::default())
}
pub fn matches_in(b: &Board, mechanics: BoardMechanics) -> Vec<(usize, usize)> {
    let mut hit = [[false; N]; N];
    for y in 0..N {
        for x in 0..N {
            if b[y][x] == 255 {
                continue;
            }
            if x + 2 < N
                && b[y][x] == b[y][x + 1]
                && b[y][x] == b[y][x + 2]
                && !mechanics.blocks((x, y), (x + 1, y))
                && !mechanics.blocks((x + 1, y), (x + 2, y))
            {
                hit[y][x] = true;
                hit[y][x + 1] = true;
                hit[y][x + 2] = true;
            }
            if y + 2 < N
                && b[y][x] == b[y + 1][x]
                && b[y][x] == b[y + 2][x]
                && !mechanics.blocks((x, y), (x, y + 1))
                && !mechanics.blocks((x, y + 1), (x, y + 2))
            {
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
/// Insert a tile at `target`, push the rest of its row/column toward the
/// current settling side, and destroy the tile squeezed out at that edge.
pub fn insert_tile(b: &mut Board, target: (usize, usize), tile: u8, mechanics: BoardMechanics) {
    let (x, y) = target;
    match mechanics.gravity() {
        Gravity::Down => {
            for row in (y + 1..N).rev() {
                b[row][x] = b[row - 1][x];
            }
        }
        Gravity::Up => {
            for row in 0..y {
                b[row][x] = b[row + 1][x];
            }
        }
        Gravity::Right => {
            for column in (x + 1..N).rev() {
                b[y][column] = b[y][column - 1];
            }
        }
        Gravity::Left => {
            for column in 0..x {
                b[y][column] = b[y][column + 1];
            }
        }
    }
    b[y][x] = tile;
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
#[path = "board/gravity.rs"]
mod gravity;
pub use gravity::*;

/// Role-specific swaps are transactional, including invalid coordinates.
#[allow(dead_code)] // Legacy probes and tests use the default-topology wrapper.
pub fn hero_swap(b: &mut Board, a: (usize, usize), c: (usize, usize), hero: usize) -> bool {
    hero_swap_in(b, a, c, hero, BoardMechanics::default())
}
pub fn hero_swap_in(
    b: &mut Board,
    a: Cell,
    c: Cell,
    hero: usize,
    mechanics: BoardMechanics,
) -> bool {
    if !adjacent(a, c, hero) || !exchange_allowed(a, c, hero, false, mechanics) {
        return false;
    }
    let original = *b;
    if crate::content::movement_profile(hero).columns && a.1 == c.1 {
        for y in 0..N {
            swap(b, (a.0, y), (c.0, y));
        }
    } else {
        swap(b, a, c);
    }
    if matches_in(b, mechanics).is_empty() {
        *b = original;
        false
    } else {
        true
    }
}
pub fn adjacent(a: (usize, usize), c: (usize, usize), hero: usize) -> bool {
    if [a.0, a.1, c.0, c.1].iter().any(|&v| v >= N) {
        return false;
    }
    let dx = a.0.abs_diff(c.0);
    let dy = a.1.abs_diff(c.1);
    dx + dy == 1 || (crate::content::movement_profile(hero).diagonal && dx == 1 && dy == 1)
}
pub fn hero_options(b: &Board, hero: usize) -> Vec<((usize, usize), (usize, usize))> {
    hero_options_in(b, hero, BoardMechanics::default())
}
pub fn hero_options_in(b: &Board, hero: usize, mechanics: BoardMechanics) -> Vec<(Cell, Cell)> {
    let mut out = Vec::new();
    for y in 0..N {
        for x in 0..N {
            for (dx, dy) in [(1, 0), (0, 1), (1, 1), (-1, 1)] {
                let (cx, cy) = (x as isize + dx, y as isize + dy);
                if cx < 0 || cy < 0 || cx >= N as isize || cy >= N as isize {
                    continue;
                }
                if crate::content::movement_profile(hero).columns && dy == 0 && y > 0 {
                    continue;
                }
                let c = (cx as usize, cy as usize);
                if hero_swap_in(&mut b.clone(), (x, y), c, hero, mechanics) {
                    out.push(((x, y), c));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod passive_tests {
    use super::*;
    fn base() -> Board {
        std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
    }
    #[test]
    fn column_swap_moves_all_rows_and_rejects_without_mutation() {
        let mut b = base();
        b[0][0] = 6;
        b[0][1] = 2;
        b[0][2] = 6;
        b[0][3] = 6;
        let original = b;
        assert!(hero_swap(&mut b, (0, 0), (1, 0), 0));
        for y in 0..N {
            assert_eq!(b[y][0], original[y][1]);
            assert_eq!(b[y][1], original[y][0]);
        }
        let mut b = base();
        let original = b;
        assert!(!hero_swap(&mut b, (0, 0), (1, 0), 0));
        assert_eq!(b, original);
        assert!(!hero_swap(&mut b, (8, 0), (7, 0), 0));
        assert_eq!(b, original);
    }
    #[test]
    fn diagonal_swap_only_for_elf_and_preserves_match_rules() {
        let mut b = base();
        b[0][0] = 6;
        b[0][1] = 6;
        b[0][2] = 2;
        b[1][3] = 6;
        let original = b;
        assert!(!hero_swap(&mut b, (2, 0), (3, 1), 2));
        assert_eq!(b, original);
        assert!(hero_swap(&mut b, (2, 0), (3, 1), 1));
        assert!(matches(&b).contains(&(0, 0)));
    }
    #[test]
    fn all_role_options_are_legal_unique_and_match_playability() {
        crate::rng::srand(711);
        for _ in 0..12 {
            let b = fresh();
            for id in 0..7 {
                let moves = hero_options(&b, id);
                let mut unique = std::collections::HashSet::new();
                for &(a, c) in &moves {
                    assert!(hero_swap(&mut b.clone(), a, c, id));
                    assert!(unique.insert((a, c)));
                }
                if id != 0 {
                    assert!(!moves.is_empty());
                }
            }
        }
    }
}

/// Refresh for the actor who will actually use the resulting board.
pub fn fresh_for(hero: usize) -> Board {
    loop {
        let b = new_board();
        if !hero_options(&b, hero).is_empty() {
            return b;
        }
    }
}

#[cfg(test)]
mod insert_tests {
    use super::*;
    #[test]
    fn insert_tile_pushes_toward_each_gravity_edge_and_destroys_it() {
        let cases = [
            (Gravity::Down, (2, 2), [1, 2, 6, 3, 4, 5, 6, 1]),
            (Gravity::Up, (2, 5), [2, 3, 4, 5, 6, 6, 1, 2]),
            (Gravity::Left, (5, 2), [2, 3, 4, 5, 6, 6, 1, 2]),
            (Gravity::Right, (2, 2), [1, 2, 6, 3, 4, 5, 6, 1]),
        ];
        for (gravity, target, expected) in cases {
            let mut mechanics = BoardMechanics::default();
            mechanics.set_gravity(gravity, false);
            let mut board = [[0; N]; N];
            for i in 0..N {
                match gravity {
                    Gravity::Down | Gravity::Up => board[i][target.0] = (i % 6 + 1) as u8,
                    Gravity::Left | Gravity::Right => board[target.1][i] = (i % 6 + 1) as u8,
                }
            }
            insert_tile(&mut board, target, 6, mechanics);
            let line: Vec<u8> = (0..N)
                .map(|i| match gravity {
                    Gravity::Down | Gravity::Up => board[i][target.0],
                    Gravity::Left | Gravity::Right => board[target.1][i],
                })
                .collect();
            assert_eq!(line, expected, "{gravity:?}");
        }
    }
}

#[cfg(test)]
mod refresh_tests {
    #[test]
    fn refreshed_boards_are_match_free_and_playable_for_actor() {
        for id in 0..7 {
            for _ in 0..20 {
                let board = super::fresh_for(id);
                assert!(super::matches(&board).is_empty());
                assert!(!super::hero_options(&board, id).is_empty());
            }
        }
    }
}

/// Enumerate actual exchange inputs, including a prepared parallel exchange.
/// This is a read-only query; column exchanges are deduplicated by column pair.
#[cfg(test)]
pub fn move_hints(b: &Board, hero: usize, armed: bool) -> Vec<((usize, usize), (usize, usize))> {
    move_hints_in(b, hero, armed, BoardMechanics::default())
}
pub fn move_hints_in(
    b: &Board,
    hero: usize,
    armed: bool,
    mechanics: BoardMechanics,
) -> Vec<(Cell, Cell)> {
    if !armed {
        return hero_options_in(b, hero, mechanics);
    }
    let mut moves = Vec::new();
    for y in 0..N {
        for x in 0..N {
            for c in [(x + 1, y), (x, y + 1)] {
                if c.0 < N && c.1 < N && exchange_allowed((x, y), c, hero, true, mechanics) {
                    moves.push(((x, y), c));
                }
            }
        }
    }
    // An armed elf can still make an ordinary diagonal swap without using
    // the orthogonal parallel preparation, matching the input handler.
    if crate::content::movement_profile(hero).diagonal {
        moves.extend(
            hero_options_in(b, hero, mechanics)
                .into_iter()
                .filter(|&(a, c)| a.0 != c.0 && a.1 != c.1),
        );
    }
    moves
}

#[cfg(test)]
mod hint_tests {
    use super::*;
    #[test]
    fn hints_cover_every_legal_input_without_changing_state() {
        crate::rng::srand(613);
        for _ in 0..8 {
            let b = fresh();
            let original = b;
            let seed = crate::rng::state();
            for hero in 0..7 {
                for armed in [false, true] {
                    let moves = move_hints(&b, hero, armed);
                    let unique: std::collections::HashSet<_> = moves.iter().copied().collect();
                    assert_eq!(unique.len(), moves.len());
                    for y in 0..N {
                        for x in 0..N {
                            for (dx, dy) in [(1isize, 0), (0, 1), (1, 1), (-1, 1)] {
                                let (cx, cy) = (x as isize + dx, y as isize + dy);
                                if cx < 0 || cy < 0 || cx >= N as isize || cy >= N as isize {
                                    continue;
                                }
                                let a = (x, y);
                                let c = (cx as usize, cy as usize);
                                let expected = (armed && dx.abs() + dy.abs() == 1)
                                    || hero_swap(&mut b.clone(), a, c, hero);
                                let canonical = if !armed
                                    && crate::content::movement_profile(hero).columns
                                    && dy == 0
                                {
                                    ((x, 0), (c.0, 0))
                                } else {
                                    (a, c)
                                };
                                assert_eq!(unique.contains(&canonical), expected);
                            }
                        }
                    }
                }
            }
            assert_eq!(b, original);
            assert_eq!(crate::rng::state(), seed);
        }
    }
    #[test]
    fn no_match_board_has_no_hints_unless_prepared() {
        let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
        for hero in 0..7 {
            assert!(move_hints(&b, hero, false).is_empty());
            assert_eq!(move_hints(&b, hero, true).len(), 2 * N * (N - 1));
        }
    }
}

/// Multi-tile exchanges cannot smuggle a tile through a blocked boundary.
pub fn exchange_allowed(a: Cell, b: Cell, hero: usize, parallel: bool, m: BoardMechanics) -> bool {
    if m.blocks(a, b) {
        return false;
    }
    if !parallel && crate::content::movement_profile(hero).columns && a.1 == b.1 {
        return (0..N).all(|y| !m.blocks((a.0, y), (b.0, y)));
    }
    if parallel {
        for d in [-1isize, 1] {
            let (p, q) = if a.0 == b.0 {
                (
                    (a.0 as isize + d, a.1 as isize),
                    (b.0 as isize + d, b.1 as isize),
                )
            } else {
                (
                    (a.0 as isize, a.1 as isize + d),
                    (b.0 as isize, b.1 as isize + d),
                )
            };
            if [p.0, p.1, q.0, q.1]
                .iter()
                .all(|&v| v >= 0 && v < N as isize)
                && m.blocks((p.0 as usize, p.1 as usize), (q.0 as usize, q.1 as usize))
            {
                return false;
            }
        }
    }
    true
}
pub fn fresh_for_in(hero: usize, m: BoardMechanics) -> Board {
    loop {
        let b = new_board();
        if !hero_options_in(&b, hero, m).is_empty() {
            return b;
        }
    }
}
#[cfg(test)]
#[path = "board/topology_tests.rs"]
mod topology_tests;

use super::*;
fn pair(g: Gravity, a: Cell, b: Cell, owner: bool) -> PortalPair {
    PortalPair {
        a: PortalEdge::at(a, g),
        b: PortalEdge::at(b, g),
        owner,
    }
}
fn marked() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| (10 + y * N + x) as u8))
}
#[test]
fn all_directions_exchange_links_and_fill_both_ends_without_loss() {
    for g in [Gravity::Down, Gravity::Up, Gravity::Left, Gravity::Right] {
        let p = pair(g, (1, 2), (4, 4), false);
        let mut m = BoardMechanics::default();
        m.set_gravity(g, false);
        m.place(p).unwrap();
        let t = m.topology().unwrap();
        assert_eq!(t.chains.iter().map(Vec::len).sum::<usize>(), N * N);
        let cells: std::collections::HashSet<_> = t.chains.iter().flatten().copied().collect();
        assert_eq!(cells.len(), N * N);
        let mut b = marked();
        let full = b;
        let seed = crate::rng::state();
        assert!(drop_board_in(&mut b, m).unwrap().is_empty());
        assert_eq!(b, full);
        assert_eq!(crate::rng::state(), seed);
        for &(ua, db) in &t.portal_links {
            b[db.1][db.0] = 255;
            assert_ne!(ua, db);
        }
        let survivors: std::collections::HashSet<_> =
            b.iter().flatten().filter(|&&v| v != 255).copied().collect();
        let falls = drop_board_in(&mut b, m).unwrap();
        for &(ua, db) in &t.portal_links {
            assert_eq!(b[db.1][db.0], full[ua.1][ua.0]);
            assert!(falls
                .iter()
                .any(|f| f.kind == full[ua.1][ua.0] && f.to == db && f.portals == vec![(ua, db)]));
        }
        assert!(b.iter().flatten().all(|&v| v != 255));
        let after: std::collections::HashSet<_> =
            b.iter().flatten().filter(|&&v| v >= 10).copied().collect();
        assert_eq!(after, survivors);
    }
}
#[test]
fn either_downstream_hole_pulls_only_the_other_entrance() {
    let mut m = BoardMechanics::default();
    m.place(pair(Gravity::Down, (1, 2), (4, 4), false)).unwrap();
    for &(source, dest) in &m.topology().unwrap().portal_links {
        let mut b = marked();
        let original = b;
        b[dest.1][dest.0] = 255;
        let falls = drop_board_in(&mut b, m).unwrap();
        assert_eq!(b[dest.1][dest.0], original[source.1][source.0]);
        assert_eq!(falls.iter().map(|f| f.portals.len()).sum::<usize>(), 1);
    }
}
#[test]
fn geometry_rejections_are_atomic_and_global_cycles_are_detected() {
    let mut m = BoardMechanics::default();
    for p in [
        pair(Gravity::Down, (1, 2), (1, 4), false),
        pair(Gravity::Down, (1, 2), (1, 2), false),
        pair(Gravity::Down, (1, 7), (4, 4), false),
        pair(Gravity::Left, (1, 2), (4, 4), false),
    ] {
        let before = m;
        assert!(m.place(p).is_err());
        assert_eq!(m, before);
    }
    m.place(pair(Gravity::Down, (1, 1), (4, 5), false)).unwrap();
    let before = m;
    // Each pair is individually legal, but these two pairs close a middle loop.
    assert!(m.place(pair(Gravity::Down, (1, 5), (4, 1), true)).is_err());
    assert_eq!(m, before);
    assert!(m.place(pair(Gravity::Down, (1, 1), (6, 2), true)).is_err());
    assert_eq!(m, before);
}
#[test]
fn removal_expiry_and_gravity_revalidation_restore_geometry() {
    let p = pair(Gravity::Down, (1, 2), (4, 4), false);
    let mut m = BoardMechanics::default();
    m.place(p).unwrap();
    assert!(!m.start_turn(true));
    assert!(m.blocks((1, 2), (1, 3)));
    m.set_gravity(Gravity::Up, true);
    assert!(m.portals[0].is_some());
    m.set_gravity(Gravity::Left, true);
    assert!(m.portals.iter().all(Option::is_none));
    assert_eq!(m.gravity(), Gravity::Left);
    assert!(!m.start_turn(false));
    assert!(m.start_turn(true));
    assert_eq!(m, BoardMechanics::default());
    m.place(p).unwrap();
    assert!(m.start_turn(false));
    assert_eq!(m, BoardMechanics::default());
}
#[test]
fn gates_cut_matches_and_exchanges_without_connecting_remote_tiles() {
    let mut b: Board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    b[1][1] = 2;
    b[2][1] = 2;
    b[3][1] = 2;
    let mut m = BoardMechanics::default();
    m.place(pair(Gravity::Down, (1, 2), (4, 4), false)).unwrap();
    assert!(matches(&b).contains(&(1, 2)));
    assert!(!m.matches(&b).contains(&(1, 2)));
    let before = b;
    assert!(!hero_swap_in(&mut b, (1, 2), (1, 3), 3, m));
    assert_eq!(b, before);
    assert!(!move_hints_in(&b, 3, true, m).contains(&((1, 2), (1, 3))));
    assert!(m.matches(&marked()).is_empty());
}
#[test]
fn seeded_refill_is_deterministic_through_long_chains_and_multiple_pairs() {
    let mut m = BoardMechanics::default();
    m.place(pair(Gravity::Down, (0, 1), (3, 5), false)).unwrap();
    m.place(pair(Gravity::Down, (3, 1), (6, 5), true)).unwrap();
    let mut a = marked();
    for y in 0..N {
        for x in 0..N {
            if (x + y) % 3 == 0 {
                a[y][x] = 255;
            }
        }
    }
    let mut b = a;
    crate::rng::srand(301);
    let fa = drop_board_in(&mut a, m).unwrap();
    let seed = crate::rng::state();
    crate::rng::srand(301);
    let fb = drop_board_in(&mut b, m).unwrap();
    assert_eq!(a, b);
    assert_eq!(seed, crate::rng::state());
    assert_eq!(
        fa.iter()
            .map(|f| (&f.from, &f.to, &f.portals))
            .collect::<Vec<_>>(),
        fb.iter()
            .map(|f| (&f.from, &f.to, &f.portals))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        m.topology()
            .unwrap()
            .chains
            .iter()
            .map(Vec::len)
            .sum::<usize>(),
        64
    );
}
#[test]
fn invalid_topology_never_mutates_board_or_rng() {
    let m = BoardMechanics {
        portals: [Some(pair(Gravity::Down, (0, 1), (0, 4), false)), None],
        ..Default::default()
    };
    let mut b = [[255; N]; N];
    let before = b;
    let seed = crate::rng::state();
    assert!(drop_board_in(&mut b, m).is_err());
    assert_eq!(before, b);
    assert_eq!(seed, crate::rng::state());
}

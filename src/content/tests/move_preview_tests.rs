use super::*;
fn base() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
#[test]
fn previews_use_role_exchange_rules_rewards_caps_and_bonus() {
    let mut b = base();
    b[0][0] = 6;
    b[0][1] = 2;
    b[0][2] = 6;
    b[0][3] = 6;
    let p = preview_move(&b, 0, 500, 0, 700, 0, false, (0, 0), (1, 0)).unwrap();
    assert_eq!((p.heal, p.damage, p.mana_gain), (0, 22, 22));
    let p = preview_move(&b, 0, 500, 0, 700, 1, false, (0, 0), (1, 0)).unwrap();
    assert_eq!((p.damage, p.mana_gain), (44, 44));
    assert!(preview_move(&base(), 0, 700, 0, 700, 0, false, (0, 0), (1, 0)).is_none());
    assert!(preview_move(&b, 0, 700, 0, 700, 0, false, (0, 0), (1, 1)).is_none());
    let mut b = base();
    b[0][0] = 6;
    b[0][1] = 6;
    b[0][2] = 2;
    b[1][3] = 6;
    let p = preview_move(&b, 1, 599, 119, 7, 0, false, (2, 0), (3, 1)).unwrap();
    assert_eq!((p.heal, p.damage, p.mana_gain), (0, 7, 1));
    assert_eq!((p.health_after, p.mana_after), (599, 120));
    let mut b = base();
    b[0][0] = 1;
    b[0][1] = 2;
    b[0][2] = 1;
    b[0][3] = 1;
    let p = preview_move(&b, 0, 699, 0, 700, 0, false, (0, 0), (1, 0)).unwrap();
    assert_eq!((p.heal, p.health_after), (1, 700));
}
#[test]
fn every_hint_has_pure_preview_including_prepared_swaps() {
    crate::rng::srand(5017);
    let b = crate::board::fresh();
    let original = b;
    let seed = crate::rng::state();
    for hero in 0..7 {
        for armed in [false, true] {
            for (a, c) in crate::board::move_hints(&b, hero, armed) {
                let p = preview_move(
                    &b,
                    hero,
                    HEROES[hero].hp - 1,
                    mana_cap(hero, false) - 1,
                    7,
                    1,
                    armed,
                    a,
                    c,
                )
                .unwrap();
                assert!((0..=1).contains(&p.heal));
                assert!((0..=1).contains(&p.mana_gain));
                assert!((0..=7).contains(&p.damage));
                assert_eq!(p.health_after, HEROES[hero].hp - 1 + p.heal);
                assert_eq!(p.mana_after, mana_cap(hero, false) - 1 + p.mana_gain);
            }
        }
    }
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
    let p = preview_move(&base(), 0, 700, 0, 700, 0, true, (0, 0), (1, 0)).unwrap();
    assert!(p.cells.is_empty());
    assert_eq!((p.heal, p.damage, p.mana_gain), (0, 0, 0));
}
#[test]
fn ranked_moves_sort_by_total_then_heal_damage_mana() {
    // Rows 0, 2, 4 and 6 each get one swap that lines up three tiles of one kind:
    // damage+mana (total 42), heal+damage, heal+mana and balanced (each total 30).
    let mut b = base();
    for (y, kind, other) in [(0, 6, 2), (2, 4, 3), (4, 5, 2), (6, 0, 3)] {
        b[y][0] = kind;
        b[y][1] = other;
        b[y][2] = kind;
        b[y][3] = kind;
    }
    b[4][4] = 0;
    assert!(crate::matches(&b).is_empty());
    let original = b;
    let seed = crate::rng::state();
    // The flesh-casting zombie has no passive reward change.
    let ranked = ranked_moves(&b, 6, 500, 0, 700, 0, false);
    let rank = |y: usize| {
        let i = ranked
            .iter()
            .position(|&((a, c), _)| (a, c) == ((0, y), (1, y)) || (a, c) == ((1, y), (0, y)))
            .unwrap();
        let p = ranked[i].1.as_ref().unwrap();
        (i, (p.heal, p.damage, p.mana_gain))
    };
    let (damage_mana, heal_damage, heal_mana, balanced) = (rank(0), rank(2), rank(4), rank(6));
    assert_eq!(damage_mana.1, (0, 21, 21));
    assert_eq!(heal_damage.1, (9, 21, 0));
    assert_eq!(heal_mana.1, (9, 0, 21));
    assert_eq!(balanced.1, (6, 12, 12));
    assert!(damage_mana.0 < heal_damage.0);
    assert!(heal_damage.0 < heal_mana.0);
    assert!(heal_mana.0 < balanced.0);
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn ranked_moves_keep_every_hint_with_its_pure_preview() {
    crate::rng::srand(2718);
    for _ in 0..6 {
        let b = crate::board::fresh();
        let original = b;
        let seed = crate::rng::state();
        for hero in 0..7 {
            for armed in [false, true] {
                let (hp, mp) = (HEROES[hero].hp / 2, HEROES[hero].mp / 2);
                let ranked = ranked_moves(&b, hero, hp, mp, 300, 0, armed);
                let mut listed: Vec<_> = ranked.iter().map(|(m, _)| *m).collect();
                let mut hints = crate::board::move_hints(&b, hero, armed);
                listed.sort();
                hints.sort();
                assert_eq!(listed, hints);
                let key = |p: &MovePreview| {
                    (
                        p.heal + p.damage + p.mana_gain,
                        p.heal,
                        p.damage,
                        p.mana_gain,
                    )
                };
                for pair in ranked.windows(2) {
                    let (first, second) = (pair[0].1.as_ref(), pair[1].1.as_ref());
                    assert!(second.is_none() || key(first.unwrap()) >= key(second.unwrap()));
                }
                for &((a, c), ref p) in &ranked {
                    let expected = preview_move(&b, hero, hp, mp, 300, 0, armed, a, c);
                    assert_eq!(p.as_ref().map(key), expected.as_ref().map(key));
                }
            }
        }
        assert_eq!(b, original);
        assert_eq!(crate::rng::state(), seed);
    }
}

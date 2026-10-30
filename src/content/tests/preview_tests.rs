use super::*;
#[test]
fn guard_consumes_pure_healing_only_without_additional_rewards() {
    let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    let original = board;
    let seed = crate::rng::state();
    let expected: Vec<_> = (0..N)
        .flat_map(|y| (0..N).filter_map(move |x| (board[y][x] == 1).then_some((x, y))))
        .collect();
    let e = effect(Skill::Guard, &mut board.clone());
    assert_eq!(e.cells, expected);
    assert!(!e.reward);
    assert_eq!(e.damage, 0);
    assert_eq!(e.mp, 0);
    assert_eq!(e.hp, HEROES[0].hp / 5);
    assert!(e
        .cells
        .iter()
        .all(|&(x, y)| board[y][x] != 4 && board[y][x] != 5));
    assert_eq!(target_cells(Skill::Guard, &board, (3, 3)), expected);
    let preview = preview_rewards(Skill::Guard, &board, (3, 3), 0, 400, 300, 700, 0);
    assert_eq!(preview.heal, HEROES[0].hp / 5);
    assert_eq!(preview.mana_after, 150);
    assert_eq!(preview.mana_gain, 0);
    assert_eq!(board, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn charge_target_preview_is_positive_and_matches_selected_columns() {
    let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    for x in 0..N {
        let p = preview_rewards(Skill::Charge, &b, (x, 3), 0, 700, 300, 2000, 0);
        let start = x.min(N - 2);
        let expected = 60
            + (0..N)
                .flat_map(|y| (start..start + 2).map(move |c| VALUES[b[y][c] as usize].1))
                .sum::<i32>();
        assert_eq!(p.damage, expected);
        assert!(p.damage >= 60);
        assert_eq!(p.mana_after, 200);
        assert_eq!(p.mana_gain, 0);
    }
}
#[test]
fn transformations_count_first_matches_and_preview_is_pure() {
    let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    let mut board = board;
    board[0][0] = 1;
    board[0][1] = 1;
    board[0][2] = 2;
    let seed = crate::rng::state();
    let original = board;
    let p = preview_rewards(Skill::Corrupt, &board, (3, 3), 6, 500, 400, 700, 0);
    assert!(p.heal >= 3 * VALUES[1].0);
    assert_eq!(board, original);
    assert_eq!(crate::rng::state(), seed);
    let affected = target_cells(Skill::Corrupt, &board, (3, 3));
    assert!(affected.contains(&(0, 0)) && affected.contains(&(2, 0)));
}
#[test]
fn target_ranges_cover_every_target() {
    let board = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    for skill in [
        Skill::Charge,
        Skill::Nature,
        Skill::Greedy,
        Skill::Chort,
        Skill::Balance,
        Skill::War,
        Skill::Drain,
    ] {
        let (low, high) = preview_range(skill, &board, 0, 400, 300, 700, 0);
        for y in 0..N {
            for x in 0..N {
                let p = preview_rewards(skill, &board, (x, y), 0, 400, 300, 700, 0);
                assert!((low.heal..=high.heal).contains(&p.heal));
                assert!((low.damage..=high.damage).contains(&p.damage));
                assert!((low.mana_gain..=high.mana_gain).contains(&p.mana_gain));
            }
        }
    }
}
#[test]
fn lightning_shows_cost_and_actual_mana() {
    let b = [[3; N]; N];
    let p = preview_rewards(Skill::Lightning, &b, (3, 3), 2, 500, 200, 700, 0);
    assert_eq!(
        p.mana_gain,
        (2 * N as i32 * VALUES[3].2).min(250 - (200 - Skill::Lightning.cost()))
    );
    assert_eq!(
        p.mana_after,
        (200 - Skill::Lightning.cost() + 2 * N as i32 * VALUES[3].2).min(250)
    );
    assert_eq!(b, [[3; N]; N]);
}
#[test]
fn healing_and_damage_obey_caps() {
    let p = preview_rewards(Skill::Guard, &[[1; N]; N], (3, 3), 0, 650, 300, 500, 0);
    assert_eq!(p.heal, 50);
    assert_eq!(p.mana_after, 150);
    let p = preview_rewards(Skill::Charge, &[[2; N]; N], (3, 3), 0, 700, 300, 20, 0);
    assert_eq!(p.damage, 20);
}

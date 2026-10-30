use super::*;
fn dead() -> Board {
    std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8))
}
#[test]
fn deadlock_respects_role_moves_and_preparation() {
    let b = dead();
    for hero in 0..7 {
        assert!(must_refresh(&b, hero, 1, 0, false));
    }
    assert!(!must_refresh(&b, 0, 700, 0, true));
    // Strike alone cannot unlock a board, Nature can.
    assert!(must_refresh(&b, 1, 600, 35, false));
    assert!(!must_refresh(&b, 1, 600, 60, false));
    let mut diagonal = b;
    diagonal[0][0] = 6;
    diagonal[0][1] = 6;
    diagonal[0][2] = 2;
    diagonal[1][3] = 6;
    assert!(!must_refresh(&diagonal, 1, 600, 0, false));
    let mut columns = b;
    columns[0][0] = 6;
    columns[0][1] = 2;
    columns[0][2] = 6;
    columns[0][3] = 6;
    assert!(!must_refresh(&columns, 0, 700, 0, false));
}
#[test]
fn skill_escape_checks_effects_caps_and_blood_before_heal() {
    let b = dead();
    let original = b;
    let seed = crate::rng::state();
    assert!(!must_refresh(&b, 0, 700, 100, false));
    assert!(!must_refresh(&b, 3, 500, 80, false));
    assert!(!must_refresh(&b, 6, 101, 0, false));
    assert!(must_refresh(&b, 6, 100, 0, false));
    assert!(!must_refresh(&b, 6, 51, 25, false));
    assert!(must_refresh(&b, 6, 50, 25, false));
    let no_targets =
        std::array::from_fn(|y| std::array::from_fn(|x| [3, 4, 5, 6][(x + 2 * y) % 4]));
    assert!(usable_skills(&no_targets, 0, 700, 150).contains(&Skill::Charge));
    assert!(!usable_skills(&no_targets, 0, 700, 150).contains(&Skill::Guard));
    assert!(!usable_skills(&no_targets, 4, 700, 60).contains(&Skill::Imp));
    assert!(usable_skills(&no_targets, 4, 700, 60).contains(&Skill::Balance));
    assert!(!usable_skills(&no_targets, 6, 775, 160).contains(&Skill::Corrupt));
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}

use super::*;
#[test]
fn clear_shapes_and_rewards_match_skill_rules() {
    let mut b = [[2; N]; N];
    let blade = effect(Skill::Charge, &mut b);
    assert_eq!(blade.cells.len(), 16);
    assert_eq!(blade.damage, 60 + 2 * N as i32 * VALUES[2].1);
    assert!(!blade.reward);
    let lightning = effect(Skill::Lightning, &mut [[3; N]; N]);
    assert_eq!(lightning.cells.len(), 16);
    assert_eq!(lightning.mp, 2 * N as i32 * VALUES[3].2);
    let meteor = effect(Skill::Meteor, &mut b);
    assert_eq!(meteor.cells.len(), 40);
    assert_eq!(
        meteor.damage,
        60 + meteor.cells.len() as i32 * (VALUES[2].1 / 2)
    );
    let imp = effect(Skill::Imp, &mut [[0; N]; N]);
    assert_eq!(imp.cells.len(), 64);
    assert!(!imp.reward);
    assert!(effect(Skill::Landslide, &mut b).armed);
    assert_eq!(effect(Skill::Strike, &mut b).bonus, 1);
}
#[test]
fn selected_targets_and_new_combos() {
    let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + y) % 7) as u8));
    let e = effect_at(Skill::Charge, &mut b.clone(), (7, 7));
    assert!(e.cells.iter().all(|&(x, _)| x >= 6));
    let mut changed = b;
    effect_at(Skill::War, &mut changed, (7, 7));
    assert_eq!(
        changed
            .iter()
            .flatten()
            .zip(b.iter().flatten())
            .filter(|(a, b)| a != b)
            .count(),
        4
    );
    assert_eq!(
        effect_at(Skill::Drain, &mut [[1; N]; N], (7, 7)).hp,
        8 * VALUES[1].0
    );
    assert_eq!(effect(Skill::Blood, &mut b.clone()).hp, 100);
    for hero in HEROES.iter() {
        assert_eq!(hero.skills.len(), 2);
    }
}
#[test]
fn knight_charge_and_guard_rules() {
    let charge = effect_at(Skill::Charge, &mut [[1; N]; N], (0, 0));
    assert_eq!(charge.damage, 60);
    assert!(!charge.reward);
    let guard = effect(Skill::Guard, &mut [[2; N]; N]);
    assert_eq!(guard.hp, HEROES[0].hp / 5);
    assert!(guard.cells.is_empty());
    assert!(!guard.armed);
    assert_eq!(guard.bonus, 0);
    assert_eq!(Skill::Guard.cost(), 150);
    assert_eq!(HEROES[0].skills, &[Skill::Charge, Skill::Guard]);
}
#[test]
fn all_heroes_can_pay_their_skills() {
    for h in HEROES.iter() {
        for s in h.skills {
            assert!(s.cost() <= h.mp);
        }
    }
}
#[test]
fn effects_are_unique_and_in_bounds() {
    for s in [
        Skill::Charge,
        Skill::Lightning,
        Skill::Meteor,
        Skill::Greedy,
        Skill::Imp,
        Skill::Chort,
    ] {
        let mut b = [[0; N]; N];
        let e = effect(s, &mut b);
        let mut cells = e.cells.clone();
        cells.sort();
        cells.dedup();
        assert_eq!(cells.len(), e.cells.len());
        assert!(cells.iter().all(|&(x, y)| x < N && y < N));
    }
}
#[test]
fn corrupt_and_nature_change_board() {
    let mut b = [[2; N]; N];
    effect(Skill::Corrupt, &mut b);
    assert!(b.iter().flatten().all(|&t| t == 1));
    effect(Skill::Nature, &mut b);
    assert_eq!(b.iter().flatten().filter(|&&t| t == 0).count(), 4);
}

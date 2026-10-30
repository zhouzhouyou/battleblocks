use super::*;
#[test]
fn payment_requires_surviving_before_any_heal() {
    assert_eq!(
        payment(6, 101, 0, Skill::Drain),
        Some(Payment {
            mana: 0,
            blood: 100
        })
    );
    assert_eq!(payment(6, 100, 0, Skill::Drain), None);
    assert_eq!(
        payment(6, 60, 25, Skill::Drain),
        Some(Payment {
            mana: 25,
            blood: 50
        })
    );
    assert_eq!(payment(6, 50, 25, Skill::Drain), None);
    assert_eq!(payment(0, 700, 25, Skill::Drain), None);
    assert_eq!(
        payment(6, 1, 50, Skill::Drain),
        Some(Payment { mana: 50, blood: 0 })
    );
}
#[test]
fn clear_passives_have_exact_rewards_and_thresholds() {
    let cells = vec![(0, 0), (1, 0), (2, 0)];
    assert_eq!(
        clear_rewards(2, 300, 0, &[[1; N]; N], &cells, 1, 0),
        (9, 0, 9)
    );
    assert_eq!(
        clear_rewards(3, 720, 490, &[[1; N]; N], &cells, 1, 0),
        (18, 0, 6)
    );
    assert_eq!(
        clear_rewards(4, 300, 0, &[[0; N]; N], &cells, 1, 0),
        (6, 24, 18)
    );
    assert_eq!(
        clear_rewards(5, 210, 0, &[[2; N]; N], &cells, 1, 0),
        (0, 45, 0)
    );
    assert_eq!(
        clear_rewards(5, 211, 0, &[[2; N]; N], &cells, 1, 0),
        (0, 36, 0)
    );
    assert_eq!(
        clear_rewards(4, 300, 0, &[[0; N]; N], &cells, 2, 1),
        (12, 50, 37)
    );
}
#[test]
fn blood_preview_is_pure_accounts_for_cost_caps_and_targets() {
    let b = std::array::from_fn(|y| std::array::from_fn(|x| ((x + 2 * y) % 7) as u8));
    let original = b;
    let seed = crate::rng::state();
    for target in [(0, 0), (3, 3), (7, 7)] {
        let p = preview_rewards(Skill::Drain, &b, target, 6, 200, 25, 700, 0);
        let e = effect_at(Skill::Drain, &mut b.clone(), target);
        assert_eq!(p.blood_cost, 50);
        assert_eq!(p.health_after, 150 + e.hp);
        assert_eq!(p.heal, e.hp - 50);
        assert_eq!(p.mana_after, 0);
    }
    let p = preview_rewards(
        Skill::Drain,
        &[[1; N]; N],
        (0, 0),
        6,
        HEROES[6].hp - 1,
        49,
        700,
        0,
    );
    assert_eq!(p.health_after, HEROES[6].hp);
    assert_eq!(p.blood_cost, 2);
    assert_eq!(b, original);
    assert_eq!(crate::rng::state(), seed);
}
#[test]
fn passive_skill_preview_matches_manual_first_wave_with_caps() {
    let b = [[0; N]; N];
    for id in 0..7 {
        let skill = Skill::Nature;
        let hp = HEROES[id].hp - 5;
        let mp = mana_cap(id, false);
        let p = preview_rewards(skill, &b, (0, 0), id, hp, mp, 700, 0);
        let mut copy = b;
        let e = effect_at(skill, &mut copy, (0, 0));
        let health = (hp + e.hp).min(HEROES[id].hp);
        let mana = mp - skill.cost() + e.mp;
        let (h, d, m) = clear_rewards(id, health, mana, &copy, &crate::matches(&copy), 1, 0);
        assert_eq!(p.health_after, (health + h).min(HEROES[id].hp));
        assert_eq!(p.damage, (e.damage + d).min(700));
        assert_eq!(p.mana_after, (mana + m).min(mana_cap(id, false)));
    }
}

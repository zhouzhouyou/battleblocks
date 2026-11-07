use super::*;
use content::HeroGroup::{Legacy, New};

#[test]
fn groups_cover_every_hero_once_with_complete_assets_and_descriptions() {
    assert_eq!(New.roster().len(), 7);
    assert_eq!(Legacy.roster().len(), 7);
    let mut ids: Vec<_> = content::HeroGroup::ALL
        .into_iter()
        .flat_map(|group| group.roster().iter().copied())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, (0..HEROES.len()).collect::<Vec<_>>());
    for id in ids {
        assert!(!content::skills(id, false).is_empty());
        assert!(!content::skills(id, true).is_empty());
        assert!(content::role_lore(id).is_some());
        assert!(sprites::SPRITES.get(id).is_some());
        assert!(sprites::AWAKENED.get(id).is_some());
    }
    assert_eq!(content::HeroGroup::for_hero(0), Some(Legacy));
    assert_eq!(content::HeroGroup::for_hero(2), Some(Legacy));
    assert_eq!(
        content::DEFAULT_AUTOPLAY_ROSTER,
        &[8, 7, 9, 10, 1, 11, 3, 4, 5, 6]
    );
}

#[test]
fn switching_groups_resets_only_that_pages_selection_and_preview() {
    let mut session = Session::new_headless();
    session.hero_id = content::KNIGHT;
    session.character_page = 2;
    session.refresh_preview(2);
    session.interface.opponent = Some(content::PRIEST);
    session.interface.hero_scroll = 5;
    session.set_hero_group(Legacy);
    assert_eq!(session.hero_id, 0);
    assert_eq!(session.preview.skill, 0);
    assert_eq!(session.interface.hero_scroll, 0);
    assert_eq!(session.interface.opponent, Some(content::PRIEST));
    assert_eq!(session.interface.opponent_group, New);
    session.hero_id = 2;
    session.refresh_preview(1);
    // Clicking the already selected tab must not reset the hero/preview.
    session.set_hero_group(Legacy);
    assert_eq!(session.hero_id, 2);
    assert_eq!(session.preview.skill, 1);
    session.interface.opponent_scroll = 4;
    session.set_opponent_group(Legacy);
    assert_eq!(session.interface.opponent, None);
    assert_eq!(session.interface.opponent_scroll, 0);
    assert_eq!(session.hero_id, 2);
    assert_eq!(session.preview.skill, 1);
    session.interface.opponent = Some(0);
    session.set_opponent_group(Legacy);
    assert_eq!(session.interface.opponent, Some(0));
    session.set_opponent_group(New);
    session.set_hero_group(New);
    assert_eq!(session.hero_id, content::DWARF);
    assert_eq!(session.preview.skill, 0);
    assert_eq!(session.interface.opponent, None);
}

#[test]
fn random_battle_and_restart_follow_opponent_group_for_every_player() {
    let mut session = Session::new_headless();
    session.prepare_headless_batch();
    for group in content::HeroGroup::ALL {
        session.set_opponent_group(group);
        for hero in 0..HEROES.len() {
            session.hero_id = hero;
            session.interface.opponent = None;
            let mut seen = std::collections::BTreeSet::new();
            for seed in 1..=64 {
                rng::srand(seed);
                session.reset_battle();
                assert!(group.roster().contains(&session.enemy_id));
                assert_ne!(session.enemy_id, hero);
                seen.insert(session.enemy_id);
            }
            let expected: std::collections::BTreeSet<_> = group
                .roster()
                .iter()
                .copied()
                .filter(|&id| id != hero)
                .collect();
            assert_eq!(seen, expected);
            let mut frame = Frame::headless(0.);
            frame.restart_command = true;
            session.apply_restart_command(&frame);
            assert!(group.roster().contains(&session.enemy_id));
            assert_ne!(session.enemy_id, hero);
        }
    }
}

#[test]
fn explicit_cross_group_opponents_and_mirrors_survive_restart() {
    let mut session = Session::new_headless();
    session.prepare_headless_batch();
    for (hero, enemy) in [(0, content::KNIGHT), (content::FIRE_ELF, 2), (2, 2)] {
        session.hero_id = hero;
        session.interface.opponent = Some(enemy);
        session.sync_selection_groups();
        session.reset_battle();
        assert_eq!(session.enemy_id, enemy);
        let mut frame = Frame::headless(0.);
        frame.restart_command = true;
        session.apply_restart_command(&frame);
        assert_eq!(session.enemy_id, enemy);
        session.sync_selection_groups();
        assert_eq!(session.hero_id, hero);
        assert_eq!(session.interface.opponent, Some(enemy));
        assert!(session.interface.hero_group.roster().contains(&hero));
        assert!(session.interface.opponent_group.roster().contains(&enemy));
    }
}

#[test]
fn console_edits_and_rollback_reveal_the_actual_hero_on_return() {
    let mut session = Session::new_headless();
    session.hero_id = content::KNIGHT;
    session.interface.opponent_group = Legacy;
    session.queue_headless_command("pause on".into());
    session.headless_step_command();
    session.queue_headless_command("hero player 2".into());
    session.headless_step_command();
    session.sync_selection_groups();
    assert_eq!(session.hero_id, 2);
    assert_eq!(session.interface.hero_group, Legacy);
    assert_eq!(session.interface.opponent_group, Legacy);
    session.queue_headless_command("undo".into());
    session.headless_step_command();
    session.sync_selection_groups();
    assert_eq!(session.hero_id, content::KNIGHT);
    assert_eq!(session.interface.hero_group, New);
    assert_eq!(session.interface.opponent_group, Legacy);
    assert!(session.interface.opponent.is_none());
}

#[test]
fn slingshot_is_a_new_role_and_only_shares_the_legacy_goblin_artwork() {
    use content::{Passive, Skill, SLINGSHOT_USER};
    assert_eq!(content::skills(3, false), &[Skill::Greedy, Skill::Blood]);
    assert_eq!(content::skills(3, true), &[Skill::Greedy, Skill::Blood]);
    assert_eq!(content::passives(3, false), &[Passive::Recycling]);
    assert_eq!(
        content::passives(3, true),
        &[Passive::Recycling, Passive::LootMastery]
    );
    assert_eq!(content::HeroGroup::for_hero(3), Some(Legacy));
    assert_eq!(content::HeroGroup::for_hero(SLINGSHOT_USER), Some(New));
    for awake in [false, true] {
        assert_eq!(
            content::skills(SLINGSHOT_USER, awake),
            &[Skill::LightSlingshot, Skill::HeavySlingshot]
        );
        assert!(content::passives(SLINGSHOT_USER, awake).is_empty());
        for skill in content::skills(3, awake) {
            assert!(content::skill_profile(
                *skill,
                content::Rules {
                    awake,
                    ..Default::default()
                }
            )
            .projectile
            .is_none());
        }
    }
    let original = &sprites::SPRITES[3];
    let shared = &sprites::SPRITES[SLINGSHOT_USER];
    assert_eq!(original.bytes(), shared.bytes());
    assert_eq!(original.indices, shared.indices);
    assert_eq!(original.durations, shared.durations);
    assert_eq!(
        sprites::AWAKENED[3].bytes(),
        sprites::AWAKENED[SLINGSHOT_USER].bytes()
    );
    assert_eq!(
        sprites::thumbnail_index(SLINGSHOT_USER),
        sprites::thumbnail_index(3)
    );
    assert_ne!(HEROES[3].zh, HEROES[SLINGSHOT_USER].zh);
}

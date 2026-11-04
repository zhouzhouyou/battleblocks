use super::*;
fn state(enemy_turn: bool, hp: i32) -> Snapshot {
    Snapshot {
        cooldowns: Default::default(),
        new_role_states: Default::default(),
        awakened: [false; 2],
        reward_skill: None,
        board_mechanics: Default::default(),
        board: [[1; 8]; 8],
        hero: 0,
        enemy: 1,
        hp,
        enemy_hp: 550,
        mp: 200,
        enemy_mp: 50,
        enemy_turn,
        action_done: false,
        pending: vec![(2, 3)],
        deadline: 0.2,
        enemy_clock: 0.5,
        bonuses: [1, 0],
        armed_sides: [true, false],
        bonus: 1,
        armed: true,
        chain: 2,
        reward: true,
        difficulty: 2,
        paused: false,
        rng_state: 42,
        charge_damage: None,
        charge_remaining: 0.,
        round: 3,
        turns: crate::turns::Log::new(),
    }
}
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn visual_action_survives_editor_checkpoints_but_changes_on_gameplay_and_undo() {
    let mut c = console();
    let first = c.checkpoint(state(false, 200));
    c.checkpoint_edit(state(false, 150));
    assert_eq!(c.gameplay_action(), first);
    let second = c.checkpoint(state(true, 150));
    assert_eq!(c.gameplay_action(), second);
    c.undo(false).unwrap();
    assert_eq!(c.gameplay_action(), first);
    c.reset_history();
    assert_eq!(c.gameplay_action(), 0);
}
#[cfg(not(target_arch = "wasm32"))]
fn console() -> Console {
    let (_, receiver) = std::sync::mpsc::channel();
    Console {
        open: false,
        zh: false,
        follow_game: true,
        scroll: 0,
        input: String::new(),
        output: VecDeque::new(),
        commands: VecDeque::new(),
        history: Vec::new(),
        records: Vec::new(),
        seq: 0,
        started: macroquad::miniquad::date::now(),
        quiet_events: false,
        batch: false,
        action: 0,
        next_action: 0,
        file: None,
        receiver,
        path: String::new(),
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn undo_player_removes_enemy_actions_and_debug_edits() {
    let mut c = console();
    let original = state(false, 700);
    c.checkpoint(original.clone());
    c.checkpoint(state(true, 650));
    c.checkpoint_edit(state(false, 600));
    let (restored, id) = c.undo(true).unwrap();
    assert_eq!(id, 1);
    assert_eq!(restored.json(), original.json());
    assert!(c.history.is_empty());
    assert!(c.undo(false).is_none());
    assert_eq!(c.checkpoint(state(false, 500)), 4); // IDs never reused after rollback.
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn undo_latest_retains_earlier_checkpoints() {
    let mut c = console();
    c.checkpoint(state(false, 700));
    c.checkpoint(state(true, 650));
    let (restored, id) = c.undo(false).unwrap();
    assert_eq!(restored.hp, 650);
    assert_eq!(id, 2);
    assert_eq!(c.history.len(), 1);
    assert_eq!(c.undo(false).unwrap().0.hp, 700);
}
#[test]
fn command_validation() {
    assert_eq!(parse("tile 8 1 6"), Ok(Command::SetTile(7, 0, 6)));
    for bad in [
        "tile 0 1 2",
        "tile 1 1 7",
        "hp player -1",
        "move 9 1 2 3",
        "ai 3",
        "undo enemy",
        "cast 4",
        "hp player 5 extra",
    ] {
        assert!(parse(bad).is_err(), "{bad}");
    }
    assert_eq!(parse("undo player"), Ok(Command::Undo(true)));
}
#[test]
fn json_escapes_control_characters() {
    assert_eq!(quote("a\n\"\\"), "\"a\\n\\\"\\\\\"");
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn console_language_follows_the_game_until_fixed() {
    let mut c = console();
    c.sync_language(true);
    assert!(c.zh);
    c.sync_language(false);
    assert!(!c.zh);
    // A fixed language ignores the game's language toggles.
    c.set_language(Some(true), false);
    c.sync_language(false);
    assert!(c.zh);
    assert!(c.output.back().unwrap().contains("中文"));
    // lang auto takes the game's current language at once and keeps following it.
    c.set_language(None, false);
    assert!(!c.zh);
    c.sync_language(true);
    assert!(c.zh);
}
#[test]
fn board_load_requires_64_valid_tiles() {
    let text = format!("board load {}", vec!["2"; 64].join(" "));
    assert_eq!(parse(&text), Ok(Command::LoadBoard([[2; 8]; 8])));
    assert!(parse("board load 2 2").is_err());
}
#[test]
fn lang_command_and_errors_are_specific() {
    assert_eq!(parse("lang zh"), Ok(Command::Lang(Some(true))));
    assert_eq!(parse("lang cn"), Ok(Command::Lang(Some(true))));
    assert_eq!(parse("lang en"), Ok(Command::Lang(Some(false))));
    assert_eq!(parse("lang auto"), Ok(Command::Lang(None)));
    for (bad, error) in [
        ("lang", ParseError::LangUsage),
        ("lang fr", ParseError::LangUsage),
        ("tile 0 1 2", ParseError::Coordinate),
        ("tile 1 1 7", ParseError::TileType),
        ("hp player -1", ParseError::Negative),
        ("hp boss 1", ParseError::Side),
        ("cast 4", ParseError::SkillSlot),
        ("pause maybe", ParseError::PauseUsage),
        ("seed -1", ParseError::Seed),
        ("tile x 1 2", ParseError::ExpectedInteger),
        ("dance", ParseError::Unknown),
    ] {
        assert_eq!(parse(bad), Err(error), "{bad}");
    }
}
#[test]
fn every_parse_error_has_both_languages() {
    use ParseError::*;
    for error in [
        MissingArgument,
        ExpectedInteger,
        Coordinate,
        Side,
        Negative,
        TileType,
        Seed,
        ArmedUsage,
        PauseUsage,
        SkillSlot,
        LangUsage,
        Unknown,
    ] {
        let cjk = |text: &str| text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
        let (en, zh) = (error.message(false), error.message(true));
        assert!(!en.is_empty() && !cjk(en), "{error:?}");
        assert!(cjk(zh), "{error:?} has no Chinese message");
    }
}
#[test]
fn wrapped_rows_stay_within_the_limit_and_keep_the_text() {
    let text = "#4 action=1 resolution: cells=[(2, 6), (3, 6), (4, 6)] reward=true multiplier_percent=100 heal=0 damage=22 mana=22 chain=2 refreshed=false actual_heal=0";
    let rows = wrap_units(text, 40.);
    assert!(rows.len() > 1);
    assert!(rows.iter().all(|row| text_units(row) <= 40.));
    assert_eq!(
        rows.join(" ").split_whitespace().collect::<Vec<_>>(),
        text.split_whitespace().collect::<Vec<_>>()
    );
    let cjk = "控制台打开时战斗暂停；move、cast、resolve 会关闭控制台并按正常规则结算。";
    let rows = wrap_units(cjk, 12.);
    assert!(rows.iter().all(|row| text_units(row) <= 12.));
    assert_eq!(rows.concat().replace(' ', ""), cjk.replace(' ', ""));
    assert_eq!(wrap_units("", 10.), vec![String::new()]);
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn help_switches_language_and_keeps_the_same_sections() {
    let mut c = console();
    c.help();
    let english: Vec<_> = c.output.drain(..).collect();
    c.zh = true;
    c.help();
    let chinese: Vec<_> = c.output.drain(..).collect();
    assert_eq!(english.len(), chinese.len());
    assert!(english.iter().all(|line| !line.contains('～')));
    assert!(chinese.iter().any(|line| line.contains("lang en")));
    // Every section line has a category column, in both languages.
    for lines in [&english, &chinese] {
        assert!(lines[1..]
            .iter()
            .all(|line| line.split_once('\t').is_some()));
    }
    assert!(c
        .rows()
        .iter()
        .all(|(_, row, x)| text_units(row) * 15. <= 1000. - x));
}
#[test]
fn console_names_events_in_chinese_but_keeps_log_identifiers() {
    assert_eq!(event_name("resolution", true), "结算");
    assert_eq!(event_name("resolution", false), "resolution");
    assert_eq!(
        event_detail("All cascades resolved; next actor", true),
        "连锁结算完毕，轮到下一方"
    );
    // Measured details are shown exactly as logged.
    assert_eq!(
        event_detail("swap [1,1] -> [2,1]", true),
        "swap [1,1] -> [2,1]"
    );
}

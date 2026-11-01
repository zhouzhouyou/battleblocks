use super::*;
#[test]
fn cooldown_counts_only_owner_turns_and_zero_is_unlimited() {
    let mut state = Cooldowns::default();
    state.cast(0, Skill::Charge);
    assert_eq!(state.remaining(0, Skill::Charge), 2);
    state.start_turn(1);
    assert_eq!(state.remaining(0, Skill::Charge), 2);
    state.start_turn(0);
    assert_eq!(state.remaining(0, Skill::Charge), 1);
    state.start_turn(0);
    assert_eq!(state.remaining(0, Skill::Charge), 0);
    state.cast(0, Skill::Strike);
    assert_eq!(state.remaining(0, Skill::Strike), 0);
    assert!(!Skill::Strike.ends_turn());
    assert!(Skill::Charge.ends_turn());
}

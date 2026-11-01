use super::*;
#[test]
fn cascades_are_bounded_and_first_wave_is_linear() {
    assert_eq!(reward_scale(1, 0), 100);
    assert_eq!(reward_scale(2, 0), 110);
    assert_eq!(reward_scale(100, 0), 130);
    assert_eq!(reward_scale(1, 1), 200);
}

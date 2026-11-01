use super::*;
#[test]
fn badges_follow_health_thresholds_and_real_payment_capacity() {
    assert_eq!(
        passive_status(5, 210, 0, false, true),
        Some(("伤害+25%", true))
    );
    assert_eq!(
        passive_status(5, 211, 0, false, true),
        Some(("未触发", false))
    );
    assert_eq!(
        passive_status(5, 270, 0, true, true),
        Some(("伤害+25%", true))
    );
    assert_eq!(
        passive_status(6, 101, 0, false, true),
        Some(("可代付", true))
    );
    assert_eq!(
        passive_status(6, 100, 0, false, true),
        Some(("代付不足", false))
    );
    assert_eq!(passive_status(6, 200, 80, true, true), None);
    assert_eq!(passive_status(0, 100, 0, false, true), None);
}

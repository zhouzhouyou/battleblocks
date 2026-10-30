use super::*;
#[test]
fn keywords_are_configured_and_percentage_triggers_floor_after_aggregation() {
    assert_eq!(Skill::Rush.operation(), TileOperation::Eliminate);
    assert_eq!(Skill::Meteor.operation(), TileOperation::Destroy);
    assert_eq!(Skill::Forge.operation(), TileOperation::Transform);
    let b = [[4; N]; N];
    let cells = [(0, 0), (1, 0), (2, 0)];
    assert_eq!(trigger_tiles(&b, &cells, 50, 50, 50), (4, 10, 0));
    let e = effect_at(Skill::Meteor, &mut b.clone(), (0, 0));
    assert_eq!(e.damage, 60 + 40 * 7 / 2);
    assert_eq!(e.hp, 0);
    assert!(!e.reward);
    let mut converted = b;
    let e = effect_at(Skill::Forge, &mut converted, (0, 0));
    assert_eq!((e.hp, e.damage, e.mp), (0, 0, 0));
    assert!(!e.reward);
    let drained = effect_at(Skill::Drain, &mut [[1; N]; N], (0, 0));
    assert_eq!((drained.hp, drained.damage, drained.mp), (48, 72, 0));
}

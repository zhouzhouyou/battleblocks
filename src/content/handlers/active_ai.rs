use super::*;
pub fn utility(
    skill: Skill,
    ctx: &EventContext,
    b: &Board,
    bonus: i32,
    swap_value: f32,
) -> Option<i32> {
    let (id, hp, mp, rules) = (ctx.hero, ctx.hp, ctx.mp, ctx.rules);
    match skill {
        Skill::Jump if !rules.statuses.contains(StatusId::ExtraReach) => {
            Some(swap_value as i32 + 8)
        }
        Skill::Entomb => Some(18 - skill_cost_in(id, skill, rules) / 5),
        Skill::Miracle => {
            let mut state = crate::role_state::RoleState::default();
            state.on_cast(skill, b, (0, 0));
            let prepared = Rules {
                statuses: state.statuses,
                ..rules
            };
            Some(
                crate::strategy::choose_force_move_in(b, id, hp, mp, bonus, prepared)
                    .map(|(a, c)| {
                        let mut next = *b;
                        state.complete_force_in(&mut next, a, c, id, rules.board_mechanics);
                        let next_rules = Rules {
                            statuses: state.statuses,
                            ..rules
                        };
                        crate::strategy::best_move_score_in(&next, id, hp, mp, next_rules) as i32
                    })
                    .unwrap_or(0)
                    - skill_cost_in(id, skill, rules) / 5,
            )
        }
        Skill::Frenzy | Skill::Rational => {
            let mut state = crate::role_state::RoleState::default();
            state.on_cast(skill, b, (0, 0));
            let next_rules = Rules {
                statuses: state.statuses,
                ..rules
            };
            let future = crate::strategy::best_move_score_in(b, id, hp, mp, next_rules) as i32;
            Some(future - swap_value as i32 - skill_cost_in(id, skill, rules) / 5)
        }
        _ => None,
    }
}

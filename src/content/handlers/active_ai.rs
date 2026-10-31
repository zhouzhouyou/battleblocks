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
                        state.complete_force(&mut next, a, c, id);
                        let next_rules = Rules {
                            statuses: state.statuses,
                            ..rules
                        };
                        crate::strategy::choose_move_in(&next, hp, id, mp, id, 1, next_rules)
                            .map(|(p, q)| {
                                let mut after = next;
                                crate::board::hero_swap(&mut after, p, q, id);
                                crate::strategy::score(
                                    &after,
                                    id,
                                    hp,
                                    mp,
                                    Rules {
                                        horizontal: p.1 == q.1,
                                        swap_clear: true,
                                        ..next_rules
                                    },
                                ) as i32
                            })
                            .unwrap_or(0)
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
            let future = crate::strategy::choose_move_in(b, hp, id, mp, id, 1, next_rules)
                .map(|(a, c)| {
                    let mut next = *b;
                    crate::board::hero_swap(&mut next, a, c, id);
                    crate::strategy::score(
                        &next,
                        id,
                        hp,
                        mp,
                        Rules {
                            swap_clear: true,
                            horizontal: a.1 == c.1,
                            ..next_rules
                        },
                    ) as i32
                })
                .unwrap_or(0);
            Some(future - swap_value as i32 - skill_cost_in(id, skill, rules) / 5)
        }
        _ => None,
    }
}

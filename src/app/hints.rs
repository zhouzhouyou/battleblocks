use super::*;
use std::rc::Rc;

type RankedHints = Vec<(Move, Option<content::MovePreview>)>;

#[derive(PartialEq, Eq)]
pub(super) struct HintKey {
    board: Board,
    hero: usize,
    hp: i32,
    mp: i32,
    opponent_hp: i32,
    bonus: i32,
    armed: bool,
    rules: content::Rules,
}

pub(super) struct HintCache {
    key: HintKey,
    ranked: Rc<RankedHints>,
}

impl Session {
    /// A derived, read-only cache shared by input and drawing, never saved as game state.
    pub(in crate::app) fn ranked_hints(&mut self, side: bool) -> Rc<RankedHints> {
        let (hero, hp, mp, opponent_hp) = if side {
            (self.enemy_id, self.enemy, self.enemy_mp, self.hp)
        } else {
            (self.hero_id, self.hp, self.mp, self.enemy)
        };
        let key = HintKey {
            board: self.b,
            hero,
            hp,
            mp,
            opponent_hp,
            bonus: self.extra_bonus,
            armed: self.armed,
            rules: self.rules(side),
        };
        if let Some(cached) = self.hint_cache.as_ref().filter(|cached| cached.key == key) {
            return Rc::clone(&cached.ranked);
        }
        let ranked = Rc::new(content::ranked_moves_in(
            &key.board,
            key.hero,
            key.hp,
            key.mp,
            key.opponent_hp,
            key.bonus,
            key.armed,
            key.rules,
        ));
        self.hint_cache = Some(HintCache {
            key,
            ranked: Rc::clone(&ranked),
        });
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_reuse_only_identical_state_and_never_change_rules_or_rng() {
        let mut session = Session::new_headless();
        let before = session.snapshot();
        let first = session.ranked_hints(false);
        assert!(Rc::ptr_eq(&first, &session.ranked_hints(false)));
        assert_eq!(rng::state(), before.rng_state);
        assert_eq!(session.b, before.board);
        session.mp += 1;
        let changed = session.ranked_hints(false);
        assert!(!Rc::ptr_eq(&first, &changed));
        session
            .board_mechanics
            .set_gravity(board::Gravity::Left, false);
        let gravity = session.ranked_hints(false);
        assert!(!Rc::ptr_eq(&changed, &gravity));
        session.new_role_states[0].statuses.insert(
            content::StatusId::ExtraReach,
            content::Skill::Jump,
            content::StatusDuration::OwnerTurnEnd(1),
        );
        assert!(!Rc::ptr_eq(&gravity, &session.ranked_hints(false)));
        session.restore(before);
        assert!(session.hint_cache.is_none());
        assert!(!Rc::ptr_eq(&first, &session.ranked_hints(false)));
    }
}

use super::planning::{plan_key, AiPlan};
#[cfg(test)]
use super::planning::{PlanAction, PlanStep, SearchPolicy};

impl Session {
    pub(super) fn drive_hint_queue(&mut self, frame: &mut Frame) {
        if self.hint_queue.is_empty() {
            return;
        }
        if frame.key(KeyCode::Escape)
            || frame.clicked(MouseButton::Right)
            || frame.clicked(MouseButton::Left)
            || [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Space]
                .iter()
                .any(|&key| frame.key(key))
            || frame.command_skill.is_some()
            || frame.command_move.is_some()
            || self.enemy_turn
            || self.action_done
            || self.hp == 0
            || self.enemy == 0
        {
            self.hint_queue.clear();
            return;
        }
        if !self.pending.is_empty() || self.presentation.busy() || self.charge_damage.is_some() {
            return;
        }
        let step = self.hint_queue.pop_front().unwrap();
        if step.before.as_ref() != plan_key(self.snapshot()) {
            self.hint_queue.clear();
            self.feedback = tr_key(self.zh, "ui.hints.plans.position_changed").to_string();
            self.feedback_time = 2.5;
            return;
        }
        step.action.inject(self, frame);
    }
}

pub(super) fn plan_pages(plans: &[AiPlan]) -> Vec<Vec<(usize, Rect)>> {
    let mut pages = vec![Vec::new()];
    let mut y = 266.;
    for (i, plan) in plans.iter().enumerate() {
        let height = 66. + plan.steps.len() as f32 * 42.;
        if y + height > 660. && !pages.last().unwrap().is_empty() {
            pages.push(Vec::new());
            y = 266.;
        }
        pages
            .last_mut()
            .unwrap()
            .push((i, Rect::new(584., y, 416., height)));
        y += height + 8.;
    }
    pages
}

pub(super) fn plan_row_at(plans: &[AiPlan], page: usize, point: Vec2) -> Option<usize> {
    plan_pages(plans)
        .get(page)?
        .iter()
        .find(|(_, r)| r.contains(point))
        .map(|(i, _)| *i)
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    fn position(hero: usize, mana: i32) -> Session {
        rng::srand(7123);
        let mut s = Session::new_headless();
        s.prepare_headless_batch();
        s.hero_id = hero;
        s.enemy_id = 1;
        s.hp = content::health_cap(hero, false);
        s.enemy = content::health_cap(1, false);
        s.mp = mana;
        s.new_role_states[0].turn_start_done = true;
        s
    }

    #[test]
    fn plans_are_ranked_cached_and_leave_live_model_and_rng_untouched() {
        let mut s = position(content::PRIEST, 100);
        s.new_role_states[0].pockets = [Some(2), Some(1)];
        let before = s.snapshot().json();
        let plans = s.ranked_ai_plans(false);
        assert!(!plans.is_empty());
        assert_eq!(s.snapshot().json(), before);
        assert!(Rc::ptr_eq(&plans, &s.ranked_ai_plans(false)));
        s.deadline += 0.25;
        s.enemy_clock += 0.25;
        assert!(Rc::ptr_eq(&plans, &s.ranked_ai_plans(false)));
        s.cooldowns.cast(0, content::Skill::Entomb);
        let cooldown_plans = s.ranked_ai_plans(false);
        assert!(!Rc::ptr_eq(&plans, &cooldown_plans));
        assert!(cooldown_plans.iter().all(|p| p
            .steps
            .iter()
            .all(|step| !matches!(step.action, PlanAction::Cast(content::Skill::Entomb, _, _)))));
        assert!(plans.windows(2).all(|p| p[0].score >= p[1].score));
        assert!(plans.iter().any(|p| p.steps.len() > 1));
        s.new_role_states[0].pockets[0] = Some(4);
        assert!(!Rc::ptr_eq(&plans, &s.ranked_ai_plans(false)));
        s.restore(s.snapshot());
        assert!(s.plan_cache.is_none());
    }

    #[test]
    fn queued_plan_runs_through_real_frames_and_matches_every_forecast() {
        let mut s = position(content::PRIEST, 100);
        s.new_role_states[0].pockets = [Some(2), Some(1)];
        let plans = s.ranked_ai_plans(false);
        let plan = plans.iter().find(|p| p.steps.len() > 1).unwrap();
        s.hint_queue = plan.steps.clone().into();
        let mut executed = 0;
        for _ in 0..4000 {
            let mut frame = Frame::headless(1. / 60.);
            s.tick(&mut frame);
            if frame.command_skill.is_some() || frame.command_move.is_some() {
                executed += 1;
            }
            if s.hint_queue.is_empty()
                && s.pending.is_empty()
                && !s.presentation.busy()
                && s.charge_damage.is_none()
            {
                break;
            }
        }
        assert_eq!(executed, plan.steps.len(), "{}", s.feedback);
        let last = plan.steps.last().unwrap();
        assert_eq!((s.hp, s.mp, s.enemy), (last.hp, last.mp, last.enemy_hp));
    }

    #[test]
    fn stale_queue_stops_before_spending_or_moving() {
        let mut s = position(0, 0);
        let plan = s.ranked_ai_plans(false)[0].clone();
        s.hint_queue = plan.steps.into();
        s.mp += 1;
        let before = s.snapshot().json();
        let mut frame = Frame::headless(1. / 60.);
        s.drive_hint_queue(&mut frame);
        assert!(s.hint_queue.is_empty());
        assert!(frame.command_skill.is_none() && frame.command_move.is_none());
        assert_eq!(before, s.snapshot().json());
    }

    #[test]
    fn no_mana_plans_use_swaps_and_repeat_casts_use_actual_increasing_costs() {
        let mut s = position(content::PRIEST, 0);
        assert!(s.ranked_ai_plans(false).iter().all(|p| p
            .steps
            .iter()
            .all(|s| matches!(s.action, PlanAction::Swap(_)))));
        s.mp = 200;
        s.new_role_states[0].pockets = [Some(2), Some(1)];
        let a = PlanAction::Cast(content::Skill::Resurrect, (0, 0), None);
        let first = s.simulate_plan_step(a).unwrap();
        let second = s.simulate_plan_step(a).unwrap();
        assert_eq!((first.mana_cost, second.mana_cost), (25, 40));
    }

    #[test]
    fn every_plan_card_and_step_fits_inside_its_page_and_hits_the_drawn_card() {
        let plans: Vec<_> = (1..=SearchPolicy::hints().depth)
            .map(|n| AiPlan {
                steps: (0..n)
                    .map(|_| PlanStep {
                        action: PlanAction::Swap(((0, 0), (1, 0))),
                        before: String::new().into(),
                        name: String::new(),
                        mana_cost: 0,
                        blood_cost: 0,
                        hp: 1,
                        mp: 0,
                        enemy_hp: 1,
                    })
                    .collect(),
                score: 0.,
                damage: 0,
                end_state: None,
            })
            .collect();
        for (page, rows) in plan_pages(&plans).iter().enumerate() {
            for &(i, r) in rows {
                assert!(r.y >= 266. && r.y + r.h <= 660.);
                assert!(r.x >= 574. && r.x + r.w <= 1010.);
                assert_eq!(plan_row_at(&plans, page, vec2(r.x + 1., r.y + 1.)), Some(i));
                let final_detail_y = r.y + 40. + (plans[i].steps.len() - 1) as f32 * 42. + 17.;
                assert!(final_detail_y < r.y + r.h - 20.);
            }
        }
    }
}

#[cfg(test)]
mod plan_regression_tests {
    use super::*;

    #[test]
    fn all_rosters_can_forecast_and_replay_without_changing_live_randomness() {
        for hero in 0..HEROES.len() {
            rng::srand(173 + hero as u64);
            let mut s = Session::new_simulation();
            s.prepare_headless_batch();
            s.hero_id = hero;
            s.enemy_id = 1;
            s.hp = content::health_cap(hero, false) - 80;
            s.enemy = 500;
            s.mp = 100;
            s.new_role_states[0].turn_start_done = true;
            s.new_role_states[0].pockets = [Some(2), Some(1)];
            let original = s.snapshot();
            let before = original.json();
            let plans = s.ranked_ai_plans(false);
            assert_eq!(s.snapshot().json(), before, "hero {hero}");
            assert!(!plans.is_empty(), "hero {hero}");
            for plan in plans.iter().take(6) {
                s.restore(original.clone());
                s.hint_queue = plan.steps.clone().into();
                let mut executed = 0;
                for _ in 0..4000 {
                    let mut frame = Frame::headless(1. / 60.);
                    s.tick(&mut frame);
                    if frame.command_move.is_some() || frame.command_skill.is_some() {
                        executed += 1;
                    }
                    if s.hint_queue.is_empty()
                        && s.pending.is_empty()
                        && !s.presentation.busy()
                        && s.charge_damage.is_none()
                    {
                        break;
                    }
                }
                assert_eq!(executed, plan.steps.len(), "hero {hero}: {}", s.feedback);
                let last = plan.steps.last().unwrap();
                assert_eq!(
                    (s.hp, s.mp, s.enemy),
                    (last.hp, last.mp, last.enemy_hp),
                    "hero {hero}"
                );
            }
        }
    }

    #[test]
    fn resurrection_can_precede_entomb_but_turn_ending_skills_stop_the_sequence() {
        rng::srand(7123);
        let mut s = Session::new_simulation();
        s.prepare_headless_batch();
        s.hero_id = content::PRIEST;
        s.enemy_id = 1;
        s.hp = 500;
        s.mp = 100;
        s.new_role_states[0].turn_start_done = true;
        s.new_role_states[0].pockets = [Some(2), Some(1)];
        let before = s.snapshot();
        assert!(s
            .simulate_plan_step(PlanAction::Cast(content::Skill::Resurrect, (0, 0), None))
            .is_some());
        assert!(!s.action_done);
        assert!(s
            .simulate_plan_step(PlanAction::Cast(content::Skill::Entomb, (2, 2), None))
            .is_some());
        assert!(s.action_done);
        assert!(s
            .simulate_plan_step(PlanAction::Swap(((0, 0), (1, 0))))
            .is_none());
        s.restore(before);
        assert!(s.hint_queue.is_empty());
    }
}

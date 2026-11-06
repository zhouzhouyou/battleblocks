use super::*;
use std::rc::Rc;

// Search is bounded to keep the interactive hint responsive. Every branch uses
// the canonical Session actions and settlement, including repeat costs and RNG.
pub(super) const PLAN_DEPTH: usize = 6;
const PLAN_BEAM: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlanAction {
    Swap(Move),
    Cast(content::Skill, (usize, usize), Option<(usize, usize)>),
}

#[derive(Clone)]
pub(super) struct PlanStep {
    pub action: PlanAction,
    pub before: Rc<str>,
    pub mana_cost: i32,
    pub blood_cost: i32,
    pub hp: i32,
    pub mp: i32,
    pub enemy_hp: i32,
    pub name: String,
}

#[derive(Clone)]
pub(super) struct AiPlan {
    pub steps: Vec<PlanStep>,
    pub score: f32,
    pub damage: i32,
}

pub(super) struct AiPlanCache {
    key: String,
    zh: bool,
    pub plans: Rc<Vec<AiPlan>>,
}

/// Ignore pacing and display history; retain every rule input and the random stream.
pub(super) fn plan_key(mut state: debug::Snapshot) -> String {
    state.deadline = 0.;
    state.enemy_clock = 0.;
    state.charge_remaining = 0.;
    format!("{}:{:?}", state.json(), state.new_role_states)
}

impl PlanAction {
    pub(super) fn inject(self, session: &mut Session, frame: &mut Frame) {
        match self {
            Self::Swap(m) => frame.command_move = Some(m),
            Self::Cast(skill, target, anchor) => {
                session.target_anchor = anchor;
                frame.command_skill = Some(skill);
                frame.command_target = Some(target);
            }
        }
    }
}

impl Session {
    pub(super) fn plan_candidates(&mut self) -> Vec<PlanAction> {
        let rules = self.rules(false);
        if self.new_role_states[0].force_armed {
            return strategy::choose_force_move_in(
                &self.b,
                self.hero_id,
                self.hp,
                self.mp,
                self.extra_bonus,
                rules,
            )
            .map(|m| vec![PlanAction::Swap(m)])
            .unwrap_or_default();
        }
        let mut actions: Vec<_> = if self.armed {
            content::ranked_moves_in(
                &self.b,
                self.hero_id,
                self.hp,
                self.mp,
                self.enemy,
                self.extra_bonus,
                true,
                rules,
            )
            .into_iter()
            .map(|(m, _)| PlanAction::Swap(m))
            .collect()
        } else {
            strategy::ranked_move_options_in(
                &self.b,
                self.hp,
                self.hero_id,
                self.mp,
                self.enemy_id,
                self.difficulty.max(1),
                rules,
            )
            .into_iter()
            .map(|(m, _)| PlanAction::Swap(m))
            .collect()
        };
        for &skill in content::skills(self.hero_id, self.awakened[0]) {
            if self.cooldowns.remaining(0, skill) > 0
                || !content::preparation_available(skill, rules, self.armed, self.extra_bonus)
                || content::payment_in(self.hero_id, self.hp, self.mp, skill, rules).is_none()
            {
                continue;
            }
            let paired = content::skill_profile(skill, rules).paired_target;
            let anchors: Vec<_> = if paired {
                (0..N)
                    .flat_map(|y| (0..N).map(move |x| Some((x, y))))
                    .filter(|a| {
                        board::PortalEdge::at(a.unwrap(), self.board_mechanics.gravity())
                            .cells()
                            .is_some()
                    })
                    .collect()
            } else {
                vec![None]
            };
            let mut targets = Vec::new();
            for anchor in anchors {
                let rules = content::Rules {
                    target_anchor: anchor,
                    ..rules
                };
                for y in 0..N {
                    for x in 0..N {
                        let target = (x, y);
                        if !content::legal_target(
                            self.hero_id,
                            self.hp,
                            skill,
                            &self.b,
                            target,
                            rules,
                        ) {
                            continue;
                        }
                        let p = content::preview_rewards_in(
                            skill,
                            &self.b,
                            target,
                            self.hero_id,
                            self.hp,
                            self.mp,
                            self.enemy,
                            self.extra_bonus,
                            rules,
                        );
                        let value = p.damage as f32
                            + p.heal as f32 * 0.8
                            + (p.mana_after - self.mp) as f32 * 0.65;
                        targets.push((value, PlanAction::Cast(skill, target, anchor)));
                    }
                }
            }
            targets.sort_by(|a, b| b.0.total_cmp(&a.0));
            // Non-targeted skills have identical target choices.
            let count = if content::targeted_in(skill, rules) {
                2
            } else {
                1
            };
            actions.extend(targets.into_iter().take(count).map(|(_, a)| a));
        }
        actions
    }

    pub(super) fn simulate_plan_step(&mut self, action: PlanAction) -> Option<PlanStep> {
        if self.action_done || self.enemy_turn || self.hp == 0 || self.enemy == 0 {
            return None;
        }
        let before = self.snapshot();
        let payment = match action {
            PlanAction::Cast(s, _, _) => {
                content::payment_in(self.hero_id, self.hp, self.mp, s, self.rules(false))
            }
            _ => None,
        };
        let name = match action {
            PlanAction::Cast(s, _, _) => content::skill_name_in(s, self.rules(false), self.zh),
            _ => String::new(),
        };
        let mut frame = Frame::headless(1.);
        action.inject(self, &mut frame);
        let accepted = match action {
            PlanAction::Cast(s, _, _) => self.cast_skill(&frame, Some(s), false),
            PlanAction::Swap(_) => {
                let count = self.turn_log.current.actions.len();
                self.player_swap(&frame, None, false);
                self.turn_log.current.actions.len() > count
            }
        };
        if !accepted {
            return None;
        }
        for _ in 0..128 {
            if self.hp == 0 || self.enemy == 0 {
                break;
            }
            self.presentation.tick(10.);
            if self.charge_damage.is_some() && !self.presentation.busy() {
                // Settle delayed impact without crossing the turn boundary.
                let done = self.action_done;
                self.action_done = false;
                self.advance_turn(&frame);
                self.action_done |= done;
            }
            if self.hp == 0 || self.enemy == 0 {
                break;
            }
            if !self.pending.is_empty() {
                self.resolve_pending(10.);
            }
            if self.pending.is_empty() && self.charge_damage.is_none() && !self.presentation.busy()
            {
                break;
            }
        }
        if self.hp > 0
            && self.enemy > 0
            && (!self.pending.is_empty()
                || self.charge_damage.is_some()
                || self.presentation.busy())
        {
            return None;
        }
        Some(PlanStep {
            action,
            before: plan_key(before).into(),
            name,
            mana_cost: payment.map_or(0, |p| p.mana),
            blood_cost: payment.map_or(0, |p| p.blood),
            hp: self.hp,
            mp: self.mp,
            enemy_hp: self.enemy,
        })
    }

    pub(super) fn ranked_ai_plans(&mut self, side: bool) -> Rc<Vec<AiPlan>> {
        let initial = self.ai_snapshot(side);
        let key = plan_key(initial.clone());
        if let Some(cache) = self
            .plan_cache
            .as_ref()
            .filter(|c| c.key == key && c.zh == self.zh)
        {
            return Rc::clone(&cache.plans);
        }
        struct RestoreRng(u64);
        impl Drop for RestoreRng {
            fn drop(&mut self) {
                rng::restore(self.0);
            }
        }
        let _rng = RestoreRng(rng::state());
        let mut model = Session::new_simulation();
        model.prepare_headless_batch();
        model.zh = self.zh;
        let mut frontier = vec![(
            initial.clone(),
            AiPlan {
                steps: vec![],
                score: 0.,
                damage: 0,
            },
        )];
        let mut finished = Vec::new();
        for depth in 0..PLAN_DEPTH {
            let mut next = Vec::new();
            for (state, plan) in frontier {
                model.restore(state.clone());
                let mut swaps = 0;
                for action in model.plan_candidates().into_iter().filter(|a| {
                    if matches!(a, PlanAction::Swap(_)) {
                        swaps += 1;
                        depth == 0 || swaps <= 5
                    } else {
                        true
                    }
                }) {
                    model.restore(state.clone());
                    if let Some(step) = model.simulate_plan_step(action) {
                        let mut candidate = plan.clone();
                        candidate.steps.push(step);
                        candidate.damage =
                            model.turn_log.current.damage - initial.turns.current.damage;
                        candidate.score = strategy::reward_value(
                            candidate.damage,
                            model.hp - initial.hp,
                            model.mp - initial.mp,
                        ) - (model.enemy - initial.enemy_hp + candidate.damage)
                            .max(0) as f32;
                        if self.difficulty == 2
                            && model.enemy > 0
                            && model.hp > 0
                            && model.action_done
                        {
                            candidate.score -= strategy::best_move_score_in(
                                &model.b,
                                model.enemy_id,
                                model.enemy,
                                model.enemy_mp,
                                model.rules(true),
                            ) * 0.35;
                        }
                        if model.enemy == 0 {
                            candidate.score += 10000.;
                        }
                        if model.hp == 0 {
                            candidate.score -= 10000.;
                        }
                        if model.action_done || model.hp == 0 || model.enemy == 0 {
                            finished.push(candidate);
                        } else {
                            next.push((model.snapshot(), candidate));
                        }
                    }
                }
            }
            next.sort_by(|a, b| b.1.score.total_cmp(&a.1.score));
            next.truncate(PLAN_BEAM);
            frontier = next;
        }
        finished.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then(a.steps.len().cmp(&b.steps.len()))
        });
        let plans = Rc::new(finished);
        self.plan_cache = Some(AiPlanCache {
            key,
            zh: self.zh,
            plans: Rc::clone(&plans),
        });
        plans
    }
}

impl Session {
    /// AI and hints read the exact same ranked alternatives. Both native AI and
    /// unattended player AI execute the first action of the highest ranked plan.
    pub(super) fn best_ai_action(&mut self, side: bool) -> Option<PlanAction> {
        self.ranked_ai_plans(side)
            .first()?
            .steps
            .first()
            .map(|s| s.action)
    }

    fn ai_snapshot(&self, side: bool) -> debug::Snapshot {
        let mut state = self.snapshot();
        if side {
            std::mem::swap(&mut state.hero, &mut state.enemy);
            std::mem::swap(&mut state.hp, &mut state.enemy_hp);
            std::mem::swap(&mut state.mp, &mut state.enemy_mp);
            state.awakened.swap(0, 1);
            state.cooldowns.0.swap(0, 1);
            state.new_role_states.swap(0, 1);
            state.bonuses.swap(0, 1);
            state.armed_sides.swap(0, 1);
            state.board_mechanics.portals.swap(0, 1);
            for pair in state.board_mechanics.portals.iter_mut().flatten() {
                pair.owner = !pair.owner;
            }
            if let Some(effect) = &mut state.board_mechanics.gravity_effect {
                effect.owner = !effect.owner;
            }
            state.enemy_turn = false;
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_ai_and_player_hints_choose_the_same_ranked_action_for_mirrored_positions() {
        for hero in 0..HEROES.len() {
            rng::srand(173 + hero as u64);
            let mut player = Session::new_simulation();
            player.prepare_headless_batch();
            player.hero_id = hero;
            player.enemy_id = 1;
            player.hp = content::health_cap(hero, false) - 80;
            player.enemy = 500;
            player.mp = 100;
            player.difficulty = 1;
            player.new_role_states[0].turn_start_done = true;
            player.new_role_states[0].pockets = [Some(2), Some(1)];
            let plans = player.ranked_ai_plans(false);
            let first = plans[0].steps[0].clone();
            assert_eq!(player.best_ai_action(false), Some(first.action));
            let mut mirrored = player.ai_snapshot(true);
            mirrored.enemy_turn = true;
            mirrored.enemy_clock = 1.1;
            let mut enemy = Session::new_simulation();
            enemy.prepare_headless_batch();
            enemy.restore(mirrored);
            assert_eq!(
                enemy.best_ai_action(true),
                Some(first.action),
                "hero {hero}"
            );
            let action_count = enemy.turn_log.current.actions.len();
            for _ in 0..4000 {
                let mut frame = Frame::headless(1. / 60.);
                enemy.tick(&mut frame);
                if enemy.hp == 0
                    || enemy.enemy == 0
                    || (enemy.turn_log.current.actions.len() > action_count
                        && enemy.pending.is_empty()
                        && enemy.charge_damage.is_none()
                        && !enemy.presentation.busy())
                {
                    break;
                }
            }
            assert_eq!(
                (enemy.enemy, enemy.enemy_mp, enemy.hp),
                (first.hp, first.mp, first.enemy_hp),
                "hero {hero}"
            );
        }
    }

    #[test]
    fn lethal_cast_stops_forecast_at_the_same_point_as_the_model() {
        let mut s = Session::new_simulation();
        s.prepare_headless_batch();
        s.hero_id = content::PRIEST;
        s.enemy = 1;
        s.awakened[1] = true;
        s.mp = 100;
        s.new_role_states[0].pockets = [Some(2), Some(1)];
        let before = s.snapshot();
        let action = PlanAction::Cast(content::Skill::Resurrect, (0, 0), None);
        let predicted = s.simulate_plan_step(action).unwrap();
        s.restore(before);
        let mut frame = Frame::headless(1. / 60.);
        action.inject(&mut s, &mut frame);
        s.tick(&mut frame);
        assert_eq!(
            (s.hp, s.mp, s.enemy),
            (predicted.hp, predicted.mp, predicted.enemy_hp)
        );
        assert_eq!(s.enemy, 0);
    }
}

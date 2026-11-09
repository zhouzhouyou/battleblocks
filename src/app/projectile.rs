use super::*;
use content::projectile::{Aim, Ammo, Flight, Point, ProjectileConfig, ORIGIN};
const SAMPLES: usize = 12;
pub(super) struct PreviewCache {
    key: String,
    totals: [f64; 3],
    complete: usize,
}
fn screen(point: Point) -> Vec2 {
    vec2(256. + point.x as f32 * 64., 110. + point.y as f32 * 64.)
}
pub(super) fn origin_screen() -> Vec2 {
    screen(ORIGIN)
}
fn ammo_rect(index: usize) -> Rect {
    Rect::new(sidebar_x(true, 800. + index as f32 * 102.), 246., 98., 36.)
}
fn draw_sling(
    skill: content::Skill,
    pull: Vec2,
    zh: bool,
    explosive: bool,
    loaded: bool,
    origin: Point,
) {
    let mut area = Rect::new(side_panel_rect(false).right() + 8., 490., 84., 202.);
    if origin.x > N as f64 / 2. {
        area.x = 1024. - area.right();
    }
    draw_rectangle(
        area.x,
        area.y,
        area.w,
        area.h,
        Color::new(0.08, 0.1, 0.14, 0.9),
    );
    outline(
        area.x,
        area.y,
        area.w,
        area.h,
        1.,
        Color::new(0.3, 0.35, 0.4, 0.7),
    );
    let text = tr_key(zh, "projectile.lane");
    label_centered(
        text,
        area.x + area.w / 2.,
        area.bottom() - 10.,
        fit_size(text, 14., area.w - 12.),
        GRAY,
    );
    let origin = screen(origin);
    crate::slingshot_art::draw(
        skill,
        Rect::new(origin.x - 26., origin.y - 9.36, 52., 52.),
        pull,
    );
    if loaded {
        crate::slingshot_art::draw_ammo(origin - pull, 4., explosive);
    }
}

fn draw_path(flight: &Flight, time: f64, color: Color) {
    // Sample the analytical flight at subpixel spacing, then place dots by arc length.
    let steps = (time * 600.).ceil().max(1.) as usize;
    let mut previous = screen(flight.position(0.));
    let mut distance = 0.;
    let mut next_dot = 0.;
    for i in 1..=steps {
        let at = screen(flight.position(time * i as f64 / steps as f64));
        let segment = previous.distance(at);
        while next_dot <= distance + segment && segment > 0. {
            let dot = previous.lerp(at, (next_dot - distance) / segment).round();
            draw_rectangle(
                dot.x - 2.,
                dot.y - 2.,
                5.,
                5.,
                Color::new(0.04, 0.07, 0.1, 0.8),
            );
            draw_rectangle(dot.x - 1., dot.y - 1., 3., 3., color);
            next_dot += 12.;
        }
        distance += segment;
        previous = at;
    }
}

pub(super) fn aftermath(flight: &Flight) -> f64 {
    if flight.explosion_radius.is_some() {
        effects::PROJECTILE_BLAST_DURATION as f64
    } else {
        0.3
    }
}

impl Session {
    pub(super) fn aim_projectile(&mut self, frame: &Frame, config: ProjectileConfig) {
        if !frame.interactive || frame.block_input {
            return;
        }
        if config.explosion.is_some() {
            let mut aim = self.projectile_aim.unwrap_or_default();
            for (index, ammo) in [Ammo::Solid, Ammo::Explosive].into_iter().enumerate() {
                if frame.clicked(MouseButton::Left) && ammo_rect(index).contains(frame.pointer) {
                    aim.ammo = ammo;
                }
            }
            if frame.key(KeyCode::Tab) {
                aim.ammo = match aim.ammo {
                    Ammo::Solid => Ammo::Explosive,
                    Ammo::Explosive => Ammo::Solid,
                };
            }
            self.projectile_aim = Some(aim);
        }
        let origin = screen(ORIGIN);
        if frame.clicked(MouseButton::Left) && frame.pointer.distance(origin) < 42. {
            self.projectile_drag = true;
        }
        if self.projectile_drag {
            let pull = origin - frame.pointer;
            let angle = (-pull.y).atan2(pull.x).to_degrees();
            self.projectile_aim = Some(config.clamp_aim(Aim {
                ammo: self.projectile_aim.unwrap_or_default().ammo,
                angle_mdeg: (angle * 1000.).round() as i32,
                power_permille: (pull.length() / 70. * 1000.).round().clamp(0., 1000.) as u16,
                ..Default::default()
            }));
            if !is_mouse_button_down(MouseButton::Left) {
                self.projectile_drag = false;
            }
        }
        let mut aim = self.projectile_aim.unwrap_or_default();
        if frame.key(KeyCode::Up) {
            aim.angle_mdeg += 1000;
        }
        if frame.key(KeyCode::Down) {
            aim.angle_mdeg -= 1000;
        }
        if frame.key(KeyCode::Right) {
            aim.power_permille = aim.power_permille.saturating_add(25);
        }
        if frame.key(KeyCode::Left) {
            aim.power_permille = aim.power_permille.saturating_sub(25);
        }
        self.projectile_aim = Some(config.clamp_aim(aim));
    }
    pub(super) fn projectile_estimate(&mut self, skill: content::Skill, aim: Aim) -> &PreviewCache {
        let key = format!("{}:{skill:?}:{aim:?}", planning::plan_key(self.snapshot()));
        if self
            .projectile_preview
            .as_ref()
            .is_none_or(|cache| cache.key != key)
        {
            struct RestoreRng(u64);
            impl Drop for RestoreRng {
                fn drop(&mut self) {
                    rng::restore(self.0);
                }
            }
            let _guard = RestoreRng(rng::state());
            let snapshot = self.snapshot();
            let mut model = Session::new_simulation();
            model.prepare_headless_batch();
            let mut totals = [0.; 3];
            let mut complete = 0;
            for sample in 0..SAMPLES {
                model.restore(snapshot.clone());
                // Independent fixed streams keep dragging stable without revealing live RNG.
                rng::srand(0x51a97501 + sample as u64 * 7919);
                model.projectile_aim = Some(aim);
                if let Some(result) =
                    model.simulate_plan_step(planning::PlanAction::Cast(skill, (3, 3), None))
                {
                    totals[0] += (result.hp - self.hp) as f64;
                    totals[1] +=
                        (model.turn_log.current.damage - snapshot.turns.current.damage) as f64;
                    totals[2] += (result.mp - self.mp) as f64;
                    complete += 1;
                }
            }
            if complete > 0 {
                for value in &mut totals {
                    *value /= complete as f64;
                }
            }
            self.projectile_preview = Some(PreviewCache {
                key,
                totals,
                complete,
            });
        }
        self.projectile_preview.as_ref().unwrap()
    }
    #[cfg(test)]
    pub(super) fn projectile_estimate_for_test(&mut self, skill: content::Skill, aim: Aim) {
        self.projectile_estimate(skill, aim);
    }
    pub(super) fn draw_projectile(&mut self, _frame: &Frame) {
        if let Some(impact) = &self.charge_damage {
            if let Some(flight) = &impact.projectile {
                let elapsed =
                    (flight.duration + aftermath(flight) - self.presentation.remaining()).max(0.);
                let recoil = (elapsed as f32 * 35.).cos() * (-elapsed as f32 * 12.).exp() * 20.;
                draw_sling(
                    impact.skill,
                    vec2(flight.velocity.x as f32, flight.velocity.y as f32).normalize() * recoil,
                    self.zh,
                    flight.ammo == Ammo::Explosive,
                    false,
                    flight.origin,
                );
                draw_path(flight, elapsed.min(flight.duration), ORANGE);
                let p = screen(flight.position(elapsed.min(flight.duration)));
                if elapsed < flight.duration {
                    for i in 1..=8 {
                        let tail =
                            screen(flight.position((elapsed - i as f64 * 0.015).max(0.))).round();
                        let size = (9 - i) as f32 * 0.8;
                        draw_rectangle(
                            tail.x - size / 2.,
                            tail.y - size / 2.,
                            size,
                            size,
                            Color::new(1., 0.65, 0.15, (9 - i) as f32 / 12.),
                        );
                    }
                }
                if elapsed < flight.duration {
                    let radius =
                        content::skill_profile(impact.skill, self.rules(impact.actor_enemy))
                            .projectile
                            .map(|c| c.radius as f32 * 64.)
                            .unwrap_or(7.);
                    crate::slingshot_art::draw_ammo(p, radius, flight.ammo == Ammo::Explosive);
                }
                let count = flight.hits.iter().filter(|h| h.time <= elapsed).count();
                let text =
                    crate::format_text(self.zh, "projectile.flight_count", &[&count.to_string()]);
                label_centered(&text, 512., 100., 16., GOLD);
                for hit in flight.hits.iter().filter(|h| h.time <= elapsed) {
                    let age = (elapsed - hit.time) as f32;
                    if age < 0.3 {
                        let c = screen(hit.position);
                        draw_circle_lines(c.x, c.y, 8. + age * 40., 2., ORANGE);
                    }
                }
                if elapsed >= flight.duration {
                    if let Some(radius) = flight.explosion_radius {
                        if let Some(effects) = &self.skill_effects {
                            effects.draw_projectile_blast(
                                p,
                                radius as f32 * 64.,
                                (elapsed - flight.duration) as f32,
                            );
                        }
                    }
                }
                return;
            }
        }
        let Some(skill) = self.targeting else {
            return;
        };
        let rules = self.rules(false);
        let Some(config) = content::skill_profile(skill, rules).projectile else {
            return;
        };
        let aim = config.clamp_aim(self.projectile_aim.unwrap_or_default());
        let flight = config.solve(&self.b, aim);
        draw_path(&flight, flight.duration, SKYBLUE);
        for hit in &flight.hits {
            let (x, y) = hit.cell;
            outline(
                256. + x as f32 * 64. + 3.,
                110. + y as f32 * 64. + 3.,
                58.,
                58.,
                3.,
                SKYBLUE,
            );
        }
        for &(x, y) in &flight.explosion_cells {
            outline(
                256. + x as f32 * 64. + 5.,
                110. + y as f32 * 64. + 5.,
                54.,
                54.,
                3.,
                ORANGE,
            );
        }
        let end = screen(flight.position(flight.duration));
        draw_circle(end.x, end.y, 5., SKYBLUE);
        if let Some(radius) = flight.explosion_radius {
            draw_circle_lines(end.x, end.y, radius as f32 * 64., 2., ORANGE);
        }
        let angle = (aim.angle_mdeg as f32 / 1000.).to_radians();
        let pull = vec2(angle.cos(), -angle.sin()) * aim.power_permille as f32 / 1000. * 35.;
        draw_sling(
            skill,
            pull,
            self.zh,
            flight.ammo == Ammo::Explosive,
            true,
            flight.origin,
        );
        let exact = content::preview_rewards_in(
            skill,
            &self.b,
            (3, 3),
            self.hero_id,
            self.hp,
            self.mp,
            self.enemy,
            self.extra_bonus,
            content::Rules {
                projectile_aim: Some(aim),
                ..rules
            },
        );
        let cost = content::skill_cost_in(self.hero_id, skill, rules);
        let totals = self.projectile_estimate(skill, aim).totals;
        let complete = self.projectile_preview.as_ref().unwrap().complete;
        flush_labels();
        draw_rectangle(
            sidebar_x(true, 790.),
            208.,
            220.,
            438.,
            Color::new(0.04, 0.07, 0.12, 0.98),
        );
        outline(sidebar_x(true, 790.), 208., 220., 438., 2., SKYBLUE);
        let name = content::skill_name_in(skill, rules, self.zh);
        label(
            &name,
            sidebar_x(true, 800.),
            237.,
            fit_size(&name, 21., 200.),
            GOLD,
        );
        if config.explosion.is_some() {
            for (index, ammo) in [Ammo::Solid, Ammo::Explosive].into_iter().enumerate() {
                let r = ammo_rect(index);
                draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(35, 45, 57, 255));
                outline(
                    r.x,
                    r.y,
                    r.w,
                    r.h,
                    2.,
                    if ammo == aim.ammo { GOLD } else { GRAY },
                );
                label_centered(
                    ammo.name(self.zh),
                    r.center().x,
                    r.y + 24.,
                    fit_size(ammo.name(self.zh), 16., r.w - 8.),
                    WHITE,
                );
            }
        }
        let random = config.direct_reward.probability.iter().any(|p| *p < 100)
            || config
                .explosion
                .is_some_and(|e| e.reward.probability.iter().any(|p| *p < 100));
        let end_key = match flight.end {
            content::projectile::End::Landing => "projectile.end.landing",
            content::projectile::End::OutOfBounds => "projectile.end.outside",
            content::projectile::End::Fuse => "projectile.end.fuse",
            content::projectile::End::Penetrations => "projectile.end.penetrations",
        };
        let rows = [
            (
                "projectile.end",
                vec![
                    format!("{:.2}", flight.duration),
                    tr_key(self.zh, end_key).to_string(),
                ],
            ),
            (
                "projectile.aim",
                vec![
                    format!("{:.1}", aim.angle_mdeg as f64 / 1000.),
                    format!("{:.0}", aim.power_permille as f64 / 10.),
                ],
            ),
            (
                "projectile.cells",
                vec![
                    flight.hits.len().to_string(),
                    flight.explosion_cells.len().to_string(),
                    flight.cells().len().to_string(),
                ],
            ),
            ("projectile.cost", vec![cost.to_string()]),
            (
                if random {
                    "projectile.direct_expected"
                } else {
                    "projectile.direct"
                },
                vec![
                    exact.heal.to_string(),
                    exact.damage.to_string(),
                    exact.mana_gain.to_string(),
                ],
            ),
            (
                if random {
                    "projectile.net_expected"
                } else {
                    "projectile.net"
                },
                vec![(exact.mana_after - self.mp).to_string()],
            ),
            ("projectile.sample", vec![complete.to_string()]),
            (
                "projectile.total",
                vec![
                    format!("{:+.1}", totals[0]),
                    format!("{:.1}", totals[1]),
                    format!("{:+.1}", totals[2]),
                ],
            ),
            (
                if config.explosion.is_some() {
                    "projectile.controls.ammo"
                } else {
                    "projectile.controls"
                },
                vec![],
            ),
            ("projectile.confirm", vec![]),
        ];
        for (i, (key, args)) in rows.iter().enumerate() {
            let refs: Vec<_> = args.iter().map(String::as_str).collect();
            let text = crate::format_text(self.zh, key, &refs);
            label(
                &text,
                sidebar_x(true, 800.),
                if config.explosion.is_some() {
                    309. + i as f32 * 31.
                } else {
                    270. + i as f32 * 33.
                },
                fit_size(&text, 16., 200.),
                if i < 5 { WHITE } else { SKYBLUE },
            );
        }
    }
}
impl Session {
    /// Analysis adapter over normal casts, settlement, refill and rollback-friendly RNG.
    pub(crate) fn projectile_sweep(samples: usize, seed: u64) -> (String, String) {
        struct RestoreRng(u64);
        impl Drop for RestoreRng {
            fn drop(&mut self) {
                rng::restore(self.0);
            }
        }
        let _guard = RestoreRng(rng::state());
        let mut model = Session::new_simulation();
        model.prepare_headless_batch();
        let mut csv=String::from("awake,skill,ammo,sample,angle,power,cost,direct,blast,cleared,heal,damage,mana,net_mana,distance,duration\n");
        let mut summary=String::from("awake,skill,ammo,shots,mean_direct,mean_blast,mean_cleared,mean_heal,mean_damage,damage_stddev,mean_mana,mean_net_mana,net_mana_stddev,damage_per_cost,mean_distance,selection_rate\n");
        for awake in [false, true] {
            let mut modes = Vec::new();
            for &skill in content::skills(content::SLINGSHOT_USER, awake) {
                if let Some(config) = content::skill_profile(
                    skill,
                    content::Rules {
                        awake,
                        ..Default::default()
                    },
                )
                .projectile
                {
                    modes.push((skill, Ammo::Solid));
                    if config.explosion.is_some() {
                        modes.push((skill, Ammo::Explosive));
                    }
                }
            }
            let mut sums = vec![[0.; 12]; modes.len()];
            let mut selections = vec![0; modes.len()];
            for sample in 0..samples {
                model.hero_id = content::SLINGSHOT_USER;
                model.enemy_id = content::DWARF;
                rng::srand(seed.wrapping_add(sample as u64));
                model.reset_battle();
                model.awakened[0] = awake;
                model.hp = content::health_cap(content::SLINGSHOT_USER, awake);
                model.mp = content::mana_cap(content::SLINGSHOT_USER, awake);
                model.new_role_states[0].turn_start_done = true;
                let initial = model.snapshot();
                let mut best = (f32::NEG_INFINITY, 0);
                for (index, &(skill, ammo)) in modes.iter().enumerate() {
                    for angle in [15000, 30000, 45000, 60000] {
                        for power in [250, 500, 750, 1000] {
                            model.restore(initial.clone());
                            let aim = Aim {
                                ammo,
                                angle_mdeg: angle,
                                power_permille: power,
                                ..Default::default()
                            };
                            let config = content::skill_profile(skill, model.rules(false))
                                .projectile
                                .unwrap();
                            let flight = config.solve(&model.b, aim);
                            let cost = content::skill_cost_in(
                                content::SLINGSHOT_USER,
                                skill,
                                model.rules(false),
                            );
                            model.projectile_aim = Some(aim);
                            if let Some(result) = model
                                .simulate_plan_step(planning::PlanAction::Cast(skill, (3, 3), None))
                            {
                                let heal = result.hp - initial.hp;
                                let damage =
                                    model.turn_log.current.damage - initial.turns.current.damage;
                                let net = result.mp - initial.mp;
                                let mana = net + cost;
                                let distance = flight.position(flight.duration).x - flight.origin.x;
                                csv.push_str(&format!("{awake},{skill:?},{ammo:?},{sample},{},{},{cost},{},{},{},{heal},{damage},{mana},{net},{distance},{}\n",angle as f64/1000.,power as f64/10.,flight.hits.len(),flight.explosion_cells.len(),flight.cells().len(),flight.duration));
                                let values = [
                                    flight.hits.len() as f64,
                                    flight.explosion_cells.len() as f64,
                                    flight.cells().len() as f64,
                                    heal as f64,
                                    damage as f64,
                                    (damage as f64).powi(2),
                                    mana as f64,
                                    net as f64,
                                    (net as f64).powi(2),
                                    damage as f64 / cost as f64,
                                    distance,
                                    1.,
                                ];
                                for (sum, value) in sums[index].iter_mut().zip(values) {
                                    *sum += value;
                                }
                                let score = strategy::reward_value(damage, heal, net);
                                if score > best.0 {
                                    best = (score, index);
                                }
                            }
                        }
                    }
                }
                if best.0.is_finite() {
                    selections[best.1] += 1;
                }
            }
            for (index, (skill, ammo)) in modes.iter().enumerate() {
                let n = sums[index][11];
                let mean = |i: usize| if n > 0. { sums[index][i] / n } else { 0. };
                let sd = |i: usize| (mean(i + 1) - mean(i).powi(2)).max(0.).sqrt();
                summary.push_str(&format!(
                    "{awake},{skill:?},{ammo:?},{n},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    mean(0),
                    mean(1),
                    mean(2),
                    mean(3),
                    mean(4),
                    sd(4),
                    mean(6),
                    mean(7),
                    sd(7),
                    mean(9),
                    mean(10),
                    selections[index] as f64 / samples as f64
                ));
            }
        }
        (csv, summary)
    }
}

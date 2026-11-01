use super::*;
pub(super) fn effect(skill: Skill, b: &mut Board, target: (usize, usize)) -> Effect {
    let mut e = Effect {
        reward: skill.operation() == TileOperation::Eliminate,
        ..Default::default()
    };
    match skill {
        Skill::Blood => e.hp = 100,
        Skill::Balance | Skill::War | Skill::Drain => {
            for y in target.1.min(N - 2)..target.1.min(N - 2) + 2 {
                for x in target.0.min(N - 2)..target.0.min(N - 2) + 2 {
                    if skill == Skill::Drain {
                        e.cells.push((x, y));
                        e.hp += VALUES[b[y][x] as usize].0 * 2;
                        e.damage += VALUES[b[y][x] as usize].0 * 3;
                    } else {
                        b[y][x] = if skill == Skill::War { 2 } else { 0 };
                    }
                }
            }
        }
        Skill::Guard => {
            for (y, row) in b.iter().enumerate() {
                for (x, &kind) in row.iter().enumerate() {
                    if kind == 1 {
                        e.cells.push((x, y));
                    }
                }
            }
            // The consumed healing tiles deliberately grant no extra rewards.
        }
        Skill::Charge => {
            e.damage = 60;
            let x = target.0.min(N - 2);
            for y in 0..N {
                for c in x..x + 2 {
                    e.cells.push((c, y));
                    e.damage += VALUES[b[y][c] as usize].1;
                }
            }
        }
        Skill::Landslide => e.armed = true,
        Skill::Strike => e.bonus = 1,
        Skill::Nature => {
            e.damage = 25;
            e.hp = 10;
            for y in target.1.min(N - 2)..target.1.min(N - 2) + 2 {
                for x in target.0.min(N - 2)..target.0.min(N - 2) + 2 {
                    b[y][x] = 0;
                }
            }
        }

        Skill::Corrupt => {
            for row in b.iter_mut() {
                for t in row {
                    if *t == 2 {
                        *t = 1;
                        e.damage += 6;
                        e.hp += 3;
                    }
                }
            }
        }
        Skill::Lightning => {
            for y in 0..N {
                for x in [y, N - 1 - y] {
                    e.cells.push((x, y));
                    e.mp += VALUES[b[y][x] as usize].2;
                }
            }
        }
        Skill::Meteor => {
            for x in 0..N {
                let c = x.min(N - 1 - x);
                for y in N / 2 - 1 - c..=N / 2 + c {
                    e.cells.push((x, y));
                }
            }
            let (_, damage, mana) = trigger_tiles(b, &e.cells, 0, 50, 50);
            e.damage = 60 + damage;
            e.mp = mana;
        }
        Skill::Greedy => {
            e.reward = true;
            for y in target.1.min(N - 2)..target.1.min(N - 2) + 2 {
                for x in target.0.min(N - 2)..target.0.min(N - 2) + 2 {
                    e.cells.push((x, y));
                    let v = VALUES[b[y][x] as usize];
                    e.hp += v.0;
                    e.damage += v.1;
                    e.mp += v.2;
                }
            }
            e.hp /= 2;
            e.damage /= 2;
            e.mp /= 2;
        }

        Skill::Imp => {
            for y in 0..N {
                for x in 0..N {
                    if b[y][x] == 0 {
                        e.cells.push((x, y));
                        e.damage += 12;
                        e.mp += 4;
                    }
                }
            }
        }
        Skill::Chort => {
            let kind = b[target.1][target.0];
            for y in 0..N {
                for x in 0..N {
                    if b[y][x] == kind {
                        e.cells.push((x, y));
                    }
                }
            }

            e.hp = e.cells.len() as i32 * 2;
            e.damage = e.cells.len() as i32 * 4;
            e.mp = e.cells.len() as i32;
        }
        _ => unreachable!("new skill dispatched to legacy effects"),
    }
    e
}

pub(super) fn cast(
    skill: Skill,
    b: &mut Board,
    target: (usize, usize),
    ctx: &EventContext,
) -> Effect {
    let original = *b;
    let mut e = effect(skill, b, target);
    if skill == Skill::Guard {
        e.hp = health_cap(ctx.hero, ctx.rules.awake) / 5;
    }
    let mut batch = RewardBatch::default();
    let tile_events = |position: (usize, usize), gains: (i32, i32, i32)| {
        let source = EventSource::Tile {
            position,
            tile: original[position.1][position.0],
            chain: 0,
            from_swap: false,
        };
        let mut part = RewardBatch::gains(gains, 0, source);
        for event in &mut part.events {
            event.derived_by(EventSource::Skill(skill), EventSide::Actor);
        }
        part.events
    };
    match skill {
        Skill::Charge => {
            batch.push(
                Resource::Damage,
                EventSide::Opponent,
                60,
                EventSource::Skill(skill),
            );
            for &cell in &e.cells {
                batch.events.extend(tile_events(
                    cell,
                    (0, VALUES[original[cell.1][cell.0] as usize].1, 0),
                ));
            }
        }
        Skill::Drain => {
            for &cell in &e.cells {
                let h = VALUES[original[cell.1][cell.0] as usize].0;
                batch.events.extend(tile_events(cell, (h * 2, h * 3, 0)));
            }
        }
        Skill::Lightning => {
            for &cell in &e.cells {
                batch.events.extend(tile_events(
                    cell,
                    (0, 0, VALUES[original[cell.1][cell.0] as usize].2),
                ));
            }
        }
        Skill::Meteor => {
            for &cell in &e.cells {
                let v = VALUES[original[cell.1][cell.0] as usize];
                batch.events.extend(tile_events(cell, (0, v.1, v.2)));
            }
            let t = batch.totals();
            batch.set_totals((0, t.1 / 2, t.2 / 2));
            batch.push(
                Resource::Damage,
                EventSide::Opponent,
                60,
                EventSource::Skill(skill),
            );
        }
        Skill::Greedy => {
            for &cell in &e.cells {
                batch
                    .events
                    .extend(tile_events(cell, VALUES[original[cell.1][cell.0] as usize]));
            }
            let t = batch.totals();
            batch.set_totals((t.0 / 2, t.1 / 2, t.2 / 2));
        }
        Skill::Imp => {
            for &cell in &e.cells {
                batch.events.extend(tile_events(cell, (0, 12, 4)));
            }
        }
        Skill::Chort => {
            for &cell in &e.cells {
                batch.events.extend(tile_events(cell, (2, 4, 1)));
            }
        }
        Skill::Corrupt => {
            for y in 0..N {
                for x in 0..N {
                    if original[y][x] == 2 {
                        batch.events.extend(tile_events((x, y), (3, 6, 0)));
                    }
                }
            }
        }
        _ => {
            batch = RewardBatch::gains(
                (e.hp, e.damage, e.mp),
                e.opponent_heal,
                EventSource::Skill(skill),
            )
        }
    }
    e.events = Some(batch);
    e
}

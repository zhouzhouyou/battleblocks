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
            e.hp = HEROES[0].hp / 5;
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

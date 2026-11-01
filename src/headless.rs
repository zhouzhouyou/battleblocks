use std::path::PathBuf;

const DEFAULT_MAX_STEPS: u64 = 200_000;
const DEFAULT_BATCH_MAX_STEPS: u64 = 20_000;

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--autoplay") {
        run_batch(&args);
        return;
    }
    let max_steps = option_value(&args, "--headless-max-steps")
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_MAX_STEPS);
    let mut script = Vec::new();
    if let Some(path) = option_value(&args, "--script") {
        let text = if path == "-" {
            let mut text = String::new();
            use std::io::Read;
            std::io::stdin()
                .read_to_string(&mut text)
                .expect("read headless script from stdin");
            text
        } else {
            std::fs::read_to_string(PathBuf::from(path)).expect("read headless script")
        };
        script.extend(
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned),
        );
    }

    let verbose = args.iter().any(|arg| arg == "--verbose");
    if !verbose {
        std::env::set_var("BATTLEBLOCKS_HEADLESS_QUIET", "1");
    }
    let mut session = crate::app::Session::new_headless();
    session.set_headless_quiet(!verbose);
    for line in script {
        session.queue_headless_command(line);
    }
    session.start();

    let mut steps = 0u64;
    while session.headless_commands_pending() {
        session.headless_step_command();
        steps += 1;
        while !session.headless_idle() {
            session.headless_advance();
            steps += 1;
            if steps > max_steps {
                eprintln!("HEADLESS_STALL steps={steps} max_steps={max_steps}");
                std::process::exit(2);
            }
        }
    }
    if steps > max_steps {
        eprintln!("HEADLESS_STALL steps={steps} max_steps={max_steps}");
        std::process::exit(2);
    }
    println!("{}", session.headless_summary(steps));
}

/// Unattended pairwise screening through the canonical `Session` model.
///
/// The player side is driven with the same `strategy` policy as the enemy side,
/// so no rules live in this runner. Output is a CSV summary on stdout (or `--out`).
fn run_batch(args: &[String]) {
    if args
        .iter()
        .any(|arg| arg == "--script" || arg == "--command")
    {
        eprintln!("--autoplay cannot be combined with --script or --command");
        std::process::exit(2);
    }
    let samples: usize = option_value(args, "--samples")
        .and_then(|value| value.parse().ok())
        .unwrap_or(24);
    if samples == 0 {
        eprintln!("--samples must be positive");
        std::process::exit(2);
    }
    let seed: u64 = option_value(args, "--seed")
        .and_then(|value| value.parse().ok())
        .unwrap_or(30_000);
    let level: usize = option_value(args, "--level")
        .and_then(|value| value.parse().ok())
        .unwrap_or(1)
        .min(2);
    let max_steps: u64 = option_value(args, "--headless-max-steps")
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_BATCH_MAX_STEPS);
    let verbose = args.iter().any(|arg| arg == "--verbose");
    let roster = match option_value(args, "--roster") {
        Some(text) => text
            .split(',')
            .map(|field| {
                field.trim().parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("invalid --roster value: {field}");
                    std::process::exit(2);
                })
            })
            .collect::<Vec<_>>(),
        None => crate::content::ROSTER.to_vec(),
    };
    if roster.len() < 2 {
        eprintln!("--roster needs at least two hero ids");
        std::process::exit(2);
    }
    let mut seen = std::collections::HashSet::new();
    for &id in &roster {
        if id >= crate::content::HEROES.len() {
            eprintln!("hero id {id} is out of range");
            std::process::exit(2);
        }
        if !seen.insert(id) {
            eprintln!("hero id {id} appears more than once");
            std::process::exit(2);
        }
    }

    std::env::set_var("BATTLEBLOCKS_HEADLESS_QUIET", "1");
    let mut session = crate::app::Session::new_headless();
    session.prepare_headless_batch();

    let n = roster.len();
    let mut wins = vec![0u64; crate::content::HEROES.len()];
    let mut games = vec![0u64; crate::content::HEROES.len()];
    let mut pairs = vec![vec![0u64; n]; n];
    let mut rounds = Vec::new();
    let mut draws = 0u64;
    let mut aborted = 0u64;
    let mut first_wins = 0u64;
    let mut total = 0u64;
    let mut nukes = vec![0u64; crate::content::HEROES.len()];
    let mut nuke_damage = vec![0i64; crate::content::HEROES.len()];
    let mut first_nuke_sum = vec![0u64; crate::content::HEROES.len()];
    let mut first_nuke_games = vec![0u64; crate::content::HEROES.len()];
    for a in 0..n {
        for b in (a + 1)..n {
            for sample in 0..samples {
                for (player, enemy) in [(a, b), (b, a)] {
                    let outcome = session.headless_batch_match(
                        roster[player],
                        roster[enemy],
                        seed + sample as u64,
                        level,
                        max_steps,
                    );
                    total += 1;
                    games[roster[player]] += 1;
                    games[roster[enemy]] += 1;
                    // Only the fire elf can cast Ember Blast; attribute the count to that side.
                    if roster[player] == crate::content::FIRE_ELF {
                        nukes[roster[player]] += outcome.nukes[0] as u64;
                        nuke_damage[roster[player]] += outcome.nuke_damage[0];
                        if outcome.first_nuke_round[0] > 0 {
                            first_nuke_sum[roster[player]] += outcome.first_nuke_round[0] as u64;
                            first_nuke_games[roster[player]] += 1;
                        }
                    }
                    if roster[enemy] == crate::content::FIRE_ELF {
                        nukes[roster[enemy]] += outcome.nukes[1] as u64;
                        nuke_damage[roster[enemy]] += outcome.nuke_damage[1];
                        if outcome.first_nuke_round[1] > 0 {
                            first_nuke_sum[roster[enemy]] += outcome.first_nuke_round[1] as u64;
                            first_nuke_games[roster[enemy]] += 1;
                        }
                    }
                    rounds.push(outcome.rounds);
                    match outcome.winner {
                        Some(0) => {
                            wins[roster[player]] += 1;
                            pairs[player][enemy] += 1;
                            first_wins += 1;
                        }
                        Some(1) => {
                            wins[roster[enemy]] += 1;
                            pairs[enemy][player] += 1;
                        }
                        _ => {
                            draws += 1;
                            if outcome.steps >= max_steps {
                                aborted += 1;
                            }
                        }
                    }
                }
            }
            if verbose {
                eprintln!(
                    "pair {}/{} {} vs {}",
                    a * (2 * n - a - 1) / 2 + b - a,
                    n * (n - 1) / 2,
                    crate::content::HEROES[roster[a]].en,
                    crate::content::HEROES[roster[b]].en
                );
            }
        }
    }
    rounds.sort_unstable();
    let median = rounds.get(rounds.len() / 2).copied().unwrap_or(0);
    let p90 = rounds
        .get(rounds.len().saturating_mul(9) / 10)
        .copied()
        .unwrap_or(0);
    let max_round = rounds.last().copied().unwrap_or(0);
    let mut out = String::new();
    out.push_str("hero,wins,games,win_percent,nukes_per_game,first_nuke_round,avg_nuke_damage\n");
    for &id in &roster {
        out.push_str(&format!(
            "{},{},{},{:.1},{:.2},{:.1},{:.1}\n",
            crate::content::HEROES[id].en,
            wins[id],
            games[id],
            wins[id] as f64 / games[id].max(1) as f64 * 100.,
            nukes[id] as f64 / games[id].max(1) as f64,
            first_nuke_sum[id] as f64 / first_nuke_games[id].max(1) as f64,
            nuke_damage[id] as f64 / nukes[id].max(1) as f64
        ));
    }
    out.push_str(&format!(
        "games={total},draws={draws},aborted={aborted},median_rounds={median},p90_rounds={p90},max_rounds={max_round},first_side_win={:.1}%\n",
        first_wins as f64 / (total - draws).max(1) as f64 * 100.
    ));
    out.push_str(&format!(
        "roster={}\n",
        roster
            .iter()
            .map(|id| crate::content::HEROES[*id].en)
            .collect::<Vec<_>>()
            .join(",")
    ));
    out.push_str(&format!("pairwise_win_counts={pairs:?}\n"));
    match option_value(args, "--out") {
        Some(path) => {
            if let Err(error) = std::fs::write(&path, out) {
                eprintln!("write {path}: {error}");
                std::process::exit(2);
            }
            eprintln!("wrote {path}");
        }
        None => print!("{out}"),
    }
}

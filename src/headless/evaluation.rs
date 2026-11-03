//! Line-oriented adapter to the canonical Session. No scoring or game rules here.
use std::io::{BufRead, Write};

pub(super) fn requested(args: &[String]) -> bool {
    args.iter()
        .any(|s| matches!(s.as_str(), "--evaluation-manifest" | "--evaluate-jobs"))
}

#[derive(Debug, PartialEq, Eq)]
struct Job {
    hero: usize,
    enemy: usize,
    seed: u64,
    level: usize,
    max_steps: u64,
}

fn parse(line: &str) -> Result<Job, String> {
    let fields: Vec<_> = line.split_whitespace().collect();
    if fields.len() != 5 {
        return Err("expected: hero enemy seed level max_steps".into());
    }
    let values = fields
        .iter()
        .map(|s| s.parse::<u64>().map_err(|_| "invalid integer".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    if values[0] >= crate::content::HEROES.len() as u64
        || values[1] >= crate::content::HEROES.len() as u64
        || values[0] == values[1]
        || values[2] == 0
        || values[3] > 2
        || values[4] == 0
    {
        return Err("invalid hero, seed, level or step limit".into());
    }
    Ok(Job {
        hero: values[0] as usize,
        enemy: values[1] as usize,
        seed: values[2],
        level: values[3] as usize,
        max_steps: values[4],
    })
}

fn termination(outcome: &crate::app::BatchOutcome) -> &'static str {
    if outcome.winner.is_some() {
        "win"
    } else if outcome.hp <= 0 && outcome.enemy_hp <= 0 {
        "draw"
    } else {
        "aborted"
    }
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    if args.iter().skip(1).any(|s| {
        !matches!(
            s.as_str(),
            "--headless" | "--evaluation-manifest" | "--evaluate-jobs"
        )
    }) || args
        .iter()
        .filter(|s| matches!(s.as_str(), "--evaluation-manifest" | "--evaluate-jobs"))
        .count()
        != 1
    {
        return Err("evaluation options cannot be combined with other modes/options".into());
    }
    if args.iter().any(|s| s == "--evaluation-manifest") {
        println!(
            "{}",
            include_str!(concat!(env!("OUT_DIR"), "/evaluation-manifest.json"))
        );
        return Ok(());
    }
    std::env::set_var("BATTLEBLOCKS_HEADLESS_QUIET", "1");
    let mut session = crate::app::Session::new_headless();
    session.prepare_headless_batch();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        let job = parse(&line)?;
        let outcome =
            session.headless_batch_match(job.hero, job.enemy, job.seed, job.level, job.max_steps);
        let winner = outcome.winner.map_or("null".to_owned(), |v| v.to_string());
        writeln!(stdout, "{{\"hero\":{},\"enemy\":{},\"seed\":{},\"level\":{},\"max_steps\":{},\"winner\":{},\"termination\":\"{}\",\"rounds\":{},\"steps\":{},\"hp\":{},\"enemy_hp\":{}}}",
            job.hero, job.enemy, job.seed, job.level, job.max_steps, winner, termination(&outcome),
            outcome.rounds, outcome.steps, outcome.hp, outcome.enemy_hp).map_err(|e| e.to_string())?;
        stdout.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_jobs_without_truncating_u64_or_clamping_levels() {
        assert_eq!(
            parse("8 7 18446744073709551615 2 123").unwrap().seed,
            u64::MAX
        );
        for bad in [
            "8 7 0 1 10",
            "8 8 1 1 10",
            "8 7 1 3 10",
            "99 7 1 1 10",
            "8 7 1 1 0",
            "8 7 1 1",
            "18446744073709551615 7 1 1 10",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn step_limit_is_not_a_draw() {
        let mut session = crate::app::Session::new_headless();
        let result = session.headless_batch_match(8, 7, 19, 1, 1);
        assert_eq!(termination(&result), "aborted");
    }
}

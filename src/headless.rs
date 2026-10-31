use std::path::PathBuf;

const DEFAULT_MAX_STEPS: u64 = 200_000;

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
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

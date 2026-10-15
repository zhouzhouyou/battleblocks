use crate::{label, Board};
use macroquad::prelude::*;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Snapshot {
    pub board: Board,
    pub hero: usize,
    pub enemy: usize,
    pub hp: i32,
    pub enemy_hp: i32,
    pub mp: i32,
    pub enemy_mp: i32,
    pub enemy_turn: bool,
    pub action_done: bool,
    pub pending: Vec<(usize, usize)>,
    pub deadline: f64,
    pub enemy_clock: f64,
    pub bonuses: [i32; 2],
    pub armed_sides: [bool; 2],
    pub bonus: i32,
    pub armed: bool,
    pub chain: i32,
    pub reward: bool,
    pub difficulty: usize,
    pub paused: bool,
    pub rng_state: u64,
    pub charge_damage: Option<(i32, bool)>,
    pub charge_remaining: f64,
}
impl Snapshot {
    fn json(&self) -> String {
        format!("{{\"board\":{:?},\"hero\":{},\"enemy\":{},\"hp\":{},\"enemy_hp\":{},\"mp\":{},\"enemy_mp\":{},\"enemy_turn\":{},\"action_done\":{},\"pending\":{},\"deadline\":{},\"enemy_clock\":{},\"bonuses\":{:?},\"armed_sides\":{:?},\"bonus\":{},\"armed\":{},\"chain\":{},\"reward\":{},\"difficulty\":{},\"paused\":{},\"rng_state\":{},\"charge_damage\":{},\"charge_remaining\":{}}}",
            self.board,self.hero,self.enemy,self.hp,self.enemy_hp,self.mp,self.enemy_mp,self.enemy_turn,self.action_done,
            cells_json(&self.pending),self.deadline,self.enemy_clock,self.bonuses,self.armed_sides,self.bonus,self.armed,self.chain,self.reward,self.difficulty,self.paused,self.rng_state,self.charge_damage.map(|(d,e)|format!("[{d},{e}]")).unwrap_or("null".into()),self.charge_remaining)
    }
}
fn cells_json(cells: &[(usize, usize)]) -> String {
    format!(
        "[{}]",
        cells
            .iter()
            .map(|(x, y)| format!("[{x},{y}]"))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Status,
    Board,
    Export,
    Undo(bool),
    Restart,
    Resolve,
    Shuffle,
    Seed(u64),
    SetTile(usize, usize, u8),
    LoadBoard(Board),
    Hero(bool, usize),
    Bonus(bool, i32),
    Armed(bool, bool),
    Hp(bool, i32),
    Mp(bool, i32),
    Turn(bool),
    Ai(usize),
    Pause(bool),
    Move((usize, usize), (usize, usize)),
    Cast(usize, (usize, usize)),
}
pub fn parse(input: &str) -> Result<Command, String> {
    let words: Vec<_> = input.split_whitespace().collect();
    let n = |i: usize| -> Result<i32, String> {
        words
            .get(i)
            .ok_or("Missing argument")?
            .parse()
            .map_err(|_| "Expected integer".into())
    };
    let coord = |i| -> Result<usize, String> {
        let v = n(i)?;
        if (1..=8).contains(&v) {
            Ok((v - 1) as usize)
        } else {
            Err("Coordinates must be 1..8 (x y)".into())
        }
    };
    let side = |i| -> Result<bool, String> {
        match words.get(i).copied() {
            Some("player") => Ok(false),
            Some("enemy") => Ok(true),
            _ => Err("Use player or enemy".into()),
        }
    };
    let value = |i| -> Result<i32, String> {
        let v = n(i)?;
        if v >= 0 {
            Ok(v)
        } else {
            Err("Value must be non-negative".into())
        }
    };
    let kind = |i| -> Result<u8, String> {
        let v = n(i)?;
        if (0..=6).contains(&v) {
            Ok(v as u8)
        } else {
            Err("Tile type must be 0..6".into())
        }
    };
    match words.first().copied().unwrap_or("") {
        "help" if words.len() == 1 => Ok(Command::Help),
        "status" if words.len() == 1 => Ok(Command::Status),
        "board" if words.len() == 1 => Ok(Command::Board),
        "export" if words.len() == 1 => Ok(Command::Export),
        "undo" if words.len() == 1 => Ok(Command::Undo(false)),
        "undo" if words.as_slice() == ["undo", "player"] => Ok(Command::Undo(true)),
        "restart" if words.len() == 1 => Ok(Command::Restart),
        "resolve" if words.len() == 1 => Ok(Command::Resolve),
        "shuffle" if words.len() == 1 => Ok(Command::Shuffle),
        "seed" if words.len() == 2 => Ok(Command::Seed(
            words[1].parse().map_err(|_| "Expected unsigned seed")?,
        )),
        "tile" if words.len() == 4 => Ok(Command::SetTile(coord(1)?, coord(2)?, kind(3)?)),
        "board" if words.len() == 66 && words[1] == "load" => {
            let mut b = [[0; 8]; 8];
            for y in 0..8 {
                for x in 0..8 {
                    b[y][x] = kind(2 + y * 8 + x)?;
                }
            }
            Ok(Command::LoadBoard(b))
        }
        "hero" if words.len() == 3 && (0..7).contains(&n(2)?) => {
            Ok(Command::Hero(side(1)?, n(2)? as usize))
        }
        "buff" if words.len() == 4 && words[2] == "bonus" && (0..=10).contains(&n(3)?) => {
            Ok(Command::Bonus(side(1)?, n(3)?))
        }
        "buff" if words.len() == 4 && words[2] == "armed" => match words[3] {
            "on" => Ok(Command::Armed(side(1)?, true)),
            "off" => Ok(Command::Armed(side(1)?, false)),
            _ => Err("Use buff player/enemy armed on/off".into()),
        },
        "hp" if words.len() == 3 => Ok(Command::Hp(side(1)?, value(2)?)),
        "mp" if words.len() == 3 => Ok(Command::Mp(side(1)?, value(2)?)),
        "turn" if words.len() == 2 => Ok(Command::Turn(side(1)?)),
        "ai" if words.len() == 2 && (0..=2).contains(&n(1)?) => Ok(Command::Ai(n(1)? as usize)),
        "pause" if words.len() == 2 => match words[1] {
            "on" => Ok(Command::Pause(true)),
            "off" => Ok(Command::Pause(false)),
            _ => Err("Use pause on/off".into()),
        },
        "move" if words.len() == 5 => Ok(Command::Move(
            (coord(1)?, coord(2)?),
            (coord(3)?, coord(4)?),
        )),
        "cast" if words.len() == 2 || words.len() == 4 => {
            let slot = n(1)?;
            if !(1..=2).contains(&slot) {
                return Err("Skill slot must be 1 or 2".into());
            }
            Ok(Command::Cast(
                (slot - 1) as usize,
                if words.len() == 4 {
                    (coord(2)?, coord(3)?)
                } else {
                    (3, 3)
                },
            ))
        }
        _ => Err("Unknown command or argument count. Type help.".into()),
    }
}
struct Checkpoint {
    gameplay: bool,
    state: Snapshot,
    actor_enemy: bool,
    action: u64,
}
pub struct Console {
    pub open: bool,
    input: String,
    output: VecDeque<String>,
    pub commands: VecDeque<String>,
    history: Vec<Checkpoint>,
    records: Vec<String>,
    seq: u64,
    pub action: u64,
    next_action: u64,
    #[cfg(not(target_arch = "wasm32"))]
    file: Option<std::io::BufWriter<std::fs::File>>,
    #[cfg(not(target_arch = "wasm32"))]
    receiver: std::sync::mpsc::Receiver<String>,
    pub path: String,
}
impl Console {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut commands = VecDeque::new();
        #[cfg(not(target_arch = "wasm32"))]
        let (file, receiver, path, open) = {
            use std::io::BufRead;
            let args: Vec<_> = std::env::args().collect();
            for pair in args.windows(2) {
                if pair[0] == "--command" {
                    commands.push_back(pair[1].clone());
                }
            }
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let default = std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("Library/Logs/BattleBlocks")
                .join(format!("session-{stamp}-{}.jsonl", std::process::id()));
            let path = args
                .windows(2)
                .find(|p| p[0] == "--log")
                .map(|p| std::path::PathBuf::from(&p[1]))
                .unwrap_or(default);
            let result = (|| {
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::File::create(&path)
            })();
            let file = match result {
                Ok(file) => Some(std::io::BufWriter::new(file)),
                Err(e) => {
                    eprintln!("Cannot open game log: {e}");
                    None
                }
            };
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                for line in std::io::stdin().lock().lines() {
                    match line {
                        Ok(line) => {
                            if sender.send(line).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
            (
                file,
                receiver,
                path.display().to_string(),
                args.iter().any(|s| s == "--debug"),
            )
        };
        #[cfg(target_arch = "wasm32")]
        let (path, open) = (
            String::from("Browser session: export downloads JSONL"),
            false,
        );
        let mut console = Self {
            open,
            input: String::new(),
            output: VecDeque::new(),
            commands,
            history: Vec::new(),
            records: Vec::new(),
            seq: 0,
            action: 0,
            next_action: 0,
            #[cfg(not(target_arch = "wasm32"))]
            file,
            #[cfg(not(target_arch = "wasm32"))]
            receiver,
            path,
        };
        console.say("Debug console: ~ / F1. Type help. Coordinates: 1..8, x y.");
        console.say(&format!("Log: {}", console.path));
        console
    }
    pub fn input(&mut self) {
        let toggle = is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::F1);
        let mut chars = Vec::new();
        while let Some(c) = get_char_pressed() {
            chars.push(c);
        }
        if toggle || chars.iter().any(|c| *c == '～' || *c == '~' || *c == '`') {
            self.open = !self.open;
        }
        if self.open {
            for c in chars {
                if !c.is_control() && !['～', '~', '`'].contains(&c) && self.input.len() < 2048 {
                    self.input.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.input.pop();
            }
            if is_key_pressed(KeyCode::Escape) {
                self.open = false;
            }
            if is_key_pressed(KeyCode::Enter) && !self.input.trim().is_empty() {
                let line = std::mem::take(&mut self.input);
                self.say(&format!("> {line}"));
                self.commands.push_back(line);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        while let Ok(line) = self.receiver.try_recv() {
            self.commands.push_back(line);
        }
    }
    pub fn say(&mut self, line: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        println!("{line}");
        self.output.push_back(line.to_string());
        while self.output.len() > 100 {
            self.output.pop_front();
        }
    }
    pub fn checkpoint(&mut self, state: Snapshot) -> u64 {
        self.checkpoint_kind(state, true)
    }
    pub fn checkpoint_edit(&mut self, state: Snapshot) -> u64 {
        self.checkpoint_kind(state, false)
    }
    fn checkpoint_kind(&mut self, state: Snapshot, gameplay: bool) -> u64 {
        self.next_action += 1;
        self.action = self.next_action;
        self.history.push(Checkpoint {
            gameplay,
            actor_enemy: state.enemy_turn,
            state,
            action: self.action,
        });
        if self.history.len() > 256 {
            self.history.remove(0);
        }
        self.action
    }
    pub fn undo(&mut self, player: bool) -> Option<(Snapshot, u64)> {
        let index = if player {
            self.history
                .iter()
                .rposition(|h| h.gameplay && !h.actor_enemy)?
        } else {
            self.history.len().checked_sub(1)?
        };
        let checkpoint = self.history.remove(index);
        self.history.truncate(index);
        self.action = self.history.last().map(|h| h.action).unwrap_or(0);
        Some((checkpoint.state, checkpoint.action))
    }
    pub fn reset_history(&mut self) {
        self.history.clear();
        self.action = 0;
    }
    pub fn event(&mut self, event: &str, detail: &str, before: &Snapshot, after: &Snapshot) {
        self.seq += 1;
        let record=format!("{{\"seq\":{},\"action\":{},\"time\":{},\"event\":{},\"actor\":{},\"detail\":{},\"before\":{},\"after\":{}}}",self.seq,self.action,get_time(),quote(event),quote(if before.enemy_turn{"enemy"}else{"player"}),quote(detail),before.json(),after.json());
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::Write;
            if let Some(file) = &mut self.file {
                if let Err(error) = writeln!(file, "{record}").and_then(|_| file.flush()) {
                    eprintln!("Game log write failed: {error}");
                    self.file = None;
                    self.say(&format!("Log write failed: {error}"));
                }
            }
        }
        self.records.push(record);
        self.say(&format!(
            "#{} action={} {event}: {detail} | HP {}/{} MP {}/{}",
            self.seq, self.action, after.hp, after.enemy_hp, after.mp, after.enemy_mp
        ));
    }
    pub fn export(&mut self) {
        let data = self.records.join("\n") + "\n";
        #[cfg(not(target_arch = "wasm32"))]
        {
            match std::fs::write(&self.path, data) {
                Ok(()) => self.say(&format!(
                    "Exported {} events: {}",
                    self.records.len(),
                    self.path
                )),
                Err(e) => self.say(&format!("Export failed: {e}")),
            }
        }
        #[cfg(target_arch = "wasm32")]
        unsafe {
            bb_debug_download(data.as_ptr(), data.len());
            self.say("Downloaded battleblocks-session.jsonl");
        }
    }
    pub fn draw(&self) {
        if !self.open {
            return;
        }
        // Labels are batched: flush the underlying scene before painting this overlay.
        crate::flush_labels();
        draw_rectangle(12., 90., 1000., 555., Color::new(0.02, 0.03, 0.06, 0.97));
        label("DEBUG CONSOLE · ~ / F1 / Esc", 28., 120., 22., GOLD);
        for (i, line) in self
            .output
            .iter()
            .rev()
            .take(19)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .enumerate()
        {
            let text: String = line.chars().take(112).collect();
            label(&text, 28., 150. + i as f32 * 23., 15., LIGHTGRAY);
        }
        let input: String = self
            .input
            .chars()
            .rev()
            .take(105)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        label(&format!("> {input}_"), 28., 622., 19., SKYBLUE);
    }
}
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn bb_debug_download(ptr: *const u8, len: usize);
}
#[cfg(test)]
mod tests {
    use super::*;
    fn state(enemy_turn: bool, hp: i32) -> Snapshot {
        Snapshot {
            board: [[1; 8]; 8],
            hero: 0,
            enemy: 1,
            hp,
            enemy_hp: 550,
            mp: 200,
            enemy_mp: 50,
            enemy_turn,
            action_done: false,
            pending: vec![(2, 3)],
            deadline: 0.2,
            enemy_clock: 0.5,
            bonuses: [1, 0],
            armed_sides: [true, false],
            bonus: 1,
            armed: true,
            chain: 2,
            reward: true,
            difficulty: 2,
            paused: false,
            rng_state: 42,
            charge_damage: None,
            charge_remaining: 0.,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn console() -> Console {
        let (_, receiver) = std::sync::mpsc::channel();
        Console {
            open: false,
            input: String::new(),
            output: VecDeque::new(),
            commands: VecDeque::new(),
            history: Vec::new(),
            records: Vec::new(),
            seq: 0,
            action: 0,
            next_action: 0,
            file: None,
            receiver,
            path: String::new(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn undo_player_removes_enemy_actions_and_debug_edits() {
        let mut c = console();
        let original = state(false, 700);
        c.checkpoint(original.clone());
        c.checkpoint(state(true, 650));
        c.checkpoint_edit(state(false, 600));
        let (restored, id) = c.undo(true).unwrap();
        assert_eq!(id, 1);
        assert_eq!(restored.json(), original.json());
        assert!(c.history.is_empty());
        assert!(c.undo(false).is_none());
        assert_eq!(c.checkpoint(state(false, 500)), 4); // IDs never reused after rollback.
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn undo_latest_retains_earlier_checkpoints() {
        let mut c = console();
        c.checkpoint(state(false, 700));
        c.checkpoint(state(true, 650));
        let (restored, id) = c.undo(false).unwrap();
        assert_eq!(restored.hp, 650);
        assert_eq!(id, 2);
        assert_eq!(c.history.len(), 1);
        assert_eq!(c.undo(false).unwrap().0.hp, 700);
    }
    #[test]
    fn command_validation() {
        assert_eq!(parse("tile 8 1 6"), Ok(Command::SetTile(7, 0, 6)));
        for bad in [
            "tile 0 1 2",
            "tile 1 1 7",
            "hp player -1",
            "move 9 1 2 3",
            "ai 3",
            "undo enemy",
            "cast 3",
            "hp player 5 extra",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        assert_eq!(parse("undo player"), Ok(Command::Undo(true)));
    }
    #[test]
    fn json_escapes_control_characters() {
        assert_eq!(quote("a\n\"\\"), "\"a\\n\\\"\\\\\"");
    }
    #[test]
    fn board_load_requires_64_valid_tiles() {
        let text = format!("board load {}", vec!["2"; 64].join(" "));
        assert_eq!(parse(&text), Ok(Command::LoadBoard([[2; 8]; 8])));
        assert!(parse("board load 2 2").is_err());
    }
}

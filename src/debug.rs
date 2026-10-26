use crate::{label, Board};
use macroquad::prelude::*;
use std::collections::VecDeque;
/// Output rows visible at once in the full-screen console.
const VISIBLE_ROWS: usize = 26;

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
    // Turn indicator records: restored by undo, left out of the log.
    pub turns: crate::turns::Log,
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
    /// Console output language: Some(true) Chinese, Some(false) English, None follows the game.
    Lang(Option<bool>),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParseError {
    MissingArgument,
    ExpectedInteger,
    Coordinate,
    Side,
    Negative,
    TileType,
    Seed,
    ArmedUsage,
    PauseUsage,
    SkillSlot,
    LangUsage,
    Unknown,
}
impl ParseError {
    pub fn message(self, zh: bool) -> &'static str {
        let (en, cn) = match self {
            Self::MissingArgument => ("Missing argument", "缺少参数"),
            Self::ExpectedInteger => ("Expected an integer", "参数应为整数"),
            Self::Coordinate => ("Coordinates must be 1–8 (x y)", "坐标必须是 1～8（x y）"),
            Self::Side => ("Use player or enemy", "请用 player 或 enemy"),
            Self::Negative => ("Value must be non-negative", "数值不能为负"),
            Self::TileType => ("Tile type must be 0–6", "方块类型必须是 0～6"),
            Self::Seed => ("Expected an unsigned seed", "种子应为非负整数"),
            Self::ArmedUsage => (
                "Use buff player|enemy armed on|off",
                "用法：buff player|enemy armed on|off",
            ),
            Self::PauseUsage => ("Use pause on|off", "用法：pause on|off"),
            Self::SkillSlot => ("Skill slot must be 1 or 2", "技能编号只能是 1 或 2"),
            Self::LangUsage => (
                "Use lang zh, lang en or lang auto (follow the game)",
                "用法：lang zh、lang en 或 lang auto（跟随游戏语言）",
            ),
            Self::Unknown => (
                "Unknown command or argument count. Type help.",
                "未知命令或参数个数不对，输入 help 查看用法",
            ),
        };
        if zh {
            cn
        } else {
            en
        }
    }
}
pub fn parse(input: &str) -> Result<Command, ParseError> {
    let words: Vec<_> = input.split_whitespace().collect();
    let n = |i: usize| -> Result<i32, ParseError> {
        words
            .get(i)
            .ok_or(ParseError::MissingArgument)?
            .parse()
            .map_err(|_| ParseError::ExpectedInteger)
    };
    let coord = |i| -> Result<usize, ParseError> {
        let v = n(i)?;
        if (1..=8).contains(&v) {
            Ok((v - 1) as usize)
        } else {
            Err(ParseError::Coordinate)
        }
    };
    let side = |i| -> Result<bool, ParseError> {
        match words.get(i).copied() {
            Some("player") => Ok(false),
            Some("enemy") => Ok(true),
            _ => Err(ParseError::Side),
        }
    };
    let value = |i| -> Result<i32, ParseError> {
        let v = n(i)?;
        if v >= 0 {
            Ok(v)
        } else {
            Err(ParseError::Negative)
        }
    };
    let kind = |i| -> Result<u8, ParseError> {
        let v = n(i)?;
        if (0..=6).contains(&v) {
            Ok(v as u8)
        } else {
            Err(ParseError::TileType)
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
            words[1].parse().map_err(|_| ParseError::Seed)?,
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
            _ => Err(ParseError::ArmedUsage),
        },
        "hp" if words.len() == 3 => Ok(Command::Hp(side(1)?, value(2)?)),
        "mp" if words.len() == 3 => Ok(Command::Mp(side(1)?, value(2)?)),
        "turn" if words.len() == 2 => Ok(Command::Turn(side(1)?)),
        "ai" if words.len() == 2 && (0..=2).contains(&n(1)?) => Ok(Command::Ai(n(1)? as usize)),
        "pause" if words.len() == 2 => match words[1] {
            "on" => Ok(Command::Pause(true)),
            "off" => Ok(Command::Pause(false)),
            _ => Err(ParseError::PauseUsage),
        },
        "move" if words.len() == 5 => Ok(Command::Move(
            (coord(1)?, coord(2)?),
            (coord(3)?, coord(4)?),
        )),
        "cast" if words.len() == 2 || words.len() == 4 => {
            let slot = n(1)?;
            if !(1..=2).contains(&slot) {
                return Err(ParseError::SkillSlot);
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
        "lang" => match words.as_slice() {
            [_, "zh" | "cn"] => Ok(Command::Lang(Some(true))),
            [_, "en"] => Ok(Command::Lang(Some(false))),
            [_, "auto" | "game"] => Ok(Command::Lang(None)),
            _ => Err(ParseError::LangUsage),
        },
        _ => Err(ParseError::Unknown),
    }
}
/// Event names as shown in the console; the JSONL log keeps the English identifiers.
fn event_name(event: &str, zh: bool) -> &str {
    if !zh {
        return event;
    }
    match event {
        "session_start" => "会话开始",
        "battle_start" => "战斗开始",
        "restart" => "重开",
        "command" => "命令",
        "move" => "交换",
        "move_rejected" => "交换无效",
        "skill" => "技能",
        "skill_hit" => "技能命中",
        "resolution" => "结算",
        "board_refresh" => "棋盘刷新",
        "turn_start" => "换人",
        "battle_end" => "战斗结束",
        "rollback" => "撤回",
        "debug_edit" => "调试修改",
        other => other,
    }
}
/// Fixed event details in Chinese; measured details (key=value) stay as logged.
fn event_detail(detail: &str, zh: bool) -> &str {
    if !zh {
        return detail;
    }
    match detail {
        "New session" => "新会话",
        "Selected hero and opponent" => "已选择角色和对手",
        "Restart battle" => "重开本局",
        "All cascades resolved; next actor" => "连锁结算完毕，轮到下一方",
        "No role-legal swap or usable skill" => "没有可行交换，也没有可用技能",
        "Player chose reroll instead of a skill" => "玩家选择重排棋盘，没有施法",
        "AI chose reroll instead of a skill" => "AI 选择重排棋盘，没有施法",
        "Player victory" => "玩家胜利",
        "Player defeat" => "玩家失败",
        other => other,
    }
}
/// Estimated label width in units of the font size (CJK about 1, ASCII about 0.58).
fn text_units(text: &str) -> f32 {
    text.chars()
        .map(|c| if c.is_ascii() { 0.58 } else { 1. })
        .sum()
}
/// Greedy wrap to `limit` font-size units, preferring to break after a space.
fn wrap_units(text: &str, limit: f32) -> Vec<String> {
    let mut rows = Vec::new();
    let mut line = String::new();
    for c in text.chars() {
        line.push(c);
        if text_units(&line) > limit {
            let cut = line
                .char_indices()
                .rev()
                .find(|&(i, ch)| ch == ' ' && i > line.len() / 2)
                .map(|(i, _)| i + 1)
                .unwrap_or(line.len() - c.len_utf8());
            let rest = line.split_off(cut);
            rows.push(line.trim_end().to_string());
            line = rest;
        }
    }
    if !line.is_empty() || rows.is_empty() {
        rows.push(line);
    }
    rows
}
struct Checkpoint {
    gameplay: bool,
    state: Snapshot,
    actor_enemy: bool,
    action: u64,
}
pub struct Console {
    pub open: bool,
    /// Output language for help, messages and event lines; the JSONL log stays English.
    pub zh: bool,
    /// Follow the game's UI language until `lang zh` / `lang en` fixes it.
    follow_game: bool,
    scroll: usize,
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
    pub fn new(zh: bool) -> Self {
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
            String::from(if zh {
                "浏览器会话，用 export 下载 JSONL"
            } else {
                "Browser session: export downloads JSONL"
            }),
            false,
        );
        let mut console = Self {
            open,
            zh,
            follow_game: true,
            scroll: 0,
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
        console.say_tr(
            "Debug console: ~ / F1 toggles, Esc closes. Type help for commands; the language follows the game, lang zh / lang en fix it.",
            "调试控制台：~ / F1 开关，Esc 关闭。输入 help 查看命令；语言跟随游戏，lang zh / lang en 可单独切换。",
        );
        let path = format!("{}{}", console.tr("Log: ", "日志："), console.path);
        console.say(&path);
        console
    }
    fn tr(&self, en: &'static str, zh: &'static str) -> &'static str {
        if self.zh {
            zh
        } else {
            en
        }
    }
    /// Say the line in the console's current language.
    pub fn say_tr(&mut self, en: &str, zh: &str) {
        let line = if self.zh { zh } else { en }.to_string();
        self.say(&line);
    }
    /// Called every frame with the game's UI language.
    pub fn sync_language(&mut self, game_zh: bool) {
        if self.follow_game {
            self.zh = game_zh;
        }
    }
    /// `lang zh` / `lang en` fix the console language; `lang auto` follows the game again.
    pub fn set_language(&mut self, choice: Option<bool>, game_zh: bool) {
        self.follow_game = choice.is_none();
        self.zh = choice.unwrap_or(game_zh);
        if self.follow_game {
            self.say_tr(
                "Console language: follows the game (English now)",
                "控制台语言：跟随游戏（当前为中文）",
            );
        } else {
            self.say_tr(
                "Console language: English (lang auto follows the game again)",
                "控制台语言：中文（lang auto 恢复跟随游戏）",
            );
        }
    }
    pub fn help(&mut self) {
        let heroes = crate::content::HEROES
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{i} {}", if self.zh { h.zh } else { h.en }))
            .collect::<Vec<_>>()
            .join(" · ");
        // A tab separates the category column from its commands.
        let lines = if self.zh {
            [
                "命令帮助：坐标是「列 行」，取值 1～8，左上角为 1 1".to_string(),
                "查看\tstatus 双方状态 · board 打印棋盘 · help 本帮助".into(),
                "棋盘\ttile x y 类型 · board load <64 个类型，按行> · shuffle 重排 · seed n 随机种子 · resolve 结算当前三消".into(),
                "数值\thp player|enemy n · mp player|enemy n · hero player|enemy 0～6 · turn player|enemy".into(),
                "强化\tbuff player|enemy bonus 0～10 · buff player|enemy armed on|off".into(),
                "行动\tmove x1 y1 x2 y2 · cast 1|2 [x y] · pause on|off · ai 0|1|2（只能在开战前设置）".into(),
                "历史\tundo 撤回上一步 · undo player 撤回到玩家上次行动前 · restart 重开 · export 导出日志".into(),
                "语言\t默认跟随游戏 · lang zh 中文 · lang en English · lang auto 恢复跟随".into(),
                "方块\t0 平衡 · 1 治疗 · 2 伤害 · 3 魔力 · 4 治疗+伤害 · 5 治疗+魔力 · 6 伤害+魔力".into(),
                format!("角色\t{heroes}"),
                "说明\t控制台打开时战斗暂停；move、cast、resolve 会关闭控制台并按正常规则结算。最多保留 256 步撤回快照，随机补充状态也会还原。".into(),
            ]
        } else {
            [
                "Command help: coordinates are column then row, 1–8; top-left is 1 1".to_string(),
                "Inspect\tstatus · board · help".into(),
                "Board\ttile x y type · board load <64 types, row order> · shuffle · seed n · resolve".into(),
                "Values\thp player|enemy n · mp player|enemy n · hero player|enemy 0–6 · turn player|enemy".into(),
                "Buffs\tbuff player|enemy bonus 0–10 · buff player|enemy armed on|off".into(),
                "Actions\tmove x1 y1 x2 y2 · cast 1|2 [x y] · pause on|off · ai 0|1|2 (before battle only)".into(),
                "History\tundo · undo player · restart · export".into(),
                "Language\tfollows the game by default · lang zh · lang en · lang auto to follow again".into(),
                "Tiles\t0 balanced · 1 heal · 2 attack · 3 mana · 4 HP+ATK · 5 HP+MP · 6 ATK+MP".into(),
                format!("Heroes\t{heroes}"),
                "Notes\tThe console freezes combat; move, cast and resolve close it and follow normal rules. Undo keeps 256 snapshots, including the random refill state.".into(),
            ]
        };
        for line in lines {
            self.say(&line);
        }
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
                self.commands.push_back(line);
                self.scroll = 0;
            }
            let wheel = mouse_wheel().1;
            if wheel > 0. || is_key_pressed(KeyCode::PageUp) {
                self.scroll += 5;
            }
            if wheel < 0. || is_key_pressed(KeyCode::PageDown) {
                self.scroll = self.scroll.saturating_sub(5);
            }
            self.scroll = self
                .scroll
                .min(self.rows().len().saturating_sub(VISIBLE_ROWS));
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
                    let line =
                        format!("{}{error}", self.tr("Log write failed: ", "日志写入失败："));
                    self.say(&line);
                }
            }
        }
        self.records.push(record);
        // The command line itself is echoed as "> line" when it runs.
        if event == "command" {
            return;
        }
        let line = if self.zh {
            format!(
                "#{} 行动{} {}：{} | 生命 {}/{} 魔力 {}/{}",
                self.seq,
                self.action,
                event_name(event, true),
                event_detail(detail, true),
                after.hp,
                after.enemy_hp,
                after.mp,
                after.enemy_mp
            )
        } else {
            format!(
                "#{} action={} {event}: {detail} | HP {}/{} MP {}/{}",
                self.seq, self.action, after.hp, after.enemy_hp, after.mp, after.enemy_mp
            )
        };
        self.say(&line);
    }
    pub fn export(&mut self) {
        let data = self.records.join("\n") + "\n";
        #[cfg(not(target_arch = "wasm32"))]
        {
            let line = match std::fs::write(&self.path, data) {
                Ok(()) => format!(
                    "{} {}: {}",
                    self.tr("Exported events:", "已导出事件："),
                    self.records.len(),
                    self.path
                ),
                Err(e) => format!("{}{e}", self.tr("Export failed: ", "导出失败：")),
            };
            self.say(&line);
        }
        #[cfg(target_arch = "wasm32")]
        unsafe {
            bb_debug_download(data.as_ptr(), data.len());
            self.say_tr(
                "Downloaded battleblocks-session.jsonl",
                "已下载 battleblocks-session.jsonl",
            );
        }
    }
    /// Output wrapped into display rows: (category of a line's first row, text, text x).
    fn rows(&self) -> Vec<(Option<&str>, String, f32)> {
        const SIZE: f32 = 15.;
        let column = if self.zh { 52. } else { 92. };
        let mut rows = Vec::new();
        for line in &self.output {
            let (category, text, x) = match line.split_once('\t') {
                Some((category, text)) => (Some(category), text, 24. + column),
                None => (None, line.as_str(), 24.),
            };
            for (i, row) in wrap_units(text, (1000. - x) / SIZE).into_iter().enumerate() {
                rows.push((category.filter(|_| i == 0), row, x));
            }
        }
        rows
    }
    pub fn draw(&self) {
        if !self.open {
            return;
        }
        // Labels are batched: flush the underlying scene before painting this overlay.
        crate::flush_labels();
        // An opaque full-screen sheet, so no battle text shows through or around it.
        draw_rectangle(0., 0., 1024., 720., Color::new(0.02, 0.03, 0.06, 1.));
        draw_rectangle_lines(8., 8., 1008., 704., 2., Color::new(0.25, 0.4, 0.55, 1.));
        label(
            self.tr(
                "DEBUG CONSOLE · ~ / F1 toggle · Esc close · wheel / PgUp PgDn scroll",
                "调试控制台 · ~ / F1 开关 · Esc 关闭 · 滚轮或 PgUp / PgDn 翻看",
            ),
            24.,
            40.,
            20.,
            GOLD,
        );
        let rows = self.rows();
        let scroll = self.scroll.min(rows.len().saturating_sub(VISIBLE_ROWS));
        let end = rows.len() - scroll;
        for (i, (category, text, x)) in rows[end.saturating_sub(VISIBLE_ROWS)..end]
            .iter()
            .enumerate()
        {
            let y = 76. + i as f32 * 23.;
            if let Some(category) = category {
                label(category, 24., y, 15., GOLD);
            }
            label(text, *x, y, 15., LIGHTGRAY);
        }
        if scroll > 0 {
            let note = if self.zh {
                format!("↑ 已上翻 {scroll} 行，向下滚动回到最新")
            } else {
                format!("↑ {scroll} rows up · scroll down for the latest")
            };
            label(&note, 24., 674., 14., SKYBLUE);
        }
        draw_line(16., 682., 1008., 682., 1., Color::new(0.25, 0.4, 0.55, 1.));
        let input: String = self
            .input
            .chars()
            .rev()
            .take(105)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        label(&format!("> {input}_"), 24., 704., 19., SKYBLUE);
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
            turns: crate::turns::Log::new(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn console() -> Console {
        let (_, receiver) = std::sync::mpsc::channel();
        Console {
            open: false,
            zh: false,
            follow_game: true,
            scroll: 0,
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
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn console_language_follows_the_game_until_fixed() {
        let mut c = console();
        c.sync_language(true);
        assert!(c.zh);
        c.sync_language(false);
        assert!(!c.zh);
        // A fixed language ignores the game's language toggles.
        c.set_language(Some(true), false);
        c.sync_language(false);
        assert!(c.zh);
        assert!(c.output.back().unwrap().contains("中文"));
        // lang auto takes the game's current language at once and keeps following it.
        c.set_language(None, false);
        assert!(!c.zh);
        c.sync_language(true);
        assert!(c.zh);
    }
    #[test]
    fn board_load_requires_64_valid_tiles() {
        let text = format!("board load {}", vec!["2"; 64].join(" "));
        assert_eq!(parse(&text), Ok(Command::LoadBoard([[2; 8]; 8])));
        assert!(parse("board load 2 2").is_err());
    }
    #[test]
    fn lang_command_and_errors_are_specific() {
        assert_eq!(parse("lang zh"), Ok(Command::Lang(Some(true))));
        assert_eq!(parse("lang cn"), Ok(Command::Lang(Some(true))));
        assert_eq!(parse("lang en"), Ok(Command::Lang(Some(false))));
        assert_eq!(parse("lang auto"), Ok(Command::Lang(None)));
        for (bad, error) in [
            ("lang", ParseError::LangUsage),
            ("lang fr", ParseError::LangUsage),
            ("tile 0 1 2", ParseError::Coordinate),
            ("tile 1 1 7", ParseError::TileType),
            ("hp player -1", ParseError::Negative),
            ("hp boss 1", ParseError::Side),
            ("cast 3", ParseError::SkillSlot),
            ("pause maybe", ParseError::PauseUsage),
            ("seed -1", ParseError::Seed),
            ("tile x 1 2", ParseError::ExpectedInteger),
            ("dance", ParseError::Unknown),
        ] {
            assert_eq!(parse(bad), Err(error), "{bad}");
        }
    }
    #[test]
    fn every_parse_error_has_both_languages() {
        use ParseError::*;
        for error in [
            MissingArgument,
            ExpectedInteger,
            Coordinate,
            Side,
            Negative,
            TileType,
            Seed,
            ArmedUsage,
            PauseUsage,
            SkillSlot,
            LangUsage,
            Unknown,
        ] {
            let cjk = |text: &str| text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
            let (en, zh) = (error.message(false), error.message(true));
            assert!(!en.is_empty() && !cjk(en), "{error:?}");
            assert!(cjk(zh), "{error:?} has no Chinese message");
        }
    }
    #[test]
    fn wrapped_rows_stay_within_the_limit_and_keep_the_text() {
        let text = "#4 action=1 resolution: cells=[(2, 6), (3, 6), (4, 6)] reward=true multiplier_percent=100 heal=0 damage=22 mana=22 chain=2 refreshed=false actual_heal=0";
        let rows = wrap_units(text, 40.);
        assert!(rows.len() > 1);
        assert!(rows.iter().all(|row| text_units(row) <= 40.));
        assert_eq!(
            rows.join(" ").split_whitespace().collect::<Vec<_>>(),
            text.split_whitespace().collect::<Vec<_>>()
        );
        let cjk = "控制台打开时战斗暂停；move、cast、resolve 会关闭控制台并按正常规则结算。";
        let rows = wrap_units(cjk, 12.);
        assert!(rows.iter().all(|row| text_units(row) <= 12.));
        assert_eq!(rows.concat().replace(' ', ""), cjk.replace(' ', ""));
        assert_eq!(wrap_units("", 10.), vec![String::new()]);
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn help_switches_language_and_keeps_the_same_sections() {
        let mut c = console();
        c.help();
        let english: Vec<_> = c.output.drain(..).collect();
        c.zh = true;
        c.help();
        let chinese: Vec<_> = c.output.drain(..).collect();
        assert_eq!(english.len(), chinese.len());
        assert!(english.iter().all(|line| !line.contains('～')));
        assert!(chinese.iter().any(|line| line.contains("lang en")));
        // Every section line has a category column, in both languages.
        for lines in [&english, &chinese] {
            assert!(lines[1..]
                .iter()
                .all(|line| line.split_once('\t').is_some()));
        }
        assert!(c
            .rows()
            .iter()
            .all(|(_, row, x)| text_units(row) * 15. <= 1000. - x));
    }
    #[test]
    fn console_names_events_in_chinese_but_keeps_log_identifiers() {
        assert_eq!(event_name("resolution", true), "结算");
        assert_eq!(event_name("resolution", false), "resolution");
        assert_eq!(
            event_detail("All cascades resolved; next actor", true),
            "连锁结算完毕，轮到下一方"
        );
        // Measured details are shown exactly as logged.
        assert_eq!(
            event_detail("swap [1,1] -> [2,1]", true),
            "swap [1,1] -> [2,1]"
        );
    }
}

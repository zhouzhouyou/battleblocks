use crate::{label, outline, Board};
use macroquad::prelude::*;
use std::collections::VecDeque;
/// Output rows visible at once in the full-screen console.
const VISIBLE_ROWS: usize = 26;

mod commands;
mod snapshot;
#[cfg(test)]
use commands::ParseError;
pub use commands::{parse, Command};
use snapshot::quote;
pub use snapshot::Snapshot;
#[cfg(test)]
mod tests;
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
        "awakening" => "觉醒",
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
    /// Seconds from a platform clock; `std::time::Instant` panics on wasm32-unknown-unknown.
    started: f64,
    quiet_events: bool,
    /// Unattended batch runs skip snapshots, history and JSONL output.
    batch: bool,
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
        let quiet_events = std::env::var_os("BATTLEBLOCKS_HEADLESS_QUIET").is_some();
        #[allow(unused_mut)]
        let mut commands = VecDeque::new();
        #[cfg(not(target_arch = "wasm32"))]
        let (file, receiver, path, open) = {
            use std::io::BufRead;
            let args: Vec<_> = std::env::args().collect();
            // The evaluation protocol owns stdin and emits its own result stream.
            // A console reader here could steal jobs or hold stdin until EOF.
            let evaluation = args.iter().any(|arg| arg == "--evaluate-jobs");
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
                if evaluation {
                    return Ok(None);
                }
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::File::create(&path).map(Some)
            })();
            let file = match result {
                Ok(file) => file.map(std::io::BufWriter::new),
                Err(e) => {
                    eprintln!("Cannot open game log: {e}");
                    None
                }
            };
            let (sender, receiver) = std::sync::mpsc::channel();
            if !evaluation {
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
            }
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
            started: macroquad::miniquad::date::now(),
            quiet_events,
            batch: false,
            action: 0,
            next_action: 0,
            #[cfg(not(target_arch = "wasm32"))]
            file,
            #[cfg(not(target_arch = "wasm32"))]
            receiver,
            path,
        };
        if !quiet_events {
            console.say_tr(
                "Debug console: ~ / F1 toggles, Esc closes. Type help for commands; the language follows the game, lang zh / lang en fix it.",
                "调试控制台：~ / F1 开关，Esc 关闭。输入 help 查看命令；语言跟随游戏，lang zh / lang en 可单独切换。",
            );
            let path = format!("{}{}", console.tr("Log: ", "日志："), console.path);
            console.say(&path);
        }
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
        self.poll_commands();
    }
    /// Drain externally supplied commands without reading keyboard or mouse state.
    pub fn poll_commands(&mut self) {
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
    /// View effects follow gameplay, not pause/resource editor checkpoints.
    pub fn gameplay_action(&self) -> u64 {
        self.history
            .iter()
            .rev()
            .find(|h| h.gameplay)
            .map(|h| h.action)
            .unwrap_or(0)
    }
    fn checkpoint_kind(&mut self, state: Snapshot, gameplay: bool) -> u64 {
        if self.batch {
            return 0;
        }
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
        self.event_payload(event, detail, None, before, after);
    }
    pub fn event_payload(
        &mut self,
        event: &str,
        detail: &str,
        payload: Option<&str>,
        before: &Snapshot,
        after: &Snapshot,
    ) {
        if self.batch {
            return;
        }
        self.seq += 1;
        let record=format!("{{\"seq\":{},\"action\":{},\"time\":{},\"event\":{},\"actor\":{},\"detail\":{},\"before\":{},\"after\":{},\"payload\":{}}}",self.seq,self.action,macroquad::miniquad::date::now()-self.started,quote(event),quote(if before.enemy_turn{"enemy"}else{"player"}),quote(detail),before.json(),after.json(),payload.unwrap_or("null"));
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
        if !self.quiet_events {
            self.say(&line);
        }
    }
    pub fn set_quiet_events(&mut self, quiet: bool) {
        self.quiet_events = quiet;
    }
    /// Unattended batch runs do not need undo snapshots or a JSONL transcript.
    pub fn set_batch(&mut self, batch: bool) {
        self.batch = batch;
    }
    pub fn quiet(&self) -> bool {
        self.quiet_events
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
        outline(8., 8., 1008., 704., 2., Color::new(0.25, 0.4, 0.55, 1.));
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

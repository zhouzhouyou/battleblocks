use super::*;
impl Session {
    pub(super) fn console_input(&mut self, frame: &mut Frame) -> bool {
        frame.was_console_open = self.console.open;
        self.console.sync_language(self.zh);
        self.console.input();
        frame.command_move = None;
        frame.command_skill = None;
        frame.command_target = None;
        frame.restart_command = false;
        // Process one command per frame so multiple CLI commands retain their order.
        if let Some(line) = self.console.commands.pop_front() {
            self.console.say(&format!("> {line}"));
            self.console
                .event("command", &line, &self.snapshot(), &self.snapshot());
            match debug::parse(&line) {
                Err(error) => self.console.say(error.message(self.console.zh)),
                Ok(command) => {
                    use debug::Command;
                    let before = self.snapshot();
                    match command {
                        Command::Help => self.console.help(),
                        Command::Status => {
                            let status = if self.console.zh {
                                format!("状态：我方 {}({hero_id}) 生命 {hp}/{} 魔力 {mp}/{} · 敌方 {}({enemy_id}) 生命 {enemy}/{} 魔力 {enemy_mp}/{} · 行动方 {} · 待结算 {} 格 · 增幅 {extra_bonus} · 三线 {} · AI {} · 暂停 {}",HEROES[self.hero_id].zh,HEROES[self.hero_id].hp,content::mana_cap(self.hero_id, self.awakened[0]),HEROES[self.enemy_id].zh,HEROES[self.enemy_id].hp,content::mana_cap(self.enemy_id, self.awakened[1]),if self.enemy_turn{"敌方"}else{"玩家"},self.pending.len(),if self.armed{"是"}else{"否"},["入门","标准","进阶"][self.difficulty],if self.paused{"是"}else{"否"},enemy = self.enemy, enemy_id = self.enemy_id, enemy_mp = self.enemy_mp, extra_bonus = self.extra_bonus, hero_id = self.hero_id, hp = self.hp, mp = self.mp)
                            } else {
                                format!("Hero={} Enemy={} HP={}/{} MP={}/{} turn={} pending={} buff={} armed={} AI={} paused={}",self.hero_id,self.enemy_id,self.hp,self.enemy,self.mp,self.enemy_mp,if self.enemy_turn{"enemy"}else{"player"},self.pending.len(),self.extra_bonus,self.armed,self.difficulty,self.paused)
                            };
                            self.console.say(&status);
                        }
                        Command::Board => {
                            self.console.say_tr(
                                "Board: one row per line, tile types 0–6, top-left is 1 1",
                                "棋盘：每行 8 个方块类型（0～6），左上角为 1 1",
                            );
                            for (y, row) in self.b.iter().enumerate() {
                                self.console.say(&format!("{}: {:?}", y + 1, row));
                            }
                        }
                        Command::Export => self.console.export(),
                        Command::Undo(player) => {
                            if let Some((state, action)) = self.console.undo(player) {
                                self.restore(state);
                                self.console.event(
                                    "rollback",
                                    &format!("Restored before action {action}; command: {line}"),
                                    &before,
                                    &self.snapshot(),
                                );
                            } else {
                                self.console
                                    .say_tr("No matching action to undo", "没有可撤回的行动");
                            }
                        }
                        Command::Restart => frame.restart_command = true,
                        Command::Move(a, c) => {
                            if self.choosing
                                || self.enemy_turn
                                || self.action_done
                                || self.presentation.busy()
                                || !self.pending.is_empty()
                                || self.hp == 0
                                || self.enemy == 0
                                || self.paused
                            {
                                self.console.say_tr("Move requires a live, unpaused player turn with no pending resolution","只能在玩家回合、未暂停且没有待结算时交换");
                            } else if !board::adjacent(a, c, self.hero_id) {
                                self.console.say_tr(
                                    "Move requires adjacent tiles",
                                    "交换的两个方块必须相邻",
                                );
                            } else {
                                frame.command_move = Some((a, c));
                                self.console.open = false;
                                self.targeting = None;
                            }
                        }
                        Command::Cast(slot, target) => {
                            let Some(&skill) =
                                content::skills(self.hero_id, self.awakened[0]).get(slot)
                            else {
                                self.console
                                    .say_tr("Skill slot is locked", "技能栏尚未解锁");
                                return false;
                            };
                            if self.choosing
                                || self.enemy_turn
                                || self.action_done
                                || self.presentation.busy()
                                || !self.pending.is_empty()
                                || self.hp == 0
                                || self.enemy == 0
                                || self.paused
                            {
                                self.console.say_tr("Cast requires a live, unpaused player turn with no pending resolution","只能在玩家回合、未暂停且没有待结算时施法");
                            } else if self.cooldowns.remaining(0, skill) > 0 {
                                self.console.say_tr("Skill is cooling down", "技能冷却中");
                            } else if content::payment_in(
                                self.hero_id,
                                self.hp,
                                self.mp,
                                skill,
                                self.rules(false),
                            )
                            .is_none()
                            {
                                self.console.say_tr("Not enough mana", "魔力不足");
                            } else if (skill == content::Skill::Strike && self.extra_bonus > 0)
                                || (skill == content::Skill::Landslide && self.armed)
                            {
                                self.console.say_tr(
                                    "Preparation skill is already active",
                                    "准备类技能已经生效",
                                );
                            } else {
                                frame.command_skill = Some(skill);
                                frame.command_target = Some(target);
                                self.console.open = false;
                                self.targeting = None;
                            }
                        }
                        Command::Ai(level) => {
                            if self.choosing {
                                self.difficulty = level;
                                self.console.say_tr(
                                    "Pre-battle AI difficulty updated",
                                    "已设置开战前的 AI 难度",
                                );
                            } else {
                                self.console.say_tr("AI difficulty is locked during battle; return to the main menu to choose a new difficulty","战斗中 AI 难度已锁定，回到主菜单才能重新选择");
                            }
                        }
                        Command::Lang(choice) => self.console.set_language(choice, self.zh),
                        command => {
                            let allowed = !self.action_done
                                && self.pending.is_empty()
                                && !self.presentation.busy();
                            if !allowed
                                && !matches!(
                                    command,
                                    Command::Hp(..)
                                        | Command::Mp(..)
                                        | Command::Ai(..)
                                        | Command::Pause(..)
                                )
                            {
                                self.console.say_tr(
                                    "Wait for resolution, or undo before editing board/turn",
                                    "请等待结算完成，或先 undo，再修改棋盘或回合",
                                );
                            } else {
                                self.console.checkpoint_edit(before.clone());
                                match command {
                                    Command::Hero(side, id) => {
                                        self.awakened[side as usize] = false;
                                        self.new_role_states[side as usize] = Default::default();
                                        self.cooldowns.0[side as usize] =
                                            [0; content::Skill::ALL.len()];
                                        if side {
                                            self.enemy_id = id;
                                            self.enemy = HEROES[id].hp;
                                            self.enemy_mp = 0;
                                            self.enemy_texture = diagnostic_texture(
                                                sprites::SPRITES[id].bytes,
                                                "no_actors",
                                            );
                                        } else {
                                            self.hero_id = id;
                                            self.hp = HEROES[id].hp;
                                            self.mp = 0;
                                            self.player_texture = diagnostic_texture(
                                                sprites::SPRITES[id].bytes,
                                                "no_actors",
                                            );
                                            self.preview = menu::Preview::new(id, 0);
                                        }
                                        self.bonuses[side as usize] = 0;
                                        self.armed_sides[side as usize] = false;
                                        if self.enemy_turn == side {
                                            self.extra_bonus = 0;
                                            self.armed = false;
                                        }
                                    }
                                    Command::Bonus(side, value) => {
                                        self.bonuses[side as usize] = value;
                                        if self.enemy_turn == side {
                                            self.extra_bonus = value;
                                        }
                                    }
                                    Command::Armed(side, value) => {
                                        self.armed_sides[side as usize] = value;
                                        if self.enemy_turn == side {
                                            self.armed = value;
                                        }
                                    }
                                    Command::SetTile(x, y, kind) => self.b[y][x] = kind,
                                    Command::LoadBoard(board) => self.b = board,
                                    Command::Shuffle => self.b = fresh(),
                                    Command::Seed(seed) => rng::srand(seed),
                                    Command::Hp(side, value) => {
                                        if side {
                                            self.enemy = value.min(content::health_cap(
                                                self.enemy_id,
                                                self.awakened[1],
                                            ))
                                        } else {
                                            self.hp = value.min(content::health_cap(
                                                self.hero_id,
                                                self.awakened[0],
                                            ))
                                        }
                                    }
                                    Command::Mp(side, value) => {
                                        if side {
                                            self.enemy_mp = value.min(content::mana_cap(
                                                self.enemy_id,
                                                self.awakened[1],
                                            ))
                                        } else {
                                            self.mp = value.min(content::mana_cap(
                                                self.hero_id,
                                                self.awakened[0],
                                            ))
                                        }
                                    }
                                    Command::Turn(side) => {
                                        self.bonuses[self.enemy_turn as usize] = self.extra_bonus;
                                        self.armed_sides[self.enemy_turn as usize] = self.armed;
                                        self.enemy_turn = side;
                                        self.extra_bonus = self.bonuses[side as usize];
                                        self.armed = self.armed_sides[side as usize];
                                        self.enemy_clock = 0.;
                                        self.turn_log.next(side);
                                    }
                                    Command::Pause(value) => self.paused = value,
                                    Command::Resolve => {
                                        self.pending = matches(&self.b);
                                        self.action_done = true;
                                        self.deadline = 0.28;
                                        self.reward = true;
                                        self.chain = 1;
                                        self.console.open = false;
                                    }
                                    _ => unreachable!(),
                                }
                                self.selected = None;
                                self.targeting = None;
                                self.choosing = false;
                                self.game_over_logged = false;
                                self.console
                                    .event("debug_edit", &line, &before, &self.snapshot());
                            }
                        }
                    }
                }
            }
        }

        true
    }
}

use crate::board::Board;
#[derive(Debug, PartialEq)]
pub enum Command {
    Shoot(usize, crate::content::projectile::Aim),
    CastPair(usize, (usize, usize), (usize, usize)),
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
    Ammunition,
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
            Self::Ammunition => (
                crate::tr_key(false, "projectile.ammo.error"),
                crate::tr_key(true, "projectile.ammo.error"),
            ),
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
            Self::SkillSlot => ("Skill slot must be 1, 2 or 3", "技能编号只能是 1、2 或 3"),
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
        "hero" if words.len() == 3 && (0..crate::content::HEROES.len() as i32).contains(&n(2)?) => {
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
        "shoot" if (4..=5).contains(&words.len()) => {
            let slot = n(1)?;
            if !(1..=3).contains(&slot) {
                return Err(ParseError::SkillSlot);
            }
            let angle = words[2]
                .parse::<f64>()
                .map_err(|_| ParseError::ExpectedInteger)?;
            let power = words[3]
                .parse::<f64>()
                .map_err(|_| ParseError::ExpectedInteger)?;
            if !angle.is_finite()
                || !power.is_finite()
                || !(0.0..90.0).contains(&angle)
                || !(0.0..=100.0).contains(&power)
            {
                return Err(ParseError::Coordinate);
            }
            let ammo = match words.get(4).copied().unwrap_or("solid") {
                "solid" => crate::content::projectile::Ammo::Solid,
                "explosive" | "blast" => crate::content::projectile::Ammo::Explosive,
                _ => return Err(ParseError::Ammunition),
            };
            Ok(Command::Shoot(
                (slot - 1) as usize,
                crate::content::projectile::Aim {
                    ammo,
                    angle_mdeg: (angle * 1000.).round() as i32,
                    power_permille: (power * 10.).round() as u16,
                    ..Default::default()
                },
            ))
        }
        "cast" if words.len() == 6 => {
            let slot = n(1)?;
            if !(1..=3).contains(&slot) {
                return Err(ParseError::SkillSlot);
            }
            Ok(Command::CastPair(
                (slot - 1) as usize,
                (coord(2)?, coord(3)?),
                (coord(4)?, coord(5)?),
            ))
        }
        "cast" if words.len() == 2 || words.len() == 4 => {
            let slot = n(1)?;
            if !(1..=3).contains(&slot) {
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

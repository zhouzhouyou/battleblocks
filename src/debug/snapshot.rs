use super::*;
#[derive(Clone)]
pub struct Snapshot {
    pub awakened: [bool; 2],
    pub cooldowns: crate::content::Cooldowns,
    pub new_role_states: [crate::role_state::RoleState; 2],
    pub reward_skill: Option<crate::content::Skill>,
    pub board: Board,
    pub board_mechanics: crate::board::BoardMechanics,
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
    pub charge_damage: Option<crate::content::PendingImpact>,
    pub charge_remaining: f64,
    // Turn indicator records: restored by undo, left out of the log.
    pub turns: crate::turns::Log,
    pub round: u32,
}
impl Snapshot {
    pub(super) fn json(&self) -> String {
        format!("{{\"board_mechanics\":{},\"new_role_states\":[{},{}],\"cooldowns\":{:?},\"reward_skill\":{},\"awakened\":{:?},\"board\":{:?},\"hero\":{},\"enemy\":{},\"hp\":{},\"enemy_hp\":{},\"mp\":{},\"enemy_mp\":{},\"enemy_turn\":{},\"action_done\":{},\"pending\":{},\"deadline\":{},\"enemy_clock\":{},\"bonuses\":{:?},\"armed_sides\":{:?},\"bonus\":{},\"armed\":{},\"chain\":{},\"reward\":{},\"difficulty\":{},\"paused\":{},\"rng_state\":{},\"charge_damage\":{},\"charge_remaining\":{},\"round\":{}}}",
            self.board_mechanics.json(),role_json(&self.new_role_states[0]),role_json(&self.new_role_states[1]),self.cooldowns.0,self.reward_skill.map(|s|quote(&format!("{s:?}"))).unwrap_or("null".into()),self.awakened,self.board,self.hero,self.enemy,self.hp,self.enemy_hp,self.mp,self.enemy_mp,self.enemy_turn,self.action_done,
            cells_json(&self.pending),self.deadline,self.enemy_clock,self.bonuses,self.armed_sides,self.bonus,self.armed,self.chain,self.reward,self.difficulty,self.paused,self.rng_state,self.charge_damage.as_ref().map(|p|p.json()).unwrap_or("null".into()),self.charge_remaining,self.round)
    }
}
fn role_json(state: &crate::role_state::RoleState) -> String {
    let pockets = state
        .pockets
        .map(|v| v.map(|v| v.to_string()).unwrap_or_else(|| "null".into()));
    let copied = state
        .copied_skill
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".into());
    format!("{{\"pockets\":[{},{}],\"copied_skill\":{},\"pocket_slot\":{},\"direct_tiles\":{:?},\"repeat_casts\":{:?},\"force_armed\":{},\"horizontal\":{},\"swap_clear\":{},\"statuses\":{}}}", pockets[0],pockets[1],copied,state.pocket_slot,state.direct_tiles,state.repeat_casts,state.force_armed,state.horizontal,state.swap_clear,statuses_json(state.statuses))
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
pub(super) fn quote(s: &str) -> String {
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

fn statuses_json(statuses: crate::content::StatusSet) -> String {
    format!(
        "[{}]",
        statuses
            .0
            .iter()
            .flatten()
            .map(|s| format!(
                "{{\"id\":\"{:?}\",\"applied_by\":\"{:?}\",\"duration\":\"{:?}\"}}",
                s.id, s.applied_by, s.duration
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

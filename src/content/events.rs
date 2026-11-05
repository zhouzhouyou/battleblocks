use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityId {
    Active(Skill),
    Passive(Passive),
    Status(StatusId),
}
impl AbilityId {
    pub fn name(self, zh: bool) -> &'static str {
        match self {
            Self::Active(s) => s.name(zh),
            Self::Passive(p) => p.name(zh),
            Self::Status(s) => s.name(zh),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventSide {
    Actor,
    Opponent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Healing,
    Damage,
    Mana,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventSource {
    Tile {
        position: (usize, usize),
        tile: u8,
        chain: usize,
        from_swap: bool,
    },
    Skill(Skill),
    Cost(Skill),
    Passive(Passive),
    Status(StatusId),
    Portal {
        position: (usize, usize),
        tile: u8,
    },
    Unspecified,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventCause {
    pub source: EventSource,
    pub by: EventSide,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardEvent {
    pub resource: Resource,
    pub recipient: EventSide,
    pub amount: i32,
    pub causes: Vec<EventCause>,
}
impl RewardEvent {
    pub fn current_source(&self) -> EventSource {
        self.causes
            .last()
            .map(|c| c.source)
            .unwrap_or(EventSource::Unspecified)
    }
    pub fn derived_by(&mut self, source: EventSource, by: EventSide) {
        self.causes.push(EventCause { source, by });
    }
    pub fn has_source(&self, source: EventSource) -> bool {
        self.causes.iter().any(|c| c.source == source)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbilityTrigger {
    pub side: EventSide,
    pub ability: AbilityId,
    pub message: Option<&'static str>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RewardBatch {
    pub events: Vec<RewardEvent>,
    pub triggers: Vec<AbilityTrigger>,
}
impl RewardBatch {
    pub fn push(
        &mut self,
        resource: Resource,
        recipient: EventSide,
        amount: i32,
        source: EventSource,
    ) {
        if amount > 0 {
            self.events.push(RewardEvent {
                resource,
                recipient,
                amount,
                causes: vec![EventCause {
                    source,
                    by: EventSide::Actor,
                }],
            });
        }
    }
    pub fn gains(gains: (i32, i32, i32), opponent_heal: i32, source: EventSource) -> Self {
        let mut b = Self::default();
        b.push(Resource::Healing, EventSide::Actor, gains.0, source);
        b.push(Resource::Damage, EventSide::Opponent, gains.1, source);
        b.push(Resource::Mana, EventSide::Actor, gains.2, source);
        b.push(
            Resource::Healing,
            EventSide::Opponent,
            opponent_heal,
            source,
        );
        b
    }
    pub fn total(&self, resource: Resource, recipient: EventSide) -> i32 {
        self.events
            .iter()
            .filter(|e| e.resource == resource && e.recipient == recipient)
            .map(|e| e.amount)
            .sum()
    }
    pub fn totals(&self) -> (i32, i32, i32) {
        (
            self.total(Resource::Healing, EventSide::Actor),
            self.total(Resource::Damage, EventSide::Opponent),
            self.total(Resource::Mana, EventSide::Actor),
        )
    }
    /// Scale cumulative contributions so integer rounding preserves the original total and every origin.
    pub fn set_total(&mut self, resource: Resource, recipient: EventSide, total: i32) {
        let old = self.total(resource, recipient);
        let mut sum = 0;
        let mut assigned = 0;
        if old == 0 {
            self.push(resource, recipient, total, EventSource::Unspecified);
            return;
        }
        for e in &mut self.events {
            if e.resource == resource && e.recipient == recipient {
                sum += e.amount;
                let next = sum * total / old;
                e.amount = next - assigned;
                assigned = next;
            }
        }
    }
    pub fn derive_resource(
        &mut self,
        from: Resource,
        recipient: EventSide,
        to: Resource,
        to_recipient: EventSide,
        amount: i32,
        source: EventSource,
        consume: bool,
    ) {
        let total = self.total(from, recipient);
        if total <= 0 || amount <= 0 {
            return;
        }
        let mut sum = 0;
        let mut allocated = 0;
        let mut derived = Vec::new();
        for event in &mut self.events {
            if event.resource == from && event.recipient == recipient {
                sum += event.amount;
                let next = sum * amount / total;
                let share = next - allocated;
                allocated = next;
                if share > 0 {
                    let mut child = event.clone();
                    child.amount = share;
                    child.resource = to;
                    child.recipient = to_recipient;
                    child.derived_by(source, EventSide::Actor);
                    derived.push(child);
                    if consume {
                        event.amount -= share;
                    }
                }
            }
        }
        self.events.extend(derived);
    }
    pub fn set_totals(&mut self, totals: (i32, i32, i32)) {
        self.set_total(Resource::Healing, EventSide::Actor, totals.0);
        self.set_total(Resource::Damage, EventSide::Opponent, totals.1);
        self.set_total(Resource::Mana, EventSide::Actor, totals.2);
    }
}
#[derive(Clone, Copy)]
pub struct EventContext {
    pub owner: EventSide,
    pub hero: usize,
    pub hp: i32,
    pub mp: i32,
    pub rules: Rules,
}
impl EventContext {
    pub fn new(hero: usize, hp: i32, mp: i32, rules: Rules) -> Self {
        Self {
            owner: EventSide::Actor,
            hero,
            hp,
            mp,
            rules,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct MovementProfile {
    pub columns: bool,
    pub diagonal: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SkillProfile {
    pub paired_target: bool,
    pub force_move: bool,
    pub random_target: bool,
    pub delayed_damage: bool,
    pub copy_slot: bool,
    pub hop: bool,
    pub initial_loot: bool,
    pub healing_only: bool,
    pub requires_unarmed: bool,
    pub requires_no_bonus: bool,
    pub zero_ai_threshold: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct AbilityCapabilities {
    pub blood_payment: bool,
    pub pockets: bool,
    pub reach_visual: bool,
    pub rejection: Option<&'static str>,
}
pub enum StateCommand {
    FillPocket { slot: usize, options: Vec<u8> },
    CopySkill { options: Vec<Skill> },
}
pub enum RuleEvent<'a> {
    PortalTransit {
        tile: u8,
        position: (usize, usize),
        batch: &'a mut RewardBatch,
    },
    AbilityEnabled {
        ability: AbilityId,
        enabled: &'a mut bool,
    },
    OpponentTurnEnd {
        cleared: &'a [u8],
        state: &'a crate::role_state::RoleState,
        commands: &'a mut Vec<StateCommand>,
    },
    TurnStart {
        board: &'a mut Board,
    },
    RefreshAbilities {
        options: &'a [Skill],
        commands: &'a mut Vec<StateCommand>,
    },
    Profile {
        value: &'a mut SkillProfile,
    },
    Capabilities {
        value: &'a mut AbilityCapabilities,
    },
    Appearance {
        dark: &'a mut bool,
    },
    ResolveSkill {
        value: &'a mut Skill,
    },
    NoSwapUsable {
        board: &'a Board,
        value: &'a mut Option<bool>,
    },
    AiUtility {
        board: &'a Board,
        bonus: i32,
        swap_value: f32,
        value: &'a mut Option<i32>,
    },
    TileReward {
        source: EventSource,
        gains: &'a mut (i32, i32, i32),
    },
    ClearScale {
        chain: usize,
        scale: &'a mut i32,
        damage_extra: &'a mut i32,
    },
    ClearBatch {
        batch: &'a mut RewardBatch,
    },
    ClearTotals {
        gains: &'a mut (i32, i32, i32),
    },
    Reward {
        reward: &'a mut RewardEvent,
        generated: &'a mut Vec<RewardEvent>,
        cancelled: &'a mut bool,
    },
    Incoming {
        reward: &'a mut RewardEvent,
    },
    ManaCap {
        value: &'a mut i32,
    },
    SkillCost {
        skill: Skill,
        value: &'a mut i32,
    },
    Payment {
        skill: Option<Skill>,
        missing: i32,
        blood: &'a mut i32,
        allowed: &'a mut bool,
    },
    Reach {
        depth: &'a mut usize,
    },
    Movement {
        profile: &'a mut MovementProfile,
    },
    SkillEffect {
        skill: Skill,
        effect: &'a mut Effect,
    },
    TurnEnd {
        batch: &'a mut RewardBatch,
    },
    CastState {
        skill: Skill,
        board: &'a Board,
        target: (usize, usize),
        state: &'a mut crate::role_state::RoleState,
    },
    Targeted {
        skill: Skill,
        value: &'a mut bool,
    },
    LegalTarget {
        skill: Skill,
        board: &'a Board,
        target: (usize, usize),
        allowed: &'a mut bool,
    },
    Cast {
        skill: Skill,
        board: &'a mut Board,
        target: (usize, usize),
        effect: &'a mut Effect,
    },
}
pub type RuleHandler =
    for<'a> fn(AbilityId, &EventContext, &mut RuleEvent<'a>, &mut Vec<AbilityTrigger>);
pub fn trigger(
    out: &mut Vec<AbilityTrigger>,
    side: EventSide,
    ability: AbilityId,
    message: Option<&'static str>,
) {
    let t = AbilityTrigger {
        side,
        ability,
        message,
    };
    if !out.contains(&t) {
        out.push(t);
    }
}
pub fn dispatch_rules(
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    triggers: &mut Vec<AbilityTrigger>,
) {
    for &p in passives(ctx.hero, ctx.rules.awake) {
        let id = AbilityId::Passive(p);
        if ability_enabled(ctx, id) {
            handlers::passive_handler(p)(id, ctx, event, triggers);
        }
    }
    for &s in skills(ctx.hero, ctx.rules.awake) {
        let s = effective_skill(s, ctx.rules);
        let id = AbilityId::Active(s);
        if ability_enabled(ctx, id) {
            handlers::active_handler(s)(id, ctx, event, triggers);
        }
    }
    for status in ctx.rules.statuses.0.iter().flatten() {
        handlers::status_handler(status.id)(AbilityId::Status(status.id), ctx, event, triggers);
    }
}
pub fn dispatch_active(
    skill: Skill,
    ctx: &EventContext,
    event: &mut RuleEvent<'_>,
    triggers: &mut Vec<AbilityTrigger>,
) {
    handlers::active_handler(skill)(AbilityId::Active(skill), ctx, event, triggers);
}
#[path = "handlers/mod.rs"]
mod handlers;

pub fn skill_profile(skill: Skill, rules: Rules) -> SkillProfile {
    let mut value = SkillProfile::default();
    let effective = effective_skill(skill, rules);
    dispatch_active(
        effective,
        &EventContext::new(0, 0, 0, rules),
        &mut RuleEvent::Profile { value: &mut value },
        &mut Vec::new(),
    );
    value
}
pub fn preparation_available(skill: Skill, rules: Rules, armed: bool, bonus: i32) -> bool {
    let p = skill_profile(skill, rules);
    !(p.requires_unarmed && armed || p.requires_no_bonus && bonus > 0)
}
pub fn ability_capabilities(hero: usize, rules: Rules) -> AbilityCapabilities {
    let mut value = AbilityCapabilities::default();
    dispatch_rules(
        &EventContext::new(hero, 0, 0, rules),
        &mut RuleEvent::Capabilities { value: &mut value },
        &mut Vec::new(),
    );
    value
}
pub fn ability_ai_utility(
    skill: Skill,
    ctx: &EventContext,
    board: &Board,
    bonus: i32,
    swap_value: f32,
) -> Option<i32> {
    let mut value = None;
    dispatch_active(
        effective_skill(skill, ctx.rules),
        ctx,
        &mut RuleEvent::AiUtility {
            board,
            bonus,
            swap_value,
            value: &mut value,
        },
        &mut Vec::new(),
    );
    value
}

pub fn ability_enabled(ctx: &EventContext, ability: AbilityId) -> bool {
    let mut enabled = true;
    for status in ctx.rules.statuses.0.iter().flatten() {
        handlers::status_handler(status.id)(
            AbilityId::Status(status.id),
            ctx,
            &mut RuleEvent::AbilityEnabled {
                ability,
                enabled: &mut enabled,
            },
            &mut Vec::new(),
        );
    }
    enabled
}

impl EventSource {
    /// Human-readable current source for feedback. Logs retain the structured chain.
    pub fn name(self, zh: bool) -> &'static str {
        match self {
            Self::Portal { .. } => super::role_text("PortalYield.name", zh),
            Self::Tile { .. } => crate::tr_key(zh, "ui.resolution.resolve_pending.clear"),
            Self::Skill(skill) => skill.name(zh),
            Self::Passive(passive) => passive.name(zh),
            Self::Status(status) => status.name(zh),
            Self::Cost(skill) => skill.name(zh),
            Self::Unspecified => crate::tr_key(zh, "ui.shared.unknown"),
        }
    }
    fn json(self) -> String {
        match self {
            Self::Tile{position:(x,y),tile,chain,from_swap}=>format!("{{\"kind\":\"tile_clear\",\"position\":[{x},{y}],\"tile\":{tile},\"chain\":{chain},\"from_swap\":{from_swap}}}"),
            Self::Skill(s)=>format!("{{\"kind\":\"skill\",\"id\":\"{s:?}\"}}"),
            Self::Passive(p)=>format!("{{\"kind\":\"passive\",\"id\":\"{p:?}\"}}"),
            Self::Status(s)=>format!("{{\"kind\":\"status\",\"id\":\"{s:?}\"}}"),
            Self::Cost(s)=>format!("{{\"kind\":\"health_cost\",\"id\":\"{s:?}\"}}"),
            Self::Portal{position:(x,y),tile}=>format!("{{\"kind\":\"portal_transit\",\"position\":[{x},{y}],\"tile\":{tile}}}"),
            Self::Unspecified=>"{\"kind\":\"unspecified\"}".into(),
        }
    }
}
impl RewardEvent {
    pub fn json(&self) -> String {
        let chain = self
            .causes
            .iter()
            .map(|c| format!("{{\"by\":\"{:?}\",\"source\":{}}}", c.by, c.source.json()))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"resource\":\"{:?}\",\"recipient\":\"{:?}\",\"amount\":{},\"source\":{},\"source_chain\":[{}]}}",self.resource,self.recipient,self.amount,self.current_source().json(),chain)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingImpact {
    pub rewards: RewardBatch,
    pub actor_enemy: bool,
    pub skill: Skill,
}
impl PendingImpact {
    pub fn json(&self) -> String {
        let triggers = self
            .rewards
            .triggers
            .iter()
            .map(|t| {
                format!(
                    "{{\"ability\":\"{:?}\",\"side\":\"{:?}\",\"message\":{}}}",
                    t.ability,
                    t.side,
                    t.message
                        .map(|s| format!("\"{s}\""))
                        .unwrap_or("null".into())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"actor_enemy\":{},\"skill\":\"{:?}\",\"events\":[{}],\"triggers\":[{}]}}",
            self.actor_enemy,
            self.skill,
            self.rewards
                .events
                .iter()
                .map(|e| e.json())
                .collect::<Vec<_>>()
                .join(","),
            triggers
        )
    }
}

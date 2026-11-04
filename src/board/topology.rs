//! Board-owned geometry. Coordinates use the game's top-left origin (y grows down).
use super::{Board, N};
pub type Cell = (usize, usize);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gravity {
    Up,
    #[default]
    Down,
    Left,
    Right,
}
impl Gravity {
    pub fn vector(self) -> (isize, isize) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }
    pub fn vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}
/// A horizontal boundary below (x,y), or a vertical boundary right of (x,y).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalEdge {
    pub cell: Cell,
    pub horizontal: bool,
}
impl PortalEdge {
    pub fn at(cell: Cell, gravity: Gravity) -> Self {
        Self {
            cell,
            horizontal: gravity.vertical(),
        }
    }
    pub fn cells(self) -> Option<(Cell, Cell)> {
        let (x, y) = self.cell;
        if x >= N || y >= N {
            return None;
        }
        if self.horizontal {
            (y + 1 < N).then_some(((x, y), (x, y + 1)))
        } else {
            (x + 1 < N).then_some(((x, y), (x + 1, y)))
        }
    }
    fn directed(self, gravity: Gravity) -> Result<(Cell, Cell), &'static str> {
        if self.horizontal != gravity.vertical() {
            return Err("portal must be perpendicular to gravity");
        }
        let (a, b) = self
            .cells()
            .ok_or("portal must be on an internal boundary")?;
        Ok(if matches!(gravity, Gravity::Down | Gravity::Right) {
            (a, b)
        } else {
            (b, a)
        })
    }
    pub fn blocks(self, a: Cell, b: Cell) -> bool {
        self.cells()
            .is_some_and(|(p, q)| (a == p && b == q) || (a == q && b == p))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalPair {
    pub a: PortalEdge,
    pub b: PortalEdge,
    pub owner: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimedGravity {
    pub direction: Gravity,
    pub owner: bool,
}
/// One pair per side. Lifetime is through the opponent's turn, until the owner's next turn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoardMechanics {
    pub portals: [Option<PortalPair>; 2],
    pub gravity_effect: Option<TimedGravity>,
}
impl BoardMechanics {
    pub fn gravity(self) -> Gravity {
        self.gravity_effect.map(|e| e.direction).unwrap_or_default()
    }
    pub fn blocks(self, a: Cell, b: Cell) -> bool {
        self.portals
            .iter()
            .flatten()
            .any(|p| p.a.blocks(a, b) || p.b.blocks(a, b))
    }
    pub fn place(&mut self, pair: PortalPair) -> Result<(), &'static str> {
        let mut next = *self;
        next.portals[pair.owner as usize] = Some(pair);
        next.topology()?;
        *self = next;
        Ok(())
    }
    pub fn set_gravity(&mut self, direction: Gravity, owner: bool) {
        self.gravity_effect = Some(TimedGravity { direction, owner });
        self.revalidate();
    }
    fn revalidate(&mut self) {
        let g = self.gravity();
        for p in &mut self.portals {
            if p.is_some_and(|p| p.a.directed(g).is_err() || p.b.directed(g).is_err()) {
                *p = None;
            }
        }
        // Reversing a valid graph reverses all its edges; it cannot create cycles.
        debug_assert!(self.topology().is_ok());
    }
    pub fn start_turn(&mut self, side: bool) -> bool {
        let before = *self;
        self.portals[side as usize] = None;
        if self.gravity_effect.is_some_and(|g| g.owner == side) {
            self.gravity_effect = None;
        }
        self.revalidate();
        before != *self
    }
    pub fn topology(self) -> Result<GravityTopology, &'static str> {
        GravityTopology::build(
            self.gravity(),
            &self.portals.iter().flatten().copied().collect::<Vec<_>>(),
        )
    }
    pub fn matches(self, board: &Board) -> Vec<Cell> {
        super::matches_in(board, self)
    }
    pub fn json(self) -> String {
        let portals = self
            .portals
            .iter()
            .flatten()
            .map(|p| {
                format!(
                    "{{\"a\":[{},{},{}],\"b\":[{},{},{}],\"owner\":{}}}",
                    p.a.cell.0,
                    p.a.cell.1,
                    p.a.horizontal,
                    p.b.cell.0,
                    p.b.cell.1,
                    p.b.horizontal,
                    p.owner
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"gravity\":\"{}\",\"gravity_owner\":{},\"portals\":[{}]}}",
            self.gravity().name(),
            self.gravity_effect
                .map(|g| g.owner.to_string())
                .unwrap_or("null".into()),
            portals
        )
    }
}
#[derive(Clone, Debug)]
pub struct GravityTopology {
    pub chains: Vec<Vec<Cell>>,
    pub portal_links: Vec<(Cell, Cell)>,
}
impl GravityTopology {
    pub fn build(gravity: Gravity, pairs: &[PortalPair]) -> Result<Self, &'static str> {
        let index = |(x, y): Cell| y * N + x;
        let cell = |i: usize| (i % N, i / N);
        let mut down = [None; N * N];
        let (dx, dy) = gravity.vector();
        for y in 0..N {
            for x in 0..N {
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx >= 0 && ny >= 0 && nx < N as isize && ny < N as isize {
                    down[index((x, y))] = Some(index((nx as usize, ny as usize)));
                }
            }
        }
        let mut used = Vec::new();
        let mut links = Vec::new();
        for p in pairs {
            let (ua, da) = p.a.directed(gravity)?;
            let (ub, db) = p.b.directed(gravity)?;
            if p.a == p.b || used.contains(&p.a) || used.contains(&p.b) {
                return Err("overlapping portal boundaries");
            }
            if if gravity.vertical() {
                ua.0 == ub.0
            } else {
                ua.1 == ub.1
            } {
                return Err("portals must be on different gravity lines");
            }
            used.extend([p.a, p.b]);
            down[index(ua)] = Some(index(db));
            down[index(ub)] = Some(index(da));
            links.extend([(ua, db), (ub, da)]);
        }
        let mut up = [None; N * N];
        for (i, to) in down.iter().enumerate() {
            if let Some(j) = *to {
                if up[j].replace(i).is_some() {
                    return Err("multiple upstream sources");
                }
            }
        }
        let mut seen = [false; N * N];
        let mut chains = Vec::new();
        // Stable source ordering preserves the original downward refill RNG order.
        for i in 0..N * N {
            if up[i].is_none() {
                let (x, y) = cell(i);
                let upstream = (x as isize - dx, y as isize - dy);
                if upstream.0 >= 0
                    && upstream.1 >= 0
                    && upstream.0 < N as isize
                    && upstream.1 < N as isize
                {
                    return Err("chain has no boundary source");
                }
                let mut chain = Vec::new();
                let mut cursor = Some(i);
                while let Some(j) = cursor {
                    if seen[j] {
                        return Err("gravity cycle");
                    }
                    seen[j] = true;
                    chain.push(cell(j));
                    cursor = down[j];
                }
                let (x, y) = *chain.last().unwrap();
                let end = (x as isize + dx, y as isize + dy);
                if end.0 >= 0 && end.1 >= 0 && end.0 < N as isize && end.1 < N as isize {
                    return Err("chain has no boundary sink");
                }
                chains.push(chain);
            }
        }
        if seen.contains(&false) {
            return Err("gravity cycle");
        }
        Ok(Self {
            chains,
            portal_links: links,
        })
    }
}

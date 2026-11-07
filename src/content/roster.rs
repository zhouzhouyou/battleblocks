use super::{DWARF, FIRE_ELF, KNIGHT, NUN, PORTAL_CUBE, PRIEST, SLINGSHOT_USER};

/// Preserve the historical batch-analysis sample independently of menu filters.
pub const DEFAULT_AUTOPLAY_ROSTER: &[usize] =
    &[DWARF, KNIGHT, PRIEST, NUN, 1, FIRE_ELF, 3, 4, 5, 6];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeroGroup {
    #[default]
    New,
    Legacy,
}

impl HeroGroup {
    pub const ALL: [Self; 2] = [Self::New, Self::Legacy];

    pub const fn roster(self) -> &'static [usize] {
        match self {
            Self::New => &[
                DWARF,
                KNIGHT,
                PRIEST,
                NUN,
                FIRE_ELF,
                PORTAL_CUBE,
                SLINGSHOT_USER,
            ],
            Self::Legacy => &[0, 1, 2, 3, 4, 5, 6],
        }
    }

    pub fn for_hero(hero: usize) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|group| group.roster().contains(&hero))
    }

    pub fn other(self) -> Self {
        match self {
            Self::New => Self::Legacy,
            Self::Legacy => Self::New,
        }
    }
}
